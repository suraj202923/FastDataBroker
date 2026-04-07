// Push notification handler (Firebase Cloud Messaging & APNs)
use crate::models::{Envelope, RecipientId};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Push notification platform
#[derive(Clone, Debug, PartialEq)]
pub enum PushPlatform {
    Firebase,
    ApplePushNotification,
    GooglePlayServices,
    WebPush,
}

/// Push handler configuration
#[derive(Clone, Debug)]
pub struct PushHandlerConfig {
    pub firebase_api_key: String,
    pub firebase_project_id: String,
    pub apns_certificate_path: String,
    pub apns_team_id: String,
    pub apns_key_id: String,
    pub max_retry_count: u32,
    pub request_timeout_ms: u64,
    pub batch_size: usize,
}

impl Default for PushHandlerConfig {
    fn default() -> Self {
        Self {
            firebase_api_key: String::new(),
            firebase_project_id: "default-project".to_string(),
            apns_certificate_path: String::new(),
            apns_team_id: String::new(),
            apns_key_id: String::new(),
            max_retry_count: 3,
            request_timeout_ms: 30000,
            batch_size: 100,
        }
    }
}

/// Push notification status
#[derive(Clone, Debug, PartialEq)]
pub enum PushStatus {
    Pending,
    Sent,
    Delivered,
    Failed(String),
    InvalidToken,
    RateLimited,
}

/// Push notification record
#[derive(Clone, Debug)]
pub struct PushRecord {
    pub device_token: String,
    pub platform: PushPlatform,
    pub attempts: u32,
    pub status: PushStatus,
}

/// Push handler for mobile notifications
pub struct PushHandler {
    config: PushHandlerConfig,
    stats: PushStats,
}

/// Push handler statistics
#[derive(Clone, Debug, Default)]
pub struct PushStats {
    total_received: Arc<AtomicU64>,
    sent: Arc<AtomicU64>,
    delivered: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
    invalid_tokens: Arc<AtomicU64>,
    rate_limited: Arc<AtomicU64>,
}

impl PushHandler {
    /// Create new push handler
    pub fn new(config: PushHandlerConfig) -> Self {
        info!(
            "Initializing PushHandler - Firebase: {}, APNs: {}",
            config.firebase_project_id, config.apns_team_id
        );

        Self {
            config,
            stats: PushStats::default(),
        }
    }

    /// Send push notification to device
    pub async fn send_push(
        &self,
        envelope: &Envelope,
        device_token: &str,
        platform: &PushPlatform,
    ) -> Result<PushStatus> {
        debug!(
            "Sending push notification via {:?} to: {}, msg_id: {}",
            platform, device_token, envelope.id
        );

        // Validate device token format
        if !self.is_valid_token(device_token) {
            warn!("Invalid device token: {}", device_token);
            self.stats.invalid_tokens.fetch_add(1, Ordering::Relaxed);
            return Ok(PushStatus::InvalidToken);
        }

        self.stats.total_received.fetch_add(1, Ordering::Relaxed);

        // Attempt send based on platform
        match platform {
            PushPlatform::Firebase => self.send_firebase(envelope, device_token).await,
            PushPlatform::ApplePushNotification => self.send_apns(envelope, device_token).await,
            PushPlatform::GooglePlayServices => self.send_fcm(envelope, device_token).await,
            PushPlatform::WebPush => self.send_webpush(envelope, device_token).await,
        }
    }

