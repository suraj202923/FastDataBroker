//! Ingestion Service - Phase 2
//!
//! Accepts messages from producers via QUIC
//! Validates, processes, and forwards to routing service

use crate::models::{Envelope, MessageId};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Ingestion statistics
#[derive(Debug, Clone)]
pub struct IngestionStats {
    pub total_received: Arc<AtomicU64>,
    pub validated: Arc<AtomicU64>,
    pub rejected: Arc<AtomicU64>,
    pub total_bytes: Arc<AtomicU64>,
}

impl Default for IngestionStats {
    fn default() -> Self {
        IngestionStats {
            total_received: Arc::new(AtomicU64::new(0)),
            validated: Arc::new(AtomicU64::new(0)),
            rejected: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Ingestion Service configuration
#[derive(Debug, Clone)]
pub struct IngestionServiceConfig {
    /// Max message size in bytes
    pub max_message_size: usize,
    /// Max recipients per message
    pub max_recipients: usize,
    /// Enable rate limiting
    pub rate_limit_per_second: u32,
}

impl Default for IngestionServiceConfig {
    fn default() -> Self {
        IngestionServiceConfig {
            max_message_size: 10 * 1024 * 1024,  // 10 MB
            max_recipients: 10000,
            rate_limit_per_second: 100000,
        }
    }
}

/// Message validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_error(error: String) -> Self {
        ValidationResult {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
        }
    }
}

/// Ingestion Service - accepts and validates messages
pub struct IngestionService {
    config: IngestionServiceConfig,
    stats: IngestionStats,
}

impl IngestionService {
    /// Create a new ingestion service
    pub fn new(config: IngestionServiceConfig) -> Self {
        IngestionService {
            config,
            stats: IngestionStats::default(),
        }
    }

    /// Ingest a message with full validation
    pub async fn ingest(&self, mut envelope: Envelope) -> Result<MessageId> {
        self.stats.total_received.fetch_add(1, Ordering::Relaxed);
        self.stats.total_bytes.fetch_add(envelope.content.len() as u64, Ordering::Relaxed);

        // Validate the message
        let validation = self.validate_envelope(&envelope)?;

        if !validation.is_valid {
            self.stats.rejected.fetch_add(1, Ordering::Relaxed);
            let errors = validation.errors.join("; ");
            warn!("Message validation failed: {}", errors);
            return Err(anyhow!("Message validation failed: {}", errors));
        }

        // Log any warnings
        for warning in &validation.warnings {
            debug!("Message warning: {}", warning);
        }

        // Mark as ingested
        let msg_id = envelope.id;
        self.stats.validated.fetch_add(1, Ordering::Relaxed);

        info!(
            "✅ Message ingested: {} from {} to {} recipients",
            msg_id,
            envelope.sender_id,
            envelope.recipient_ids.len()
        );

        Ok(msg_id)
    }

    /// Validate an envelope before processing
    fn validate_envelope(&self, envelope: &Envelope) -> Result<ValidationResult> {
        let mut result = ValidationResult::valid();

        // Check message size
        if envelope.content.len() > self.config.max_message_size {
            result.is_valid = false;
            result.errors.push(format!(
                "Message too large: {} > {}",
                envelope.content.len(),
                self.config.max_message_size
            ));
        }

        // Check recipient count
        if envelope.recipient_ids.is_empty() {
            result.is_valid = false;
            result.errors.push("No recipients specified".to_string());
        } else if envelope.recipient_ids.len() > self.config.max_recipients {
            result.is_valid = false;
            result.errors.push(format!(
                "Too many recipients: {} > {}",
                envelope.recipient_ids.len(),
                self.config.max_recipients
            ));
        }

        // Check sender
        if envelope.sender_id.is_empty() {
            result.is_valid = false;
            result.errors.push("Sender ID is empty".to_string());
        }

        // Check subject
        if envelope.subject.is_empty() {
            result.warnings.push("Message subject is empty".to_string());
        }

        // Validate priority
        if envelope.priority == 0 {
            result.warnings.push("Priority is 0 (minimum), consider using higher value".to_string());
        }

        // Check for duplicate recipients
        let unique_recipients: std::collections::HashSet<_> = envelope.recipient_ids.iter().cloned().collect();
        if unique_recipients.len() != envelope.recipient_ids.len() {
            result.warnings.push("Duplicate recipients detected, will be deduplicated".to_string());
        }

        Ok(result)
    }

    /// Get ingestion statistics
    pub fn stats(&self) -> IngestionStats {
        self.stats.clone()
    }

    /// Get service configuration
    pub fn config(&self) -> &IngestionServiceConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ingest_valid_message() {
        let service = IngestionService::new(IngestionServiceConfig::default());

        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test Message".to_string(),
            b"Hello, World!".to_vec(),
        );

        let result = service.ingest(envelope).await;
        assert!(result.is_ok());
        assert_eq!(service.stats.validated.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_reject_empty_recipients() {
        let service = IngestionService::new(IngestionServiceConfig::default());

        let mut envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test".to_string(),
            vec![],
        );
        envelope.recipient_ids.clear();

        let result = service.ingest(envelope).await;
        assert!(result.is_err());
        assert_eq!(service.stats.rejected.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_reject_oversized_message() {
        let mut config = IngestionServiceConfig::default();
        config.max_message_size = 100;

        let service = IngestionService::new(config);

        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test".to_string(),
            vec![0u8; 1000],  // 1000 bytes, larger than limit
        );

        let result = service.ingest(envelope).await;
        assert!(result.is_err());
    }
}
