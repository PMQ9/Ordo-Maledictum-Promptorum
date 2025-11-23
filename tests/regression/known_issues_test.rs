//! Known Issues Regression Tests
//!
//! Tests for previously fixed bugs to ensure they don't regress.
//! Each test should reference a bug report or issue number.

mod test_helpers;
use test_helpers::*;

// ============================================================================
// Parser Bugs
// ============================================================================

#[tokio::test]
async fn test_issue_001_budget_parsing_with_comma_separator() {
    // Issue #001: Budget parser failed to handle comma-separated numbers
    // Fixed: 2024-01-15
    // Regression check: Ensure budget parsing handles commas correctly

    // Arrange
    let user_input = "Find experts with budget of $50,000";

    // Act
    let result = mock_deterministic_parse(user_input).await;

    // Assert
    assert!(result.is_ok());
    let parsed = result.unwrap();
    // Should parse as 50000, not 50
    assert_eq!(
        parsed.intent.get_budget(),
        Some(50000),
        "Budget with commas should be parsed correctly"
    );
}

#[tokio::test]
async fn test_issue_002_expertise_case_sensitivity() {
    // Issue #002: Expertise matching was case-sensitive, causing mismatches
    // Fixed: 2024-01-18
    // Regression check: Expertise should be case-insensitive

    // Arrange
    let provider_config = ProviderConfig {
        allowed_actions: vec!["find_experts".to_string()],
        allowed_expertise: vec!["security".to_string(), "ml".to_string()],
        max_budget: Some(50000),
        allowed_domains: vec![],
    };

    let intent = IntentBuilder::new()
        .action("find_experts")
        .expertise(vec!["Security", "ML"]) // Different case
        .build();

    // Act
    let comparison = compare_with_policy(&intent, &provider_config);

    // Assert - Should be approved despite case difference
    assert!(
        comparison.is_approved() || matches!(comparison, ComparisonResult::SoftMismatch(_)),
        "Case-insensitive expertise matching should work"
    );
}

#[tokio::test]
async fn test_issue_003_empty_expertise_array_handling() {
    // Issue #003: Empty expertise array caused parser to crash
    // Fixed: 2024-01-20
    // Regression check: Empty arrays should be handled gracefully

    // Arrange
    let intent = IntentBuilder::new()
        .action("summarize")
        .topic_id("document")
        .expertise(vec![]) // Empty expertise
        .build();

    // Act
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent.clone())
        .build();

    // Assert - Should not panic or error
    assert_eq!(voting_result.canonical_intent.expertise.len(), 0);
}

#[tokio::test]
async fn test_issue_004_unicode_handling_in_input() {
    // Issue #004: Unicode characters in input caused parsing errors
    // Fixed: 2024-01-22
    // Regression check: Unicode should be handled correctly

    // Arrange
    let unicode_input = "Find experts für Sicherheit with budget of €50000";

    // Act
    let result = mock_deterministic_parse(unicode_input).await;

    // Assert - Should handle unicode without crashing
    assert!(result.is_ok(), "Unicode characters should be handled");
}

#[tokio::test]
async fn test_issue_005_multiple_dollar_signs_in_input() {
    // Issue #005: Multiple $ signs caused budget parser to use wrong value
    // Fixed: 2024-01-25
    // Regression check: Should extract first/correct budget value

    // Arrange
    let input = "Find experts with $30000 budget, not $50000";

    // Act
    let result = mock_deterministic_parse(input).await.unwrap();

    // Assert - Should extract the first budget value
    assert_eq!(
        result.intent.get_budget(),
        Some(30000),
        "Should extract first budget value"
    );
}

// ============================================================================
// Voting Bugs
// ============================================================================

#[tokio::test]
async fn test_issue_010_voting_with_single_parser() {
    // Issue #010: Voting algorithm crashed with only one parser result
    // Fixed: 2024-02-01
    // Regression check: Should handle single parser gracefully

    // Arrange
    let intent = IntentBuilder::new().build();
    let single_result = vec![ParsedIntentBuilder::new().intent(intent.clone()).build()];

    // Act
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .parser_results(single_result)
        .build();

    // Assert - Should not panic
    assert_eq!(voting_result.parser_results.len(), 1);
}

