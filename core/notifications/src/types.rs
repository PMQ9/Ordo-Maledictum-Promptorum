use serde::{Deserialize, Serialize};

/// Priority level for notifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Type of notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    ApprovalRequest,
    Alert,
    Info,
    Warning,
    Error,
}

/// Email message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    /// Recipient email address(es)
    pub to: Vec<String>,

    /// Email subject
    pub subject: String,

    /// Email body (can be plain text or HTML)
    pub body: String,

    /// Whether the body is HTML
    #[serde(default)]
    pub is_html: bool,

    /// CC recipients (optional)
    #[serde(default)]
    pub cc: Vec<String>,
}

/// Slack message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    /// Message text (supports Slack markdown)
    pub text: String,

    /// Optional channel override (e.g., "#alerts")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,

    /// Optional username override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Optional emoji icon (e.g., ":robot_face:")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_emoji: Option<String>,

    /// Optional attachments for rich formatting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<SlackAttachment>>,
}

/// Slack attachment for rich messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAttachment {
    /// Attachment color (e.g., "good", "warning", "danger", or hex color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Attachment title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Attachment text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Fields for structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<SlackField>>,

    /// Footer text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,

    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<i64>,
}

/// Slack field for attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackField {
    pub title: String,
    pub value: String,
    #[serde(default)]
    pub short: bool,
}

/// Microsoft Teams message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsMessage {
    /// Message title
    pub title: String,

    /// Message text (supports markdown)
    pub text: String,

    /// Theme color (hex color without #)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_color: Option<String>,

    /// Sections for structured content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<TeamsSection>>,

    /// Potential actions (buttons)
    #[serde(skip_serializing_if = "Option::is_none", rename = "potentialAction")]
    pub potential_action: Option<Vec<TeamsAction>>,
}

/// Teams message section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsSection {
    #[serde(skip_serializing_if = "Option::is_none", rename = "activityTitle")]
    pub activity_title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "activitySubtitle")]
    pub activity_subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<Vec<TeamsFact>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Teams fact (key-value pair)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsFact {
    pub name: String,
    pub value: String,
}

/// Teams action (button)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsAction {
    #[serde(rename = "@type")]
    pub action_type: String,

    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<TeamsActionTarget>>,
}

/// Teams action target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsActionTarget {
    pub os: String,
    pub uri: String,
}

/// Approval request notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Request ID for tracking
    pub request_id: String,

    /// Title/summary of the approval request
    pub title: String,

    /// Detailed description
    pub description: String,

    /// User who initiated the request
    pub requested_by: String,

    /// Timestamp of the request
    pub timestamp: String,

    /// Intent that requires approval
    pub intent_summary: String,

    /// Approval URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_url: Option<String>,

    /// Priority level
    pub priority: NotificationPriority,
}

/// Alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub alert_id: String,

    /// Alert title
    pub title: String,

    /// Alert message
    pub message: String,

    /// Alert type
    pub alert_type: NotificationType,

    /// Priority level
    pub priority: NotificationPriority,

    /// Timestamp
    pub timestamp: String,

    /// Additional context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}
