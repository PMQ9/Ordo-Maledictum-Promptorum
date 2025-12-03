//! Caching helper for cogitator system prompts
//!
//! Provides optional caching of system prompts to Redis to reduce API token costs.
//! Falls back gracefully if Redis is unavailable.

#[cfg(feature = "caching")]
use intent_redis_cache::RedisCache;
use intent_schema::cache::cache_keys;
#[cfg(feature = "caching")]
use intent_schema::cache::CacheBackend;
#[cfg(feature = "caching")]
use std::sync::OnceLock;
#[cfg(feature = "caching")]
use tokio::sync::Mutex;

#[cfg(feature = "caching")]
type CacheState = Mutex<Option<RedisCache>>;

#[cfg(feature = "caching")]
static COGITATOR_PROMPT_CACHE: OnceLock<CacheState> = OnceLock::new();

#[cfg(feature = "caching")]
async fn get_cache() -> Option<&'static CacheState> {
    COGITATOR_PROMPT_CACHE
        .get_or_init(|| Mutex::new(None))
        .into()
}

/// Get or create cached system prompt for cogitators
///
/// Caches the system prompt in Redis for 24 hours to avoid recomputing
/// the same static prompt thousands of times per day.
///
/// Falls back to returning the provided prompt if caching is unavailable.
pub async fn get_cached_system_prompt(
    #[allow(unused_variables)] prompt_key: &str,
    #[allow(unused_variables)] ttl_secs: usize,
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
                    tracing::info!("Cogitator system prompt cache initialized (Redis)");
                } else {
                    tracing::debug!(
                        "Redis unavailable for cogitator prompt caching, will recompute"
                    );
                }
            }

            // Try to get from cache
            if let Some(cache) = cache_lock.as_ref() {
                if let Ok(Some(cached_bytes)) = cache.get(prompt_key).await {
                    if let Ok(prompt) = String::from_utf8(cached_bytes) {
                        tracing::debug!("Cogitator system prompt cache hit: {}", prompt_key);
                        return prompt;
                    }
                }

                // Not in cache, build it
                let prompt = build_prompt();

                // Try to cache it
                if let Ok(prompt_bytes) = prompt.clone().into_bytes() {
                    if let Err(e) = cache.set(prompt_key, prompt_bytes, ttl_secs).await {
                        tracing::debug!("Failed to cache cogitator prompt: {}", e);
                        // Silently fail - we'll just recompute next time
                    } else {
                        tracing::debug!(
                            "Cogitator system prompt cached: {} (TTL: {}s)",
                            prompt_key,
                            ttl_secs
                        );
                    }
                }

                return prompt;
            }
        }
    }

    // Fallback: no caching
    build_prompt()
}

/// Cache system prompt for corruption detection
pub async fn cache_corruption_system_prompt(prompt: &str) -> String {
    get_cached_system_prompt(
        cache_keys::COGITATOR_SYSTEM_PROMPT_KEY,
        cache_keys::COGITATOR_SYSTEM_PROMPT_TTL_SECS,
        || prompt.to_string(),
    )
    .await
}

/// Cache system prompt for batch diagnostics
pub async fn cache_batch_diagnostic_system_prompt(prompt: &str) -> String {
    get_cached_system_prompt(
        cache_keys::BATCH_DIAGNOSTIC_SYSTEM_PROMPT_KEY,
        cache_keys::BATCH_DIAGNOSTIC_SYSTEM_PROMPT_TTL_SECS,
        || prompt.to_string(),
    )
    .await
}

/// Hash input for vault deduplication (SHA256)
///
/// Used to create cache keys for vault corruption test results.
/// Allows caching of corruption test results for identical inputs.
pub fn hash_input(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get cached vault corruption test result for identical inputs (5 minute TTL)
///
/// If the same user input is tested for corruption multiple times within 5 minutes,
/// returns the previously cached corruption consensus result instead of calling all cogitators.
///
/// This helps with: demo testing, user retries, copy-paste submissions.
pub async fn get_cached_corruption_test(
    #[allow(unused_variables)] input_hash: &str,
) -> Option<String> {
    #[cfg(feature = "caching")]
    {
        if let Some(cache_state) = get_cache().await {
            let cache_lock = cache_state.lock().await;
            if let Some(cache) = cache_lock.as_ref() {
                let cache_key = cache_keys::vault_corruption_key(input_hash);
                if let Ok(Some(cached_bytes)) = cache.get(&cache_key).await {
                    if let Ok(result_str) = String::from_utf8(cached_bytes) {
                        tracing::debug!(
                            "Vault corruption test cache hit for input: {}",
                            input_hash
                        );
                        return Some(result_str);
                    }
                }
            }
        }
    }

    None
}

/// Cache vault corruption test result for future identical inputs (5 minute TTL)
pub async fn cache_corruption_test(
    #[allow(unused_variables)] input_hash: &str,
    #[allow(unused_variables)] result: &str,
) {
    #[cfg(feature = "caching")]
    {
        if let Some(cache_state) = get_cache().await {
            let cache_lock = cache_state.lock().await;
            if let Some(cache) = cache_lock.as_ref() {
                let cache_key = cache_keys::vault_corruption_key(input_hash);
                if let Ok(result_bytes) = result.as_bytes().to_vec() {
                    if let Err(e) = cache
                        .set(
                            &cache_key,
                            result_bytes,
                            cache_keys::VAULT_CORRUPTION_TTL_SECS,
                        )
                        .await
                    {
                        tracing::debug!("Failed to cache vault corruption test: {}", e);
                    } else {
                        tracing::debug!(
                            "Vault corruption test cached for input: {} (TTL: {}s)",
                            input_hash,
                            cache_keys::VAULT_CORRUPTION_TTL_SECS
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_caching_fallback() {
        let prompt = "test cogitator prompt";
        let cached =
            get_cached_system_prompt("test_cogitator_key", 3600, || prompt.to_string()).await;
        assert_eq!(cached, prompt);
    }

    #[test]
    fn test_input_hashing() {
        let input = "Check if this is malicious";
        let hash = hash_input(input);

        // Hash should be deterministic
        let hash2 = hash_input(input);
        assert_eq!(hash, hash2);

        // Hash should be different for different inputs
        let different_hash = hash_input("Different input");
        assert_ne!(hash, different_hash);

        // Hash should be valid hex (64 chars for SHA256)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
