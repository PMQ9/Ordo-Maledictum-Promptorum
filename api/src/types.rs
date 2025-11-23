//! Request and response types for the API

use intent_schema::Intent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Process Endpoint Types
// ============================================================================

/// Request to process user input
#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    /// The raw user input text
    pub user_input: String,

    /// User identifier
    pub user_id: String,

    /// Session identifier
    pub session_id: String,
}

/// Response from processing user input
#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    /// Unique ID for this request (can be used to check approval status)
    pub request_id: Uuid,

    /// Status of the request
    pub status: ProcessStatus,

    /// The trusted intent (if approved and generated)
    pub trusted_intent: Option<Intent>,

    /// Processing result (if executed)
    pub result: Option<serde_json::Value>,

    /// Human-readable message
    pub message: String,

    /// Detailed pipeline information
    pub pipeline_info: PipelineInfo,
}

/// Status of a process request
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    /// Input was blocked as malicious
    Blocked,

    /// Request requires human approval (pending)
    PendingApproval,

    /// Request was approved and processed successfully
    Completed,

    /// Request was denied (either by policy or human)
    Denied,

    /// Request failed during processing
    Failed,
}

/// Information about each stage of the pipeline
#[derive(Debug, Serialize)]
pub struct PipelineInfo {
    /// Malicious detection result
    pub malicious_detection: Option<MaliciousDetectionInfo>,

    /// Parser results
    pub parser_results: Option<Vec<ParserResultInfo>>,

    /// Voting result
    pub voting_result: Option<VotingResultInfo>,

    /// Comparison result
    pub comparison_result: Option<ComparisonResultInfo>,
}

#[derive(Debug, Serialize)]
pub struct MaliciousDetectionInfo {
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParserResultInfo {
    pub parser_id: String,
    pub success: bool,
    pub confidence: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct VotingResultInfo {
    pub confidence_level: String,
    pub average_similarity: f64,
    pub requires_human_review: bool,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
pub struct ComparisonResultInfo {
    pub result: String, // "approved", "soft_mismatch", "hard_mismatch"
    pub message: String,
    pub reasons: Vec<String>,
}

// ============================================================================
// Approval Endpoint Types
// ============================================================================

/// Response for approval status check
#[derive(Debug, Serialize)]
pub struct ApprovalStatusResponse {
    pub id: Uuid,
    pub status: ApprovalStatus,
    pub intent: Option<Intent>,
    pub reason: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub decision: Option<ApprovalDecisionInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
}

#[derive(Debug, Serialize)]
pub struct ApprovalDecisionInfo {
    pub approved: bool,
    pub approver_id: String,
    pub reason: String,
    pub decided_at: chrono::DateTime<chrono::Utc>,
}

/// Request to submit approval decision
#[derive(Debug, Deserialize)]
pub struct ApprovalDecisionRequest {
    /// Whether to approve (true) or deny (false)
    pub approved: bool,

    /// Identifier of the approver
    pub approver_id: String,

    /// Reason for the decision
    pub reason: String,
}

/// Response after submitting approval decision
#[derive(Debug, Serialize)]
pub struct ApprovalDecisionResponse {
    pub id: Uuid,
    pub approved: bool,
    pub message: String,
}

// ============================================================================
// Ledger Endpoint Types
// ============================================================================

/// Query parameters for ledger queries
#[derive(Debug, Deserialize)]
pub struct LedgerQueryParams {
    /// User ID filter
    pub user_id: Option<String>,

    /// Session ID filter
    pub session_id: Option<String>,

    /// Start timestamp (ISO 8601)
    pub start_time: Option<String>,

    /// End timestamp (ISO 8601)
    pub end_time: Option<String>,

    /// Filter for entries requiring elevation
    pub elevation_only: Option<bool>,

    /// Filter for blocked entries
    pub blocked_only: Option<bool>,

    /// Maximum number of results
    pub limit: Option<i64>,
}

/// Response containing ledger entries
#[derive(Debug, Serialize)]
pub struct LedgerQueryResponse {
    pub entries: Vec<LedgerEntryResponse>,
    pub count: usize,
}

/// Ledger entry in response format
#[derive(Debug, Serialize)]
pub struct LedgerEntryResponse {
    pub id: Uuid,
    pub user_id: String,
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_input: String,
    pub malicious_blocked: bool,
    pub voting_confidence: String,
    pub comparison_decision: String,
    pub required_approval: bool,
    pub was_executed: bool,
}

/// Response containing ledger statistics
#[derive(Debug, Serialize)]
pub struct LedgerStatsResponse {
    pub total_entries: i64,
    pub total_users: i64,
    pub total_sessions: i64,
    pub blocked_entries: i64,
    pub elevation_events: i64,
    pub oldest_entry: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_entry: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// Health Check Types
// ============================================================================

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub services: ServiceHealthStatus,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealthStatus {
    pub database: bool,
    pub parsers: bool,
    pub ledger: bool,
}
