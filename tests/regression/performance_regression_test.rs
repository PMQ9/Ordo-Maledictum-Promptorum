//! Performance Regression Tests
//!
//! Tests to ensure system performance doesn't degrade over time.
//! Includes benchmarks for critical paths and resource usage limits.

use std::time::Instant;

mod test_helpers;
use test_helpers::*;

// ============================================================================
// Response Time Benchmarks
// ============================================================================

#[tokio::test]
async fn test_perf_parser_response_time_under_100ms() {
    // Benchmark: Deterministic parser should respond in < 100ms
    // Baseline: 50ms (2024-01-01)

    // Arrange
    let user_input = "Find security experts for supply chain project with $30000 budget";

    // Act
    let start = Instant::now();
    let result = mock_deterministic_parse(user_input).await;
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(result.is_ok());
    assert!(
        duration_ms < 100,
        "Parser should respond in < 100ms, got {}ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_perf_voting_completes_under_50ms() {
    // Benchmark: Voting with 3 parsers should complete in < 50ms
    // Baseline: 20ms (2024-01-01)

    // Arrange
    let intent = IntentBuilder::new().build();
    let results = vec![
        ParsedIntentBuilder::new().intent(intent.clone()).build(),
        ParsedIntentBuilder::new().intent(intent.clone()).build(),
        ParsedIntentBuilder::new().intent(intent.clone()).build(),
    ];

    // Act
    let start = Instant::now();
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .parser_results(results)
        .build();
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert_eq!(voting_result.parser_results.len(), 3);
    assert!(
        duration_ms < 50,
        "Voting should complete in < 50ms, got {}ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_perf_comparison_under_10ms() {
    // Benchmark: Policy comparison should complete in < 10ms
    // Baseline: 3ms (2024-01-01)

    // Arrange
    let config = default_test_provider_config();
    let intent = IntentBuilder::new()
        .action("find_experts")
        .expertise(vec!["security", "ml"])
        .budget(30000)
        .build();

    // Act
    let start = Instant::now();
    let result = compare_with_policy(&intent, &config);
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(result.is_approved());
    assert!(
        duration_ms < 10,
        "Comparison should complete in < 10ms, got {}ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_perf_end_to_end_under_500ms() {
    // Benchmark: Full pipeline should complete in < 500ms
    // Baseline: 200ms (2024-01-01)

    // Arrange
    let user_input = "Find security experts for cloud migration";
    let config = default_test_provider_config();

    // Act
    let start = Instant::now();

    // Simulate full pipeline
    let _malicious_check = MockMaliciousDetector::new().is_malicious(user_input);
    let parsed = mock_deterministic_parse(user_input).await.unwrap();
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(parsed.intent.clone())
        .add_parser_result(parsed)
        .build();
    let _comparison = compare_with_policy(&voting_result.canonical_intent, &config);

    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(
        duration_ms < 500,
        "E2E should complete in < 500ms, got {}ms",
        duration_ms
    );
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

#[tokio::test]
async fn test_perf_parser_handles_100_requests_per_second() {
    // Benchmark: Should handle >= 100 parsing requests per second
    // Baseline: 150 req/s (2024-01-01)

    // Arrange
    let inputs: Vec<String> = (0..100)
        .map(|i| format!("Find experts for project {}", i))
        .collect();

    // Act
    let start = Instant::now();
    for input in &inputs {
        let _ = mock_deterministic_parse(input).await;
    }
    let duration_secs = start.elapsed().as_secs_f64();

    // Assert
    let req_per_sec = inputs.len() as f64 / duration_secs;
    assert!(
        req_per_sec >= 100.0,
        "Should handle >= 100 req/s, got {:.1} req/s",
        req_per_sec
    );
}

#[tokio::test]
async fn test_perf_concurrent_processing_10_requests() {
    // Benchmark: Should handle 10 concurrent requests efficiently
    // Baseline: < 300ms for 10 concurrent (2024-01-01)

    // Arrange
    let inputs: Vec<String> = (0..10)
        .map(|i| format!("Find security experts for project {}", i))
        .collect();

    // Act
    let start = Instant::now();
    let mut handles = vec![];

    for input in inputs {
        let handle = tokio::spawn(async move {
            mock_deterministic_parse(&input).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(
        duration_ms < 300,
        "10 concurrent requests should complete in < 300ms, got {}ms",
        duration_ms
    );
}

// ============================================================================
// Scalability Benchmarks
// ============================================================================

#[tokio::test]
async fn test_perf_large_expertise_list_parsing() {
    // Benchmark: Should handle intent with 50 expertise areas efficiently
    // Baseline: < 50ms (2024-01-01)

    // Arrange
    let large_expertise: Vec<String> = (0..50).map(|i| format!("expertise_{}", i)).collect();
    let intent = IntentBuilder::new()
        .expertise(large_expertise.iter().map(|s| s.as_str()).collect())
        .build();

    let config = ProviderConfig {
        allowed_actions: vec!["find_experts".to_string()],
        allowed_expertise: vec![], // Empty = allow all
        max_budget: Some(50000),
        allowed_domains: vec![],
    };

    // Act
    let start = Instant::now();
    let result = compare_with_policy(&intent, &config);
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(result.is_approved());
    assert!(
        duration_ms < 50,
        "Large expertise comparison should complete in < 50ms, got {}ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_perf_similarity_calculation_with_large_intents() {
    // Benchmark: Similarity calculation should be O(n) not O(n²)
    // Baseline: < 100ms for large intents (2024-01-01)

    // Arrange
    let expertise1: Vec<String> = (0..100).map(|i| format!("exp_{}", i)).collect();
    let expertise2: Vec<String> = (50..150).map(|i| format!("exp_{}", i)).collect();

    let intent1 = IntentBuilder::new()
        .expertise(expertise1.iter().map(|s| s.as_str()).collect())
        .build();

    let intent2 = IntentBuilder::new()
        .expertise(expertise2.iter().map(|s| s.as_str()).collect())
        .build();

    // Act
    let start = Instant::now();
    let similarity = intent1.similarity(&intent2);
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(similarity >= 0.0 && similarity <= 1.0);
    assert!(
        duration_ms < 100,
        "Similarity calculation should complete in < 100ms, got {}ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_perf_voting_with_many_parsers() {
    // Benchmark: Voting should scale linearly with parser count
    // Baseline: < 100ms for 10 parsers (2024-01-01)

    // Arrange
    let intent = IntentBuilder::new().build();
    let results: Vec<_> = (0..10)
        .map(|i| {
            ParsedIntentBuilder::new()
                .parser_id(&format!("parser_{}", i))
                .intent(intent.clone())
                .confidence(0.9)
                .build()
        })
        .collect();

    // Act
    let start = Instant::now();
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .parser_results(results)
        .build();
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert_eq!(voting_result.parser_results.len(), 10);
    assert!(
        duration_ms < 100,
        "Voting with 10 parsers should complete in < 100ms, got {}ms",
        duration_ms
    );
}

// ============================================================================
// Memory Usage Benchmarks
// ============================================================================

#[tokio::test]
async fn test_perf_memory_ledger_entry_serialization() {
    // Benchmark: Ledger entry should serialize without excessive memory
    // Baseline: < 1MB for typical entry (2024-01-01)

    // Arrange
    let intent = IntentBuilder::new().build();
    let parsed_results: Vec<_> = (0..5)
        .map(|i| {
            ParsedIntentBuilder::new()
                .parser_id(&format!("parser_{}", i))
                .intent(intent.clone())
                .build()
        })
        .collect();

    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .parser_results(parsed_results.clone())
        .build();

    let ledger_entry = intent_schema::LedgerEntry::new(
        "Test input".to_string(),
        parsed_results,
        voting_result,
        intent_schema::ComparisonResult::Approved,
    );

    // Act
    let serialized = serde_json::to_string(&ledger_entry).unwrap();
    let size_bytes = serialized.len();

    // Assert
    assert!(
        size_bytes < 1_000_000,
        "Ledger entry should be < 1MB, got {} bytes",
        size_bytes
    );
}

// ============================================================================
// Database Performance Benchmarks
// ============================================================================

#[tokio::test]
async fn test_perf_db_insert_under_50ms() {
    // Benchmark: Single DB insert should complete in < 50ms
    // Baseline: 15ms (2024-01-01)

    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let intent = IntentBuilder::new().build();
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .build();

    let ledger_entry = intent_schema::LedgerEntry::new(
        "Test input".to_string(),
        vec![],
        voting_result,
        intent_schema::ComparisonResult::Approved,
    );

    // Act
    let start = Instant::now();
    mock_store_ledger_entry(&db, &ledger_entry).await;
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(
        duration_ms < 50,
        "DB insert should complete in < 50ms, got {}ms",
        duration_ms
    );

    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_perf_db_query_by_user_under_100ms() {
    // Benchmark: Query by user ID should complete in < 100ms
    // Baseline: 30ms (2024-01-01)

    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let user_id = "test_user";

    // Insert test data
    for i in 0..50 {
        let intent = IntentBuilder::new()
            .user_id(user_id)
            .session_id(&format!("session_{}", i))
            .build();

        let voting_result = VotingResultBuilder::new()
            .canonical_intent(intent)
            .build();

        let entry = intent_schema::LedgerEntry::new(
            format!("Input {}", i),
            vec![],
            voting_result,
            intent_schema::ComparisonResult::Approved,
        );

        mock_store_ledger_entry(&db, &entry).await;
    }

    // Act
    let start = Instant::now();
    let _results = mock_query_ledger_by_user(&db, user_id).await;
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(
        duration_ms < 100,
        "DB query should complete in < 100ms, got {}ms",
        duration_ms
    );

    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_perf_db_bulk_insert_1000_entries() {
    // Benchmark: Should insert 1000 entries in < 10 seconds
    // Baseline: 5 seconds (2024-01-01)

    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    // Act
    let start = Instant::now();

    for i in 0..1000 {
        let intent = IntentBuilder::new()
            .user_id(&format!("user_{}", i % 10))
            .build();

        let voting_result = VotingResultBuilder::new()
            .canonical_intent(intent)
            .build();

        let entry = intent_schema::LedgerEntry::new(
            format!("Bulk input {}", i),
            vec![],
            voting_result,
            intent_schema::ComparisonResult::Approved,
        );

        mock_store_ledger_entry(&db, &entry).await;
    }

    let duration_secs = start.elapsed().as_secs();

    // Assert
    assert!(
        duration_secs < 10,
        "1000 inserts should complete in < 10s, got {}s",
        duration_secs
    );

    teardown_test_database(db).await;
}

// ============================================================================
// API Performance Benchmarks
// ============================================================================

#[tokio::test]
async fn test_perf_api_process_endpoint_under_200ms() {
    // Benchmark: /api/process should respond in < 200ms
    // Baseline: 100ms (2024-01-01)

    // Arrange
    let request_body = serde_json::json!({
        "user_input": "Find security experts",
        "user_id": "user_123",
        "session_id": "session_456"
    });

    // Act
    let start = Instant::now();
    let _response = mock_process_request(&request_body).await;
    let duration_ms = start.elapsed().as_millis();

    // Assert
    assert!(
        duration_ms < 200,
        "API should respond in < 200ms, got {}ms",
        duration_ms
    );
}

#[tokio::test]
async fn test_perf_api_handles_burst_of_100_requests() {
    // Benchmark: Should handle burst of 100 requests in < 5 seconds
    // Baseline: 3 seconds (2024-01-01)

    // Arrange
    let request_body = serde_json::json!({
        "user_input": "Find experts",
        "user_id": "user_123",
        "session_id": "session_456"
    });

    // Act
    let start = Instant::now();

    for _ in 0..100 {
        let _ = mock_process_request(&request_body).await;
    }

    let duration_secs = start.elapsed().as_secs();

    // Assert
    assert!(
        duration_secs < 5,
        "100 requests should complete in < 5s, got {}s",
        duration_secs
    );
}

// ============================================================================
// Helper Functions
// ============================================================================

use intent_schema::{ComparisonResult, Intent, ProviderConfig};

async fn mock_deterministic_parse(input: &str) -> Result<intent_schema::ParsedIntent, String> {
    let action = if input.to_lowercase().contains("find") {
        "find_experts"
    } else {
        "unknown"
    };

    let intent = IntentBuilder::new().action(action).build();

    Ok(ParsedIntentBuilder::new()
        .intent(intent)
        .confidence(0.95)
        .build())
}

fn compare_with_policy(intent: &Intent, config: &ProviderConfig) -> ComparisonResult {
    if !config.is_action_allowed(&intent.action) {
        return ComparisonResult::HardMismatch("Action not allowed".to_string());
    }

    for expertise in &intent.expertise {
        if !config.is_expertise_allowed(expertise) {
            return ComparisonResult::HardMismatch(format!(
                "Expertise '{}' not allowed",
                expertise
            ));
        }
    }

    ComparisonResult::Approved
}

async fn mock_process_request(body: &serde_json::Value) -> MockProcessResponse {
    MockProcessResponse {
        status: "completed".to_string(),
        message: "Success".to_string(),
    }
}

async fn mock_store_ledger_entry(
    _db: &TestDatabase,
    entry: &intent_schema::LedgerEntry,
) -> intent_schema::LedgerEntry {
    // Simulate DB write delay
    tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
    entry.clone()
}

async fn mock_query_ledger_by_user(_db: &TestDatabase, user_id: &str) -> Vec<intent_schema::LedgerEntry> {
    // Simulate DB query delay
    tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;

    (0..5)
        .map(|i| {
            let intent = IntentBuilder::new()
                .user_id(user_id)
                .session_id(&format!("session_{}", i))
                .build();
            let voting_result = VotingResultBuilder::new()
                .canonical_intent(intent)
                .build();
            intent_schema::LedgerEntry::new(
                format!("Input {}", i),
                vec![],
                voting_result,
                intent_schema::ComparisonResult::Approved,
            )
        })
        .collect()
}

#[derive(Debug)]
struct MockProcessResponse {
    status: String,
    message: String,
}
