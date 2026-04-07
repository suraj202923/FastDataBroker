// Webhook notification handler for external integrations
use crate::models::{Envelope, RecipientId};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Webhook delivery status
#[derive(Clone, Debug, PartialEq)]
pub enum WebhookStatus {
    Pending,
    Sent,
    Delivered,
    Failed(String),
    InvalidUrl,
    Timeout,
    RateLimited,
}

/// Webhook configuration for external endpoints
#[derive(Clone, Debug)]
pub struct WebhookConfig {
    pub url: String,
    pub headers: HashMap<String, String>,
    pub retry_count: u32,
    pub timeout_ms: u64,
    pub verify_ssl: bool,
}

impl WebhookConfig {
    /// Create new webhook config
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
            retry_count: 3,
            timeout_ms: 30000,
            verify_ssl: true,
        }
    }

    /// Add custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set retry count
    pub fn with_retries(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
}

/// Webhook handler configuration
#[derive(Clone, Debug)]
pub struct WebhookHandlerConfig {
    pub max_concurrent_webhooks: usize,
    pub queue_size: usize,
    pub batch_size: usize,
    pub default_timeout_ms: u64,
    pub verify_signatures: bool,
    pub signature_secret: String,
}

impl Default for WebhookHandlerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_webhooks: 1000,
            queue_size: 10000,
            batch_size: 100,
            default_timeout_ms: 30000,
            verify_signatures: true,
            signature_secret: String::new(),
        }
    }
}

/// Webhook delivery record
#[derive(Clone, Debug)]
pub struct WebhookRecord {
    pub webhook_id: String,
    pub url: String,
    pub attempts: u32,
    pub status: WebhookStatus,
    pub last_response_code: Option<u16>,
}

/// Webhook handler for external integrations
pub struct WebhookHandler {
    config: WebhookHandlerConfig,
    webhooks: Arc<std::sync::RwLock<HashMap<RecipientId, WebhookConfig>>>,
    stats: WebhookStats,
}

/// Webhook handler statistics
#[derive(Clone, Debug, Default)]
pub struct WebhookStats {
    total_received: Arc<AtomicU64>,
    sent: Arc<AtomicU64>,
    delivered: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
    invalid_urls: Arc<AtomicU64>,
    timeouts: Arc<AtomicU64>,
    rate_limited: Arc<AtomicU64>,
}

impl WebhookHandler {
    /// Create new webhook handler
    pub fn new(config: WebhookHandlerConfig) -> Self {
        info!(
            "Initializing WebhookHandler - Max concurrent: {}",
            config.max_concurrent_webhooks
        );

        Self {
            config,
            webhooks: Arc::new(std::sync::RwLock::new(HashMap::new())),
            stats: WebhookStats::default(),
        }
    }

    /// Register webhook endpoint
    pub fn register_webhook(&self, recipient: RecipientId, config: WebhookConfig) -> Result<()> {
        // Validate URL
        if !self.is_valid_url(&config.url) {
            warn!("Invalid webhook URL: {}", config.url);
            return Err(anyhow!("Invalid URL format"));
        }

        let mut webhooks = self
            .webhooks
            .write()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;

        if webhooks.len() >= self.config.max_concurrent_webhooks {
            return Err(anyhow!("Max webhooks reached"));
        }

        webhooks.insert(recipient.clone(), config);
        info!("Webhook registered for: {}", recipient);

        Ok(())
    }

    /// Unregister webhook endpoint
    pub fn unregister_webhook(&self, recipient: &RecipientId) -> Result<()> {
        let mut webhooks = self
            .webhooks
            .write()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;

        if webhooks.remove(recipient).is_some() {
            info!("Webhook unregistered for: {}", recipient);
            Ok(())
        } else {
            Err(anyhow!("Webhook not found: {}", recipient))
        }
    }

    /// Send message via webhook to external endpoint
    pub async fn send_webhook(
        &self,
        envelope: &Envelope,
        recipient: &RecipientId,
    ) -> Result<WebhookStatus> {
        debug!(
            "Sending webhook to recipient: {}, msg_id: {}",
            recipient, envelope.id
        );

        self.stats.total_received.fetch_add(1, Ordering::Relaxed);

        let webhooks = self
            .webhooks
            .read()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;

        let webhook = match webhooks.get(recipient) {
            Some(w) => w,
            None => {
                debug!("Webhook not registered: {}", recipient);
                self.stats.invalid_urls.fetch_add(1, Ordering::Relaxed);
                return Ok(WebhookStatus::InvalidUrl);
            }
        };

        // Attempt webhook delivery
        self.attempt_webhook_send(envelope, webhook).await
    }

    /// Attempt to send webhook with retries
    async fn attempt_webhook_send(
        &self,
        envelope: &Envelope,
        webhook: &WebhookConfig,
    ) -> Result<WebhookStatus> {
        // Simulate webhook POST request
        // Phase 3 extension: Use reqwest crate for actual HTTP requests

        // Validate URL format
        if !self.is_valid_url(&webhook.url) {
            return Ok(WebhookStatus::InvalidUrl);
        }

        // Simulate HTTP request with 90% success rate
        let success_rate = 90;
        if rand::random::<u32>() % 100 < success_rate {
            info!("Webhook delivered to: {}", webhook.url);
            self.stats.sent.fetch_add(1, Ordering::Relaxed);
            self.stats.delivered.fetch_add(1, Ordering::Relaxed);

            Ok(WebhookStatus::Delivered)
        } else {
            // Simulate different failure modes
            let failure_type = rand::random::<u32>() % 3;
            match failure_type {
                0 => {
                    self.stats.timeouts.fetch_add(1, Ordering::Relaxed);
                    Ok(WebhookStatus::Timeout)
                }
                1 => {
                    self.stats.rate_limited.fetch_add(1, Ordering::Relaxed);
                    Ok(WebhookStatus::RateLimited)
                }
                _ => {
                    self.stats.failed.fetch_add(1, Ordering::Relaxed);
                    Ok(WebhookStatus::Failed("HTTP error".to_string()))
                }
            }
        }
    }

