//! Delivery Service - Phase 2
//!
//! Delivers messages to recipient mailboxes
//! Handles retries, confirmations, and delivery tracking

use crate::models::{Envelope, RecipientId};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Delivery statistics
#[derive(Debug, Clone)]
pub struct DeliveryStats {
    pub total_deliveries: Arc<AtomicU64>,
    pub successful: Arc<AtomicU64>,
    pub failed: Arc<AtomicU64>,
    pub retried: Arc<AtomicU64>,
}

impl Default for DeliveryStats {
    fn default() -> Self {
        DeliveryStats {
            total_deliveries: Arc::new(AtomicU64::new(0)),
            successful: Arc::new(AtomicU64::new(0)),
            failed: Arc::new(AtomicU64::new(0)),
            retried: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
    Expired,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryStatus::Pending => "PENDING",
            DeliveryStatus::Delivered => "DELIVERED",
            DeliveryStatus::Failed => "FAILED",
            DeliveryStatus::Retrying => "RETRYING",
            DeliveryStatus::Expired => "EXPIRED",
        }
    }
}

/// Delivery record
#[derive(Debug, Clone)]
pub struct DeliveryRecord {
    pub message_id: String,
    pub recipient_id: RecipientId,
    pub status: DeliveryStatus,
    pub attempts: u64,
    pub last_attempt: u64,
    pub next_retry: Option<u64>,
}

/// Delivery Service configuration
#[derive(Debug, Clone)]
pub struct DeliveryServiceConfig {
    /// Max retry attempts
    pub max_retries: u64,
    /// Initial backoff in seconds
    pub initial_backoff_seconds: u64,
    /// Max backoff in seconds
    pub max_backoff_seconds: u64,
}

impl Default for DeliveryServiceConfig {
    fn default() -> Self {
        DeliveryServiceConfig {
            max_retries: 5,
            initial_backoff_seconds: 60,
            max_backoff_seconds: 3600,
        }
    }
}

/// Delivery Service - delivers messages to mailboxes
pub struct DeliveryService {
    config: DeliveryServiceConfig,
    stats: DeliveryStats,
    delivery_records: Arc<RwLock<std::collections::HashMap<String, DeliveryRecord>>>,
}

