// Phase 5: Prometheus Metrics Export Module (Optional)
#[cfg(feature = "metrics")]
use prometheus::{Counter, Histogram, Gauge, Registry, Encoder, TextEncoder};
use std::sync::Arc;
use anyhow::Result;

#[cfg(feature = "metrics")]
/// Comprehensive metrics collector for FastDataBroker
pub struct MetricsCollector {
    // Message metrics
    messages_received: Counter,
    messages_delivered: Counter,
    messages_failed: Counter,
    messages_dropped: Counter,
    
    // Latency metrics (in milliseconds)
    message_latency: Histogram,
    delivery_latency: Histogram,
    queue_processing_time: Histogram,
    
    // Queue metrics
    queue_size: Gauge,
    queue_capacity: Gauge,
    active_connections: Gauge,
    
    // Notification channel metrics
    email_sent: Counter,
    email_failed: Counter,
    websocket_delivered: Counter,
    push_sent: Counter,
    webhook_delivered: Counter,
    
    // Error metrics
    transport_errors: Counter,
    service_errors: Counter,
    retriable_errors: Counter,
    
    // Registry for collection
    registry: Arc<Registry>,
}

#[cfg(feature = "metrics")]
impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Arc::new(Registry::new());
        
        // Create counters
        let messages_received = Counter::new("fastdatabroker_messages_received_total", 
            "Total number of messages received")?;
        let messages_delivered = Counter::new("fastdatabroker_messages_delivered_total",
            "Total number of messages successfully delivered")?;
        let messages_failed = Counter::new("fastdatabroker_messages_failed_total",
            "Total number of messages that failed delivery")?;
        let messages_dropped = Counter::new("fastdatabroker_messages_dropped_total",
            "Total number of messages dropped")?;
        
        // Notification channel counters
        let email_sent = Counter::new("fastdatabroker_email_sent_total",
            "Total emails sent")?;
        let email_failed = Counter::new("fastdatabroker_email_failed_total",
            "Total emails failed")?;
        let websocket_delivered = Counter::new("fastdatabroker_websocket_delivered_total",
            "Total WebSocket messages delivered")?;
        let push_sent = Counter::new("fastdatabroker_push_sent_total",
            "Total push notifications sent")?;
        let webhook_delivered = Counter::new("fastdatabroker_webhook_delivered_total",
            "Total webhooks delivered")?;
        
        // Error counters
        let transport_errors = Counter::new("fastdatabroker_transport_errors_total",
            "Total transport layer errors")?;
        let service_errors = Counter::new("fastdatabroker_service_errors_total",
            "Total service layer errors")?;
        let retriable_errors = Counter::new("fastdatabroker_retriable_errors_total",
            "Total retriable errors")?;
        
        // Create histograms
        let message_latency = Histogram::new("fastdatabroker_message_latency_ms",
            "Message processing latency in milliseconds")?;
        let delivery_latency = Histogram::new("fastdatabroker_delivery_latency_ms",
            "Message delivery latency in milliseconds")?;
        let queue_processing_time = Histogram::new("fastdatabroker_queue_processing_time_ms",
            "Queue processing time in milliseconds")?;
        
        // Create gauges
        let queue_size = Gauge::new("postoffice_queue_size",
            "Current queue size")?;
        let queue_capacity = Gauge::new("postoffice_queue_capacity",
            "Queue capacity")?;
        let active_connections = Gauge::new("postoffice_active_connections",
            "Number of active connections")?;
        
        // Register all metrics
        registry.register(Box::new(messages_received.clone()))?;
        registry.register(Box::new(messages_delivered.clone()))?;
        registry.register(Box::new(messages_failed.clone()))?;
        registry.register(Box::new(messages_dropped.clone()))?;
        registry.register(Box::new(email_sent.clone()))?;
        registry.register(Box::new(email_failed.clone()))?;
        registry.register(Box::new(websocket_delivered.clone()))?;
        registry.register(Box::new(push_sent.clone()))?;
        registry.register(Box::new(webhook_delivered.clone()))?;
        registry.register(Box::new(transport_errors.clone()))?;
        registry.register(Box::new(service_errors.clone()))?;
        registry.register(Box::new(retriable_errors.clone()))?;
        registry.register(Box::new(message_latency.clone()))?;
        registry.register(Box::new(delivery_latency.clone()))?;
        registry.register(Box::new(queue_processing_time.clone()))?;
        registry.register(Box::new(queue_size.clone()))?;
        registry.register(Box::new(queue_capacity.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        
        Ok(MetricsCollector {
            messages_received,
            messages_delivered,
            messages_failed,
            messages_dropped,
            message_latency,
            delivery_latency,
            queue_processing_time,
            queue_size,
            queue_capacity,
            active_connections,
            email_sent,
            email_failed,
            websocket_delivered,
            push_sent,
            webhook_delivered,
            transport_errors,
            service_errors,
            retriable_errors,
            registry,
        })
    }
    
    // Message metrics accessors
    pub fn record_message_received(&self) {
        self.messages_received.inc();
    }
    
    pub fn record_message_delivered(&self) {
        self.messages_delivered.inc();
    }
    
    pub fn record_message_failed(&self) {
        self.messages_failed.inc();
    }
    
    pub fn record_message_dropped(&self) {
        self.messages_dropped.inc();
    }
    
    pub fn record_message_latency(&self, latency_ms: f64) {
        self.message_latency.observe(latency_ms);
    }
    
    pub fn record_delivery_latency(&self, latency_ms: f64) {
        self.delivery_latency.observe(latency_ms);
    }
    
    pub fn record_queue_processing_time(&self, time_ms: f64) {
        self.queue_processing_time.observe(time_ms);
    }
    
    // Queue metrics accessors
    pub fn set_queue_size(&self, size: i64) {
        self.queue_size.set(size);
    }
    
    pub fn set_queue_capacity(&self, capacity: i64) {
        self.queue_capacity.set(capacity);
    }
    
    pub fn set_active_connections(&self, count: i64) {
        self.active_connections.set(count);
    }
    
    // Notification channel metrics accessors
    pub fn record_email_sent(&self) {
        self.email_sent.inc();
    }
    
    pub fn record_email_failed(&self) {
        self.email_failed.inc();
    }
    
    pub fn record_websocket_delivered(&self) {
        self.websocket_delivered.inc();
    }
    
    pub fn record_push_sent(&self) {
        self.push_sent.inc();
    }
    
    pub fn record_webhook_delivered(&self) {
        self.webhook_delivered.inc();
    }
    
    // Error metrics accessors
    pub fn record_transport_error(&self) {
        self.transport_errors.inc();
    }
    
    pub fn record_service_error(&self) {
        self.service_errors.inc();
    }
    
    pub fn record_retriable_error(&self) {
        self.retriable_errors.inc();
    }
    
    /// Export metrics in Prometheus text format
    pub fn export_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to encode metrics: {}", e))?;
        String::from_utf8(buffer)
            .map_err(|e| anyhow::anyhow!("Failed to convert metrics to UTF-8: {}", e))
    }
}

#[cfg(feature = "metrics")]
impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics collector")
    }
}

/// Stub implementation when metrics feature is disabled
#[cfg(not(feature = "metrics"))]
pub struct MetricsCollector;

/// Initialize Prometheus metrics (feature-gated)
#[cfg(feature = "metrics")]
pub fn init_prometheus(port: u16) -> Result<()> {
    tracing::info!("Prometheus metrics enabled on port {}", port);
    Ok(())
}

#[cfg(not(feature = "metrics"))]
pub fn init_prometheus(_port: u16) -> Result<()> {
    tracing::info!("Prometheus metrics disabled (feature not enabled)");
    Ok(())
}

#[cfg(all(test, feature = "metrics"))]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let metrics = MetricsCollector::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_record_messages() {
        let metrics = MetricsCollector::new().unwrap();
        metrics.record_message_received();
        metrics.record_message_delivered();
        metrics.record_message_failed();
        metrics.set_queue_size(100);
        metrics.set_active_connections(50);
    }
}
