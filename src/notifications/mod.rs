// Notification system orchestrating all channels
pub mod email;
pub mod push;
pub mod websocket;
pub mod webhook;

use crate::models::{Envelope, RecipientId, MessageId};
use anyhow::Result;
use email::{EmailHandler, EmailHandlerConfig, EmailStatus};
use push::{PushHandler, PushHandlerConfig, PushPlatform, PushStatus};
use websocket::{WebSocketHandler, WebSocketHandlerConfig, WebSocketStatus};
use webhook::{WebhookHandler, WebhookHandlerConfig, WebhookStatus, WebhookConfig};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Notification delivery channel
#[derive(Clone, Debug, PartialEq)]
pub enum NotificationChannel {
    Email,
    WebSocket,
    Push(PushPlatform),
    Webhook,
}

/// Notification broker orchestrating all delivery channels
pub struct NotificationBroker {
    email_handler: Arc<EmailHandler>,
    websocket_handler: Arc<WebSocketHandler>,
    push_handler: Arc<PushHandler>,
    webhook_handler: Arc<WebhookHandler>,
}

impl NotificationBroker {
    /// Create new notification broker with default configuration
    pub fn new() -> Self {
        Self::with_config(
            EmailHandlerConfig::default(),
            WebSocketHandlerConfig::default(),
            PushHandlerConfig::default(),
            WebhookHandlerConfig::default(),
        )
    }

    /// Create notification broker with custom configuration
    pub fn with_config(
        email_config: EmailHandlerConfig,
        websocket_config: WebSocketHandlerConfig,
        push_config: PushHandlerConfig,
        webhook_config: WebhookHandlerConfig,
    ) -> Self {
        info!("Initializing NotificationBroker with 4 channels");

        Self {
            email_handler: Arc::new(EmailHandler::new(email_config)),
            websocket_handler: Arc::new(WebSocketHandler::new(websocket_config)),
            push_handler: Arc::new(PushHandler::new(push_config)),
            webhook_handler: Arc::new(WebhookHandler::new(webhook_config)),
        }
    }

    /// Deliver message through all available channels
    pub async fn deliver_omnibus(
        &self,
        envelope: &Envelope,
        recipient: &RecipientId,
    ) -> Result<OmnibusDeliveryResult> {
        debug!(
            "Omnibus delivery for: {}, msg_id: {}",
            recipient, envelope.id
        );

        let mut email_result = None;
        let mut websocket_result = None;
        let mut push_results = Vec::new();
        let mut webhook_result = None;

        // Email delivery
        if let Ok(status) = self
            .email_handler
            .send_email(envelope, recipient)
            .await
        {
            email_result = Some(status);
        }

        // WebSocket delivery (to connected clients)
        if let Ok(status) = self
            .websocket_handler
            .deliver_message(envelope, recipient)
            .await
        {
            websocket_result = Some(status);
        }

        // Push notifications (to registered devices)
        for platform in &[PushPlatform::Firebase, PushPlatform::ApplePushNotification] {
            if let Ok(status) = self
                .push_handler
                .send_push(envelope, &format!("{}_token", recipient), platform)
                .await
            {
                push_results.push((platform.clone(), status));
            }
        }

        // Webhook delivery
        if let Ok(status) = self
            .webhook_handler
            .send_webhook(envelope, recipient)
            .await
        {
            webhook_result = Some(status);
        }

        // Count delivered channels before moving values
        let delivered_channels = self.count_delivered(&email_result, &websocket_result, &push_results, &webhook_result);

        Ok(OmnibusDeliveryResult {
            message_id: envelope.id.clone(),
            recipient: recipient.clone(),
            email: email_result,
            websocket: websocket_result,
            push: push_results,
            webhook: webhook_result,
            delivered_channels,
        })
    }

    /// Deliver through specific channel
    pub async fn deliver_channel(
        &self,
        envelope: &Envelope,
        recipient: &RecipientId,
        channel: &NotificationChannel,
    ) -> Result<ChannelDeliveryResult> {
        debug!(
            "Channel delivery via {:?} to: {}, msg_id: {}",
            channel, recipient, envelope.id
        );

        let result = match channel {
            NotificationChannel::Email => {
                let status = self.email_handler.send_email(envelope, recipient).await?;
                ChannelDeliveryResult {
                    channel: channel.clone(),
                    success: matches!(status, EmailStatus::Sent),
                    details: format!("{:?}", status),
                }
            }
            NotificationChannel::WebSocket => {
                let status = self
                    .websocket_handler
                    .deliver_message(envelope, recipient)
                    .await?;
                ChannelDeliveryResult {
                    channel: channel.clone(),
                    success: matches!(status, WebSocketStatus::Delivered),
                    details: format!("{:?}", status),
                }
            }
            NotificationChannel::Push(platform) => {
                let status = self
                    .push_handler
                    .send_push(envelope, &format!("{}_token", recipient), platform)
                    .await?;
                ChannelDeliveryResult {
                    channel: channel.clone(),
                    success: matches!(status, PushStatus::Delivered),
                    details: format!("{:?}", status),
                }
            }
            NotificationChannel::Webhook => {
                let status = self
                    .webhook_handler
                    .send_webhook(envelope, recipient)
                    .await?;
                ChannelDeliveryResult {
                    channel: channel.clone(),
                    success: matches!(status, WebhookStatus::Delivered),
                    details: format!("{:?}", status),
                }
            }
        };

        Ok(result)
    }