    /// Send via Firebase Cloud Messaging
    async fn send_firebase(&self, envelope: &Envelope, token: &str) -> Result<PushStatus> {
        if self.config.firebase_api_key.is_empty() {
            warn!("Firebase API key not configured");
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            return Ok(PushStatus::Failed("Firebase not configured".to_string()));
        }

        // Simulate Firebase send
        if self.attempt_send(envelope, token).await? {
            info!("Firebase push sent to: {}", token);
            self.stats.sent.fetch_add(1, Ordering::Relaxed);
            self.stats.delivered.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Delivered)
        } else {
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Failed("Firebase send failed".to_string()))
        }
    }

    /// Send via Apple Push Notification service
    async fn send_apns(&self, envelope: &Envelope, token: &str) -> Result<PushStatus> {
        if self.config.apns_certificate_path.is_empty() {
            warn!("APNs certificate not configured");
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            return Ok(PushStatus::Failed("APNs not configured".to_string()));
        }

        // Simulate APNs send
        if self.attempt_send(envelope, token).await? {
            info!("APNs push sent to: {}", token);
            self.stats.sent.fetch_add(1, Ordering::Relaxed);
            self.stats.delivered.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Delivered)
        } else {
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Failed("APNs send failed".to_string()))
        }
    }

    /// Send via Google FCM
    async fn send_fcm(&self, envelope: &Envelope, token: &str) -> Result<PushStatus> {
        // Simulate FCM send
        if self.attempt_send(envelope, token).await? {
            info!("FCM push sent to: {}", token);
            self.stats.sent.fetch_add(1, Ordering::Relaxed);
            self.stats.delivered.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Delivered)
        } else {
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Failed("FCM send failed".to_string()))
        }
    }

    /// Send via Web Push API
    async fn send_webpush(&self, envelope: &Envelope, token: &str) -> Result<PushStatus> {
        // Simulate Web Push send
        if self.attempt_send(envelope, token).await? {
            info!("Web Push sent to: {}", token);
            self.stats.sent.fetch_add(1, Ordering::Relaxed);
            self.stats.delivered.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Delivered)
        } else {
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            Ok(PushStatus::Failed("Web Push failed".to_string()))
        }
    }

    /// Validate device token format
    fn is_valid_token(&self, token: &str) -> bool {
        // Basic validation: token should be alphanumeric + common special chars and reasonable length
        !token.is_empty() && token.len() > 10 && token.len() < 1000 && token.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '=' || c == '/' || c == '+')
    }

    /// Attempt to send push (with simulated rate limiting)
    async fn attempt_send(&self, _envelope: &Envelope, _token: &str) -> Result<bool> {
        // Simulate 92% success rate (some rate limiting)
        let success_rate = 92;
        if rand::random::<u32>() % 100 < success_rate {
            Ok(true)
        } else {
            self.stats.rate_limited.fetch_add(1, Ordering::Relaxed);
            Ok(false)
        }
    }

    /// Batch send push notifications
    pub async fn batch_send(
        &self,
        envelope: &Envelope,
        tokens: Vec<&str>,
        platform: &PushPlatform,
    ) -> Result<(usize, usize)> {
        let mut sent = 0;
        let mut failed = 0;

        for token in tokens.iter().take(self.config.batch_size) {
            match self.send_push(envelope, token, platform).await? {
                PushStatus::Delivered | PushStatus::Sent => sent += 1,
                _ => failed += 1,
            }
        }

        info!(
            "Batch push send: {} sent, {} failed",
            sent, failed
        );

        Ok((sent, failed))
    }

    /// Get push handler statistics
    pub fn stats(&self) -> PushHandlerStats {
        PushHandlerStats {
            total_received: self.stats.total_received.load(Ordering::Relaxed),
            sent: self.stats.sent.load(Ordering::Relaxed),
            delivered: self.stats.delivered.load(Ordering::Relaxed),
            failed: self.stats.failed.load(Ordering::Relaxed),
            invalid_tokens: self.stats.invalid_tokens.load(Ordering::Relaxed),
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

/// Push handler statistics snapshot
#[derive(Clone, Debug)]
pub struct PushHandlerStats {
    pub total_received: u64,
    pub sent: u64,
    pub delivered: u64,
    pub failed: u64,
    pub invalid_tokens: u64,
    pub rate_limited: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_device_token() {
        let handler = PushHandler::new(PushHandlerConfig::default());

        assert!(handler.is_valid_token("abcd1234567890"));
        assert!(handler.is_valid_token("firebase_token_12345:aGVsbG8="));
        assert!(!handler.is_valid_token(""));
        assert!(!handler.is_valid_token("short"));
    }

    #[tokio::test]
    async fn test_send_firebase_push() {
        let mut config = PushHandlerConfig::default();
        config.firebase_api_key = "test-key".to_string();
        let handler = PushHandler::new(config);

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["device-1".to_string()],
            "Notification".to_string(),
            b"You have a new message".to_vec(),
        );

        let result = handler
            .send_push(&envelope, "android_device_token_12345", &PushPlatform::Firebase)
            .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            PushStatus::Delivered | PushStatus::Failed(_)
        ));
    }

    #[tokio::test]
    async fn test_send_apns_push() {
        let mut config = PushHandlerConfig::default();
        config.apns_certificate_path = "/path/to/cert.p8".to_string();
        let handler = PushHandler::new(config);

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["device-1".to_string()],
            "Notification".to_string(),
            b"Message for iOS".to_vec(),
        );

        let result = handler
            .send_push(&envelope, "ios_device_token_12345", &PushPlatform::ApplePushNotification)
            .await;

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            PushStatus::Delivered | PushStatus::Failed(_)
        ));
    }

    #[tokio::test]
    async fn test_reject_invalid_token() {
        let handler = PushHandler::new(PushHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["device".to_string()],
            "Test".to_string(),
            b"Content".to_vec(),
        );

        let result = handler
            .send_push(&envelope, "short", &PushPlatform::Firebase)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PushStatus::InvalidToken);
    }

    #[tokio::test]
    async fn test_batch_send_push() {
        let handler = PushHandler::new(PushHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["batch".to_string()],
            "Batch".to_string(),
            b"Message".to_vec(),
        );

        let tokens = vec!["token1_1234567890", "token2_1234567890", "token3_1234567890"];

        let result = handler
            .batch_send(&envelope, tokens, &PushPlatform::Firebase)
            .await;

        assert!(result.is_ok());
        let (sent, failed) = result.unwrap();
        assert!(sent + failed == 3);
    }

    #[tokio::test]
    async fn test_push_stats_tracking() {
        let handler = PushHandler::new(PushHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["device".to_string()],
            "Test".to_string(),
            b"Content".to_vec(),
        );

        for _ in 0..5 {
            let _ = handler
                .send_push(&envelope, "device_token_1234567890", &PushPlatform::Firebase)
                .await;
        }

        let stats = handler.stats();
        assert_eq!(stats.total_received, 5);
    }
}
