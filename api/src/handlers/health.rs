//! Health check handler

use crate::state::AppState;
use crate::types::{HealthResponse, ServiceHealthStatus};
use axum::{extract::State, Json};
use std::sync::Arc;

/// GET /health - Health check endpoint
///
/// Returns the health status of the application and its services.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    // Check database health
    let database_healthy = sqlx::query("SELECT 1")
        .fetch_optional(&state.db_pool)
        .await
        .is_ok();

    // Check parsers health (always true if initialized)
    let parsers_healthy = true;

    // Check ledger health (same as database)
    let ledger_healthy = database_healthy;

    Json(HealthResponse {
        status: if database_healthy && parsers_healthy && ledger_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        services: ServiceHealthStatus {
            database: database_healthy,
            parsers: parsers_healthy,
            ledger: ledger_healthy,
        },
    })
}
