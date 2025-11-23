use crate::error::{Result, SupervisionError};
use crate::models::{ApprovalRequest, ApprovalStatus, HumanApproval};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Trait for approval storage backends
#[async_trait::async_trait]
pub trait ApprovalStorage: Send + Sync {
    /// Store a new approval request
    async fn store_request(&self, request: ApprovalRequest) -> Result<()>;

    /// Retrieve an approval request by ID
    async fn get_request(&self, request_id: &str) -> Result<Option<ApprovalRequest>>;

    /// Update an existing approval request
    async fn update_request(&self, request: ApprovalRequest) -> Result<()>;

    /// List all pending approval requests
    async fn list_pending(&self) -> Result<Vec<ApprovalRequest>>;

    /// Store a human approval decision
    async fn store_approval(&self, approval: HumanApproval) -> Result<()>;

    /// Get approval decision for a request
    async fn get_approval(&self, request_id: &str) -> Result<Option<HumanApproval>>;

    /// List all approval requests with a specific status
    async fn list_by_status(&self, status: ApprovalStatus) -> Result<Vec<ApprovalRequest>>;

    /// Delete expired requests
    async fn cleanup_expired(&self) -> Result<usize>;
}

/// In-memory implementation of approval storage
/// Suitable for development and testing, or for systems with low approval volume
#[derive(Clone)]
pub struct InMemoryStorage {
    requests: Arc<RwLock<HashMap<String, ApprovalRequest>>>,
    approvals: Arc<RwLock<HashMap<String, HumanApproval>>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage instance
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            approvals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ApprovalStorage for InMemoryStorage {
    async fn store_request(&self, request: ApprovalRequest) -> Result<()> {
        let mut requests = self.requests.write().await;
        debug!(
            request_id = %request.id,
            risk_level = ?request.risk_level,
            "Storing approval request"
        );
        requests.insert(request.id.clone(), request);
        Ok(())
    }

    async fn get_request(&self, request_id: &str) -> Result<Option<ApprovalRequest>> {
        let requests = self.requests.read().await;
        Ok(requests.get(request_id).cloned())
    }

    async fn update_request(&self, request: ApprovalRequest) -> Result<()> {
        let mut requests = self.requests.write().await;

        if !requests.contains_key(&request.id) {
            return Err(SupervisionError::ApprovalNotFound(request.id.clone()));
        }

        debug!(
            request_id = %request.id,
            status = %request.status,
            "Updating approval request"
        );
        requests.insert(request.id.clone(), request);
        Ok(())
    }

    async fn list_pending(&self) -> Result<Vec<ApprovalRequest>> {
        let requests = self.requests.read().await;
        Ok(requests
            .values()
            .filter(|r| r.status == ApprovalStatus::Pending)
            .cloned()
            .collect())
    }

    async fn store_approval(&self, approval: HumanApproval) -> Result<()> {
        let mut approvals = self.approvals.write().await;
        info!(
            request_id = %approval.request_id,
            approved = approval.approved,
            approver = %approval.approver_id,
            "Storing human approval decision"
        );
        approvals.insert(approval.request_id.clone(), approval);
        Ok(())
    }

    async fn get_approval(&self, request_id: &str) -> Result<Option<HumanApproval>> {
        let approvals = self.approvals.read().await;
        Ok(approvals.get(request_id).cloned())
    }

    async fn list_by_status(&self, status: ApprovalStatus) -> Result<Vec<ApprovalRequest>> {
        let requests = self.requests.read().await;
        Ok(requests
            .values()
            .filter(|r| r.status == status)
            .cloned()
            .collect())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let mut requests = self.requests.write().await;
        let mut expired_ids = Vec::new();

        for (id, request) in requests.iter_mut() {
            if request.status == ApprovalStatus::Pending && request.is_expired() {
                request.update_status(ApprovalStatus::Expired);
                expired_ids.push(id.clone());
            }
        }

        let count = expired_ids.len();
        if count > 0 {
            warn!(count = count, "Marked expired approval requests");
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApprovalReason, Intent, RiskLevel};
    use std::collections::HashMap;

    fn create_test_request() -> ApprovalRequest {
        let intent = Intent {
            action: "test_action".to_string(),
            topic: Some("test_topic".to_string()),
            parameters: HashMap::new(),
            content_refs: vec![],
        };

        let reason = ApprovalReason {
            reason_type: "test_reason".to_string(),
            description: "Test description".to_string(),
            metadata: HashMap::new(),
        };

        ApprovalRequest::new(
            intent,
            vec![reason],
            RiskLevel::Medium,
            "test input".to_string(),
            vec![],
        )
    }

    #[tokio::test]
    async fn test_store_and_retrieve_request() {
        let storage = InMemoryStorage::new();
        let request = create_test_request();
        let request_id = request.id.clone();

        storage.store_request(request.clone()).await.unwrap();

        let retrieved = storage.get_request(&request_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, request_id);
    }

    #[tokio::test]
    async fn test_list_pending() {
        let storage = InMemoryStorage::new();
        let request = create_test_request();

        storage.store_request(request).await.unwrap();

        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[tokio::test]
    async fn test_update_request() {
        let storage = InMemoryStorage::new();
        let mut request = create_test_request();
        let request_id = request.id.clone();

        storage.store_request(request.clone()).await.unwrap();

        request.update_status(ApprovalStatus::Approved);
        storage.update_request(request).await.unwrap();

        let updated = storage.get_request(&request_id).await.unwrap().unwrap();
        assert_eq!(updated.status, ApprovalStatus::Approved);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_approval() {
        let storage = InMemoryStorage::new();
        let request = create_test_request();
        let request_id = request.id.clone();

        storage.store_request(request).await.unwrap();

        let approval = HumanApproval::new(
            request_id.clone(),
            true,
            "approver1".to_string(),
            Some("Looks good".to_string()),
        );

        storage.store_approval(approval).await.unwrap();

        let retrieved = storage.get_approval(&request_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().approved, true);
    }
}
