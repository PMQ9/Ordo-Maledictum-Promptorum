//! Ledger query handlers

use crate::error::AppError;
use crate::state::AppState;
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// GET /api/ledger/query - Query ledger entries
///
/// Supports various filters:
/// - user_id: Filter by user
/// - session_id: Filter by session
/// - start_time/end_time: Time range filter
/// - elevation_only: Only entries requiring elevation
/// - blocked_only: Only blocked entries
/// - limit: Maximum number of results
pub async fn query_ledger(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LedgerQueryParams>,
) -> Result<Json<LedgerQueryResponse>, AppError> {
    info!("Querying ledger with params: {:?}", params);

    let entries = if let Some(user_id) = &params.user_id {
        // Query by user ID
        state.ledger.query_by_user(user_id, params.limit).await?
    } else if let Some(session_id) = &params.session_id {
        // Query by session ID
        state.ledger.query_by_session(session_id).await?
    } else if params.start_time.is_some() || params.end_time.is_some() {
        // Query by time range
        let start = params
            .start_time
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));

        let end = params
            .end_time
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        state
            .ledger
            .query_by_time_range(start, end, params.limit)
            .await?
    } else if params.elevation_only == Some(true) {
        // Query elevation events
        state.ledger.query_elevation_events(params.limit).await?
    } else if params.blocked_only == Some(true) {
        // Query blocked entries
        state.ledger.query_blocked_entries(params.limit).await?
    } else {
        // Default: return recent entries
        let end = Utc::now();
        let start = end - chrono::Duration::days(7);
        state
            .ledger
            .query_by_time_range(start, end, params.limit.or(Some(100)))
            .await?
    };

    // Convert to response format
    let response_entries: Vec<LedgerEntryResponse> = entries
        .iter()
        .map(|entry| LedgerEntryResponse {
            id: entry.id,
            user_id: entry.user_id.clone(),
            session_id: entry.session_id.clone(),
            timestamp: entry.timestamp,
            user_input: entry.user_input.clone(),
            malicious_blocked: entry.malicious_blocked,
            voting_confidence: format!("{:?}", entry.voting_result.agreement_level),
            comparison_decision: format!("{:?}", entry.comparison_result.decision),
            required_approval: entry.elevation_event.is_some(),
            was_executed: entry.processing_output.is_some(),
        })
        .collect();

    let count = response_entries.len();

    Ok(Json(LedgerQueryResponse {
        entries: response_entries,
        count,
    }))
}

/// GET /api/ledger/:id - Get specific ledger entry
///
/// Returns a single ledger entry by its UUID.
pub async fn get_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<intent_ledger::LedgerEntry>, AppError> {
    info!(entry_id = %id, "Fetching ledger entry");

    let entry = state.ledger.query_by_id(id).await?;

    Ok(Json(entry))
}

/// GET /api/ledger/stats - Get ledger statistics
///
/// Returns aggregate statistics about the ledger.
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LedgerStatsResponse>, AppError> {
    info!("Fetching ledger statistics");

    let stats = state.ledger.get_stats().await?;

    Ok(Json(LedgerStatsResponse {
        total_entries: stats.total_entries,
        total_users: stats.total_users,
        total_sessions: stats.total_sessions,
        blocked_entries: stats.blocked_entries,
        elevation_events: stats.elevation_events,
        oldest_entry: stats.oldest_entry,
        newest_entry: stats.newest_entry,
    }))
}
