use crate::error::{Result, SupervisionError};
use crate::models::{
    ApprovalReason, ApprovalRequest, ApprovalStatus, HumanApproval, Intent, RiskLevel,
    SupervisionConfig,
};
use crate::notifications::{MultiChannelNotifier, NotificationService};
use crate::storage::{ApprovalStorage, InMemoryStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Main supervision service for managing human approval requests
pub struct SupervisionService<S: ApprovalStorage = InMemoryStorage> {
    storage: Arc<S>,
    notifier: Arc<dyn NotificationService>,
    config: SupervisionConfig,
    cleanup_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl SupervisionService<InMemoryStorage> {
    /// Create a new supervision service with default in-memory storage
    pub fn new(config: SupervisionConfig) -> Self {
        let storage = Arc::new(InMemoryStorage::new());
        let notifier = Arc::new(MultiChannelNotifier::new(config.clone()));

        Self {
            storage,
            notifier,
            config,
            cleanup_task: Arc::new(RwLock::new(None)),
        }
    }
}

impl<S: ApprovalStorage + 'static> SupervisionService<S> {
    /// Create a new supervision service with custom storage
    pub fn with_storage(storage: S, config: SupervisionConfig) -> Self {
        let storage = Arc::new(storage);
        let notifier = Arc::new(MultiChannelNotifier::new(config.clone()));

        Self {
            storage,
            notifier,
            config,
            cleanup_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a supervision service with custom storage and notifier
    pub fn with_custom(
        storage: S,
        notifier: Arc<dyn NotificationService>,
        config: SupervisionConfig,
    ) -> Self {
        Self {
            storage: Arc::new(storage),
            notifier,
            config,
            cleanup_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the background cleanup task for expired requests
    pub async fn start_cleanup_task(&self) {
        let storage = Arc::clone(&self.storage);
        let interval_hours = self.config.default_expiration_hours.min(6); // Check at least every 6 hours

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                (interval_hours * 3600) as u64,
            ));

            loop {
                interval.tick().await;
                debug!("Running expired request cleanup");

                match storage.cleanup_expired().await {
                    Ok(count) if count > 0 => {
                        info!(count = count, "Cleaned up expired approval requests");
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to cleanup expired requests");
                    }
                    _ => {}
                }
            }
        });

        let mut task = self.cleanup_task.write().await;
        *task = Some(handle);
    }

    /// Stop the background cleanup task
    pub async fn stop_cleanup_task(&self) {
        let mut task = self.cleanup_task.write().await;
        if let Some(handle) = task.take() {
            handle.abort();
        }
    }

    /// Request human approval for a mismatched or high-risk intent
    ///
    /// # Arguments
    /// * `intent` - The intent that requires approval
    /// * `reasons` - Reasons why approval is needed
    /// * `risk_level` - Risk level of the intent
    /// * `raw_input` - The original user input
    /// * `parsed_intents` - All parsed intents from different parsers for comparison
    ///
    /// # Returns
    /// The created approval request
    pub async fn request_approval(
        &self,
        intent: Intent,
        reasons: Vec<ApprovalReason>,
        risk_level: RiskLevel,
        raw_input: String,
        parsed_intents: Vec<serde_json::Value>,
    ) -> Result<ApprovalRequest> {
        // Create the approval request
        let mut request =
            ApprovalRequest::new(intent, reasons, risk_level, raw_input, parsed_intents);

        // Set expiration based on config
        if let Some(expires_at) = request.expires_at.as_mut() {
            *expires_at =
                chrono::Utc::now() + chrono::Duration::hours(self.config.default_expiration_hours);
        }

        info!(
            request_id = %request.id,
            risk_level = ?request.risk_level,
            action = %request.intent.action,
            "Creating approval request"
        );

        // Store the request
        self.storage.store_request(request.clone()).await?;

        // Send notifications
        if let Err(e) = self.notifier.notify_approval_request(&request).await {
            warn!(
                request_id = %request.id,
                error = %e,
                "Failed to send approval notification, but request was saved"
            );
        }

        Ok(request)
    }

    /// Check the status of an approval request
    ///
    /// # Arguments
    /// * `request_id` - The ID of the approval request
    ///
    /// # Returns
    /// The human approval decision if one has been made, None otherwise
    pub async fn check_approval_status(&self, request_id: &str) -> Result<Option<HumanApproval>> {
        debug!(request_id, "Checking approval status");

        // Get the request to check if it exists and its current status
        let request = self
            .storage
            .get_request(request_id)
            .await?
            .ok_or_else(|| SupervisionError::ApprovalNotFound(request_id.to_string()))?;

        // Check if expired and update status if needed
        if request.status == ApprovalStatus::Pending && request.is_expired() {
            let mut updated_request = request.clone();
            updated_request.update_status(ApprovalStatus::Expired);
            self.storage.update_request(updated_request).await?;

            if self.config.auto_deny_expired {
                warn!(request_id, "Approval request expired");
                self.notifier.notify_expired(request_id).await?;
            }

            return Ok(None);
        }

        // Get the approval decision if one exists
        self.storage.get_approval(request_id).await
    }

    /// Submit a human approval decision
    ///
    /// # Arguments
    /// * `request_id` - The ID of the approval request
    /// * `approved` - Whether the request is approved
    /// * `approver_id` - Identifier of the person making the decision
    /// * `comments` - Optional comments about the decision
    ///
    /// # Returns
    /// The updated approval request
    pub async fn submit_approval(
        &self,
        request_id: &str,
        approved: bool,
        approver_id: String,
        comments: Option<String>,
    ) -> Result<ApprovalRequest> {
        // Get the request
        let mut request = self
            .storage
            .get_request(request_id)
            .await?
            .ok_or_else(|| SupervisionError::ApprovalNotFound(request_id.to_string()))?;

        // Check if already processed
        if request.status != ApprovalStatus::Pending {
            return Err(SupervisionError::AlreadyProcessed(format!(
                "Request already has status: {}",
                request.status
            )));
        }

        // Check if expired
        if request.is_expired() {
            return Err(SupervisionError::InvalidRequest(
                "Cannot approve expired request".to_string(),
            ));
        }

        info!(
            request_id,
            approved,
            approver = %approver_id,
            "Processing approval decision"
        );

        // Create the approval decision
        let approval = HumanApproval::new(
            request_id.to_string(),
            approved,
            approver_id.clone(),
            comments.clone(),
        );

        // Update request status
        let new_status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Denied
        };
        request.update_status(new_status);

        // Store both the updated request and the approval decision
        self.storage.update_request(request.clone()).await?;
        self.storage.store_approval(approval).await?;

        // Send notifications
        if approved {
            let _ = self
                .notifier
                .notify_approved(request_id, &approver_id)
                .await;
        } else {
            let _ = self
                .notifier
                .notify_denied(request_id, &approver_id, comments.as_deref())
                .await;
        }

        Ok(request)
    }

    /// List all pending approval requests
    pub async fn list_pending(&self) -> Result<Vec<ApprovalRequest>> {
        self.storage.list_pending().await
    }

    /// List all approval requests with a specific status
    pub async fn list_by_status(&self, status: ApprovalStatus) -> Result<Vec<ApprovalRequest>> {
        self.storage.list_by_status(status).await
    }

    /// Get a specific approval request by ID
    pub async fn get_request(&self, request_id: &str) -> Result<Option<ApprovalRequest>> {
        self.storage.get_request(request_id).await
    }

    /// Manually cleanup expired requests
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let count = self.storage.cleanup_expired().await?;

        if count > 0 && self.config.auto_deny_expired {
            info!(count = count, "Expired {} approval requests", count);
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> SupervisionConfig {
        SupervisionConfig {
            default_expiration_hours: 24,
            notification_channels: vec![],
            notification_emails: vec![],
            slack_webhook_url: None,
            teams_webhook_url: None,
            auto_deny_expired: true,
        }
    }

    fn create_test_intent() -> Intent {
        Intent {
            action: "test_action".to_string(),
            topic: Some("test_topic".to_string()),
            parameters: HashMap::new(),
            content_refs: vec![],
        }
    }

    fn create_test_reason() -> ApprovalReason {
        ApprovalReason {
            reason_type: "intent_mismatch".to_string(),
            description: "Parsed intent doesn't match provider config".to_string(),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_request_approval() {
        let config = create_test_config();
        let service = SupervisionService::new(config);

        let intent = create_test_intent();
        let reason = create_test_reason();

        let request = service
            .request_approval(
                intent,
                vec![reason],
                RiskLevel::Medium,
                "test input".to_string(),
                vec![],
            )
            .await
            .unwrap();

        assert_eq!(request.status, ApprovalStatus::Pending);
        assert!(!request.id.is_empty());
    }

    #[tokio::test]
    async fn test_check_approval_status() {
        let config = create_test_config();
        let service = SupervisionService::new(config);

        let intent = create_test_intent();
        let reason = create_test_reason();

        let request = service
            .request_approval(
                intent,
                vec![reason],
                RiskLevel::Medium,
                "test input".to_string(),
                vec![],
            )
            .await
            .unwrap();

        let status = service.check_approval_status(&request.id).await.unwrap();

        assert!(status.is_none()); // No decision made yet
    }

    #[tokio::test]
    async fn test_submit_approval() {
        let config = create_test_config();
        let service = SupervisionService::new(config);

        let intent = create_test_intent();
        let reason = create_test_reason();

        let request = service
            .request_approval(
                intent,
                vec![reason],
                RiskLevel::Medium,
                "test input".to_string(),
                vec![],
            )
            .await
            .unwrap();

        let updated = service
            .submit_approval(
                &request.id,
                true,
                "approver1".to_string(),
                Some("Looks good".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(updated.status, ApprovalStatus::Approved);

        let approval = service
            .check_approval_status(&request.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(approval.approved, true);
        assert_eq!(approval.approver_id, "approver1");
    }

    #[tokio::test]
    async fn test_submit_denial() {
        let config = create_test_config();
        let service = SupervisionService::new(config);

        let intent = create_test_intent();
        let reason = create_test_reason();

        let request = service
            .request_approval(
                intent,
                vec![reason],
                RiskLevel::High,
                "test input".to_string(),
                vec![],
            )
            .await
            .unwrap();

        let updated = service
            .submit_approval(
                &request.id,
                false,
                "approver1".to_string(),
                Some("Too risky".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(updated.status, ApprovalStatus::Denied);
    }

    #[tokio::test]
    async fn test_list_pending() {
        let config = create_test_config();
        let service = SupervisionService::new(config);

        let intent = create_test_intent();
        let reason = create_test_reason();

        // Create multiple requests
        for _ in 0..3 {
            service
                .request_approval(
                    intent.clone(),
                    vec![reason.clone()],
                    RiskLevel::Medium,
                    "test input".to_string(),
                    vec![],
                )
                .await
                .unwrap();
        }

        let pending = service.list_pending().await.unwrap();
        assert_eq!(pending.len(), 3);
    }

    #[tokio::test]
    async fn test_cannot_approve_twice() {
        let config = create_test_config();
        let service = SupervisionService::new(config);

        let intent = create_test_intent();
        let reason = create_test_reason();

        let request = service
            .request_approval(
                intent,
                vec![reason],
                RiskLevel::Medium,
                "test input".to_string(),
                vec![],
            )
            .await
            .unwrap();

        // First approval should succeed
        service
            .submit_approval(&request.id, true, "approver1".to_string(), None)
            .await
            .unwrap();

        // Second approval should fail
        let result = service
            .submit_approval(&request.id, false, "approver2".to_string(), None)
            .await;

        assert!(result.is_err());
    }
}
