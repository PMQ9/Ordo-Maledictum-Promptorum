use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

pub mod cache_helper;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Entry not found: {0}")]
    NotFound(Uuid),

    #[error("Invalid query parameters: {0}")]
    InvalidQuery(String),
}

pub type Result<T> = std::result::Result<T, LedgerError>;

/// Decision made by the comparator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparatorDecision {
    Approved,
    SoftMismatch,
    HardMismatch,
    Blocked,
}

/// Level of agreement between parsers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgreementLevel {
    FullAgreement,
    MinorDiscrepancy,
    MajorDiscrepancy,
}

/// Status of privilege elevation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ElevationStatus {
    Pending,
    Approved,
    Denied,
    Timeout,
}

/// Voting result from parser ensemble
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    pub agreement_level: AgreementLevel,
    pub confidence: f64,
    pub canonical_intent: Option<serde_json::Value>,
    pub parser_results: Vec<serde_json::Value>,
}

/// Comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub decision: ComparatorDecision,
    pub mismatches: Vec<String>,
    pub requires_elevation: bool,
    pub explanation: String,
}

/// Privilege elevation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationEvent {
    pub requested_at: DateTime<Utc>,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub status: ElevationStatus,
    pub reason: String,
}

/// Processing output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOutput {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Complete ledger entry representing one complete flow through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,

    // User input
    pub user_input: String,
    pub user_input_hash: String,

    // Malicious detection (optional)
    pub malicious_score: Option<f64>,
    pub malicious_blocked: bool,

    // Parsing phase
    pub voting_result: VotingResult,

    // Comparison phase
    pub comparison_result: ComparisonResult,

    // Elevation (if needed)
    pub elevation_event: Option<ElevationEvent>,

    // Trusted intent
    pub trusted_intent: Option<serde_json::Value>,

    // Processing phase
    pub processing_output: Option<ProcessingOutput>,

    // Metadata
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl LedgerEntry {
    /// Create a new ledger entry with generated UUID and current timestamp
    pub fn new(session_id: String, user_id: String, user_input: String) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(user_input.as_bytes());
        let user_input_hash = format!("{:x}", hasher.finalize());

        Self {
            id: Uuid::new_v4(),
            session_id,
            user_id,
            timestamp: Utc::now(),
            user_input,
            user_input_hash,
            malicious_score: None,
            malicious_blocked: false,
            voting_result: VotingResult {
                parser_results: Vec::new(),
                canonical_intent: None,
                confidence: 0.0,
                agreement_level: AgreementLevel::MajorDiscrepancy,
            },
            comparison_result: ComparisonResult {
                decision: ComparatorDecision::Blocked,
                mismatches: Vec::new(),
                requires_elevation: false,
                explanation: String::new(),
            },
            elevation_event: None,
            trusted_intent: None,
            processing_output: None,
            ip_address: None,
            user_agent: None,
        }
    }
}

