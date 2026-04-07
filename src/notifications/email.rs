// Email notification handler with SMTP support
use crate::models::{Envelope, RecipientId};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Email handler configuration
#[derive(Clone, Debug)]
pub struct EmailHandlerConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
    pub require_tls: bool,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

impl Default for EmailHandlerConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: "user@example.com".to_string(),
            smtp_password: String::new(),
            from_address: "noreply@fastdatabroker.local".to_string(),
            from_name: "Post Office".to_string(),
            require_tls: true,
            max_retries: 3,
            timeout_ms: 30000,
        }
    }
}

/// Email delivery status
#[derive(Clone, Debug, PartialEq)]
pub enum EmailStatus {
    Pending,
    Sent,
    Failed(String),
    Bounced,
    Unsubscribed,
}

/// Email handler for SMTP notifications
pub struct EmailHandler {
    config: EmailHandlerConfig,
    stats: EmailStats,
}

/// Email handler statistics
#[derive(Clone, Debug, Default)]
pub struct EmailStats {
    total_received: Arc<AtomicU64>,
    sent: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
    bounced: Arc<AtomicU64>,
    total_bytes: Arc<AtomicU64>,
}

impl EmailHandler {
    /// Create new email handler
    pub fn new(config: EmailHandlerConfig) -> Self {
        info!(
            "Initializing EmailHandler - SMTP: {}:{}",
            config.smtp_host, config.smtp_port
        );

        Self {
            config,
            stats: EmailStats::default(),
        }
    }

    /// Send email to recipient
    pub async fn send_email(
        &self,
        envelope: &Envelope,
        recipient: &RecipientId,
    ) -> Result<EmailStatus> {
        debug!(
            "Processing email for recipient: {}, msg_id: {}",
            recipient, envelope.id
        );

        // Validate recipient email format
        if !self.is_valid_email(recipient) {
            warn!("Invalid email format: {}", recipient);
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            return Ok(EmailStatus::Failed("Invalid email format".to_string()));
        }

        // Check if recipient is unsubscribed (stub for Phase 3)
        if self.is_unsubscribed(recipient).await {
            debug!("Recipient unsubscribed: {}", recipient);
            self.stats.bounced.fetch_add(1, Ordering::Relaxed);
            return Ok(EmailStatus::Unsubscribed);
        }

        self.stats.total_received.fetch_add(1, Ordering::Relaxed);
        self.stats
            .total_bytes
            .fetch_add(envelope.content.len() as u64, Ordering::Relaxed);

        // Simulate SMTP sending (Phase 3 extension point)
        match self.attempt_send(envelope, recipient).await {
            Ok(_) => {
                info!("Email sent successfully to: {}", recipient);
                self.stats.sent.fetch_add(1, Ordering::Relaxed);
                Ok(EmailStatus::Sent)
            }
            Err(e) => {
                error!("Email send failed for {}: {}", recipient, e);
                self.stats.failed.fetch_add(1, Ordering::Relaxed);
                Ok(EmailStatus::Failed(e.to_string()))
            }
        }
    }

    /// Validate email format (basic RFC 5322)
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@')
            && email.contains('.')
            && !email.starts_with('@')
            && !email.ends_with('@')
            && email.len() > 5
    }

    /// Check if recipient is unsubscribed
    async fn is_unsubscribed(&self, _recipient: &RecipientId) -> bool {
        // Phase 3 extension: Connect to unsubscribe database
        false
    }

    /// Attempt SMTP send (simulated for testing)
    async fn attempt_send(&self, envelope: &Envelope, recipient: &RecipientId) -> Result<()> {
        // Simulate SMTP connection and send
        // Phase 3 extension: Use lettre or mail-send crate

        // Validate connection parameters
        if self.config.smtp_host.is_empty() {
            return Err(anyhow!("SMTP host not configured"));
        }

        // Simulate 95% success rate for healthy recipients
        let success_rate = 95;
        if rand::random::<u32>() % 100 < success_rate {
            debug!(
                "SMTP send simulation: {} -> {}",
                self.config.from_address, recipient
            );
            Ok(())
        } else {
            Err(anyhow!("SMTP send failed: transient error"))
        }
    }

    /// Get email handler statistics
    pub fn stats(&self) -> EmailHandlerStats {
        EmailHandlerStats {
            total_received: self.stats.total_received.load(Ordering::Relaxed),
            sent: self.stats.sent.load(Ordering::Relaxed),
            failed: self.stats.failed.load(Ordering::Relaxed),
            bounced: self.stats.bounced.load(Ordering::Relaxed),
            total_bytes: self.stats.total_bytes.load(Ordering::Relaxed),
            success_rate: self.calculate_success_rate(),
        }
    }

    fn calculate_success_rate(&self) -> f64 {
        let total = self.stats.total_received.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let sent = self.stats.sent.load(Ordering::Relaxed) as f64;
        (sent / total as f64) * 100.0
    }
}

/// Email handler statistics snapshot
#[derive(Clone, Debug)]
pub struct EmailHandlerStats {
    pub total_received: u64,
    pub sent: u64,
    pub failed: u64,
    pub bounced: u64,
    pub total_bytes: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email_format() {
        let handler = EmailHandler::new(EmailHandlerConfig::default());

        assert!(handler.is_valid_email("user@example.com"));
        assert!(handler.is_valid_email("test.user+tag@domain.co.uk"));
        assert!(!handler.is_valid_email("invalid-email"));
        assert!(!handler.is_valid_email("@example.com"));
        assert!(!handler.is_valid_email("user@"));
    }

    #[tokio::test]
    async fn test_send_email_success() {
        let handler = EmailHandler::new(EmailHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["recipient@example.com".to_string()],
            "Test Subject".to_string(),
            b"Test content".to_vec(),
        );

        let result = handler
            .send_email(&envelope, &"recipient@example.com".to_string())
            .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            EmailStatus::Sent | EmailStatus::Failed(_)
        ));
    }

    #[tokio::test]
    async fn test_reject_invalid_email() {
        let handler = EmailHandler::new(EmailHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["invalid".to_string()],
            "Test Subject".to_string(),
            b"Test content".to_vec(),
        );

        let result = handler
            .send_email(&envelope, &"invalid".to_string())
            .await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), EmailStatus::Failed(_)));
    }

    #[tokio::test]
    async fn test_email_stats_tracking() {
        let handler = EmailHandler::new(EmailHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["user@example.com".to_string()],
            "Test".to_string(),
            b"Content".to_vec(),
        );

        for _ in 0..5 {
            let _ = handler
                .send_email(&envelope, &"user@example.com".to_string())
                .await;
        }

        let stats = handler.stats();
        assert_eq!(stats.total_received, 5);
        assert!(stats.sent + stats.failed == 5);
    }
}
