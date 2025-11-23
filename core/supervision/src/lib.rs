//! Intent Supervision Module
//!
//! This module provides human supervision capabilities for the Intent Segregation system.
//! It manages approval requests for mismatched or high-risk intents, sends notifications
//! via multiple channels (email, Slack, Teams), tracks pending approvals, and returns
//! approval/denial decisions.
//!
//! # Features
//!
//! - **Approval Request Management**: Create and track approval requests for intents
//!   that require human review
//! - **Multi-Channel Notifications**: Send notifications via email, Slack, and Microsoft Teams
//! - **Flexible Storage**: In-memory storage for development, with support for custom
//!   storage backends (database, etc.)
//! - **Risk-Based Workflow**: Categorize requests by risk level (Low, Medium, High, Critical)
//! - **Expiration Handling**: Automatic expiration and cleanup of stale requests
//! - **Audit Trail**: Complete tracking of all approval decisions
//!
//! # Architecture
//!
//! The supervision module is triggered when:
//! - Intent parsers disagree on the interpretation
//! - Parsed intent doesn't match provider-allowed intents
//! - High-risk actions are detected
//! - Unusual parameter patterns are found
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use intent_supervision::{
//!     SupervisionService, SupervisionConfig, Intent, ApprovalReason,
//!     RiskLevel, ApprovalStatus,
//! };
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure the supervision service
//!     let config = SupervisionConfig {
//!         default_expiration_hours: 24,
//!         notification_channels: vec![],
//!         notification_emails: vec!["admin@example.com".to_string()],
//!         slack_webhook_url: None,
//!         teams_webhook_url: None,
//!         auto_deny_expired: true,
//!     };
//!
//!     // Create the service
//!     let service = SupervisionService::new(config);
//!
//!     // Start background cleanup task
//!     service.start_cleanup_task().await;
//!
//!     // Create an intent that needs approval
//!     let intent = Intent {
//!         action: "delete_data".to_string(),
//!         topic: Some("user_records".to_string()),
//!         parameters: HashMap::new(),
//!         content_refs: vec![],
//!     };
//!
//!     // Define why approval is needed
//!     let reason = ApprovalReason {
//!         reason_type: "high_risk_action".to_string(),
//!         description: "Deletion operation requires approval".to_string(),
//!         metadata: HashMap::new(),
//!     };
//!
//!     // Request approval
//!     let request = service
//!         .request_approval(
//!             intent,
//!             vec![reason],
//!             RiskLevel::High,
//!             "Delete all user records from 2023".to_string(),
//!             vec![],
//!         )
//!         .await?;
//!
//!     println!("Approval request created: {}", request.id);
//!
//!     // Check status later
//!     let approval = service.check_approval_status(&request.id).await?;
//!
//!     match approval {
//!         Some(decision) => {
//!             if decision.approved {
//!                 println!("Request approved by {}", decision.approver_id);
//!                 // Proceed with the action
//!             } else {
//!                 println!("Request denied by {}", decision.approver_id);
//!                 // Reject the action
//!             }
//!         }
//!         None => {
//!             println!("Still pending approval");
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Module Organization
//!
//! - `models`: Data structures for approval requests, decisions, and configuration
//! - `service`: Main SupervisionService implementation
//! - `storage`: Storage backends for persisting approval requests
//! - `notifications`: Multi-channel notification system
//! - `error`: Error types and result aliases

pub mod error;
pub mod models;
pub mod notifications;
pub mod service;
pub mod storage;

// Re-export commonly used types
pub use error::{Result, SupervisionError};
pub use models::{
    ApprovalReason, ApprovalRequest, ApprovalStatus, HumanApproval, Intent, NotificationChannel,
    RiskLevel, SupervisionConfig,
};
pub use service::SupervisionService;
pub use storage::{ApprovalStorage, InMemoryStorage};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_module_exports() {
        // Verify all main types are accessible
        let _config: SupervisionConfig = SupervisionConfig::default();
        let _status: ApprovalStatus = ApprovalStatus::Pending;
        let _risk: RiskLevel = RiskLevel::Medium;
    }

    #[tokio::test]
    async fn test_end_to_end_workflow() {
        // Test the complete approval workflow
        let config = SupervisionConfig {
            default_expiration_hours: 24,
            notification_channels: vec![],
            notification_emails: vec![],
            slack_webhook_url: None,
            teams_webhook_url: None,
            auto_deny_expired: true,
        };

        let service = SupervisionService::new(config);

        // 1. Request approval
        let intent = Intent {
            action: "sensitive_action".to_string(),
            topic: Some("critical_data".to_string()),
            parameters: HashMap::new(),
            content_refs: vec![],
        };

        let reason = ApprovalReason {
            reason_type: "intent_mismatch".to_string(),
            description: "Action not in allowed list".to_string(),
            metadata: HashMap::new(),
        };

        let request = service
            .request_approval(
                intent,
                vec![reason],
                RiskLevel::Critical,
                "Do something sensitive with critical data".to_string(),
                vec![],
            )
            .await
            .unwrap();

        // 2. Check status (should be pending)
        let status = service.check_approval_status(&request.id).await.unwrap();
        assert!(status.is_none());

        // 3. List pending requests
        let pending = service.list_pending().await.unwrap();
        assert_eq!(pending.len(), 1);

        // 4. Submit approval
        let updated = service
            .submit_approval(
                &request.id,
                true,
                "security_admin".to_string(),
                Some("Approved after review".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(updated.status, ApprovalStatus::Approved);

        // 5. Check status again (should have approval)
        let approval = service
            .check_approval_status(&request.id)
            .await
            .unwrap()
            .unwrap();

        assert!(approval.approved);
        assert_eq!(approval.approver_id, "security_admin");
    }
}
