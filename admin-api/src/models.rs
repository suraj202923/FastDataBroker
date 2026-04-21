use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// System Configuration Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub id: String,
    pub broker_url: String,
    pub max_brokers: i32,
    pub replication_factor: i32,
    pub log_level: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfigRequest {
    pub broker_url: Option<String>,
    pub max_brokers: Option<i32>,
    pub replication_factor: Option<i32>,
    pub log_level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub broker_connected: bool,
    pub database_healthy: bool,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct DetailedHealth {
    pub status: HealthStatus,
    pub broker: BrokerHealth,
    pub database: DatabaseHealth,
    pub system: SystemMetrics,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize)]
pub struct BrokerHealth {
    pub connected: bool,
    pub url: String,
    pub response_time_ms: u32,
    pub active_connections: u32,
}

#[derive(Debug, Serialize)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub query_time_ms: u32,
    pub pool_size: u32,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u32,
    pub active_tenants: u32,
    pub total_message_volume: u64,
}

// ============================================================================
// Cluster Environment Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEnvironment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub region: String,
    pub broker_addresses: String,  // JSON string
    pub replication_factor: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterEnvironmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub region: String,
    pub broker_addresses: Vec<String>,
    pub replication_factor: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ClusterEnvironmentResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub region: String,
    pub broker_addresses: Vec<String>,
    pub replication_factor: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// Tenant Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub tenant_id: String,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub status: String,
    pub max_message_size: i32,
    pub rate_limit_rps: i32,
    pub max_connections: i32,
    pub retention_days: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub tenant_id: String,
    pub name: String,
    pub email: String,
    pub status: String,
    pub limits: TenantLimits,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSecret {
    pub secret_id: String,
    pub tenant_id: String,
    pub secret_key: String,
    pub secret_value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantSecretRequest {
    pub secret_key: String,
    pub secret_value: String,
}

#[derive(Debug, Serialize)]
pub struct TenantSecretResponse {
    pub secret_id: String,
    pub secret_key: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantLimits {
    pub max_message_size: i32,
    pub rate_limit_rps: i32,
    pub max_connections: i32,
    pub retention_days: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantLimitsRequest {
    pub max_message_size: Option<i32>,
    pub rate_limit_rps: Option<i32>,
    pub max_connections: Option<i32>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TenantUsage {
    pub tenant_id: String,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub storage_used_mb: f32,
    pub bandwidth_used_gb: f32,
    pub active_connections: u32,
    pub last_activity: String,
}

#[derive(Debug, Serialize)]
pub struct TenantListResponse {
    pub tenant_id: String,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TenantUsageOverview>,
}

#[derive(Debug, Serialize)]
pub struct TenantUsageOverview {
    pub messages_sent: u64,
    pub active_connections: u32,
    pub storage_used_mb: f32,
}

// ============================================================================
// Notification Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub id: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub use_tls: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSmtpConfigRequest {
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_email: Option<String>,
    pub use_tls: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmtpTestRequest {
    pub recipient_email: String,
}

#[derive(Debug, Serialize)]
pub struct SmtpConfigResponse {
    pub host: String,
    pub port: i32,
    pub from_email: String,
    pub use_tls: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub id: String,
    pub event_type: String,
    pub enabled: bool,
    pub recipient_email: String,
    pub notification_channels: String,  // JSON array
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationSettingsRequest {
    pub event_type: String,
    pub enabled: bool,
    pub recipient_email: String,
    pub notification_channels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationSettingsResponse {
    pub id: String,
    pub event_type: String,
    pub enabled: bool,
    pub recipient_email: String,
    pub notification_channels: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub event_id: String,
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub severity: String,  // critical, warning, info
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationEventResponse {
    pub event_id: String,
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub created_at: String,
}

// ============================================================================
// API Info Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ApiInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<EndpointInfo>,
    pub documentation: String,
}

#[derive(Debug, Serialize)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub description: String,
    pub authentication: bool,
}

// ============================================================================
// Error Response Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub timestamp: String,
}
