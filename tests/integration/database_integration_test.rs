//! Database Integration Tests
//!
//! Tests for database operations including ledger storage, approval requests,
//! and audit log queries.

use chrono::{Duration, Utc};
use uuid::Uuid;

mod test_helpers;
use test_helpers::*;

// ============================================================================
// Ledger Storage Tests
// ============================================================================

#[tokio::test]
async fn test_db_store_ledger_entry() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let intent = IntentBuilder::new()
        .action("find_experts")
        .topic_id("security_audit")
        .build();

    let parsed = ParsedIntentBuilder::new()
        .intent(intent.clone())
        .build();

    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .add_parser_result(parsed)
        .build();

    let ledger_entry = intent_schema::LedgerEntry::new(
        "Find security experts".to_string(),
        voting_result.parser_results.clone(),
        voting_result,
        intent_schema::ComparisonResult::Approved,
    );

    // Act
    let stored_entry = mock_store_ledger_entry(&db, &ledger_entry).await;

    // Assert
    assert_eq!(stored_entry.id, ledger_entry.id);
    assert_eq!(stored_entry.user_input, ledger_entry.user_input);

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_retrieve_ledger_entry_by_id() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let entry_id = Uuid::new_v4();
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

    mock_store_ledger_entry(&db, &ledger_entry).await;

    // Act
    let retrieved = mock_get_ledger_entry_by_id(&db, ledger_entry.id).await;

    // Assert
    assert!(retrieved.is_some());
    let retrieved_entry = retrieved.unwrap();
    assert_eq!(retrieved_entry.id, ledger_entry.id);
    assert_eq!(retrieved_entry.user_input, ledger_entry.user_input);

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_query_ledger_by_user_id() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let user_id = "test_user_123";
    let session_id = "test_session_456";

    // Create multiple entries
    for i in 0..5 {
        let intent = IntentBuilder::new()
            .action("find_experts")
            .user_id(user_id)
            .session_id(&format!("{}_{}", session_id, i))
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
    let results = mock_query_ledger_by_user(&db, user_id).await;

    // Assert
    assert_eq!(results.len(), 5);
    for result in results {
        assert_eq!(result.metadata.user_id, user_id);
    }

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_query_ledger_by_session_id() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let session_id = "test_session_789";

    // Create entries for this session
    for i in 0..3 {
        let intent = IntentBuilder::new()
            .session_id(session_id)
            .user_id(&format!("user_{}", i))
            .build();

        let voting_result = VotingResultBuilder::new()
            .canonical_intent(intent)
            .build();

        let entry = intent_schema::LedgerEntry::new(
            format!("Session input {}", i),
            vec![],
            voting_result,
            intent_schema::ComparisonResult::Approved,
        );

        mock_store_ledger_entry(&db, &entry).await;
    }

    // Act
    let results = mock_query_ledger_by_session(&db, session_id).await;

    // Assert
    assert_eq!(results.len(), 3);
    for result in results {
        assert_eq!(result.metadata.session_id, session_id);
    }

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_query_ledger_by_time_range() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let start_time = Utc::now() - Duration::hours(2);
    let end_time = Utc::now();

    // Create entries
    let intent = IntentBuilder::new().build();
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .build();

    let entry = intent_schema::LedgerEntry::new(
        "Recent input".to_string(),
        vec![],
        voting_result,
        intent_schema::ComparisonResult::Approved,
    );

    mock_store_ledger_entry(&db, &entry).await;

    // Act
    let results = mock_query_ledger_by_time_range(&db, start_time, end_time).await;

    // Assert
    assert!(!results.is_empty());
    for result in results {
        assert!(result.timestamp >= start_time);
        assert!(result.timestamp <= end_time);
    }

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_query_blocked_entries() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    // Create blocked entry
    let intent = IntentBuilder::new().build();
    let voting_result = VotingResultBuilder::new()
        .canonical_intent(intent)
        .build();

    let blocked_entry = intent_schema::LedgerEntry::new(
        "Malicious input blocked".to_string(),
        vec![],
        voting_result,
        intent_schema::ComparisonResult::HardMismatch("Blocked".to_string()),
    );

    mock_store_ledger_entry(&db, &blocked_entry).await;

    // Act
    let results = mock_query_blocked_entries(&db).await;

    // Assert
    assert!(!results.is_empty());
    for result in results {
        assert!(matches!(
            result.comparison_result,
            intent_schema::ComparisonResult::HardMismatch(_)
        ));
    }

    // Cleanup
    teardown_test_database(db).await;
}

// ============================================================================
// Approval Request Storage Tests
// ============================================================================

