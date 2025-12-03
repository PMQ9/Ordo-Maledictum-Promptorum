//! Caching helper for ledger queries
//!
//! Provides optional caching of ledger queries to Redis to reduce database load.
//! Falls back gracefully if Redis is unavailable.

#[cfg(feature = "caching")]
use intent_redis_cache::RedisCache;
#[cfg(feature = "caching")]
use intent_schema::cache::cache_keys;
#[cfg(feature = "caching")]
use intent_schema::cache::CacheBackend;
#[cfg(feature = "caching")]
use std::sync::OnceLock;
#[cfg(feature = "caching")]
use tokio::sync::Mutex;

use crate::LedgerStats;

#[cfg(feature = "caching")]
type CacheState = Mutex<Option<RedisCache>>;

#[cfg(feature = "caching")]
static LEDGER_QUERY_CACHE: OnceLock<CacheState> = OnceLock::new();

#[cfg(feature = "caching")]
async fn get_cache() -> Option<&'static CacheState> {
    LEDGER_QUERY_CACHE.get_or_init(|| Mutex::new(None)).into()
}

/// Check if ledger stats are cached
///
/// Returns true if stats are available in cache, false otherwise.
pub async fn stats_cached() -> bool {
    #[cfg(feature = "caching")]
    {
        if let Some(cache_state) = get_cache().await {
            let cache_lock = cache_state.lock().await;
            if let Some(cache) = cache_lock.as_ref() {
                if let Ok(exists) = cache.exists(cache_keys::LEDGER_STATS_KEY).await {
                    return exists;
                }
            }
        }
    }
    false
}

/// Cache ledger statistics
///
/// Stores statistics in Redis with TTL for 60 seconds.
pub async fn cache_ledger_stats(#[allow(unused_variables)] stats: &LedgerStats) {
    #[cfg(feature = "caching")]
    {
        if let Some(cache_state) = get_cache().await {
            let cache_lock = cache_state.lock().await;
            if let Some(cache) = cache_lock.as_ref() {
                if let Ok(stats_bytes) = serde_json::to_vec(stats) {
                    if let Err(e) = cache
                        .set(
                            cache_keys::LEDGER_STATS_KEY,
                            stats_bytes,
                            cache_keys::LEDGER_STATS_TTL_SECS,
                        )
                        .await
                    {
                        tracing::debug!("Failed to cache ledger statistics: {}", e);
                    } else {
                        tracing::debug!(
                            "Ledger statistics cached (TTL: {}s)",
                            cache_keys::LEDGER_STATS_TTL_SECS
                        );
                    }
                }
            }
        }
    }
}

/// Cache key generator for user ledger queries
pub fn user_ledger_cache_key(user_id: &str, limit: i64) -> String {
    format!("ledger:user:{}:{}", user_id, limit)
}

/// Cache key generator for session ledger queries
pub fn session_ledger_cache_key(session_id: &str) -> String {
    format!("ledger:session:{}", session_id)
}

/// Cache key generator for single entry queries
pub fn entry_ledger_cache_key(id: &uuid::Uuid) -> String {
    format!("ledger:entry:{}", id)
}

/// Invalidate all ledger caches (called on append)
pub async fn invalidate_all_caches() {
    #[cfg(feature = "caching")]
    {
        if let Some(cache_state) = get_cache().await {
            let cache_lock = cache_state.lock().await;
            if let Some(cache) = cache_lock.as_ref() {
                // Clear stats cache
                if let Err(e) = cache.delete(cache_keys::LEDGER_STATS_KEY).await {
                    tracing::debug!("Failed to invalidate ledger stats cache: {}", e);
                } else {
                    tracing::debug!("Ledger stats cache invalidated due to new entry");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let user_key = user_ledger_cache_key("user123", 100);
        assert_eq!(user_key, "ledger:user:user123:100");

        let session_key = session_ledger_cache_key("session456");
        assert_eq!(session_key, "ledger:session:session456");
    }
}
