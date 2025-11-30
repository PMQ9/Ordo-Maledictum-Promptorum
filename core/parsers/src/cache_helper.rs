//! Caching helper for parser system prompts
//!
//! Provides optional caching of system prompts to Redis to reduce API token costs.
//! Falls back gracefully if Redis is unavailable.

#[cfg(feature = "caching")]
use intent_redis_cache::RedisCache;
#[cfg(feature = "caching")]
use intent_schema::cache::CacheBackend;
use intent_schema::cache::cache_keys;
#[cfg(feature = "caching")]
use std::sync::OnceLock;
#[cfg(feature = "caching")]
use tokio::sync::Mutex;

#[cfg(feature = "caching")]
type CacheState = Mutex<Option<RedisCache>>;

#[cfg(feature = "caching")]
static SYSTEM_PROMPT_CACHE: OnceLock<CacheState> = OnceLock::new();

#[cfg(feature = "caching")]
async fn get_cache() -> Option<&'static CacheState> {
    SYSTEM_PROMPT_CACHE.get_or_init(|| Mutex::new(None)).into()
}

/// Get or create cached system prompt for parsers
///
/// Caches the system prompt in Redis for 24 hours to avoid recomputing
/// the same static prompt thousands of times per day.
///
/// Falls back to returning the provided prompt if caching is unavailable.
pub async fn get_cached_system_prompt(
    #[allow(unused_variables)]
    prompt_key: &str,
    #[allow(unused_variables)]
    ttl_secs: usize,
    build_prompt: impl Fn() -> String,
) -> String {
    #[cfg(feature = "caching")]
    {
        if let Some(cache_state) = get_cache().await {
            let mut cache_lock = cache_state.lock().await;

            // Initialize cache if needed
            if cache_lock.is_none() {
                if let Ok(cache) = RedisCache::new_from_env().await {
                    *cache_lock = Some(cache);
                    tracing::info!("System prompt cache initialized (Redis)");
                } else {
                    tracing::debug!("Redis unavailable for system prompt caching, will recompute");
                }
            }

            // Try to get from cache
            if let Some(cache) = cache_lock.as_ref() {
                if let Ok(Some(cached_bytes)) = cache.get(prompt_key).await {
                    if let Ok(prompt) = String::from_utf8(cached_bytes) {
                        tracing::debug!("System prompt cache hit: {}", prompt_key);
                        return prompt;
                    }
                }

                // Not in cache, build it
                let prompt = build_prompt();

                // Try to cache it
                if let Ok(prompt_bytes) = prompt.clone().into_bytes() {
                    if let Err(e) = cache.set(prompt_key, prompt_bytes, ttl_secs).await {
                        tracing::debug!("Failed to cache system prompt: {}", e);
                        // Silently fail - we'll just recompute next time
                    } else {
                        tracing::debug!("System prompt cached: {} (TTL: {}s)", prompt_key, ttl_secs);
                    }
                }

                return prompt;
            }
        }
    }

    // Fallback: no caching
    build_prompt()
}

/// Cache system prompt for parsers (variant for static prompts)
pub async fn cache_parser_system_prompt(prompt: &str) -> String {
    get_cached_system_prompt(
        cache_keys::PARSER_SYSTEM_PROMPT_KEY,
        cache_keys::PARSER_SYSTEM_PROMPT_TTL_SECS,
        || prompt.to_string(),
    )
    .await
}

/// Cache system prompt for cogitators
pub async fn cache_cogitator_system_prompt(prompt: &str) -> String {
    get_cached_system_prompt(
        cache_keys::COGITATOR_SYSTEM_PROMPT_KEY,
        cache_keys::COGITATOR_SYSTEM_PROMPT_TTL_SECS,
        || prompt.to_string(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_caching_fallback() {
        let prompt = "test prompt";
        let cached = get_cached_system_prompt("test_key", 3600, || prompt.to_string()).await;
        assert_eq!(cached, prompt);
    }
}
