use thiserror::Error;

/// Notification-related errors
#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Email sending failed: {0}")]
    EmailError(String),

    #[error("Slack webhook failed: {0}")]
    SlackError(String),

    #[error("Teams webhook failed: {0}")]
    TeamsError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),

    #[error("Missing configuration: {0}")]
    MissingConfig(String),
}

pub type Result<T> = std::result::Result<T, NotificationError>;