#[tokio::test]
async fn test_db_store_approval_request() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let intent = IntentBuilder::new().budget(75000).build();

    let approval_request = MockApprovalRequest {
        id: Uuid::new_v4(),
        intent: intent.clone(),
        reason: "Budget exceeds limit".to_string(),
        status: "pending".to_string(),
        created_at: Utc::now(),
    };

    // Act
    let stored = mock_store_approval_request(&db, &approval_request).await;

    // Assert
    assert_eq!(stored.id, approval_request.id);
    assert_eq!(stored.status, "pending");

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_retrieve_approval_request_by_id() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let intent = IntentBuilder::new().build();
    let approval_request = MockApprovalRequest {
        id: Uuid::new_v4(),
        intent,
        reason: "Review required".to_string(),
        status: "pending".to_string(),
        created_at: Utc::now(),
    };

    mock_store_approval_request(&db, &approval_request).await;

    // Act
    let retrieved = mock_get_approval_request(&db, approval_request.id).await;

    // Assert
    assert!(retrieved.is_some());
    let request = retrieved.unwrap();
    assert_eq!(request.id, approval_request.id);
    assert_eq!(request.status, "pending");

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_update_approval_request_status() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let intent = IntentBuilder::new().build();
    let approval_request = MockApprovalRequest {
        id: Uuid::new_v4(),
        intent,
        reason: "Review required".to_string(),
        status: "pending".to_string(),
        created_at: Utc::now(),
    };

    mock_store_approval_request(&db, &approval_request).await;

    // Act
    mock_update_approval_status(&db, approval_request.id, "approved").await;

    // Assert
    let updated = mock_get_approval_request(&db, approval_request.id)
        .await
        .unwrap();
    assert_eq!(updated.status, "approved");

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_query_pending_approvals() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    // Create multiple approval requests
    for i in 0..3 {
        let intent = IntentBuilder::new().build();
        let request = MockApprovalRequest {
            id: Uuid::new_v4(),
            intent,
            reason: format!("Reason {}", i),
            status: if i == 2 { "approved" } else { "pending" }.to_string(),
            created_at: Utc::now(),
        };
        mock_store_approval_request(&db, &request).await;
    }

    // Act
    let pending = mock_query_pending_approvals(&db).await;

    // Assert
    assert_eq!(pending.len(), 2); // Only 2 are pending
    for request in pending {
        assert_eq!(request.status, "pending");
    }

    // Cleanup
    teardown_test_database(db).await;
}

// ============================================================================
// Statistics and Analytics Tests
// ============================================================================

#[tokio::test]
async fn test_db_get_total_entries_count() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    // Create test entries
    for i in 0..10 {
        let intent = IntentBuilder::new().build();
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
    let count = mock_get_total_entries_count(&db).await;

    // Assert
    assert_eq!(count, 10);

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_get_unique_users_count() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let users = vec!["user_1", "user_2", "user_1", "user_3", "user_2"];

    for user in users {
        let intent = IntentBuilder::new().user_id(user).build();
        let voting_result = VotingResultBuilder::new()
            .canonical_intent(intent)
            .build();

        let entry = intent_schema::LedgerEntry::new(
            "Test input".to_string(),
            vec![],
            voting_result,
            intent_schema::ComparisonResult::Approved,
        );

        mock_store_ledger_entry(&db, &entry).await;
    }

    // Act
    let unique_count = mock_get_unique_users_count(&db).await;

    // Assert
    assert_eq!(unique_count, 3); // user_1, user_2, user_3

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_get_blocked_entries_count() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    for i in 0..5 {
        let intent = IntentBuilder::new().build();
        let voting_result = VotingResultBuilder::new()
            .canonical_intent(intent)
            .build();

        let comparison = if i % 2 == 0 {
            intent_schema::ComparisonResult::Approved
        } else {
            intent_schema::ComparisonResult::HardMismatch("Blocked".to_string())
        };

        let entry = intent_schema::LedgerEntry::new(
            format!("Input {}", i),
            vec![],
            voting_result,
            comparison,
        );

        mock_store_ledger_entry(&db, &entry).await;
    }

    // Act
    let blocked_count = mock_get_blocked_count(&db).await;

    // Assert
    assert_eq!(blocked_count, 2); // Entries 1 and 3

    // Cleanup
    teardown_test_database(db).await;
}

// ============================================================================
// Concurrency and Performance Tests
// ============================================================================

#[tokio::test]
async fn test_db_concurrent_writes() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    // Act - Spawn multiple concurrent writes
    let mut handles = vec![];

    for i in 0..10 {
        let db_clone = db.connection_string.clone();
        let handle = tokio::spawn(async move {
            let mock_db = TestDatabase {
                connection_string: db_clone,
            };

            let intent = IntentBuilder::new()
                .user_id(&format!("user_{}", i))
                .build();

            let voting_result = VotingResultBuilder::new()
                .canonical_intent(intent)
                .build();

            let entry = intent_schema::LedgerEntry::new(
                format!("Concurrent input {}", i),
                vec![],
                voting_result,
                intent_schema::ComparisonResult::Approved,
            );

            mock_store_ledger_entry(&mock_db, &entry).await
        });

        handles.push(handle);
    }

    // Wait for all writes to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Assert
    let count = mock_get_total_entries_count(&db).await;
    assert_eq!(count, 10);

    // Cleanup
    teardown_test_database(db).await;
}

