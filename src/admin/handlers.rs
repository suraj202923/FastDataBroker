// Admin REST API Handlers
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::admin::logs::{TenantLogger, LogType};
use crate::admin::metrics::MetricsCollector;
use crate::config::AppSettings;

pub struct AdminHandlers {
    pub app_settings: web::Data<AppSettings>,
    pub logger: web::Data<TenantLogger>,
    pub metrics: web::Data<MetricsCollector>,
}

// Request/Response Models

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantConfigResponse {
    pub tenant_id: String,
    pub tenant_name: String,
    pub api_key_prefix: String,
    pub rate_limit_rps: u32,
    pub max_connections: usize,
    pub max_message_size: u64,
    pub retention_days: u32,
    pub enabled: bool,
    pub features: serde_json::Value,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantRequest {
    pub rate_limit_rps: Option<u32>,
    pub max_connections: Option<usize>,
    pub max_message_size: Option<u64>,
    pub retention_days: Option<u32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsResponse {
    pub tenant_id: String,
    pub logs: Vec<serde_json::Value>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub tenant_id: String,
    pub current_rps: f64,
    pub peak_rps: f64,
    pub total_requests: u64,
    pub current_connections: usize,
    pub peak_connections: usize,
    pub failed_requests: u64,
    pub failure_rate_percent: f64,
    pub rate_limit_violations: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitsResponse {
    pub tenant_id: String,
    pub rps_limit: u32,
    pub rps_current: f64,
    pub rps_usage_percent: f64,
    pub connections_limit: usize,
    pub connections_current: usize,
    pub connections_usage_percent: f64,
    pub message_size_limit: u64,
    pub message_size_largest: u64,
    pub retention_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub total_tenants: usize,
}

impl AdminHandlers {
    pub fn new(
        app_settings: web::Data<AppSettings>,
        logger: web::Data<TenantLogger>,
        metrics: web::Data<MetricsCollector>,
    ) -> Self {
        Self {
            app_settings,
            logger,
            metrics,
        }
    }

    // ==================== Tenant Configuration ====================

    /// GET /api/admin/tenants - List all tenants
    pub async fn list_tenants(&self) -> Result<HttpResponse> {
        let tenants: Vec<TenantConfigResponse> = self
            .app_settings
            .get_all_tenants()
            .iter()
            .map(|t| TenantConfigResponse {
                tenant_id: t.tenant_id.clone(),
                tenant_name: t.tenant_name.clone(),
                api_key_prefix: t.api_key_prefix.clone(),
                rate_limit_rps: t.rate_limit_rps,
                max_connections: t.max_connections,
                max_message_size: t.max_message_size,
                retention_days: t.retention_days,
                enabled: t.enabled,
                features: serde_json::to_value(&t.features).unwrap_or(serde_json::json!({})),
                metadata: serde_json::to_value(&t.metadata).unwrap_or(serde_json::json!({})),
            })
            .collect();

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tenants": tenants,
            "total": tenants.len(),
        })))
    }

    /// GET /api/admin/tenants/{tenant_id} - Get specific tenant config
    pub async fn get_tenant(&self, tenant_id: web::Path<String>) -> Result<HttpResponse> {
        match self.app_settings.get_tenant(&tenant_id) {
            Some(tenant) => {
                let response = TenantConfigResponse {
                    tenant_id: tenant.tenant_id.clone(),
                    tenant_name: tenant.tenant_name.clone(),
                    api_key_prefix: tenant.api_key_prefix.clone(),
                    rate_limit_rps: tenant.rate_limit_rps,
                    max_connections: tenant.max_connections,
                    max_message_size: tenant.max_message_size,
                    retention_days: tenant.retention_days,
                    enabled: tenant.enabled,
                    features: serde_json::to_value(&tenant.features)
                        .unwrap_or(serde_json::json!({})),
                    metadata: serde_json::to_value(&tenant.metadata)
                        .unwrap_or(serde_json::json!({})),
                };

                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "tenant": response,
                })))
            }
            None => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Tenant '{}' not found", tenant_id),
            }))),
        }
    }

    /// PUT /api/admin/tenants/{tenant_id} - Update tenant configuration
    pub async fn update_tenant(
        &self,
        tenant_id: web::Path<String>,
        req: web::Json<UpdateTenantRequest>,
    ) -> Result<HttpResponse> {
        match self.app_settings.get_tenant(&tenant_id) {
            Some(mut tenant) => {
                // Update fields if provided
                if let Some(rps) = req.rate_limit_rps {
                    tenant.rate_limit_rps = rps;
                }
                if let Some(conns) = req.max_connections {
                    tenant.max_connections = conns;
                }
                if let Some(size) = req.max_message_size {
                    tenant.max_message_size = size;
                }
                if let Some(retention) = req.retention_days {
                    tenant.retention_days = retention;
                }
                if let Some(enabled) = req.enabled {
                    tenant.enabled = enabled;
                }

                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": format!("Tenant '{}' updated successfully", tenant_id),
                    "tenant": {
                        "tenant_id": tenant.tenant_id,
                        "rate_limit_rps": tenant.rate_limit_rps,
                        "max_connections": tenant.max_connections,
                        "max_message_size": tenant.max_message_size,
                        "retention_days": tenant.retention_days,
                        "enabled": tenant.enabled,
                    }
                })))
            }
            None => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Tenant '{}' not found", tenant_id),
            }))),
        }
    }

    // ==================== Logs ====================

    /// GET /api/admin/tenants/{tenant_id}/logs?type=request&limit=100
    pub async fn get_logs(
        &self,
        tenant_id: web::Path<String>,
        query: web::Query<LogsQuery>,
    ) -> Result<HttpResponse> {
        let filter_type = match query.log_type.as_deref() {
            Some("request") => Some(LogType::Request),
            Some("error") => Some(LogType::Error),
            Some("rate_limit") => Some(LogType::RateLimit),
            Some("auth_failure") => Some(LogType::AuthFailure),
            Some("system") => Some(LogType::System),
            _ => None,
        };

        let limit = query.limit.unwrap_or(100).min(10000);
        let logs = self.logger.get_tenant_logs(&tenant_id, limit, filter_type);

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tenant_id": tenant_id.as_str(),
            "logs": logs,
            "total_count": logs.len(),
            "filter": {
                "type": query.log_type.as_ref(),
                "limit": limit,
            }
        })))
    }

    /// GET /api/admin/tenants/{tenant_id}/errors - Get error logs only
    pub async fn get_error_logs(
        &self,
        tenant_id: web::Path<String>,
        query: web::Query<PaginationQuery>,
    ) -> Result<HttpResponse> {
        let limit = query.limit.unwrap_or(100).min(10000);
        let errors = self.logger.get_error_logs(&tenant_id, limit);

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tenant_id": tenant_id.as_str(),
            "error_logs": errors,
            "total_count": errors.len(),
        })))
    }

    /// GET /api/admin/tenants/{tenant_id}/rate-limits - Get rate limit violations
    pub async fn get_rate_limit_logs(
        &self,
        tenant_id: web::Path<String>,
        query: web::Query<PaginationQuery>,
    ) -> Result<HttpResponse> {
        let limit = query.limit.unwrap_or(100).min(10000);
        let violations = self.logger.get_rate_limit_logs(&tenant_id, limit);

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tenant_id": tenant_id.as_str(),
            "rate_limit_violations": violations,
            "total_count": violations.len(),
        })))
    }

    /// GET /api/admin/tenants/{tenant_id}/log-stats - Get log statistics
    pub async fn get_log_stats(&self, tenant_id: web::Path<String>) -> Result<HttpResponse> {
        let stats = self.logger.get_log_stats(&tenant_id);

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "tenant_id": tenant_id.as_str(),
            "statistics": {
                "total_entries": stats.total_entries,
                "error_count": stats.error_count,
                "rate_limit_violations": stats.rate_limit_violations,
                "request_count": stats.request_count,
                "avg_request_duration_ms": stats.avg_request_duration_ms,
            }
        })))
    }

    // ==================== Metrics & Limits ====================

    /// GET /api/admin/tenants/{tenant_id}/metrics - Get current metrics
    pub async fn get_metrics(&self, tenant_id: web::Path<String>) -> Result<HttpResponse> {
        match self.metrics.get_metrics(&tenant_id) {
            Some(metrics) => {
                let response = MetricsResponse {
                    tenant_id: metrics.tenant_id.clone(),
                    current_rps: metrics.current_rps,
                    peak_rps: metrics.peak_rps,
                    total_requests: metrics.total_requests,
                    current_connections: metrics.current_connections,
                    peak_connections: metrics.peak_connections,
                    failed_requests: metrics.failed_requests,
                    failure_rate_percent: if metrics.total_requests > 0 {
                        (metrics.failed_requests as f64 / metrics.total_requests as f64) * 100.0
                    } else {
                        0.0
                    },
                    rate_limit_violations: metrics.rate_limit_violations,
                };

                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "metrics": response,
                })))
            }
            None => Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "metrics": {
                    "tenant_id": tenant_id.as_str(),
                    "current_rps": 0.0,
                    "total_requests": 0,
                }
            }))),
        }
    }

    /// GET /api/admin/tenants/{tenant_id}/limits - Get usage vs limits
    pub async fn get_limits(&self, tenant_id: web::Path<String>) -> Result<HttpResponse> {
        let tenant = self.app_settings.get_tenant(&tenant_id);
        let metrics = self.metrics.get_metrics(&tenant_id);

        match (tenant, metrics) {
            (Some(t), Some(m)) => {
                let rps_usage = (m.current_rps / t.rate_limit_rps as f64) * 100.0;
                let conn_usage = (m.current_connections as f64 / t.max_connections as f64) * 100.0;

                let response = LimitsResponse {
                    tenant_id: t.tenant_id.clone(),
                    rps_limit: t.rate_limit_rps,
                    rps_current: m.current_rps,
                    rps_usage_percent: rps_usage.min(100.0),
                    connections_limit: t.max_connections,
                    connections_current: m.current_connections,
                    connections_usage_percent: conn_usage.min(100.0),
                    message_size_limit: t.max_message_size,
                    message_size_largest: m.largest_message_size,
                    retention_days: t.retention_days,
                };

                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "limits": response,
                })))
            }
            (Some(t), None) => {
                let response = LimitsResponse {
                    tenant_id: t.tenant_id.clone(),
                    rps_limit: t.rate_limit_rps,
                    rps_current: 0.0,
                    rps_usage_percent: 0.0,
                    connections_limit: t.max_connections,
                    connections_current: 0,
                    connections_usage_percent: 0.0,
                    message_size_limit: t.max_message_size,
                    message_size_largest: 0,
                    retention_days: t.retention_days,
                };

                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "limits": response,
                })))
            }
            (None, _) => Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Tenant '{}' not found", tenant_id),
            }))),
        }
    }

    /// GET /api/admin/health - Health check
    pub async fn health(&self) -> Result<HttpResponse> {
        let tenants = self.app_settings.get_all_tenants();

        Ok(HttpResponse::Ok().json(HealthResponse {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "1.0.0".to_string(),
            total_tenants: tenants.len(),
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub log_type: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
}
