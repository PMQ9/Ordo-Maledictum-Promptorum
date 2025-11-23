use thiserror::Error;

#[derive(Error, Debug)]
pub enum SupervisionError {
    #[error("Approval request not found: {0}")]
    ApprovalNotFound(String),

    #[error("Invalid approval request: {0}")]
    InvalidRequest(String),

    #[error("Notification failed: {0}")]
    NotificationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid approval status transition: from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Approval already processed: {0}")]
    AlreadyProcessed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SupervisionError>;
