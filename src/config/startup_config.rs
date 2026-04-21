// ============================================================================
// Startup Configuration (v3.1) - Load from startup.json
// ============================================================================
// This module loads the complete system configuration from startup.json
// All system settings are configurable through a single JSON file

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use anyhow::{Result, Context};

/// Complete startup configuration loaded from startup.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub cache: CacheConfig,
    pub authentication: AuthenticationConfig,
    pub rate_limiting: RateLimitingConfig,
    pub logging: LoggingSystemConfig,
    pub monitoring: MonitoringConfig,
    pub resilience: ResilienceConfig,
    pub performance: PerformanceConfig,
    pub admin_api: AdminApiConfig,
    pub clustering: ClusteringConfig,
    pub security: SecurityConfig,
    pub features: FeaturesConfig,
    pub deployment: DeploymentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub tls_version: String,
    pub max_idle_timeout_ms: u32,
    pub max_concurrent_streams: u32,
    pub max_datagram_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub mode: String,
    pub sled: SledConfig,
    pub json: JsonStorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SledConfig {
    pub enabled: bool,
    pub path: String,
    pub cache_capacity: u64,
    pub compression: bool,
    pub compression_factor: u32,
    pub use_compression: bool,
    pub flush_every_ms: u32,
    pub snapshot_after_ops: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStorageConfig {
    pub enabled: bool,
    pub tenants_path: String,
    pub create_if_missing: bool,
    pub backup_enabled: bool,
    pub backup_path: String,
    pub backup_retention_days: u32,
    pub backup_schedule_utc_hour: u32,
    pub backup_schedule_utc_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub tenant_config: TenantConfigCacheConfig,
    pub metrics: MetricsCacheConfig,
    pub psk_verification: PskVerificationCacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfigCacheConfig {
    pub enabled: bool,
    pub max_size_mb: u32,
    pub reload_interval_seconds: u32,
    pub hot_reload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCacheConfig {
    pub enabled: bool,
    pub max_entries_per_tenant: u32,
    pub flush_interval_ms: u32,
    pub memory_limit_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PskVerificationCacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u32,
    pub max_cached_entries: u32,
    pub lru_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    pub method: String,
    pub psk_hash_algorithm: String,
    pub psk_rotation_enabled: bool,
    pub psk_rotation_days: u32,
    pub require_tls: bool,
    pub min_psk_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub default_rps: u32,
    pub burst_multiplier: f32,
    pub window_size_ms: u32,
    pub per_tenant_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSystemConfig {
    pub ndjson: NdjsonLoggingConfig,
    pub rotation: LogRotationConfig,
    pub archive: ArchiveConfig,
    pub level: String,
    pub include_request_headers: bool,
    pub include_request_body: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdjsonLoggingConfig {
    pub enabled: bool,
    pub path: String,
    pub buffer_size_mb: u32,
    pub flush_interval_ms: u32,
    pub sync_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub enabled: bool,
    pub schedule_utc_hour: u32,
    pub schedule_utc_minute: u32,
    pub keep_live_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub enabled: bool,
    pub format: String,
    pub compression: String,
    pub compression_level: u32,
    pub path: String,
    pub schedule_utc_hour: u32,
    pub schedule_utc_minute: u32,
    pub retention_days: u32,
    pub cleanup_batch_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub health_check_interval_ms: u32,
    pub metrics_path: String,
    pub prometheus_enabled: bool,
    pub hot_reload_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    pub connection_timeout_ms: u32,
    pub read_timeout_ms: u32,
    pub write_timeout_ms: u32,
    pub max_connections_per_tenant: u32,
    pub max_queue_size_per_tenant: u64,
    pub overflow_strategy: String,
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: u32,
    pub worker_queue_size: u32,
    pub batch_size: u32,
    pub enable_vectored_io: bool,
    pub enable_tcp_nodelay: bool,
    pub enable_zero_copy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub auth_required: bool,
    pub auth_token: String,
    pub cors_enabled: bool,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringConfig {
    pub enabled: bool,
    pub node_id: String,
    pub cluster_name: String,
    pub peer_discovery: String,
    pub peers: Vec<String>,
    pub gossip_interval_ms: u32,
    pub sync_interval_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls: TlsConfig,
    pub rate_limit_by_ip: bool,
    pub ip_whitelist_enabled: bool,
    pub ip_whitelist: Vec<String>,
    pub ip_blacklist_enabled: bool,
    pub ip_blacklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: Option<String>,
    pub verify_client_cert: bool,
    pub session_resumption: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub priority_queue: bool,
    pub message_compression: bool,
    pub message_encryption: bool,
    pub message_signing: bool,
    pub dead_letter_queue: bool,
    pub message_deduplication: bool,
    pub ordered_delivery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub environment: String,
    pub graceful_shutdown_timeout_ms: u32,
    pub startup_init_timeout_ms: u32,
    pub config_watch_enabled: bool,
    pub config_watch_interval_ms: u32,
}

impl StartupConfig {
    /// Load configuration from startup.json file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Read the file
        let content = fs::read_to_string(path)
            .context(format!("Failed to read startup config from {:?}", path))?;
        
        // Parse as JSON
        let config: StartupConfig = serde_json::from_str(&content)
            .context(format!(
                "Failed to parse startup config from {:?}. Invalid JSON format",
                path
            ))?;
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load with default path (./startup.json)
    pub fn from_default_path() -> Result<Self> {
        Self::from_file("./startup.json")
    }
    
    /// Load with environment override (looks for STARTUP_CONFIG env var)
    pub fn from_env_or_default() -> Result<Self> {
        let path = std::env::var("STARTUP_CONFIG")
            .unwrap_or_else(|_| "./startup.json".to_string());
        
        Self::from_file(path)
    }
    
    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Validate server settings
        if self.server.port == 0 {
            anyhow::bail!("Server port must be greater than 0");
        }
        
        // Validate PSK length
        if self.authentication.min_psk_length < 16 {
            anyhow::bail!("PSK minimum length should be at least 16 bytes");
        }
        
        // Validate cache settings
        if self.cache.metrics.flush_interval_ms == 0 {
            anyhow::bail!("Metrics flush interval must be greater than 0");
        }
        
        // Validate logging paths exist or can be created
        if self.logging.ndjson.enabled {
            let log_path = Path::new(&self.logging.ndjson.path);
            if let Some(parent) = log_path.parent() {
                if parent != Path::new("") && !parent.exists() {
                    fs::create_dir_all(parent)
                        .context("Failed to create logging directory")?;
                }
            }
        }
        
        // Validate storage paths
        if self.storage.json.enabled {
            let tenants_path = Path::new(&self.storage.json.tenants_path);
            if !tenants_path.exists() && self.storage.json.create_if_missing {
                fs::create_dir_all(tenants_path)
                    .context("Failed to create tenants directory")?;
            }
        }
        
        Ok(())
    }
    
    /// Get server bind address
    pub fn server_bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
    
    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "priority_queue" => self.features.priority_queue,
            "message_compression" => self.features.message_compression,
            "message_encryption" => self.features.message_encryption,
            "message_signing" => self.features.message_signing,
            "dead_letter_queue" => self.features.dead_letter_queue,
            "message_deduplication" => self.features.message_deduplication,
            "ordered_delivery" => self.features.ordered_delivery,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let config = StartupConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 6000,
                protocol: "QUIC".to_string(),
                tls_version: "1.3".to_string(),
                max_idle_timeout_ms: 30000,
                max_concurrent_streams: 1000,
                max_datagram_size: 1200,
            },
            storage: StorageConfig {
                mode: "hybrid".to_string(),
                sled: SledConfig {
                    enabled: true,
                    path: "/tmp/sled".to_string(),
                    cache_capacity: 1073741824,
                    compression: true,
                    compression_factor: 5,
                    use_compression: true,
                    flush_every_ms: 500,
                    snapshot_after_ops: 1000000,
                },
                json: JsonStorageConfig {
                    enabled: true,
                    tenants_path: "/tmp/tenants".to_string(),
                    create_if_missing: true,
                    backup_enabled: true,
                    backup_path: "/tmp/backups".to_string(),
                    backup_retention_days: 7,
                    backup_schedule_utc_hour: 2,
                    backup_schedule_utc_minute: 0,
                },
            },
            cache: CacheConfig {
                tenant_config: TenantConfigCacheConfig {
                    enabled: true,
                    max_size_mb: 50,
                    reload_interval_seconds: 3600,
                    hot_reload: true,
                },
                metrics: MetricsCacheConfig {
                    enabled: true,
                    max_entries_per_tenant: 1000,
                    flush_interval_ms: 100,
                    memory_limit_mb: 100,
                },
                psk_verification: PskVerificationCacheConfig {
                    enabled: true,
                    ttl_seconds: 86400,
                    max_cached_entries: 10000,
                    lru_mode: true,
                },
            },
            authentication: AuthenticationConfig {
                method: "PSK".to_string(),
                psk_hash_algorithm: "SHA256".to_string(),
                psk_rotation_enabled: true,
                psk_rotation_days: 90,
                require_tls: true,
                min_psk_length: 32,
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                default_rps: 1000,
                burst_multiplier: 1.5,
                window_size_ms: 1000,
                per_tenant_override: true,
            },
            logging: LoggingSystemConfig {
                ndjson: NdjsonLoggingConfig {
                    enabled: true,
                    path: "/tmp/logs/app.log".to_string(),
                    buffer_size_mb: 10,
                    flush_interval_ms: 50,
                    sync_on_error: true,
                },
                rotation: LogRotationConfig {
                    enabled: true,
                    schedule_utc_hour: 0,
                    schedule_utc_minute: 0,
                    keep_live_files: 3,
                },
                archive: ArchiveConfig {
                    enabled: true,
                    format: "parquet".to_string(),
                    compression: "snappy".to_string(),
                    compression_level: 5,
                    path: "/tmp/logs/archive".to_string(),
                    schedule_utc_hour: 1,
                    schedule_utc_minute: 0,
                    retention_days: 30,
                    cleanup_batch_size: 1000,
                },
                level: "info".to_string(),
                include_request_headers: false,
                include_request_body: false,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                health_check_interval_ms: 5000,
                metrics_path: "/metrics".to_string(),
                prometheus_enabled: true,
                hot_reload_enabled: true,
            },
            resilience: ResilienceConfig {
                connection_timeout_ms: 10000,
                read_timeout_ms: 30000,
                write_timeout_ms: 30000,
                max_connections_per_tenant: 100,
                max_queue_size_per_tenant: 1000000,
                overflow_strategy: "DROP_OLDEST".to_string(),
                circuit_breaker: CircuitBreakerConfig {
                    enabled: true,
                    failure_threshold: 100,
                    success_threshold: 50,
                    timeout_ms: 60000,
                },
            },
            performance: PerformanceConfig {
                worker_threads: 0,
                worker_queue_size: 10000,
                batch_size: 100,
                enable_vectored_io: true,
                enable_tcp_nodelay: true,
                enable_zero_copy: true,
            },
            admin_api: AdminApiConfig {
                enabled: true,
                host: "127.0.0.1".to_string(),
                port: 3000,
                auth_required: true,
                auth_token: "changeme_in_production".to_string(),
                cors_enabled: false,
                cors_origins: vec![],
            },
            clustering: ClusteringConfig {
                enabled: false,
                node_id: "node-1".to_string(),
                cluster_name: "fastdatabroker-cluster".to_string(),
                peer_discovery: "static".to_string(),
                peers: vec![],
                gossip_interval_ms: 1000,
                sync_interval_ms: 5000,
            },
            security: SecurityConfig {
                tls: TlsConfig {
                    cert_path: "/certs/server.crt".to_string(),
                    key_path: "/certs/server.key".to_string(),
                    ca_cert_path: None,
                    verify_client_cert: false,
                    session_resumption: true,
                },
                rate_limit_by_ip: false,
                ip_whitelist_enabled: false,
                ip_whitelist: vec![],
                ip_blacklist_enabled: false,
                ip_blacklist: vec![],
            },
            features: FeaturesConfig {
                priority_queue: true,
                message_compression: true,
                message_encryption: false,
                message_signing: false,
                dead_letter_queue: true,
                message_deduplication: true,
                ordered_delivery: true,
            },
            deployment: DeploymentConfig {
                environment: "production".to_string(),
                graceful_shutdown_timeout_ms: 30000,
                startup_init_timeout_ms: 60000,
                config_watch_enabled: true,
                config_watch_interval_ms: 5000,
            },
        };
        
        // Validation should succeed
        assert!(config.validate().is_ok());
    }
}
