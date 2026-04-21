// ============================================================================
// FastDataBroker QUIC Authentication & Rate Limiting
// API key validation and request throttling middleware
// ============================================================================

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use hex;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use crate::logging::AtomicLogger;

/// API Key with metadata
#[derive(Clone, Debug)]
pub struct ApiKey {
    pub id: String,
    pub key_hash: String,           // SHA-256 hash
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub rate_limit_rps: u32,        // Requests per second
    pub scopes: Vec<String>,
    pub last_used: Option<DateTime<Utc>>,
}

/// Rate limit bucket
#[derive(Clone, Debug)]
struct RateLimitBucket {
    requests: u32,
    window_start: Instant,
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    InvalidKey,
    KeyExpired,
    KeyDisabled,
    RateLimitExceeded { limit: u32 },
    MissingAuth,
    InvalidFormat,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidKey => write!(f, "Invalid API key"),
            AuthError::KeyExpired => write!(f, "API key expired"),
            AuthError::KeyDisabled => write!(f, "API key disabled"),
            AuthError::RateLimitExceeded { limit } => {
                write!(f, "Rate limit exceeded ({} req/sec)", limit)
            }
            AuthError::MissingAuth => write!(f, "Missing authentication"),
            AuthError::InvalidFormat => write!(f, "Invalid auth format"),
        }
    }
}

/// QUIC Authentication validator
pub struct QuicAuthValidator {
    // Map of API key hashes to full key metadata
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    // Rate limit buckets per client
    rate_limits: Arc<RwLock<HashMap<String, RateLimitBucket>>>,
    // Metrics
    pub auth_success: Arc<AtomicU32>,
    pub auth_failures: Arc<AtomicU32>,
    pub rate_limit_hits: Arc<AtomicU32>,
    // Optimized atomic logger for frequent operations
    logger: AtomicLogger,
}

impl QuicAuthValidator {
    pub fn new() -> Self {
        QuicAuthValidator {
            keys: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            auth_success: Arc::new(AtomicU32::new(0)),
            auth_failures: Arc::new(AtomicU32::new(0)),
            rate_limit_hits: Arc::new(AtomicU32::new(0)),
            logger: AtomicLogger::new(),
        }
    }

    /// Generate a new API key for a client
    pub async fn generate_key(&self, client_id: &str, rate_limit_rps: u32) -> String {
        let key = format!("sk_prod_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let key_hash = self.hash_key(&key);

        let api_key = ApiKey {
            id: format!("key_{}", uuid::Uuid::new_v4()),
            key_hash,
            client_id: client_id.to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(90)),
            is_active: true,
            rate_limit_rps,
            scopes: vec!["messages:write".to_string(), "messages:read".to_string()],
            last_used: None,
        };

        // Store in database (mock for now)
        let mut keys = self.keys.write().await;
        keys.insert(api_key.key_hash.clone(), api_key.clone());

        info!(
            "🔑 Generated new API key for client '{}': {} (rate limit: {} req/sec)",
            client_id, api_key.id, rate_limit_rps
        );

        key
    }

    /// Validate API key from QUIC client
    pub async fn validate_key(&self, api_key: &str) -> Result<ApiKey, AuthError> {
        let key_hash = self.hash_key(api_key);

        let keys = self.keys.read().await;
        let key = keys
            .get(&key_hash)
            .ok_or(AuthError::InvalidKey)?
            .clone();

        drop(keys); // Release read lock

        // Check if key is active
        if !key.is_active {
            self.auth_failures.fetch_add(1, Ordering::Relaxed);
            self.logger.warn();  // Record in atomic logger (nanosecond overhead)
            warn!("🚫 Auth failed: Key disabled for client '{}'", key.client_id);
            return Err(AuthError::KeyDisabled);
        }

        // Check expiration
        if let Some(expires) = key.expires_at {
            if Utc::now() > expires {
                self.auth_failures.fetch_add(1, Ordering::Relaxed);
                self.logger.warn();  // Record in atomic logger (nanosecond overhead)
                warn!("🚫 Auth failed: Key expired for client '{}'", key.client_id);
                return Err(AuthError::KeyExpired);
            }
        }

        // Check rate limit
        self.check_rate_limit(&key).await?;

        self.auth_success.fetch_add(1, Ordering::Relaxed);
        self.logger.info();  // Record in atomic logger (nanosecond overhead)
        debug!("✅ Auth success for client '{}'", key.client_id);

        Ok(key)
    }

    /// Check if client has exceeded rate limit
    async fn check_rate_limit(&self, key: &ApiKey) -> Result<(), AuthError> {
        let mut limits = self.rate_limits.write().await;
        let now = Instant::now();

        let bucket = limits.entry(key.client_id.clone()).or_insert_with(|| {
            RateLimitBucket {
                requests: 0,
                window_start: now,
            }
        });

        // Reset bucket if window expired (1 second)
        if now.duration_since(bucket.window_start) >= Duration::from_secs(1) {
            bucket.requests = 0;
            bucket.window_start = now;
        }

        // Check if limit exceeded
        if bucket.requests >= key.rate_limit_rps {
            self.rate_limit_hits.fetch_add(1, Ordering::Relaxed);
            self.logger.warn();  // Record in atomic logger (nanosecond overhead)
            warn!(
                "⚠️  Rate limit exceeded for client '{}': {} req/sec limit",
                key.client_id, key.rate_limit_rps
            );
            return Err(AuthError::RateLimitExceeded {
                limit: key.rate_limit_rps,
            });
        }

        bucket.requests += 1;
        Ok(())
    }

    /// Revoke an API key
    pub async fn revoke_key(&self, api_key: &str) -> Result<(), AuthError> {
        let key_hash = self.hash_key(api_key);

        let mut keys = self.keys.write().await;
        if let Some(key) = keys.get_mut(&key_hash) {
            key.is_active = false;
            self.logger.info();  // Record in atomic logger (nanosecond overhead)
            info!("🗑️  Revoked API key for client '{}'", key.client_id);
            return Ok(());
        }

        Err(AuthError::InvalidKey)
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u32, u32, u32) {
        (
            self.auth_success.load(Ordering::Relaxed),
            self.auth_failures.load(Ordering::Relaxed),
            self.rate_limit_hits.load(Ordering::Relaxed),
        )
    }

    /// Get atomic logger statistics
    pub fn get_logger_stats(&self) -> crate::logging::AtomicStats {
        self.logger.stats()
    }

    /// Hash API key using SHA-256
    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl Default for QuicAuthValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_and_validate_key() {
        let auth = QuicAuthValidator::new();
        let api_key = auth.generate_key("test-client", 1000).await;

        let result = auth.validate_key(&api_key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_key() {
        let auth = QuicAuthValidator::new();

        let result = auth.validate_key("invalid_key").await;
        assert!(matches!(result, Err(AuthError::InvalidKey)));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let auth = QuicAuthValidator::new();
        let api_key = auth.generate_key("test-client", 5).await; // 5 req/sec

        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(auth.validate_key(&api_key).await.is_ok());
        }

        // 6th should fail
        let result = auth.validate_key(&api_key).await;
        assert!(matches!(
            result,
            Err(AuthError::RateLimitExceeded { limit: 5 })
        ));
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let auth = QuicAuthValidator::new();
        let api_key = auth.generate_key("test-client", 100).await;

        // Should work before revocation
        assert!(auth.validate_key(&api_key).await.is_ok());

        // Revoke it
        assert!(auth.revoke_key(&api_key).await.is_ok());

        // Should fail after revocation
        assert!(matches!(
            auth.validate_key(&api_key).await,
            Err(AuthError::KeyDisabled)
        ));
    }
}