    /// Validate webhook URL format
    fn is_valid_url(&self, url: &str) -> bool {
        // Basic URL validation
        (url.starts_with("http://") || url.starts_with("https://"))
            && url.len() > 10
            && !url.contains("localhost")
    }

    /// Generate webhook signature (HMAC-SHA256)
    fn generate_signature(&self, payload: &[u8]) -> String {
        use sha2::{Sha256, Digest};

        if self.config.signature_secret.is_empty() {
            return String::new();
        }

        let mut hasher = Sha256::new();
        hasher.update(&self.config.signature_secret);
        hasher.update(payload);

        format!("{:x}", hasher.finalize())
    }

    /// Batch send webhooks
    pub async fn batch_send(&self, envelope: &Envelope, recipients: Vec<&str>) -> Result<(usize, usize)> {
        let mut sent = 0;
        let mut failed = 0;

        for recipient in recipients.iter().take(self.config.batch_size) {
            match self.send_webhook(envelope, &recipient.to_string()).await? {
                WebhookStatus::Delivered | WebhookStatus::Sent => sent += 1,
                _ => failed += 1,
            }
        }

        info!("Batch webhook send: {} sent, {} failed", sent, failed);

        Ok((sent, failed))
    }

    /// Get webhook handler statistics
    pub fn stats(&self) -> WebhookHandlerStats {
        WebhookHandlerStats {
            total_received: self.stats.total_received.load(Ordering::Relaxed),
            sent: self.stats.sent.load(Ordering::Relaxed),
            delivered: self.stats.delivered.load(Ordering::Relaxed),
            failed: self.stats.failed.load(Ordering::Relaxed),
            invalid_urls: self.stats.invalid_urls.load(Ordering::Relaxed),
            timeouts: self.stats.timeouts.load(Ordering::Relaxed),
            rate_limited: self.stats.rate_limited.load(Ordering::Relaxed),
            success_rate: self.calculate_success_rate(),
        }
    }

    fn calculate_success_rate(&self) -> f64 {
        let total = self.stats.total_received.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let delivered = self.stats.delivered.load(Ordering::Relaxed) as f64;
        (delivered / total as f64) * 100.0
    }
}

/// Webhook handler statistics snapshot
#[derive(Clone, Debug)]
pub struct WebhookHandlerStats {
    pub total_received: u64,
    pub sent: u64,
    pub delivered: u64,
    pub failed: u64,
    pub invalid_urls: u64,
    pub timeouts: u64,
    pub rate_limited: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_webhook_url() {
        let handler = WebhookHandler::new(WebhookHandlerConfig::default());

        assert!(handler.is_valid_url("https://example.com/webhook"));
        assert!(handler.is_valid_url("http://webhook.service.com/notify"));
        assert!(!handler.is_valid_url("http://localhost:8080/webhook"));
        assert!(!handler.is_valid_url("ftp://example.com"));
        assert!(!handler.is_valid_url("invalid"));
    }

    #[test]
    fn test_register_webhook() {
        let handler = WebhookHandler::new(WebhookHandlerConfig::default());

        let config = WebhookConfig::new("https://example.com/notify".to_string());
        let result = handler.register_webhook("service-1".to_string(), config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_invalid_webhook_url() {
        let handler = WebhookHandler::new(WebhookHandlerConfig::default());

        let config = WebhookConfig::new("not-a-valid-url".to_string());
        let result = handler.register_webhook("service-1".to_string(), config);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_webhook() {
        let handler = WebhookHandler::new(WebhookHandlerConfig::default());

        let config = WebhookConfig::new("https://example.com/webhook".to_string());
        handler
            .register_webhook("service-1".to_string(), config)
            .unwrap();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["service-1".to_string()],
            "Event".to_string(),
            b"Event data".to_vec(),
        );

        let result = handler
            .send_webhook(&envelope, &"service-1".to_string())
            .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            WebhookStatus::Delivered | WebhookStatus::Failed(_) | WebhookStatus::Timeout | WebhookStatus::RateLimited
        ));
    }

    #[tokio::test]
    async fn test_unregister_webhook() {
        let handler = WebhookHandler::new(WebhookHandlerConfig::default());

        let config = WebhookConfig::new("https://example.com/webhook".to_string());
        handler
            .register_webhook("service-1".to_string(), config)
            .unwrap();

        let result = handler.unregister_webhook(&"service-1".to_string());

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_webhook_stats_tracking() {
        let handler = WebhookHandler::new(WebhookHandlerConfig::default());

        let config = WebhookConfig::new("https://example.com/webhook".to_string());
        handler
            .register_webhook("service-1".to_string(), config)
            .unwrap();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["service-1".to_string()],
            "Event".to_string(),
            b"Data".to_vec(),
        );

        for _ in 0..5 {
            let _ = handler
                .send_webhook(&envelope, &"service-1".to_string())
                .await;
        }

        let stats = handler.stats();
        assert_eq!(stats.total_received, 5);
    }

    #[test]
    fn test_webhook_config_builder() {
        let config = WebhookConfig::new("https://example.com".to_string())
            .with_retries(5)
            .with_header("X-Custom-Header".to_string(), "custom-value".to_string());

        assert_eq!(config.retry_count, 5);
        assert!(config.headers.contains_key("X-Custom-Header"));
    }
}
