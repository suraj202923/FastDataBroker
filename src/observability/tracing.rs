// Phase 5: Distributed Tracing Module (Enhanced Observability)
use std::sync::Arc;
use tracing::Span;
use anyhow::Result;

/// Tracing initialization configuration
#[derive(Debug, Clone)]
pub struct TracingInit {
    /// Service name for tracing
    pub service_name: String,
    /// Enable JSON output for structured logging
    pub json_output: bool,
    /// Environment filter (e.g., "info,postoffice=debug")
    pub env_filter: String,
    /// Sample rate 0.0-1.0 (not actively used but reserved)
    pub sample_rate: f64,
}

impl Default for TracingInit {
    fn default() -> Self {
        Self {
            service_name: "postoffice".to_string(),
            json_output: true,
            env_filter: "info".to_string(),
            sample_rate: 1.0,
        }
    }
}

/// Initialize tracing infrastructure
pub fn init_tracing(config: TracingInit) -> Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.env_filter))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if config.json_output {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .try_init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer())
            .try_init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;
    }

    tracing::info!(
        "Tracing initialized for service: {}",
        config.service_name
    );

    Ok(())
}

/// Get current span for tracing context
pub fn get_tracer() -> Arc<dyn std::any::Any + Send + Sync> {
    Arc::new(())
}

/// Trace wrapper for async operations with custom attributes
pub struct SpanBuilder {
    name: String,
    attributes: Vec<(String, String)>,
}

impl SpanBuilder {
    /// Create new span builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
        }
    }

    /// Add attribute to span
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((key.into(), value.into()));
        self
    }

    /// Build and execute operation
    pub async fn execute<F, T>(&self, operation: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        use tracing::Instrument;
        let span = tracing::info_span!(
            "operation",
            name = %self.name,
            attr_count = %self.attributes.len()
        );
        operation.instrument(span).await
    }
}

/// Distributed trace context for cross-service calls
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace ID for correlating requests
    pub trace_id: String,
    /// Span ID within trace
    pub span_id: String,
    /// Parent span ID if nested
    pub parent_span_id: Option<String>,
    /// Service name that originated trace
    pub origin_service: String,
}

impl TraceContext {
    /// Create new trace context
    pub fn new(service_name: impl Into<String>) -> Self {
        use uuid::Uuid;
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            origin_service: service_name.into(),
        }
    }

    /// Create child span context
    pub fn create_child_span(&self) -> Self {
        use uuid::Uuid;
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            origin_service: self.origin_service.clone(),
        }
    }

    /// Log structured event with context
    pub fn log_event(&self, event: impl Into<String>) {
        tracing::info!(
            trace_id = %self.trace_id,
            span_id = %self.span_id,
            origin_service = %self.origin_service,
            event = %event.into(),
            "trace_event"
        );
    }
}

/// Macro for easy span creation
#[macro_export]
macro_rules! traced_operation {
    ($operation:expr => $code:block) => {{
        use tracing::Instrument;
        let span = tracing::debug_span!($operation);
        async { $code }.instrument(span).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingInit::default();
        assert_eq!(config.service_name, "postoffice");
        assert!(config.json_output);
        assert_eq!(config.sample_rate, 1.0);
    }

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("test_span")
            .with_attribute("key1", "value1")
            .with_attribute("key2", "value2");
        
        assert_eq!(span.name, "test_span");
        assert_eq!(span.attributes.len(), 2);
    }

    #[test]
    fn test_trace_context_creation() {
        let ctx = TraceContext::new("postoffice");
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
        assert_eq!(ctx.origin_service, "postoffice");
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_trace_context_child_span() {
        let parent = TraceContext::new("postoffice");
        let child = parent.create_child_span();
        
        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id.clone()));
    }
}

