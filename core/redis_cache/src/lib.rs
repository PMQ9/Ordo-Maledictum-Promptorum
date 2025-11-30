use intent_schema::cache::{CacheBackend, CacheError};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedisCacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),

    #[error("Redis command error: {0}")]
    CommandError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<RedisCacheError> for CacheError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::ConnectionError(msg) => CacheError::ConnectionError(msg),
            RedisCacheError::CommandError(msg) => CacheError::Other(msg),
            RedisCacheError::SerializationError(msg) => CacheError::SerializationError(msg),
            RedisCacheError::ConfigError(msg) => CacheError::Other(msg),
        }
    }
}

impl From<redis::RedisError> for RedisCacheError {
    fn from(err: redis::RedisError) -> Self {
        RedisCacheError::CommandError(err.to_string())
    }
}

pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            db: 0,
        }
    }
}

impl RedisConfig {
    pub fn from_env() -> Result<Self, RedisCacheError> {
        let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("REDIS_PORT")
            .unwrap_or_else(|_| "6379".to_string())
            .parse()
            .map_err(|_| RedisCacheError::ConfigError("Invalid REDIS_PORT".to_string()))?;
        let password = std::env::var("REDIS_PASSWORD").ok();
        let db: u32 = std::env::var("REDIS_DB")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .map_err(|_| RedisCacheError::ConfigError("Invalid REDIS_DB".to_string()))?;

        Ok(Self {
            host,
            port,
            password,
            db,
        })
    }

    fn connection_string(&self) -> String {
        if let Some(password) = &self.password {
            format!("redis://:{}@{}:{}/{}", password, self.host, self.port, self.db)
        } else {
            format!("redis://{}:{}/{}", self.host, self.port, self.db)
        }
    }
}

pub struct RedisCache {
    conn: Arc<ConnectionManager>,
}

impl RedisCache {
    pub async fn new(config: RedisConfig) -> Result<Self, RedisCacheError> {
        let client = redis::Client::open(config.connection_string())
            .map_err(|e| RedisCacheError::ConnectionError(e.to_string()))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| RedisCacheError::ConnectionError(e.to_string()))?;

        tracing::info!("Redis cache connected to {}:{}", config.host, config.port);

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn new_from_env() -> Result<Self, RedisCacheError> {
        let config = RedisConfig::from_env()?;
        Self::new(config).await
    }

    pub async fn health_check(&self) -> Result<bool, RedisCacheError> {
        let mut conn = (*self.conn).clone();
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| RedisCacheError::CommandError(format!("PING failed: {}", e)))?;

        Ok(pong == "PONG")
    }

    pub async fn flush_db(&self) -> Result<(), RedisCacheError> {
        let mut conn = (*self.conn).clone();
        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .map_err(|e| RedisCacheError::CommandError(format!("FLUSHDB failed: {}", e)))?;

        tracing::warn!("Redis database flushed");
        Ok(())
    }
}

#[async_trait::async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut conn = (*self.conn).clone();
        let value: Option<Vec<u8>> = conn
            .get(key)
            .await
            .map_err(|e| RedisCacheError::CommandError(format!("GET {} failed: {}", key, e)))?;

        if value.is_some() {
            tracing::debug!("Cache hit: {}", key);
        } else {
            tracing::debug!("Cache miss: {}", key);
        }

        Ok(value)
    }

    async fn set(&self, key: &str, value: Vec<u8>, ttl_secs: usize) -> Result<(), CacheError> {
        let mut conn = (*self.conn).clone();

        if ttl_secs > 0 {
            let ttl: u64 = ttl_secs as u64;
            let _: () = conn.set_ex::<_, _, ()>(key, value, ttl)
                .await
                .map_err(|e| {
                    RedisCacheError::CommandError(format!("SET with TTL {} failed: {}", key, e))
                })?;
        } else {
            let _: () = conn.set::<_, _, ()>(key, value)
                .await
                .map_err(|e| RedisCacheError::CommandError(format!("SET {} failed: {}", key, e)))?;
        }

        tracing::debug!("Cache set: {} (TTL: {}s)", key, ttl_secs);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = (*self.conn).clone();
        let _: () = conn.del::<_, ()>(key)
            .await
            .map_err(|e| RedisCacheError::CommandError(format!("DEL {} failed: {}", key, e)))?;

        tracing::debug!("Cache deleted: {}", key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = (*self.conn).clone();
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| RedisCacheError::CommandError(format!("EXISTS {} failed: {}", key, e)))?;

        Ok(exists)
    }

    async fn clear_all(&self) -> Result<(), CacheError> {
        self.flush_db()
            .await
            .map_err(|e| CacheError::Other(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_redis_connection() {
        let config = RedisConfig::default();
        let cache = RedisCache::new(config).await;
        assert!(cache.is_ok());

        if let Ok(cache) = cache {
            let health = cache.health_check().await;
            assert!(health.is_ok());
            assert!(health.unwrap());
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_and_get() {
        let config = RedisConfig::default();
        let cache = RedisCache::new(config).await.unwrap();

        let key = "test_key";
        let value = b"test_value".to_vec();

        let result = cache.set(key, value.clone(), 60).await;
        assert!(result.is_ok());

        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete() {
        let config = RedisConfig::default();
        let cache = RedisCache::new(config).await.unwrap();

        let key = "delete_test";
        cache.set(key, b"value".to_vec(), 60).await.unwrap();

        let exists = cache.exists(key).await.unwrap();
        assert!(exists);

        cache.delete(key).await.unwrap();

        let exists = cache.exists(key).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    #[ignore]
    async fn test_ttl_expiration() {
        let config = RedisConfig::default();
        let cache = RedisCache::new(config).await.unwrap();

        let key = "ttl_test";
        cache.set(key, b"value".to_vec(), 1).await.unwrap();

        let retrieved = cache.get(key).await.unwrap();
        assert!(retrieved.is_some());

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let retrieved = cache.get(key).await.unwrap();
        assert!(retrieved.is_none());
    }
}