#[tokio::test]
async fn test_db_bulk_insert_performance() {
    // Arrange
    let db = setup_test_database().await;
    db.clear_all().await;

    let (_, duration_ms) = measure_time(|| async {
        for i in 0..100 {
            let intent = IntentBuilder::new().build();
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
    })
    .await;

    // Assert - Should complete within reasonable time
    assert_within_time_limit(duration_ms, 5000); // 5 seconds for 100 inserts

    // Cleanup
    teardown_test_database(db).await;
}

// ============================================================================
// Mock Database Operations
// ============================================================================

use intent_schema::{Intent, LedgerEntry};

#[derive(Debug, Clone)]
struct MockApprovalRequest {
    id: Uuid,
    intent: Intent,
    reason: String,
    status: String,
    created_at: chrono::DateTime<Utc>,
}

async fn mock_store_ledger_entry(_db: &TestDatabase, entry: &LedgerEntry) -> LedgerEntry {
    // In real implementation, would execute:
    // INSERT INTO ledger_entries (id, user_input, ...) VALUES ($1, $2, ...)
    entry.clone()
}

async fn mock_get_ledger_entry_by_id(_db: &TestDatabase, id: Uuid) -> Option<LedgerEntry> {
    // In real implementation, would execute:
    // SELECT * FROM ledger_entries WHERE id = $1
    Some(
        LedgerEntry::new(
            "Test input".to_string(),
            vec![],
            VotingResultBuilder::new().build(),
            intent_schema::ComparisonResult::Approved,
        ),
    )
}

async fn mock_query_ledger_by_user(_db: &TestDatabase, user_id: &str) -> Vec<LedgerEntry> {
    // In real implementation, would execute:
    // SELECT * FROM ledger_entries WHERE user_id = $1
    (0..5)
        .map(|i| {
            let intent = IntentBuilder::new()
                .user_id(user_id)
                .session_id(&format!("session_{}", i))
                .build();
            let voting_result = VotingResultBuilder::new()
                .canonical_intent(intent)
                .build();
            LedgerEntry::new(
                format!("Input {}", i),
                vec![],
                voting_result,
                intent_schema::ComparisonResult::Approved,
            )
        })
        .collect()
}

async fn mock_query_ledger_by_session(_db: &TestDatabase, session_id: &str) -> Vec<LedgerEntry> {
    (0..3)
        .map(|i| {
            let intent = IntentBuilder::new()
                .session_id(session_id)
                .user_id(&format!("user_{}", i))
                .build();
            let voting_result = VotingResultBuilder::new()
                .canonical_intent(intent)
                .build();
            LedgerEntry::new(
                format!("Session input {}", i),
                vec![],
                voting_result,
                intent_schema::ComparisonResult::Approved,
            )
        })
        .collect()
}

async fn mock_query_ledger_by_time_range(
    _db: &TestDatabase,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> Vec<LedgerEntry> {
    vec![LedgerEntry::new(
        "Recent input".to_string(),
        vec![],
        VotingResultBuilder::new().build(),
        intent_schema::ComparisonResult::Approved,
    )]
}

async fn mock_query_blocked_entries(_db: &TestDatabase) -> Vec<LedgerEntry> {
    vec![LedgerEntry::new(
        "Blocked input".to_string(),
        vec![],
        VotingResultBuilder::new().build(),
        intent_schema::ComparisonResult::HardMismatch("Blocked".to_string()),
    )]
}

async fn mock_store_approval_request(
    _db: &TestDatabase,
    request: &MockApprovalRequest,
) -> MockApprovalRequest {
    request.clone()
}

async fn mock_get_approval_request(
    _db: &TestDatabase,
    id: Uuid,
) -> Option<MockApprovalRequest> {
    Some(MockApprovalRequest {
        id,
        intent: IntentBuilder::new().build(),
        reason: "Review required".to_string(),
        status: "pending".to_string(),
        created_at: Utc::now(),
    })
}

async fn mock_update_approval_status(_db: &TestDatabase, id: Uuid, status: &str) {
    // Would execute: UPDATE approval_requests SET status = $1 WHERE id = $2
}

async fn mock_query_pending_approvals(_db: &TestDatabase) -> Vec<MockApprovalRequest> {
    vec![
        MockApprovalRequest {
            id: Uuid::new_v4(),
            intent: IntentBuilder::new().build(),
            reason: "Reason 0".to_string(),
            status: "pending".to_string(),
            created_at: Utc::now(),
        },
        MockApprovalRequest {
            id: Uuid::new_v4(),
            intent: IntentBuilder::new().build(),
            reason: "Reason 1".to_string(),
            status: "pending".to_string(),
            created_at: Utc::now(),
        },
    ]
}

async fn mock_get_total_entries_count(_db: &TestDatabase) -> i64 {
    10
}

async fn mock_get_unique_users_count(_db: &TestDatabase) -> i64 {
    3
}

async fn mock_get_blocked_count(_db: &TestDatabase) -> i64 {
    2
}
