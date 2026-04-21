// Admin Metrics - Tracks tenant usage and limits
use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMetrics {
    pub tenant_id: String,
    pub current_rps: f64,
    pub peak_rps: f64,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub current_connections: usize,
    pub peak_connections: usize,
    pub total_messages_processed: u64,
    pub total_bytes_processed: u64,
    pub avg_message_size: f64,
    pub largest_message_size: u64,
    pub rate_limit_violations: u64,
    pub uptime_seconds: u64,
    pub last_request_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsWindow {
    pub timestamp: DateTime<Utc>,
    pub rps: f64,
    pub active_connections: usize,
    pub errors: u64,
}

/// Tracks metrics for all tenants
pub struct MetricsCollector {
    metrics: Arc<DashMap<String, TenantMetrics>>,
    windows: Arc<DashMap<String, Vec<MetricsWindow>>>,
    max_windows: usize,
}

impl MetricsCollector {
    pub fn new(max_windows: usize) -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
            windows: Arc::new(DashMap::new()),
            max_windows,
        }
    }

    /// Initialize metrics for a new tenant
    pub fn init_tenant(&self, tenant_id: &str) {
        if !self.metrics.contains_key(tenant_id) {
            self.metrics.insert(
                tenant_id.to_string(),
                TenantMetrics {
                    tenant_id: tenant_id.to_string(),
                    current_rps: 0.0,
                    peak_rps: 0.0,
                    total_requests: 0,
                    failed_requests: 0,
                    current_connections: 0,
                    peak_connections: 0,
                    total_messages_processed: 0,
                    total_bytes_processed: 0,
                    avg_message_size: 0.0,
                    largest_message_size: 0,
                    rate_limit_violations: 0,
                    uptime_seconds: 0,
                    last_request_time: None,
                    created_at: Utc::now(),
                },
            );
        }
    }

    /// Record a request
    pub fn record_request(
        &self,
        tenant_id: &str,
        duration_ms: f64,
        message_size: u64,
        success: bool,
    ) {
        self.init_tenant(tenant_id);

        if let Some(mut m) = self.metrics.get_mut(tenant_id) {
            m.total_requests += 1;
            m.total_bytes_processed += message_size;
            m.total_messages_processed += 1;
            m.last_request_time = Some(Utc::now());

            // Update average message size
            if m.total_messages_processed > 0 {
                m.avg_message_size =
                    m.total_bytes_processed as f64 / m.total_messages_processed as f64;
            }

            // Track largest message
            if message_size > m.largest_message_size {
                m.largest_message_size = message_size;
            }

            if !success {
                m.failed_requests += 1;
            }
        }
    }

    /// Record active connection count
    pub fn record_connection_count(&self, tenant_id: &str, count: usize) {
        self.init_tenant(tenant_id);

        if let Some(mut m) = self.metrics.get_mut(tenant_id) {
            m.current_connections = count;
            if count > m.peak_connections {
                m.peak_connections = count;
            }
        }
    }

    /// Record RPS measurement
    pub fn record_rps(&self, tenant_id: &str, rps: f64) {
        self.init_tenant(tenant_id);

        let window = {
            let mut m = self.metrics.get_mut(tenant_id).unwrap();
            m.current_rps = rps;
            if rps > m.peak_rps {
                m.peak_rps = rps;
            }

            // Capture values for window
            MetricsWindow {
                timestamp: Utc::now(),
                rps,
                active_connections: m.current_connections,
                errors: m.failed_requests,
            }
        };

        // Update windows
        self.windows
            .entry(tenant_id.to_string())
            .or_insert_with(Vec::new)
            .push(window);

        // Maintain max_windows size
        if let Some(mut windows) = self.windows.get_mut(tenant_id) {
            if windows.len() > self.max_windows {
                windows.remove(0);
            }
        }
    }

    /// Record rate limit violation
    pub fn record_rate_limit_violation(&self, tenant_id: &str) {
        self.init_tenant(tenant_id);

        if let Some(mut m) = self.metrics.get_mut(tenant_id) {
            m.rate_limit_violations += 1;
        }
    }

    /// Get metrics for a tenant
    pub fn get_metrics(&self, tenant_id: &str) -> Option<TenantMetrics> {
        self.metrics.get(tenant_id).map(|m| m.clone())
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<TenantMetrics> {
        self.metrics.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get metrics history for a tenant
    pub fn get_metrics_history(&self, tenant_id: &str) -> Vec<MetricsWindow> {
        self.windows
            .get(tenant_id)
            .map(|w| w.clone())
            .unwrap_or_default()
    }

    /// Get tenant usage statistics
    pub fn get_usage_stats(&self, tenant_id: &str) -> Option<UsageStats> {
        self.get_metrics(tenant_id).map(|m| {
            UsageStats {
                tenant_id: m.tenant_id.clone(),
                rps_current: m.current_rps,
                rps_peak: m.peak_rps,
                total_requests: m.total_requests,
                total_bytes: m.total_bytes_processed,
                current_connections: m.current_connections,
                peak_connections: m.peak_connections,
                failed_requests: m.failed_requests,
                failure_rate: if m.total_requests > 0 {
                    (m.failed_requests as f64 / m.total_requests as f64) * 100.0
                } else {
                    0.0
                },
                rate_limit_violations: m.rate_limit_violations,
                avg_message_size_bytes: m.avg_message_size as u64,
                largest_message_bytes: m.largest_message_size,
            }
        })
    }

    /// Reset metrics for a tenant
    pub fn reset_metrics(&self, tenant_id: &str) {
        self.metrics.remove(tenant_id);
        self.windows.remove(tenant_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub tenant_id: String,
    pub rps_current: f64,
    pub rps_peak: f64,
    pub total_requests: u64,
    pub total_bytes: u64,
    pub current_connections: usize,
    pub peak_connections: usize,
    pub failed_requests: u64,
    pub failure_rate: f64,
    pub rate_limit_violations: u64,
    pub avg_message_size_bytes: u64,
    pub largest_message_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new(100);

        collector.record_request("tenant-1", 25.5, 1024, true);
        collector.record_rps("tenant-1", 100.0);
        collector.record_connection_count("tenant-1", 10);

        let metrics = collector.get_metrics("tenant-1").unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.total_bytes_processed, 1024);
        assert_eq!(metrics.current_connections, 10);
        assert_eq!(metrics.current_rps, 100.0);
    }

    #[test]
    fn test_rate_limit_tracking() {
        let collector = MetricsCollector::new(100);

        collector.record_rate_limit_violation("tenant-1");
        collector.record_rate_limit_violation("tenant-1");

        let metrics = collector.get_metrics("tenant-1").unwrap();
        assert_eq!(metrics.rate_limit_violations, 2);
    }
}
