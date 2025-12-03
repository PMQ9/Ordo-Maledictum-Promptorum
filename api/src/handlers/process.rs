//! Main processing pipeline handler

use crate::error::AppError;
use crate::state::{AppState, PendingApproval};
use crate::types::*;
use axum::{extract::State, Json};
use intent_ledger::{
    AgreementLevel as LedgerAgreementLevel, ComparatorDecision,
    ComparisonResult as LedgerComparisonResult, ElevationEvent, ElevationStatus, LedgerEntry,
    ProcessingOutput,
};
use intent_schema::AgreementLevel;
use malicious_detector::DetectionResult;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// POST /api/process - Process user input through the full pipeline
///
/// This endpoint orchestrates all modules:
/// 1. Malicious input detection
/// 2. Parser ensemble
/// 3. Voting module
/// 4. Intent comparator
/// 5. Human approval (if needed)
/// 6. Trusted intent generator
/// 7. Processing engine
/// 8. Ledger recording
pub async fn process_input(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProcessRequest>,
) -> Result<Json<ProcessResponse>, AppError> {
    let start_time = std::time::Instant::now();
    let request_id = Uuid::new_v4();

    info!(
        request_id = %request_id,
        user_id = %request.user_id,
        session_id = %request.session_id,
        "Processing user input"
    );

    // Initialize pipeline info
    let mut pipeline_info = PipelineInfo {
        malicious_detection: None,
        parser_results: None,
        voting_result: None,
        comparison_result: None,
    };

    // Initialize ledger entry
    let mut ledger_entry = LedgerEntry::new(
        request.session_id.clone(),
        request.user_id.clone(),
        request.user_input.clone(),
    );

    // Step 1: Malicious input detection
    info!(request_id = %request_id, "Step 1: Malicious input detection");
    let detection_result = state.detector.detect(&request.user_input);

    match detection_result {
        DetectionResult::Blocked(reason) => {
            warn!(request_id = %request_id, reason = %reason, "Input blocked as malicious");

            pipeline_info.malicious_detection = Some(MaliciousDetectionInfo {
                blocked: true,
                reason: Some(reason.clone()),
            });

            ledger_entry.malicious_blocked = true;
            ledger_entry.comparison_result = LedgerComparisonResult {
                decision: ComparatorDecision::Blocked,
                mismatches: vec![reason.clone()],
                requires_elevation: false,
                explanation: format!("Input blocked by malicious detector: {}", reason),
            };

            // Record in ledger
            state.ledger.append(ledger_entry).await?;

            return Ok(Json(ProcessResponse {
                request_id,
                status: ProcessStatus::Blocked,
                trusted_intent: None,
                result: None,
                message: format!("Input blocked: {}", reason),
                pipeline_info,
            }));
        }
        DetectionResult::Clean => {
            info!(request_id = %request_id, "Input passed malicious detection");
            pipeline_info.malicious_detection = Some(MaliciousDetectionInfo {
                blocked: false,
                reason: None,
            });
        }
    }

    // Step 2: Parser ensemble
    info!(request_id = %request_id, "Step 2: Running parser ensemble");
    let ensemble_result = state
        .parser_ensemble
        .parse_all(&request.user_input, &request.user_id, &request.session_id)
        .await;

    if ensemble_result.success_count == 0 {
        error!(request_id = %request_id, "All parsers failed");
        return Err(AppError::ParserError("All parsers failed".to_string()));
    }

    let parser_results = ensemble_result.results;

    pipeline_info.parser_results = Some(
        parser_results
            .iter()
            .map(|pr| ParserResultInfo {
                parser_id: pr.parser_id.clone(),
                success: true,
                confidence: Some(pr.confidence),
            })
            .collect(),
    );

    // Store parser results in ledger
    ledger_entry.voting_result.parser_results = parser_results
        .iter()
        .map(|pr| serde_json::to_value(&pr.intent).unwrap())
        .collect();

    // Step 3: Voting module
    info!(request_id = %request_id, "Step 3: Running voting module");
    let voting_result = match state.voting.vote(parser_results.clone(), None).await {
        Ok(result) => result,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Voting failed");
            return Err(AppError::VotingError(e.to_string()));
        }
    };

    let requires_human_review = voting_result.has_conflict();
    let average_confidence = voting_result.average_confidence();

    pipeline_info.voting_result = Some(VotingResultInfo {
        confidence_level: format!("{:?}", voting_result.agreement_level),
        average_similarity: average_confidence as f64,
        requires_human_review,
        explanation: format!(
            "{:?} agreement with {} parsers",
            voting_result.agreement_level,
            voting_result.parser_results.len()
        ),
    });

    // Store voting result in ledger
    ledger_entry.voting_result.agreement_level = match voting_result.agreement_level {
        AgreementLevel::HighConfidence => LedgerAgreementLevel::FullAgreement,
        AgreementLevel::LowConfidence => LedgerAgreementLevel::MinorDiscrepancy,
        AgreementLevel::Conflict => LedgerAgreementLevel::MajorDiscrepancy,
    };
    ledger_entry.voting_result.confidence = average_confidence as f64;
    ledger_entry.voting_result.canonical_intent =
        Some(serde_json::to_value(&voting_result.canonical_intent).unwrap());

    let canonical_intent = voting_result.canonical_intent;

    // Step 4: Intent comparator
    info!(request_id = %request_id, "Step 4: Running intent comparator");
    let comparison_result = match state
        .comparator
        .compare(&canonical_intent, &state.provider_config)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Comparison failed");
            return Err(AppError::ComparisonError(e.to_string()));
        }
    };

    let comparison_info = ComparisonResultInfo {
        result: if comparison_result.is_approved() {
            "approved".to_string()
        } else if comparison_result.is_soft_mismatch() {
            "soft_mismatch".to_string()
        } else {
            "hard_mismatch".to_string()
        },
        message: comparison_result.message().to_string(),
        reasons: comparison_result
            .reasons()
            .iter()
            .map(|r| r.description.clone())
            .collect(),
    };

    pipeline_info.comparison_result = Some(comparison_info);

    // Store comparison result in ledger
    ledger_entry.comparison_result = LedgerComparisonResult {
        decision: if comparison_result.is_approved() {
            ComparatorDecision::Approved
        } else if comparison_result.is_soft_mismatch() {
            ComparatorDecision::SoftMismatch
        } else {
            ComparatorDecision::HardMismatch
        },
        mismatches: comparison_result
            .reasons()
            .iter()
            .map(|r| r.description.clone())
            .collect(),
        requires_elevation: comparison_result.is_hard_mismatch() || requires_human_review,
        explanation: comparison_result.message().to_string(),
    };

    // Step 5: Check if human approval is needed
    let requires_approval = requires_human_review
        || comparison_result.is_hard_mismatch()
        || state.provider_config.require_human_approval;

    if requires_approval {
        info!(request_id = %request_id, "Human approval required");

        // Create pending approval
        let approval = PendingApproval {
            id: request_id,
            user_id: request.user_id.clone(),
            session_id: request.session_id.clone(),
            intent: canonical_intent.clone(),
            reason: if requires_human_review {
                format!(
                    "Parser conflict: {:?} agreement",
                    voting_result.agreement_level
                )
            } else {
                format!("Policy mismatch: {}", comparison_result.message())
            },
            created_at: chrono::Utc::now(),
        };

        state.add_pending_approval(approval.clone()).await;

        // Record elevation event in ledger
        ledger_entry.elevation_event = Some(ElevationEvent {
            requested_at: chrono::Utc::now(),
            approved_by: None,
            approved_at: None,
            status: ElevationStatus::Pending,
            reason: approval.reason.clone(),
        });

        // Save to ledger
        state.ledger.append(ledger_entry).await?;

        return Ok(Json(ProcessResponse {
            request_id,
            status: ProcessStatus::PendingApproval,
            trusted_intent: None,
            result: None,
            message: format!(
                "Request requires human approval. Use GET /api/approvals/{} to check status.",
                request_id
            ),
            pipeline_info,
        }));
    }

    // Check if intent is denied
    if comparison_result.is_hard_mismatch() && !requires_approval {
        warn!(request_id = %request_id, "Intent denied by policy");

        state.ledger.append(ledger_entry).await?;

        return Ok(Json(ProcessResponse {
            request_id,
            status: ProcessStatus::Denied,
            trusted_intent: None,
            result: None,
            message: format!("Intent denied: {}", comparison_result.message()),
            pipeline_info,
        }));
    }

    // Step 6: Generate trusted intent
    info!(request_id = %request_id, "Step 6: Generating trusted intent");
    let trusted_intent = canonical_intent.clone();
    ledger_entry.trusted_intent = Some(serde_json::to_value(&trusted_intent).unwrap());

    // Step 7: Execute through processing engine
    info!(request_id = %request_id, "Step 7: Executing through processing engine");
    let execution_start = std::time::Instant::now();

    let processing_result = match state.engine.execute(&trusted_intent).await {
        Ok(result) => result,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Processing failed");

            ledger_entry.processing_output = Some(ProcessingOutput {
                success: false,
                result: None,
                error: Some(e.to_string()),
                execution_time_ms: execution_start.elapsed().as_millis() as u64,
            });

            state.ledger.append(ledger_entry).await?;

            return Err(AppError::ProcessingError(e.to_string()));
        }
    };

    ledger_entry.processing_output = Some(ProcessingOutput {
        success: true,
        result: Some(serde_json::to_value(&processing_result).unwrap()),
        error: None,
        execution_time_ms: execution_start.elapsed().as_millis() as u64,
    });

    // Step 8: Record in ledger
    state.ledger.append(ledger_entry).await?;

    let total_time = start_time.elapsed();
    info!(
        request_id = %request_id,
        duration_ms = total_time.as_millis(),
        "Processing completed successfully"
    );

    Ok(Json(ProcessResponse {
        request_id,
        status: ProcessStatus::Completed,
        trusted_intent: Some(trusted_intent),
        result: Some(serde_json::to_value(&processing_result).unwrap()),
        message: "Intent processed successfully".to_string(),
        pipeline_info,
    }))
}
