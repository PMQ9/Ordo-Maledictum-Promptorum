//! Caching layer for cost optimization
//!
//! Implements Redis-based caching to reduce API calls and database queries:
//! - System prompts (40% of LLM input tokens)
//! - User ledger queries (immutable, cacheable)
//! - Session history (immutable within session)
//! - Parser results (for identical inputs)
//! - Vault corruption test results (for identical inputs)
//! - Ledger statistics (stable between requests)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Cache keys and TTLs for different data types
pub mod cache_keys {
    /// System prompt for LLM parsers (24 hour TTL - rarely changes)
    pub const PARSER_SYSTEM_PROMPT_KEY: &str = "system_prompt:parser:v1";
    pub const PARSER_SYSTEM_PROMPT_TTL_SECS: usize = 86400; // 24 hours

    /// System prompt for sacrificial cogitators (24 hour TTL)
    pub const COGITATOR_SYSTEM_PROMPT_KEY: &str = "system_prompt:cogitator:v1";
    pub const COGITATOR_SYSTEM_PROMPT_TTL_SECS: usize = 86400; // 24 hours

    /// Batch diagnostic system prompt (24 hour TTL)
    pub const BATCH_DIAGNOSTIC_SYSTEM_PROMPT_KEY: &str = "system_prompt:batch_diag:v1";
    pub const BATCH_DIAGNOSTIC_SYSTEM_PROMPT_TTL_SECS: usize = 86400; // 24 hours

    /// User ledger history (cache until new entries, 1 hour TTL)
    pub fn user_ledger_key(user_id: &str) -> String {
        format!("ledger:user:{}", user_id)
    }
    pub const USER_LEDGER_TTL_SECS: usize = 3600; // 1 hour

    /// Session ledger history (session-duration TTL, 24 hour max)
    pub fn session_ledger_key(session_id: &str) -> String {
        format!("ledger:session:{}", session_id)
    }
    pub const SESSION_LEDGER_TTL_SECS: usize = 86400; // 24 hours

    /// Individual ledger entry by ID (immutable, 7 day TTL)
    pub fn ledger_entry_key(entry_id: &str) -> String {
        format!("ledger:entry:{}", entry_id)
    }
    pub const LEDGER_ENTRY_TTL_SECS: usize = 604800; // 7 days

    /// Ledger statistics (5 minute TTL - updates frequently during operation)
    pub const LEDGER_STATS_KEY: &str = "ledger:stats";
    pub const LEDGER_STATS_TTL_SECS: usize = 300; // 5 minutes

    /// Parser result for input (5 minute TTL - inputs change frequently)
    pub fn parser_result_key(input_hash: &str) -> String {
        format!("parser:result:{}", input_hash)
    }
    pub const PARSER_RESULT_TTL_SECS: usize = 300; // 5 minutes

    /// Vault corruption test result for input (5 minute TTL)
    pub fn vault_corruption_key(input_hash: &str) -> String {
        format!("vault:corruption:{}", input_hash)
    }
    pub const VAULT_CORRUPTION_TTL_SECS: usize = 300; // 5 minutes

    /// Approved users cache (1 day TTL)
    pub const APPROVED_USERS_KEY: &str = "approval:users";
    pub const APPROVED_USERS_TTL_SECS: usize = 86400; // 24 hours
}

/// Trait for cache backends (Redis, in-memory, etc.)
#[async_trait::async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get value from cache
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Set value in cache with TTL
    async fn set(&self, key: &str, value: Vec<u8>, ttl_secs: usize) -> Result<(), CacheError>;

    /// Delete value from cache
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Invalidate all caches (careful - expensive operation)
    async fn clear_all(&self) -> Result<(), CacheError>;
}

/// Cache errors
#[derive(Debug, Clone)]
pub enum CacheError {
    /// Connection error
    ConnectionError(String),

    /// Serialization error
    SerializationError(String),

    /// Deserialization error
    DeserializationError(String),

    /// Cache key not found
    NotFound,

    /// Other error
    Other(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionError(msg) => write!(f, "Cache connection error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Cache serialization error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "Cache deserialization error: {}", msg),
            Self::NotFound => write!(f, "Cache key not found"),
            Self::Other(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cacheable intent parser result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedParserResult {
    /// The parsed intent
    pub action: String,
    pub topic_id: String,
    pub expertise_areas: Vec<String>,
    pub max_budget: Option<u32>,
    pub max_results: Option<u32>,
    pub confidence: f32,
    pub parser_name: String,
}

/// Cacheable vault corruption test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCorruptionResult {
    /// Is the input corrupted/malicious?
    pub is_corrupted: bool,

    /// Risk score from consensus
    pub risk_score: f32,

    /// Number of cogitators that flagged as suspicious
    pub suspicious_count: usize,

    /// Total cogitators tested
    pub total_cogitators: usize,
}

/// Cache utility functions
pub mod utils {
    use sha2::{Digest, Sha256};

    /// Generate cache key from input (for deduplication)
    pub fn hash_input(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Serialize to JSON bytes
    pub fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, super::CacheError> {
        serde_json::to_vec(value).map_err(|e| super::CacheError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON bytes
    pub fn deserialize<T: serde::de::DeserializeOwned>(
        bytes: Vec<u8>,
    ) -> Result<T, super::CacheError> {
        serde_json::from_slice(&bytes)
            .map_err(|e| super::CacheError::DeserializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let user_id = "user123";
        let key = cache_keys::user_ledger_key(user_id);
        assert_eq!(key, "ledger:user:user123");
    }

    #[test]
    fn test_input_hashing() {
        let input = "test input";
        let hash1 = utils::hash_input(input);
        let hash2 = utils::hash_input(input);
        assert_eq!(hash1, hash2); // Same input produces same hash
    }

    #[test]
    fn test_serialization() {
        let result = CachedParserResult {
            action: "math_question".to_string(),
            topic_id: "algebra".to_string(),
            expertise_areas: vec![],
            max_budget: None,
            max_results: None,
            confidence: 0.95,
            parser_name: "Claude".to_string(),
        };

        let serialized = utils::serialize(&result).unwrap();
        let deserialized: CachedParserResult = utils::deserialize(serialized).unwrap();
        assert_eq!(deserialized.action, "math_question");
    }
}
