//! Error handling for the API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application-level errors
#[derive(Debug)]
pub enum AppError {
    /// Malicious input detected
    MaliciousInput(String),

    /// Parser errors
    ParserError(String),

    /// Voting errors
    VotingError(String),

    /// Comparison errors
    ComparisonError(String),

    /// Intent generation errors
    GenerationError(String),

    /// Processing errors
    ProcessingError(String),

    /// Ledger errors
    LedgerError(String),

    /// Database errors
    DatabaseError(String),

    /// Configuration errors
    ConfigError(String),

    /// Approval errors
    ApprovalNotFound,
    ApprovalAlreadyDecided,
    ApprovalPending,

    /// Validation errors
    ValidationError(String),

    /// Not found errors
    NotFound(String),

    /// Internal server errors
    InternalError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaliciousInput(msg) => write!(f, "Malicious input detected: {}", msg),
            Self::ParserError(msg) => write!(f, "Parser error: {}", msg),
            Self::VotingError(msg) => write!(f, "Voting error: {}", msg),
            Self::ComparisonError(msg) => write!(f, "Comparison error: {}", msg),
            Self::GenerationError(msg) => write!(f, "Intent generation error: {}", msg),
            Self::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            Self::LedgerError(msg) => write!(f, "Ledger error: {}", msg),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::ApprovalNotFound => write!(f, "Approval request not found"),
            Self::ApprovalAlreadyDecided => write!(f, "Approval already decided"),
            Self::ApprovalPending => write!(f, "Approval is pending"),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            Self::MaliciousInput(msg) => (StatusCode::BAD_REQUEST, "malicious_input", msg),
            Self::ParserError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "parser_error", msg),
            Self::VotingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "voting_error", msg),
            Self::ComparisonError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "comparison_error", msg)
            }
            Self::GenerationError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "generation_error", msg)
            }
            Self::ProcessingError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "processing_error", msg)
            }
            Self::LedgerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "ledger_error", msg),
            Self::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error", msg),
            Self::ConfigError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error", msg),
            Self::ApprovalNotFound => (
                StatusCode::NOT_FOUND,
                "approval_not_found",
                "Approval request not found".to_string(),
            ),
            Self::ApprovalAlreadyDecided => (
                StatusCode::CONFLICT,
                "approval_already_decided",
                "Approval has already been decided".to_string(),
            ),
            Self::ApprovalPending => (
                StatusCode::ACCEPTED,
                "approval_pending",
                "Approval is pending human review".to_string(),
            ),
            Self::ValidationError(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            Self::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        let body = Json(json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// Implement From for common error types
impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        Self::ConfigError(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<intent_ledger::LedgerError> for AppError {
    fn from(err: intent_ledger::LedgerError) -> Self {
        Self::LedgerError(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}