impl DeliveryService {
    /// Create a new delivery service
    pub fn new(config: DeliveryServiceConfig) -> Self {
        DeliveryService {
            config,
            stats: DeliveryStats::default(),
            delivery_records: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Deliver a message to a recipient
    pub async fn deliver(
        &self,
        envelope: &Envelope,
        recipient: &RecipientId,
    ) -> Result<DeliveryStatus> {
        self.stats.total_deliveries.fetch_add(1, Ordering::Relaxed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record_key = format!("{}:{}", envelope.id, recipient);

        // Get or create delivery record
        let mut records = self.delivery_records.write().await;
        let record = records
            .entry(record_key.clone())
            .or_insert_with(|| DeliveryRecord {
                message_id: envelope.id.to_string(),
                recipient_id: recipient.clone(),
                status: DeliveryStatus::Pending,
                attempts: 0,
                last_attempt: now,
                next_retry: None,
            });

        // Check if message has already been delivered
        if record.status == DeliveryStatus::Delivered {
            debug!(
                "Message {} already delivered to {}",
                envelope.id, recipient
            );
            return Ok(DeliveryStatus::Delivered);
        }

        // Check TTL
        if let Some(ttl) = envelope.ttl_seconds {
            if now > envelope.timestamp + ttl {
                record.status = DeliveryStatus::Expired;
                self.stats.failed.fetch_add(1, Ordering::Relaxed);
                warn!(
                    "Message {} expired before delivery to {}",
                    envelope.id, recipient
                );
                return Ok(DeliveryStatus::Expired);
            }
        }

        // Attempt delivery
        match self.attempt_delivery(envelope, recipient).await {
            Ok(_) => {
                record.status = DeliveryStatus::Delivered;
                record.attempts += 1;
                record.last_attempt = now;
                self.stats.successful.fetch_add(1, Ordering::Relaxed);

                info!(
                    "✅ Message {} delivered to {} (attempt {})",
                    envelope.id, recipient, record.attempts
                );
                Ok(DeliveryStatus::Delivered)
            }
            Err(e) => {
                record.attempts += 1;

                if record.attempts >= self.config.max_retries {
                    record.status = DeliveryStatus::Failed;
                    self.stats.failed.fetch_add(1, Ordering::Relaxed);

                    warn!(
                        "❌ Message {} delivery to {} failed after {} attempts",
                        envelope.id, recipient, record.attempts
                    );
                    Ok(DeliveryStatus::Failed)
                } else {
                    // Schedule retry with exponential backoff
                    let backoff = self.calculate_backoff(record.attempts);
                    record.next_retry = Some(now + backoff);
                    record.status = DeliveryStatus::Retrying;
                    self.stats.retried.fetch_add(1, Ordering::Relaxed);

                    debug!(
                        "⏳ Message {} delivery to {} will retry in {} seconds (attempt {})",
                        envelope.id, recipient, backoff, record.attempts
                    );
                    Ok(DeliveryStatus::Retrying)
                }
            }
        }
    }

    /// Attempt to deliver message to recipient
    async fn attempt_delivery(&self, envelope: &Envelope, recipient: &RecipientId) -> Result<()> {
        // TODO: Phase 2 - Implement actual delivery to mailbox
        // For now, simulate successful delivery with 90% success rate
        use rand::Rng;

        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.9) {
            Ok(())
        } else {
            Err(anyhow!("Simulated delivery failure"))
        }
    }

    /// Calculate exponential backoff
    fn calculate_backoff(&self, attempt: u64) -> u64 {
        let mut backoff = self.config.initial_backoff_seconds * 2u64.saturating_pow(attempt as u32);
        backoff = backoff.min(self.config.max_backoff_seconds);
        backoff
    }

    /// Get messages pending retry
    pub async fn get_pending_retries(&self) -> Vec<DeliveryRecord> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let records = self.delivery_records.read().await;
        records
            .values()
            .filter(|r| {
                r.status == DeliveryStatus::Retrying
                    && r.next_retry.map_or(false, |retry| now >= retry)
            })
            .cloned()
            .collect()
    }

    /// Retry pending deliveries
    pub async fn retry_pending(&self, envelope: &Envelope) -> Result<u64> {
        let pending = self.get_pending_retries().await;
        let mut retry_count = 0;

        for record in pending {
            match self
                .deliver(envelope, &record.recipient_id)
                .await
            {
                Ok(_) => retry_count += 1,
                Err(e) => warn!("Retry failed for {}: {}", record.recipient_id, e),
            }
        }

        Ok(retry_count as u64)
    }

    /// Get delivery statistics
    pub fn stats(&self) -> DeliveryStats {
        self.stats.clone()
    }
}

impl Default for DeliveryService {
    fn default() -> Self {
        Self::new(DeliveryServiceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_successful_delivery() {
        let service = DeliveryService::new(DeliveryServiceConfig::default());
        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["recipient".to_string()],
            "Test".to_string(),
            vec![],
        );

        let status = service.deliver(&envelope, &"recipient".to_string()).await;
        assert!(status.is_ok());
    }

    #[test]
    fn test_exponential_backoff() {
        let service = DeliveryService::new(DeliveryServiceConfig {
            initial_backoff_seconds: 60,
            max_backoff_seconds: 3600,
            ..Default::default()
        });

        assert_eq!(service.calculate_backoff(0), 60);
        assert_eq!(service.calculate_backoff(1), 120);
        assert_eq!(service.calculate_backoff(2), 240);
        assert_eq!(service.calculate_backoff(3), 480);
    }

    #[test]
    fn test_backoff_capping() {
        let service = DeliveryService::new(DeliveryServiceConfig {
            initial_backoff_seconds: 60,
            max_backoff_seconds: 1000,
            ..Default::default()
        });

        let backoff = service.calculate_backoff(10);
        assert!(backoff <= 1000);
    }
}
