//! Approval workflow handlers

use crate::error::AppError;
use crate::state::{AppState, ApprovalDecision};
use crate::types::*;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// GET /api/approvals/:id - Check approval status
///
/// Returns the current status of an approval request.
pub async fn get_approval_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApprovalStatusResponse>, AppError> {
    info!(approval_id = %id, "Checking approval status");

    // Get pending approval
    let pending = state
        .get_pending_approval(id)
        .await
        .ok_or(AppError::ApprovalNotFound)?;

    // Check if decision has been made
    let decision = state.get_approval_decision(id).await;

    let (status, decision_info) = match decision {
        Some(d) => {
            let status = if d.approved {
                ApprovalStatus::Approved
            } else {
                ApprovalStatus::Denied
            };
            let info = Some(ApprovalDecisionInfo {
                approved: d.approved,
                approver_id: d.approver_id,
                reason: d.reason,
                decided_at: d.decided_at,
            });
            (status, info)
        }
        None => (ApprovalStatus::Pending, None),
    };

    Ok(Json(ApprovalStatusResponse {
        id,
        status,
        intent: Some(pending.intent),
        reason: pending.reason,
        created_at: pending.created_at,
        decision: decision_info,
    }))
}

/// POST /api/approvals/:id - Submit approval decision
///
/// Submits a human approval or denial decision for a pending request.
pub async fn submit_approval_decision(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<ApprovalDecisionRequest>,
) -> Result<Json<ApprovalDecisionResponse>, AppError> {
    info!(
        approval_id = %id,
        approved = request.approved,
        approver = %request.approver_id,
        "Submitting approval decision"
    );

    // Check if pending approval exists
    let pending = state
        .get_pending_approval(id)
        .await
        .ok_or(AppError::ApprovalNotFound)?;

    // Check if already decided
    if state.is_approval_decided(id).await {
        warn!(approval_id = %id, "Approval already decided");
        return Err(AppError::ApprovalAlreadyDecided);
    }

    // Create decision
    let decision = ApprovalDecision {
        approved: request.approved,
        approver_id: request.approver_id.clone(),
        reason: request.reason.clone(),
        decided_at: chrono::Utc::now(),
    };

    // Store decision
    state.submit_approval_decision(id, decision.clone()).await;

    // If approved, we would continue processing here
    // For now, we just record the decision
    // In a production system, this would trigger the processing pipeline to continue

    let message = if request.approved {
        format!(
            "Intent approved by {}. Processing will continue.",
            request.approver_id
        )
    } else {
        format!("Intent denied by {}.", request.approver_id)
    };

    info!(
        approval_id = %id,
        approved = request.approved,
        "Approval decision recorded"
    );

    Ok(Json(ApprovalDecisionResponse {
        id,
        approved: request.approved,
        message,
    }))
}
