// WebSocket notification handler for real-time delivery
use crate::models::{Envelope, RecipientId};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// WebSocket handler configuration
#[derive(Clone, Debug)]
pub struct WebSocketHandlerConfig {
    pub listen_addr: String,
    pub listen_port: u16,
    pub max_connections: usize,
    pub message_buffer_size: usize,
    pub heartbeat_interval_ms: u64,
    pub connection_timeout_ms: u64,
}

impl Default for WebSocketHandlerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 8080,
            max_connections: 10000,
            message_buffer_size: 1000,
            heartbeat_interval_ms: 30000,
            connection_timeout_ms: 300000,
        }
    }
}

/// WebSocket connection status
#[derive(Clone, Debug, PartialEq)]
pub enum WebSocketStatus {
    Connected,
    Delivering,
    Delivered,
    Disconnected,
    TimedOut,
    BufferFull,
}

/// WebSocket client connection info
#[derive(Clone, Debug)]
pub struct WebSocketClient {
    pub client_id: String,
    pub connected_at: u64,
    pub last_heartbeat: u64,
    pub messages_received: u64,
}

/// WebSocket handler for real-time notifications
pub struct WebSocketHandler {
    config: WebSocketHandlerConfig,
    clients: Arc<RwLock<HashMap<RecipientId, WebSocketClient>>>,
    stats: WebSocketStats,
}

/// WebSocket handler statistics
#[derive(Clone, Debug, Default)]
pub struct WebSocketStats {
    total_received: Arc<AtomicU64>,
    delivered: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
    active_connections: Arc<AtomicU64>,
    messages_buffered: Arc<AtomicU64>,
}

