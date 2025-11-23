//! # Intent Notifications Module
//!
//! This module provides a notification service for the Intent Segregation security architecture.
//! It supports sending notifications via:
//! - Email (SMTP using lettre)
//! - Slack (webhooks)
//! - Microsoft Teams (webhooks)
//!
//! ## Features
//!
//! - Send approval requests to multiple channels
//! - Send security alerts
//! - Rich message formatting for Slack and Teams
//! - HTML email support with plain text fallback
//!
//! ## Example
//!
//! ```no_run
//! use intent_notifications::{
//!     NotificationService, NotificationConfig, EmailConfig,
//!     SlackConfig, TeamsConfig, EmailMessage
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure notification channels
//!     let config = NotificationConfig {
//!         email: Some(EmailConfig {
//!             smtp_server: "smtp.gmail.com".to_string(),
//!             smtp_port: 587,
//!             smtp_user: "your-email@example.com".to_string(),
//!             smtp_password: "your-password".to_string(),
//!             from_address: "your-email@example.com".to_string(),
//!             from_name: "Intent Security".to_string(),
//!             use_starttls: true,
//!         }),
//!         slack: Some(SlackConfig {
//!             webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL".to_string(),
//!             default_channel: None,
//!             username: "Intent Bot".to_string(),
//!         }),
//!         teams: Some(TeamsConfig {
//!             webhook_url: "https://outlook.office.com/webhook/YOUR/WEBHOOK/URL".to_string(),
//!             theme_color: "0076D7".to_string(),
//!         }),
//!     };
//!
//!     // Create the notification service
//!     let service = NotificationService::new(config);
//!
//!     // Send an email
//!     let email = EmailMessage {
//!         to: vec!["recipient@example.com".to_string()],
//!         subject: "Test Notification".to_string(),
//!         body: "This is a test message.".to_string(),
//!         is_html: false,
//!         cc: vec![],
//!     };
//!
//!     service.send_email(&email).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod service;
pub mod types;

// Re-export main types for convenience
pub use config::{EmailConfig, NotificationConfig, SlackConfig, TeamsConfig};
pub use error::{NotificationError, Result};
pub use service::NotificationService;
pub use types::{
    Alert, ApprovalRequest, EmailMessage, NotificationPriority, NotificationType, SlackAttachment,
    SlackField, SlackMessage, TeamsAction, TeamsActionTarget, TeamsFact, TeamsMessage,
    TeamsSection,
};