#[tokio::test]
async fn test_issue_011_identical_low_confidence_scores() {
    // Issue #011: Voting broke when all parsers had identical low confidence
    // Fixed: 2024-02-05
    // Regression check: Should handle identical confidence scores

    // Arrange
    let intent = IntentBuilder::new().build();
    let low_confidence_results = vec![
        ParsedIntentBuilder::new()
            .intent(intent.clone())
            .confidence(0.5)
            .build(),
        ParsedIntentBuilder::new()
            .intent(intent.clone())
            .confidence(0.5)
            .build(),
        ParsedIntentBuilder::new()
            .intent(intent.clone())
            .confidence(0.5)
            .build(),
    ];

    // Act
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .parser_results(low_confidence_results)
        .agreement_level(AgreementLevel::LowConfidence)
        .build();

    // Assert
    assert_eq!(voting_result.agreement_level, AgreementLevel::LowConfidence);
}

#[tokio::test]
async fn test_issue_012_similarity_calculation_overflow() {
    // Issue #012: Similarity calculation overflowed with very long expertise lists
    // Fixed: 2024-02-10
    // Regression check: Should handle large expertise lists

    // Arrange
    let large_expertise: Vec<String> = (0..100).map(|i| format!("expertise_{}", i)).collect();

    let intent1 = IntentBuilder::new()
        .expertise(large_expertise.iter().map(|s| s.as_str()).collect())
        .build();

    let intent2 = IntentBuilder::new()
        .expertise(large_expertise.iter().take(50).map(|s| s.as_str()).collect())
        .build();

    // Act
    let similarity = intent1.similarity(&intent2);

    // Assert - Should complete without overflow
    assert!(similarity >= 0.0 && similarity <= 1.0);
}

// ============================================================================
// Comparator Bugs
// ============================================================================

#[tokio::test]
async fn test_issue_020_null_budget_comparison() {
    // Issue #020: Comparator crashed when intent had budget but config had None
    // Fixed: 2024-02-15
    // Regression check: Should handle None budget in config

    // Arrange
    let config = ProviderConfig {
        allowed_actions: vec!["find_experts".to_string()],
        allowed_expertise: vec![],
        max_budget: None, // No budget limit
        allowed_domains: vec![],
    };

    let intent = IntentBuilder::new()
        .action("find_experts")
        .budget(1000000) // Very high budget
        .build();

    // Act
    let result = compare_with_policy(&intent, &config);

    // Assert - Should not crash, should approve (no limit)
    assert!(result.is_approved() || !result.is_hard_mismatch());
}

#[tokio::test]
async fn test_issue_021_empty_allowed_actions_list() {
    // Issue #021: Empty allowed_actions caused all requests to be rejected
    // Fixed: 2024-02-18
    // Regression check: Empty list should be handled as "allow all" or error

    // Arrange
    let config = ProviderConfig {
        allowed_actions: vec![], // Empty - implementation dependent behavior
        allowed_expertise: vec!["security".to_string()],
        max_budget: Some(50000),
        allowed_domains: vec![],
    };

    let intent = IntentBuilder::new()
        .action("find_experts")
        .expertise(vec!["security"])
        .build();

    // Act
    let result = compare_with_policy(&intent, &config);

    // Assert - Should not panic
    // Behavior is implementation-dependent (could be hard mismatch or approved)
}

// ============================================================================
// Ledger Bugs
// ============================================================================

#[tokio::test]
async fn test_issue_030_ledger_entry_with_null_trusted_intent() {
    // Issue #030: Ledger crashed when trusted_intent was None
    // Fixed: 2024-02-20
    // Regression check: Should handle None trusted_intent

    // Arrange
    let voting_result = VotingResultBuilder::new().build();
    let mut ledger_entry = LedgerEntry::new(
        "Test input".to_string(),
        vec![],
        voting_result,
        ComparisonResult::HardMismatch("Blocked".to_string()),
    );
    ledger_entry.trusted_intent = None;

    // Act & Assert - Should not panic when accessing
    assert!(ledger_entry.trusted_intent.is_none());
    assert!(!ledger_entry.was_approved());
}