impl WebSocketHandler {
    /// Create new WebSocket handler
    pub fn new(config: WebSocketHandlerConfig) -> Self {
        info!(
            "Initializing WebSocketHandler - Listening on {}:{}",
            config.listen_addr, config.listen_port
        );

        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            stats: WebSocketStats::default(),
        }
    }

    /// Register a WebSocket client connection
    pub async fn register_client(
        &self,
        client_id: String,
        recipient: RecipientId,
    ) -> Result<String> {
        let mut clients = self.clients.write().await;

        // Check connection limit
        if clients.len() >= self.config.max_connections {
            return Err(anyhow!("Max WebSocket connections reached"));
        }

        let client = WebSocketClient {
            client_id: client_id.clone(),
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            last_heartbeat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            messages_received: 0,
        };

        clients.insert(recipient.clone(), client);
        self.stats.active_connections.fetch_add(1, Ordering::Relaxed);

        info!("WebSocket client registered: {}", recipient);
        Ok(recipient)
    }

    /// Unregister a WebSocket client
    pub async fn unregister_client(&self, recipient: &RecipientId) -> Result<()> {
        let mut clients = self.clients.write().await;

        if clients.remove(recipient).is_some() {
            self.stats
                .active_connections
                .fetch_sub(1, Ordering::Relaxed);
            info!("WebSocket client unregistered: {}", recipient);
            Ok(())
        } else {
            Err(anyhow!("Client not found: {}", recipient))
        }
    }

    /// Deliver message via WebSocket to connected client
    pub async fn deliver_message(
        &self,
        envelope: &Envelope,
        recipient: &RecipientId,
    ) -> Result<WebSocketStatus> {
        debug!(
            "Attempting WebSocket delivery to: {}, msg_id: {}",
            recipient, envelope.id
        );

        self.stats.total_received.fetch_add(1, Ordering::Relaxed);

        let clients = self.clients.read().await;

        // Check if client is connected
        let client = match clients.get(recipient) {
            Some(c) => c,
            None => {
                debug!("Client not connected: {}", recipient);
                return Ok(WebSocketStatus::Disconnected);
            }
        };

        // Check connection timeout
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if now - client.last_heartbeat > self.config.connection_timeout_ms {
            return Ok(WebSocketStatus::TimedOut);
        }

        // Simulate message delivery
        if self.attempt_frame_send(envelope, client).await? {
            self.stats.delivered.fetch_add(1, Ordering::Relaxed);
            debug!("WebSocket message delivered to: {}", recipient);
            Ok(WebSocketStatus::Delivered)
        } else {
            self.stats.failed.fetch_add(1, Ordering::Relaxed);
            warn!("WebSocket delivery failed for: {}", recipient);
            Ok(WebSocketStatus::BufferFull)
        }
    }

    /// Attempt to send WebSocket frame
    async fn attempt_frame_send(&self, envelope: &Envelope, _client: &WebSocketClient) -> Result<bool> {
        // Check message buffer
        let buffered = self.stats.messages_buffered.load(Ordering::Relaxed);
        if buffered >= self.config.message_buffer_size as u64 {
            return Ok(false); // Buffer full
        }

        // Simulate frame send (Phase 3: use tokio-tungstenite)
        self.stats
            .messages_buffered
            .fetch_add(envelope.content.len() as u64, Ordering::Relaxed);

        Ok(true)
    }

    /// Broadcast message to all connected clients
    pub async fn broadcast_message(&self, envelope: &Envelope) -> Result<usize> {
        let clients = self.clients.read().await;
        let mut delivered = 0;

        for recipient in clients.keys() {
            if let Ok(WebSocketStatus::Delivered) = self.deliver_message(envelope, recipient).await {
                delivered += 1;
            }
        }

        info!(
            "Broadcast message delivered to {}/{} clients",
            delivered,
            clients.len()
        );

        Ok(delivered)
    }

    /// Get current connection count
    pub async fn connection_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get WebSocket handler statistics
    pub fn stats(&self) -> WebSocketHandlerStats {
        WebSocketHandlerStats {
            total_received: self.stats.total_received.load(Ordering::Relaxed),
            delivered: self.stats.delivered.load(Ordering::Relaxed),
            failed: self.stats.failed.load(Ordering::Relaxed),
            active_connections: self.stats.active_connections.load(Ordering::Relaxed),
            messages_buffered: self.stats.messages_buffered.load(Ordering::Relaxed),
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

/// WebSocket handler statistics snapshot
#[derive(Clone, Debug)]
pub struct WebSocketHandlerStats {
    pub total_received: u64,
    pub delivered: u64,
    pub failed: u64,
    pub active_connections: u64,
    pub messages_buffered: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_client() {
        let handler = WebSocketHandler::new(WebSocketHandlerConfig::default());

        let result = handler
            .register_client("client-1".to_string(), "user-123".to_string())
            .await;

        assert!(result.is_ok());
        assert_eq!(handler.connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_unregister_client() {
        let handler = WebSocketHandler::new(WebSocketHandlerConfig::default());

        handler
            .register_client("client-1".to_string(), "user-123".to_string())
            .await
            .unwrap();

        let result = handler.unregister_client(&"user-123".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(handler.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_deliver_to_connected_client() {
        let handler = WebSocketHandler::new(WebSocketHandlerConfig::default());

        handler
            .register_client("client-1".to_string(), "user-123".to_string())
            .await
            .unwrap();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["user-123".to_string()],
            "Real-time Update".to_string(),
            b"New message available".to_vec(),
        );

        let result = handler
            .deliver_message(&envelope, &"user-123".to_string())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), WebSocketStatus::Delivered);
    }

    #[tokio::test]
    async fn test_deliver_to_disconnected_client() {
        let handler = WebSocketHandler::new(WebSocketHandlerConfig::default());

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["user-999".to_string()],
            "Update".to_string(),
            b"Message".to_vec(),
        );

        let result = handler
            .deliver_message(&envelope, &"user-999".to_string())
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), WebSocketStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let handler = WebSocketHandler::new(WebSocketHandlerConfig::default());

        // Register 3 clients
        for i in 0..3 {
            handler
                .register_client(format!("client-{}", i), format!("user-{}", i))
                .await
                .unwrap();
        }

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["broadcast".to_string()],
            "Broadcast".to_string(),
            b"To all".to_vec(),
        );

        let delivered = handler.broadcast_message(&envelope).await.unwrap();

        assert!(delivered > 0);
    }

    #[tokio::test]
    async fn test_websocket_stats_tracking() {
        let handler = WebSocketHandler::new(WebSocketHandlerConfig::default());

        handler
            .register_client("client-1".to_string(), "user-123".to_string())
            .await
            .unwrap();

        let envelope = Envelope::new(
            "producer".to_string(),
            vec!["user-123".to_string()],
            "Test".to_string(),
            b"Content".to_vec(),
        );

        for _ in 0..5 {
            let _ = handler
                .deliver_message(&envelope, &"user-123".to_string())
                .await;
        }

        let stats = handler.stats();
        assert_eq!(stats.total_received, 5);
        assert_eq!(stats.active_connections, 1);
    }
}
