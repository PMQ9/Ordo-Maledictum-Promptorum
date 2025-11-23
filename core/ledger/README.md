# Intent Ledger

An append-only, immutable ledger for tracking all intent processing flows in the Intent Segregation Cybersecurity Architecture.

## Overview

The Intent Ledger is a PostgreSQL-backed audit trail that stores complete records of every request processed through the system. Once written, entries cannot be modified or deleted, ensuring a complete and tamper-proof audit history.

## Features

- **Append-Only**: Entries can only be inserted, never updated or deleted
- **Immutable**: Database triggers enforce immutability at the PostgreSQL level
- **Comprehensive Tracking**: Captures all phases of intent processing:
  - User input and malicious detection
  - Parser ensemble results and voting
  - Comparator decisions
  - Privilege elevation events
  - Trusted intent generation
  - Processing outputs
- **Audit Queries**: Pre-built queries for common audit scenarios
- **Statistics**: Aggregate statistics for monitoring and analysis
- **Indexed**: Optimized indexes for fast queries by user, session, time range, etc.

## Database Schema

### Main Table: `ledger_entries`

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Unique identifier |
| `session_id` | VARCHAR | Session grouping |
| `user_id` | VARCHAR | User identifier |
| `timestamp` | TIMESTAMPTZ | Entry creation time |
| `user_input` | TEXT | Original user input |
| `user_input_hash` | VARCHAR | SHA-256 hash for duplicate detection |
| `malicious_score` | DOUBLE PRECISION | Malicious detection score (0-1) |
| `malicious_blocked` | BOOLEAN | Whether input was blocked |
| `voting_result` | JSONB | Parser voting results |
| `comparison_result` | JSONB | Comparator decision |
| `elevation_event` | JSONB | Privilege elevation request (optional) |
| `trusted_intent` | JSONB | Validated intent (optional) |
| `processing_output` | JSONB | Execution results (optional) |
| `ip_address` | VARCHAR | Client IP address |
| `user_agent` | TEXT | Client user agent |

### Audit Views

The schema includes several pre-built views for common audit queries:

- `v_blocked_entries`: All malicious entries that were blocked
- `v_elevation_events`: All privilege elevation requests
- `v_processing_stats`: Success/failure statistics by time
- `v_parser_agreement`: Parser agreement analysis

## Usage

### Setup

1. Add to your `Cargo.toml`:
```toml
[dependencies]
intent-ledger = { path = "core/ledger" }
```

2. Set up the database connection:
```rust
use intent_ledger::{IntentLedger, LedgerEntry};

#[tokio::main]
async fn main() -> Result<()> {
    // Create ledger with connection pool
    let ledger = IntentLedger::new(
        "postgresql://user:pass@localhost/intent_db",
        10  // max connections
    ).await?;

    // Run migrations on first startup
    ledger.run_migrations().await?;

    Ok(())
}
```

### Appending Entries

```rust
use intent_ledger::{LedgerEntry, VotingResult, ComparisonResult};

// Create a new entry
let mut entry = LedgerEntry::new(
    "session_abc123".to_string(),
    "user_456".to_string(),
    "Find me experts in machine learning".to_string(),
);

// Fill in processing phases as they complete
entry.voting_result = VotingResult {
    agreement_level: AgreementLevel::FullAgreement,
    confidence: 0.95,
    canonical_intent: Some(intent_json),
    parser_results: vec![parser1_result, parser2_result],
};

entry.comparison_result = ComparisonResult {
    decision: ComparatorDecision::Approved,
    mismatches: vec![],
    requires_elevation: false,
    explanation: "All checks passed".to_string(),
};

// Append to ledger (returns UUID)
let entry_id = ledger.append(entry).await?;
```

### Querying Entries

