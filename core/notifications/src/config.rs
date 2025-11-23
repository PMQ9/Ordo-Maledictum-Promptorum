use serde::{Deserialize, Serialize};

/// Notification system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub slack: Option<SlackConfig>,

    #[serde(default)]
    pub teams: Option<TeamsConfig>,

    #[serde(default)]
    pub email: Option<EmailConfig>,
}

/// Slack webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Slack webhook URL
    /// Example: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
    pub webhook_url: String,

    /// Default channel (optional, webhook usually has a default)
    #[serde(default)]
    pub default_channel: Option<String>,

    /// Bot username to display
    #[serde(default = "default_slack_username")]
    pub username: String,
}

fn default_slack_username() -> String {
    "Intent Security Bot".to_string()
}

/// Microsoft Teams webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    /// Teams webhook URL
    /// Example: "https://outlook.office.com/webhook/YOUR/WEBHOOK/URL"
    pub webhook_url: String,

    /// Theme color for cards (hex color)
    #[serde(default = "default_teams_color")]
    pub theme_color: String,
}

fn default_teams_color() -> String {
    "0076D7".to_string() // Microsoft Blue
}

/// Email configuration using SMTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server address
    /// Example: "smtp.gmail.com"
    pub smtp_server: String,

    /// SMTP port (usually 587 for STARTTLS or 465 for SSL)
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,

    /// SMTP username
    pub smtp_user: String,

    /// SMTP password
    pub smtp_password: String,

    /// From address for emails
    pub from_address: String,

    /// From name (display name)
    #[serde(default = "default_from_name")]
    pub from_name: String,

    /// Use STARTTLS encryption
    #[serde(default = "default_use_starttls")]
    pub use_starttls: bool,
}

fn default_smtp_port() -> u16 {
    587
}

fn default_from_name() -> String {
    "Intent Security System".to_string()
}

fn default_use_starttls() -> bool {
    true
}

impl NotificationConfig {
    /// Check if email is configured
    pub fn has_email(&self) -> bool {
        self.email.is_some()
    }

    /// Check if Slack is configured
    pub fn has_slack(&self) -> bool {
        self.slack.is_some()
    }

    /// Check if Teams is configured
    pub fn has_teams(&self) -> bool {
        self.teams.is_some()
    }
}
