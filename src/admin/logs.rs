// Admin Logging - Captures request/error logs per tenant
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub tenant_id: String,
    pub log_type: LogType,
    pub message: String,
    pub duration_ms: f64,
    pub status_code: u16,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogType {
    #[serde(rename = "request")]
    Request,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "rate_limit")]
    RateLimit,
    #[serde(rename = "auth_failure")]
    AuthFailure,
    #[serde(rename = "system")]
    System,
}

impl LogType {
    pub fn as_str(&self) -> &str {
        match self {
            LogType::Request => "request",
            LogType::Error => "error",
            LogType::RateLimit => "rate_limit",
            LogType::AuthFailure => "auth_failure",
            LogType::System => "system",
        }
    }
}

/// Thread-safe logger for tenant operations
pub struct TenantLogger {
    logs: Arc<Mutex<VecDeque<LogEntry>>>,
    max_entries: usize,
}

impl TenantLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Log a new entry
    pub fn log(
        &self,
        tenant_id: String,
        log_type: LogType,
        message: String,
        duration_ms: f64,
        status_code: u16,
        details: Option<String>,
    ) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            tenant_id,
            log_type,
            message,
            duration_ms,
            status_code,
            details,
        };

        if let Ok(mut logs) = self.logs.lock() {
            if logs.len() >= self.max_entries {
                logs.pop_front();
            }
            logs.push_back(entry);
        }
    }

    /// Get logs for a specific tenant
    pub fn get_tenant_logs(
        &self,
        tenant_id: &str,
        limit: usize,
        filter_type: Option<LogType>,
    ) -> Vec<LogEntry> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter()
                .filter(|entry| {
                    entry.tenant_id == tenant_id
                        && filter_type.as_ref().map_or(true, |ft| &entry.log_type == ft)
                })
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get recent logs across all tenants
    pub fn get_recent_logs(&self, limit: usize) -> Vec<LogEntry> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get error logs for specific tenant
    pub fn get_error_logs(&self, tenant_id: &str, limit: usize) -> Vec<LogEntry> {
        self.get_tenant_logs(tenant_id, limit, Some(LogType::Error))
    }

    /// Get rate limit violations
    pub fn get_rate_limit_logs(&self, tenant_id: &str, limit: usize) -> Vec<LogEntry> {
        self.get_tenant_logs(tenant_id, limit, Some(LogType::RateLimit))
    }

    /// Search logs by message
    pub fn search_logs(&self, tenant_id: &str, search_term: &str, limit: usize) -> Vec<LogEntry> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter()
                .filter(|entry| {
                    entry.tenant_id == tenant_id
                        && (entry.message.contains(search_term)
                            || entry.details.as_ref().map_or(false, |d| d.contains(search_term)))
                })
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all logs for a tenant
    pub fn clear_logs(&self, tenant_id: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.retain(|entry| entry.tenant_id != tenant_id);
        }
    }

    /// Get statistics for logs
    pub fn get_log_stats(&self, tenant_id: &str) -> LogStatistics {
        if let Ok(logs) = self.logs.lock() {
            let tenant_logs: Vec<_> = logs.iter()
                .filter(|e| e.tenant_id == tenant_id)
                .collect();

            let total = tenant_logs.len();
            let errors = tenant_logs.iter().filter(|e| e.log_type == LogType::Error).count();
            let rate_limits = tenant_logs.iter().filter(|e| e.log_type == LogType::RateLimit).count();
            let requests = tenant_logs.iter().filter(|e| e.log_type == LogType::Request).count();
            
            let avg_duration = if requests > 0 {
                tenant_logs.iter()
                    .filter(|e| e.log_type == LogType::Request)
                    .map(|e| e.duration_ms)
                    .sum::<f64>() / requests as f64
            } else {
                0.0
            };

            LogStatistics {
                total_entries: total,
                error_count: errors,
                rate_limit_violations: rate_limits,
                request_count: requests,
                avg_request_duration_ms: avg_duration,
            }
        } else {
            LogStatistics::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogStatistics {
    pub total_entries: usize,
    pub error_count: usize,
    pub rate_limit_violations: usize,
    pub request_count: usize,
    pub avg_request_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_logger() {
        let logger = TenantLogger::new(1000);

        logger.log(
            "tenant-1".to_string(),
            LogType::Request,
            "GET /api/messages".to_string(),
            25.5,
            200,
            None,
        );

        let logs = logger.get_tenant_logs("tenant-1", 10, None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].log_type, LogType::Request);
    }

    #[test]
    fn test_error_logs() {
        let logger = TenantLogger::new(1000);

        logger.log(
            "tenant-1".to_string(),
            LogType::Error,
            "Connection timeout".to_string(),
            50.0,
            500,
            Some("Database unreachable".to_string()),
        );

        let errors = logger.get_error_logs("tenant-1", 10);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].status_code, 500);
    }
}