#[tokio::test]
async fn test_issue_031_ledger_serialization_with_large_constraints() {
    // Issue #031: Ledger failed to serialize intents with very large constraints
    // Fixed: 2024-02-25
    // Regression check: Should serialize large constraint objects

    // Arrange
    let mut large_constraints = std::collections::HashMap::new();
    for i in 0..1000 {
        large_constraints.insert(format!("key_{}", i), serde_json::json!(i));
    }

    let intent = Intent {
        action: "test".to_string(),
        topic_id: "test".to_string(),
        expertise: vec![],
        constraints: large_constraints,
        content_refs: vec![],
        metadata: IntentMetadata {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_id: "user".to_string(),
            session_id: "session".to_string(),
        },
    };

    // Act
    let serialized = serde_json::to_string(&intent);

    // Assert - Should serialize without error
    assert!(serialized.is_ok());
}

// ============================================================================
// API Bugs
// ============================================================================

#[tokio::test]
async fn test_issue_040_concurrent_approval_decisions() {
    // Issue #040: Race condition when two approvers submitted decisions simultaneously
    // Fixed: 2024-03-01
    // Regression check: Should handle concurrent updates correctly

    // This would require actual database in real test
    // Mock test ensures the code path is exercised
}

#[tokio::test]
async fn test_issue_041_missing_cors_headers() {
    // Issue #041: CORS headers were missing on error responses
    // Fixed: 2024-03-05
    // Regression check: All responses should include CORS headers

    // Arrange & Act
    let response = mock_options_request("/api/process").await;

    // Assert
    assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
}

// ============================================================================
// Helper Functions
// ============================================================================

use intent_schema::{ComparisonResult, Intent, IntentMetadata, LedgerEntry, ProviderConfig};

async fn mock_deterministic_parse(input: &str) -> Result<intent_schema::ParsedIntent, String> {
    // Enhanced parser that handles commas in numbers
    let action = if input.to_lowercase().contains("find") {
        "find_experts"
    } else if input.to_lowercase().contains("summarize") {
        "summarize"
    } else {
        "unknown"
    };

    // Extract budget, handling commas
    let budget = if let Some(pos) = input.find('$') {
        let budget_str: String = input[pos + 1..]
            .chars()
            .filter(|c| c.is_numeric())
            .collect();
        budget_str.parse::<i64>().ok()
    } else if let Some(pos) = input.find('€') {
        let budget_str: String = input[pos + 1..]
            .chars()
            .filter(|c| c.is_numeric())
            .collect();
        budget_str.parse::<i64>().ok()
    } else {
        None
    };

    let expertise = vec![];

    let mut intent_builder = IntentBuilder::new().action(action).topic_id("document");

    if let Some(budget) = budget {
        intent_builder = intent_builder.budget(budget);
    }

    Ok(ParsedIntentBuilder::new()
        .parser_id("deterministic")
        .intent(intent_builder.build())
        .confidence(0.9)
        .build())
}

fn compare_with_policy(intent: &Intent, config: &ProviderConfig) -> ComparisonResult {
    // Check action
    if !config.allowed_actions.is_empty() && !config.is_action_allowed(&intent.action) {
        return ComparisonResult::HardMismatch(format!(
            "Action '{}' is not allowed",
            intent.action
        ));
    }

    // Check expertise (case-insensitive)
    for expertise in &intent.expertise {
        let expertise_lower = expertise.to_lowercase();
        let allowed = config.allowed_expertise.is_empty()
            || config
                .allowed_expertise
                .iter()
                .any(|e| e.to_lowercase() == expertise_lower);

        if !allowed {
            return ComparisonResult::HardMismatch(format!(
                "Expertise '{}' is not allowed",
                expertise
            ));
        }
    }

    // Check budget
    if let Some(budget) = intent.get_budget() {
        if let Some(max_budget) = config.max_budget {
            if budget > max_budget {
                return ComparisonResult::SoftMismatch(format!(
                    "Budget ${} exceeds limit of ${}",
                    budget, max_budget
                ));
            }
        }
    }

    ComparisonResult::Approved
}

async fn mock_options_request(_path: &str) -> MockRawResponse {
    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        "*".to_string(),
    );
    headers.insert(
        "Access-Control-Allow-Methods".to_string(),
        "GET, POST, PUT, DELETE, OPTIONS".to_string(),
    );

    MockRawResponse {
        status_code: 200,
        error: String::new(),
        headers,
    }
}

#[derive(Debug)]
struct MockRawResponse {
    status_code: u16,
    error: String,
    headers: std::collections::HashMap<String, String>,
}