```rust
// Query by user
let user_entries = ledger.query_by_user("user_456", Some(50)).await?;

// Query by session
let session_entries = ledger.query_by_session("session_abc123").await?;

// Query by ID
let entry = ledger.query_by_id(entry_id).await?;

// Query by time range
use chrono::{Utc, Duration};
let start = Utc::now() - Duration::days(7);
let end = Utc::now();
let recent = ledger.query_by_time_range(start, end, Some(1000)).await?;

// Query elevation events
let elevations = ledger.query_elevation_events(Some(100)).await?;

// Query blocked entries
let blocked = ledger.query_blocked_entries(Some(100)).await?;

// Get statistics
let stats = ledger.get_stats().await?;
println!("Total entries: {}", stats.total_entries);
println!("Blocked: {}", stats.blocked_entries);
```

## Append-Only Enforcement

The ledger enforces immutability at multiple levels:

1. **API Level**: The `IntentLedger` struct only provides `append()` method, no update/delete
2. **Database Level**: PostgreSQL triggers reject any UPDATE or DELETE operations:

```sql
-- Attempting to update will fail with:
ERROR: ledger_entries is append-only: UPDATE operations are not allowed
HINT: The ledger is immutable. Only INSERT operations are permitted.
```

## Audit Queries

### Direct SQL Access

For advanced audit queries, you can query the views directly:

```sql
-- Recent blocked entries
SELECT * FROM v_blocked_entries
WHERE timestamp > NOW() - INTERVAL '24 hours';

-- Pending elevation requests
SELECT * FROM v_elevation_events
WHERE elevation_status = 'pending';

-- Processing success rate by hour
SELECT * FROM v_processing_stats
WHERE time_bucket > NOW() - INTERVAL '7 days';

-- Parser agreement trends
SELECT * FROM v_parser_agreement
WHERE time_bucket > NOW() - INTERVAL '24 hours';
```

### Using the stats function

```sql
-- Get overall statistics
SELECT * FROM get_ledger_stats();
```

## Performance Considerations

- Indexes are created for common query patterns (user_id, session_id, timestamp)
- GIN indexes on JSONB columns enable efficient JSON queries
- Partial indexes optimize queries for recent data and filtered conditions
- Consider partitioning by timestamp for very large datasets (100M+ entries)

## Security

- All user input is hashed (SHA-256) for duplicate detection
- Original user input is preserved for forensic analysis
- IP addresses and user agents are captured for security auditing
- The append-only nature ensures tampering is immediately evident

## Migration

The initial migration (`20250101000001_init.sql`) includes:

- Table creation with proper constraints
- All necessary indexes
- Trigger functions for append-only enforcement
- Audit views
- Helper functions
- Comprehensive comments

To run migrations:
```rust
ledger.run_migrations().await?;
```

## Example: Complete Flow

```rust
use intent_ledger::*;

async fn process_user_request(
    ledger: &IntentLedger,
    user_id: String,
    session_id: String,
    input: String,
) -> Result<Uuid> {
    // Create entry
    let mut entry = LedgerEntry::new(session_id, user_id, input);

    // Phase 1: Malicious detection
    entry.malicious_score = Some(0.05);
    entry.malicious_blocked = false;

    // Phase 2: Parsing and voting
    // ... (parsers run)
    entry.voting_result = voting_result;

    // Phase 3: Comparison
    // ... (comparator runs)
    entry.comparison_result = comparison_result;

    // Phase 4: Elevation (if needed)
    if comparison_result.requires_elevation {
        entry.elevation_event = Some(elevation_event);
    }

    // Phase 5: Trusted intent
    entry.trusted_intent = Some(trusted_intent_json);

    // Phase 6: Processing
    entry.processing_output = Some(processing_output);

    // Append complete entry
    ledger.append(entry).await
}
```

## Testing

Unit tests are included in `src/lib.rs`. Integration tests require a running PostgreSQL database:

```bash
# Start test database
docker run -d -p 5432:5432 \
  -e POSTGRES_DB=intent_ledger_test \
  -e POSTGRES_PASSWORD=test \
  postgres:15

# Run tests
DATABASE_URL=postgresql://postgres:test@localhost/intent_ledger_test \
  cargo test
```

## License

Part of the Intent-Segregation-Cybersecurity-Architecture-for-AI project.
