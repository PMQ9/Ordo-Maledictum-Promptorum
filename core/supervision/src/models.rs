use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Status of an approval request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    /// Request is pending human review
    Pending,
    /// Request has been approved by a human
    Approved,
    /// Request has been denied by a human
    Denied,
    /// Request has expired without a decision
    Expired,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "pending"),
            ApprovalStatus::Approved => write!(f, "approved"),
            ApprovalStatus::Denied => write!(f, "denied"),
            ApprovalStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Risk level of an approval request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Notification channel for approval requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannel {
    Email,
    Slack,
    Teams,
}

/// Reason for requiring human approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalReason {
    /// Type of reason (e.g., "intent_mismatch", "high_risk_action", "parser_disagreement")
    pub reason_type: String,
    /// Detailed description of why approval is needed
    pub description: String,
    /// Additional context or metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// User intent that requires approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// The action being requested
    pub action: String,
    /// Topic or domain of the intent
    pub topic: Option<String>,
    /// Parameters for the intent
    pub parameters: HashMap<String, serde_json::Value>,
    /// References to user content
    pub content_refs: Vec<String>,
}

/// Request for human approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique identifier for this approval request
    pub id: String,
    /// The intent that requires approval
    pub intent: Intent,
    /// Reason(s) why approval is needed
    pub reasons: Vec<ApprovalReason>,
    /// Risk level of this request
    pub risk_level: RiskLevel,
    /// Current status of the request
    pub status: ApprovalStatus,
    /// When the request was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the request was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// When the request will expire if not processed
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The original raw user input
    pub raw_input: String,
    /// Parsed intents from different parsers (for comparison)
    pub parsed_intents: Vec<serde_json::Value>,
    /// Requestor information
    pub requestor_id: Option<String>,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(
        intent: Intent,
        reasons: Vec<ApprovalReason>,
        risk_level: RiskLevel,
        raw_input: String,
        parsed_intents: Vec<serde_json::Value>,
    ) -> Self {
        let now = chrono::Utc::now();
        let expires_at = Some(now + chrono::Duration::hours(24));

        Self {
            id: Uuid::new_v4().to_string(),
            intent,
            reasons,
            risk_level,
            status: ApprovalStatus::Pending,
            created_at: now,
            updated_at: now,
            expires_at,
            raw_input,
            parsed_intents,
            requestor_id: None,
        }
    }

    /// Check if the request has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Update the status of the request
    pub fn update_status(&mut self, new_status: ApprovalStatus) {
        self.status = new_status;
        self.updated_at = chrono::Utc::now();
    }
}

/// Human decision on an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanApproval {
    /// The approval request ID this decision is for
    pub request_id: String,
    /// Whether the request was approved
    pub approved: bool,
    /// Comments from the approver
    pub comments: Option<String>,
    /// Who approved or denied
    pub approver_id: String,
    /// When the decision was made
    pub decided_at: chrono::DateTime<chrono::Utc>,
    /// Any modifications to the intent (if approved with changes)
    pub modified_intent: Option<Intent>,
}

impl HumanApproval {
    /// Create a new approval decision
    pub fn new(
        request_id: String,
        approved: bool,
        approver_id: String,
        comments: Option<String>,
    ) -> Self {
        Self {
            request_id,
            approved,
            comments,
            approver_id,
            decided_at: chrono::Utc::now(),
            modified_intent: None,
        }
    }

    /// Create an approval with a modified intent
    pub fn with_modified_intent(
        request_id: String,
        approver_id: String,
        modified_intent: Intent,
        comments: Option<String>,
    ) -> Self {
        Self {
            request_id,
            approved: true,
            comments,
            approver_id,
            decided_at: chrono::Utc::now(),
            modified_intent: Some(modified_intent),
        }
    }
}

/// Configuration for the supervision service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionConfig {
    /// Default expiration time for approval requests (in hours)
    pub default_expiration_hours: i64,
    /// Notification channels to use
    pub notification_channels: Vec<NotificationChannel>,
    /// Email addresses for approval notifications
    pub notification_emails: Vec<String>,
    /// Slack webhook URL
    pub slack_webhook_url: Option<String>,
    /// Teams webhook URL
    pub teams_webhook_url: Option<String>,
    /// Whether to automatically deny expired requests
    pub auto_deny_expired: bool,
}

impl Default for SupervisionConfig {
    fn default() -> Self {
        Self {
            default_expiration_hours: 24,
            notification_channels: vec![NotificationChannel::Email],
            notification_emails: vec![],
            slack_webhook_url: None,
            teams_webhook_url: None,
            auto_deny_expired: true,
        }
    }
}