    /// Register webhook endpoint
    pub fn register_webhook(&self, recipient: RecipientId, config: WebhookConfig) -> Result<()> {
        self.webhook_handler.register_webhook(recipient, config)
    }

    /// Unregister webhook endpoint
    pub fn unregister_webhook(&self, recipient: &RecipientId) -> Result<()> {
        self.webhook_handler.unregister_webhook(recipient)
    }

    /// Register WebSocket client
    pub async fn register_websocket_client(
        &self,
        client_id: String,
        recipient: RecipientId,
    ) -> Result<String> {
        self.websocket_handler
            .register_client(client_id, recipient)
            .await
    }

    /// Unregister WebSocket client
    pub async fn unregister_websocket_client(&self, recipient: &RecipientId) -> Result<()> {
        self.websocket_handler.unregister_client(recipient).await
    }

    /// Get notification broker statistics
    pub fn stats(&self) -> NotificationBrokerStats {
        NotificationBrokerStats {
            email_stats: self.email_handler.stats(),
            websocket_stats: self.websocket_handler.stats(),
            push_stats: self.push_handler.stats(),
            webhook_stats: self.webhook_handler.stats(),
        }
    }

    fn count_delivered(
        &self,
        email: &Option<EmailStatus>,
        websocket: &Option<WebSocketStatus>,
        push: &[(PushPlatform, PushStatus)],
        webhook: &Option<WebhookStatus>,
    ) -> usize {
        let mut count = 0;

        if let Some(EmailStatus::Sent) = email {
            count += 1;
        }

        if let Some(WebSocketStatus::Delivered) = websocket {
            count += 1;
        }

        for (_, status) in push {
            if matches!(status, PushStatus::Delivered) {
                count += 1;
            }
        }

        if let Some(WebhookStatus::Delivered) = webhook {
            count += 1;
        }

        count
    }
}

impl Default for NotificationBroker {
    fn default() -> Self {
        Self::new()
    }
}

/// Omnibus delivery result (all channels)
#[derive(Clone, Debug)]
pub struct OmnibusDeliveryResult {
    pub message_id: MessageId,
    pub recipient: RecipientId,
    pub email: Option<EmailStatus>,
    pub websocket: Option<WebSocketStatus>,
    pub push: Vec<(PushPlatform, PushStatus)>,
    pub webhook: Option<WebhookStatus>,
    pub delivered_channels: usize,
}

/// Channel-specific delivery result
#[derive(Clone, Debug)]
pub struct ChannelDeliveryResult {
    pub channel: NotificationChannel,
    pub success: bool,
    pub details: String,
}

/// Consolidated notification broker statistics
#[derive(Clone, Debug)]
pub struct NotificationBrokerStats {
    pub email_stats: email::EmailHandlerStats,
    pub websocket_stats: websocket::WebSocketHandlerStats,
    pub push_stats: push::PushHandlerStats,
    pub webhook_stats: webhook::WebhookHandlerStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broker_creation() {
        let broker = NotificationBroker::new();
        let stats = broker.stats();

        assert_eq!(stats.email_stats.total_received, 0);
        assert_eq!(stats.websocket_stats.total_received, 0);
        assert_eq!(stats.push_stats.total_received, 0);
        assert_eq!(stats.webhook_stats.total_received, 0);
    }

    #[tokio::test]
    async fn test_omnibus_delivery() {
        let broker = NotificationBroker::new();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["recipient-1".to_string()],
            "Multi-channel Test".to_string(),
            b"Test message to all channels".to_vec(),
        );

        let result = broker
            .deliver_omnibus(&envelope, &"recipient-1".to_string())
            .await;

        assert!(result.is_ok());
        let delivery = result.unwrap();
        assert_eq!(delivery.recipient, "recipient-1");
        assert!(delivery.delivered_channels >= 0);
    }

    #[tokio::test]
    async fn test_channel_specific_delivery() {
        let broker = NotificationBroker::new();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["user@example.com".to_string()],
            "Email Test".to_string(),
            b"Email content".to_vec(),
        );

        let result = broker
            .deliver_channel(&envelope, &"user@example.com".to_string(), &NotificationChannel::Email)
            .await;

        assert!(result.is_ok());
        let delivery = result.unwrap();
        assert_eq!(delivery.channel, NotificationChannel::Email);
    }

    #[tokio::test]
    async fn test_register_websocket_client() {
        let broker = NotificationBroker::new();

        let result = broker
            .register_websocket_client("client-1".to_string(), "user-123".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_webhook() {
        let broker = NotificationBroker::new();

        let config = WebhookConfig::new("https://example.com/webhook".to_string());
        let result = broker.register_webhook("service-1".to_string(), config);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broker_stats() {
        let broker = NotificationBroker::new();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["user@example.com".to_string()],
            "Test".to_string(),
            b"Content".to_vec(),
        );

        let _ = broker
            .deliver_channel(&envelope, &"user@example.com".to_string(), &NotificationChannel::Email)
            .await;

        let stats = broker.stats();
        assert!(stats.email_stats.total_received > 0);
    }
}