/// Database row representation for queries
#[derive(Debug, FromRow)]
struct LedgerRow {
    id: Uuid,
    session_id: String,
    user_id: String,
    timestamp: DateTime<Utc>,
    user_input: String,
    user_input_hash: String,
    malicious_score: Option<f64>,
    malicious_blocked: bool,
    voting_result: serde_json::Value,
    comparison_result: serde_json::Value,
    elevation_event: Option<serde_json::Value>,
    trusted_intent: Option<serde_json::Value>,
    processing_output: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl TryFrom<LedgerRow> for LedgerEntry {
    type Error = LedgerError;

    fn try_from(row: LedgerRow) -> Result<Self> {
        Ok(LedgerEntry {
            id: row.id,
            session_id: row.session_id,
            user_id: row.user_id,
            timestamp: row.timestamp,
            user_input: row.user_input,
            user_input_hash: row.user_input_hash,
            malicious_score: row.malicious_score,
            malicious_blocked: row.malicious_blocked,
            voting_result: serde_json::from_value(row.voting_result)?,
            comparison_result: serde_json::from_value(row.comparison_result)?,
            elevation_event: row
                .elevation_event
                .map(serde_json::from_value)
                .transpose()?,
            trusted_intent: row.trusted_intent,
            processing_output: row
                .processing_output
                .map(serde_json::from_value)
                .transpose()?,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
        })
    }
}

/// Append-only intent ledger backed by PostgreSQL
///
/// This ledger stores all LedgerEntry records immutably. Once written,
/// entries cannot be modified or deleted, ensuring a complete audit trail.
#[derive(Clone)]
pub struct IntentLedger {
    pool: PgPool,
}

impl IntentLedger {
    /// Create a new IntentLedger with the given database URL
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection string (e.g., "postgresql://user:pass@localhost/dbname")
    /// * `max_connections` - Maximum number of database connections in the pool
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Create a new IntentLedger from an existing connection pool
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run database migrations
    ///
    /// This should be called once during application startup to ensure
    /// the database schema is up to date.
    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| LedgerError::Database(sqlx::Error::Migrate(Box::new(e))))?;
        Ok(())
    }

    /// Append a new entry to the ledger
    ///
    /// This is the only write operation supported. Entries are immutable
    /// once written and cannot be updated or deleted.
    ///
    /// # Arguments
    /// * `entry` - The LedgerEntry to append
    ///
    /// # Returns
    /// The UUID of the newly created entry
    pub async fn append(&self, entry: LedgerEntry) -> Result<Uuid> {
        let id = entry.id;

        // Serialize complex JSON fields
        let voting_result = serde_json::to_value(&entry.voting_result)?;
        let comparison_result = serde_json::to_value(&entry.comparison_result)?;
        let elevation_event = entry
            .elevation_event
            .as_ref()
            .map(|e| serde_json::to_value(e))
            .transpose()?;
        let processing_output = entry
            .processing_output
            .as_ref()
            .map(|p| serde_json::to_value(p))
            .transpose()?;

        sqlx::query!(
            r#"
            INSERT INTO ledger_entries (
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6,
                $7, $8,
                $9, $10,
                $11, $12, $13,
                $14, $15
            )
            "#,
            id,
            entry.session_id,
            entry.user_id,
            entry.timestamp,
            entry.user_input,
            entry.user_input_hash,
            entry.malicious_score,
            entry.malicious_blocked,
            voting_result,
            comparison_result,
            elevation_event,
            entry.trusted_intent,
            processing_output,
            entry.ip_address,
            entry.user_agent,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Appended ledger entry: {}", id);
        Ok(id)
    }

    /// Query entries by user ID
    ///
    /// Returns all ledger entries for a specific user, ordered by timestamp.
    ///
    /// # Arguments
    /// * `user_id` - The user ID to query
    /// * `limit` - Maximum number of entries to return (optional)
    pub async fn query_by_user(
        &self,
        user_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<LedgerEntry>> {
        let limit = limit.unwrap_or(100);

        let rows = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            FROM ledger_entries
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(LedgerEntry::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// Query entries by session ID
    ///
    /// Returns all ledger entries for a specific session, ordered by timestamp.
    ///
    /// # Arguments
    /// * `session_id` - The session ID to query
    pub async fn query_by_session(&self, session_id: &str) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            FROM ledger_entries
            WHERE session_id = $1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(LedgerEntry::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// Query entries by ID
    ///
    /// Returns a single ledger entry by its UUID.
    ///
    /// # Arguments
    /// * `id` - The entry UUID
    pub async fn query_by_id(&self, id: Uuid) -> Result<LedgerEntry> {
        let row = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            FROM ledger_entries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(LedgerError::NotFound(id))?;

        LedgerEntry::try_from(row)
    }

    /// Query entries within a time range
    ///
    /// Returns all ledger entries between the given timestamps.
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (inclusive)
    /// * `limit` - Maximum number of entries to return (optional)
    pub async fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<i64>,
    ) -> Result<Vec<LedgerEntry>> {
        let limit = limit.unwrap_or(1000);

        let rows = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            FROM ledger_entries
            WHERE timestamp BETWEEN $1 AND $2
            ORDER BY timestamp ASC
            LIMIT $3
            "#,
        )
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(LedgerEntry::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// Query entries that required privilege elevation
    ///
    /// Returns all entries where privilege elevation was requested.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of entries to return (optional)
    pub async fn query_elevation_events(&self, limit: Option<i64>) -> Result<Vec<LedgerEntry>> {
        let limit = limit.unwrap_or(100);

        let rows = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            FROM ledger_entries
            WHERE elevation_event IS NOT NULL
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(LedgerEntry::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// Query entries that were blocked as malicious
    ///
    /// Returns all entries flagged and blocked by malicious input detection.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of entries to return (optional)
    pub async fn query_blocked_entries(&self, limit: Option<i64>) -> Result<Vec<LedgerEntry>> {
        let limit = limit.unwrap_or(100);

        let rows = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT
                id, session_id, user_id, timestamp,
                user_input, user_input_hash,
                malicious_score, malicious_blocked,
                voting_result, comparison_result,
                elevation_event, trusted_intent, processing_output,
                ip_address, user_agent
            FROM ledger_entries
            WHERE malicious_blocked = true
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(LedgerEntry::try_from)
            .collect::<Result<Vec<_>>>()
    }

    /// Get total count of entries in the ledger
    pub async fn count_entries(&self) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM ledger_entries
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Get statistics about the ledger
    pub async fn get_stats(&self) -> Result<LedgerStats> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_entries,
                COUNT(DISTINCT user_id) as total_users,
                COUNT(DISTINCT session_id) as total_sessions,
                SUM(CASE WHEN malicious_blocked THEN 1 ELSE 0 END) as blocked_entries,
                SUM(CASE WHEN elevation_event IS NOT NULL THEN 1 ELSE 0 END) as elevation_events,
                MIN(timestamp) as oldest_entry,
                MAX(timestamp) as newest_entry
            FROM ledger_entries
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(LedgerStats {
            total_entries: stats.total_entries.unwrap_or(0),
            total_users: stats.total_users.unwrap_or(0),
            total_sessions: stats.total_sessions.unwrap_or(0),
            blocked_entries: stats.blocked_entries.unwrap_or(0),
            elevation_events: stats.elevation_events.unwrap_or(0),
            oldest_entry: stats.oldest_entry,
            newest_entry: stats.newest_entry,
        })
    }
}

/// Statistics about the ledger
#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerStats {
    pub total_entries: i64,
    pub total_users: i64,
    pub total_sessions: i64,
    pub blocked_entries: i64,
    pub elevation_events: i64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ledger_entry_creation() {
        let entry = LedgerEntry::new(
            "session123".to_string(),
            "user456".to_string(),
            "What is 2 + 2?".to_string(),
        );

        assert_eq!(entry.session_id, "session123");
        assert_eq!(entry.user_id, "user456");
        assert_eq!(entry.user_input, "What is 2 + 2?");
        assert!(!entry.user_input_hash.is_empty());
    }

    // Integration tests require a running PostgreSQL database
    // Example usage:
    //
    // #[tokio::test]
    // async fn test_ledger_append_and_query() {
    //     let ledger = IntentLedger::new("postgresql://localhost/test", 5)
    //         .await
    //         .unwrap();
    //     ledger.run_migrations().await.unwrap();
    //
    //     let entry = LedgerEntry::new(
    //         "session123".to_string(),
    //         "user456".to_string(),
    //         "What is 5 * 7?".to_string(),
    //     );
    //     let id = ledger.append(entry).await.unwrap();
    //     let retrieved = ledger.query_by_id(id).await.unwrap();
    //     assert_eq!(retrieved.id, id);
    // }
}
