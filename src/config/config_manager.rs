// ============================================================================
// Configuration Manager (v3.1) - Unified Configuration Loader
// ============================================================================
// Loads startup.json and initializes the entire v3.1 system

use anyhow::Result;
use std::path::Path;

use crate::config::{
    StartupConfig, CacheSystem, TenantJsonConfig, 
    MetricsFlushWorker, ConfigReloadWorker, BackupWorker
};

/// Main configuration manager for v3.1
pub struct ConfigManager {
    pub startup_config: StartupConfig,
    pub cache_system: CacheSystem,
    pub metrics_flush_worker: Option<MetricsFlushWorker>,
    pub config_reload_worker: Option<ConfigReloadWorker>,
    pub backup_worker: Option<BackupWorker>,
}

impl ConfigManager {
    /// Initialize configuration manager from startup.json
    pub async fn init_from_startup_json<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Load startup.json
        let startup_config = StartupConfig::from_file(path)?;
        
        println!("✅ Loaded startup configuration");
        println!("   Server: {}:{}", startup_config.server.host, startup_config.server.port);
        println!("   Storage Mode: {}", startup_config.storage.mode);
        println!("   Tenants Path: {}", startup_config.storage.json.tenants_path);
        
        // Create cache system
        let cache_system = CacheSystem::new(
            startup_config.cache.psk_verification.ttl_seconds
        );
        
        // Load all tenant configurations into cache
        if startup_config.storage.json.enabled {
            let tenants_path = &startup_config.storage.json.tenants_path;
            match TenantJsonConfig::load_all(tenants_path) {
                Ok(configs) => {
                    let tenant_count = configs.len();
                    cache_system.tenant_config.load_all(configs);
                    println!("✅ Loaded {} tenant configurations from {}", tenant_count, tenants_path);
                }
                Err(e) => {
                    eprintln!("⚠️  Warning: Failed to load tenant configurations: {}", e);
                    eprintln!("   System will continue with empty tenant cache");
                }
            }
        }
        
        // Start background workers
        let metrics_flush_worker = if startup_config.cache.metrics.enabled {
            let worker = MetricsFlushWorker::start(
                cache_system.clone(),
                startup_config.storage.json.tenants_path.clone(),
                startup_config.cache.metrics.flush_interval_ms,
                startup_config.cache.metrics.max_entries_per_tenant as usize,
            );
            println!("✅ Started metrics flush worker (interval: {}ms)", 
                     startup_config.cache.metrics.flush_interval_ms);
            Some(worker)
        } else {
            None
        };
        
        let config_reload_worker = if startup_config.cache.tenant_config.hot_reload {
            let worker = ConfigReloadWorker::start(
                cache_system.clone(),
                startup_config.storage.json.tenants_path.clone(),
                startup_config.cache.tenant_config.reload_interval_seconds,
            );
            println!("✅ Started config reload worker (interval: {}s)", 
                     startup_config.cache.tenant_config.reload_interval_seconds);
            Some(worker)
        } else {
            None
        };
        
        let backup_worker = if startup_config.storage.json.backup_enabled {
            let worker = BackupWorker::start(
                startup_config.storage.json.tenants_path.clone(),
                startup_config.storage.json.backup_path.clone(),
                startup_config.storage.json.backup_schedule_utc_hour,
                startup_config.storage.json.backup_schedule_utc_minute,
            );
            println!("✅ Started backup worker (schedule: {}:{:02} UTC)", 
                     startup_config.storage.json.backup_schedule_utc_hour,
                     startup_config.storage.json.backup_schedule_utc_minute);
            Some(worker)
        } else {
            None
        };
        
        Ok(ConfigManager {
            startup_config,
            cache_system,
            metrics_flush_worker,
            config_reload_worker,
            backup_worker,
        })
    }
    
    /// Initialize from default path (./startup.json)
    pub async fn init() -> Result<Self> {
        Self::init_from_startup_json("./startup.json").await
    }
    
    /// Initialize from environment (STARTUP_CONFIG env var or default)
    pub async fn init_from_env() -> Result<Self> {
        let path = std::env::var("STARTUP_CONFIG")
            .unwrap_or_else(|_| "./startup.json".to_string());
        Self::init_from_startup_json(path).await
    }
    
    /// Get detailed configuration report
    pub fn get_report(&self) -> String {
        format!(
            r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║          FastDataBroker v3.1 Configuration Report                         ║
╚═══════════════════════════════════════════════════════════════════════════╝

📊 SERVER CONFIGURATION
  • Protocol: {} on {}:{}
  • Max Concurrent Streams: {}
  • Max Idle Timeout: {}ms
  
💾 STORAGE CONFIGURATION
  • Mode: {}
  • Sled Enabled: {}
  • JSON Storage Enabled: {}
  • Tenants Path: {}
  
🗂️  TENANT CACHE
  • Loaded Tenants: {}
  • Max Size: {}MB
  • Hot Reload: {} (interval: {}s)
  
📈 METRICS
  • Enabled: {}
  • Flush Interval: {}ms
  • Max Entries Per Tenant: {}
  • Memory Limit: {}MB
  
🔐 AUTHENTICATION
  • Method: {}
  • PSK Hash: {}
  • Min PSK Length: {} bytes
  • TLS Required: {}
  
⏱️  RATE LIMITING
  • Enabled: {}
  • Default RPS: {}
  • Burst Multiplier: {}
  • Per-Tenant Override: {}
  
📝 LOGGING
  • NDJSON Enabled: {}
  • Log Level: {}
  • Buffer Size: {}MB
  • Flush Interval: {}ms
  • Archive Enabled: {} (format: {})
  • Rotation Enabled: {} (schedule: {}:00 UTC)
  • Archive Schedule: {}:00 UTC
  • Retention: {} days
  
🛡️  RESILIENCE
  • Connection Timeout: {}ms
  • Max Connections Per Tenant: {}
  • Max Queue Size Per Tenant: {}
  • Overflow Strategy: {}
  • Circuit Breaker: {} (threshold: {})
  
⚙️  PERFORMANCE
  • Worker Threads: {}
  • Batch Size: {}
  • Vectored I/O: {}
  • TCP_NODELAY: {}
  • Zero-Copy: {}
  
🔧 FEATURES ENABLED
  • Priority Queue: {}
  • Message Compression: {}
  • Message Encryption: {}
  • Message Signing: {}
  • Dead Letter Queue: {}
  • Message Deduplication: {}
  • Ordered Delivery: {}
  
🚀 DEPLOYMENT
  • Environment: {}
  • Config Watch: {} (interval: {}ms)
  • Graceful Shutdown Timeout: {}ms

╚═══════════════════════════════════════════════════════════════════════════╝
"#,
            self.startup_config.server.protocol,
            self.startup_config.server.host,
            self.startup_config.server.port,
            self.startup_config.server.max_concurrent_streams,
            self.startup_config.server.max_idle_timeout_ms,
            
            self.startup_config.storage.mode,
            self.startup_config.storage.sled.enabled,
            self.startup_config.storage.json.enabled,
            self.startup_config.storage.json.tenants_path,
            
            self.cache_system.tenant_config.size(),
            self.startup_config.cache.tenant_config.max_size_mb,
            self.startup_config.cache.tenant_config.hot_reload,
            self.startup_config.cache.tenant_config.reload_interval_seconds,
            
            self.startup_config.cache.metrics.enabled,
            self.startup_config.cache.metrics.flush_interval_ms,
            self.startup_config.cache.metrics.max_entries_per_tenant,
            self.startup_config.cache.metrics.memory_limit_mb,
            
            self.startup_config.authentication.method,
            self.startup_config.authentication.psk_hash_algorithm,
            self.startup_config.authentication.min_psk_length,
            self.startup_config.authentication.require_tls,
            
            self.startup_config.rate_limiting.enabled,
            self.startup_config.rate_limiting.default_rps,
            self.startup_config.rate_limiting.burst_multiplier,
            self.startup_config.rate_limiting.per_tenant_override,
            
            self.startup_config.logging.ndjson.enabled,
            self.startup_config.logging.level,
            self.startup_config.logging.ndjson.buffer_size_mb,
            self.startup_config.logging.ndjson.flush_interval_ms,
            self.startup_config.logging.archive.enabled,
            self.startup_config.logging.archive.format,
            self.startup_config.logging.rotation.enabled,
            self.startup_config.logging.rotation.schedule_utc_hour,
            self.startup_config.logging.archive.schedule_utc_hour,
            self.startup_config.logging.archive.retention_days,
            
            self.startup_config.resilience.connection_timeout_ms,
            self.startup_config.resilience.max_connections_per_tenant,
            self.startup_config.resilience.max_queue_size_per_tenant,
            self.startup_config.resilience.overflow_strategy,
            self.startup_config.resilience.circuit_breaker.enabled,
            self.startup_config.resilience.circuit_breaker.failure_threshold,
            
            if self.startup_config.performance.worker_threads == 0 {
                "auto".to_string()
            } else {
                self.startup_config.performance.worker_threads.to_string()
            },
            self.startup_config.performance.batch_size,
            self.startup_config.performance.enable_vectored_io,
            self.startup_config.performance.enable_tcp_nodelay,
            self.startup_config.performance.enable_zero_copy,
            
            self.startup_config.features.priority_queue,
            self.startup_config.features.message_compression,
            self.startup_config.features.message_encryption,
            self.startup_config.features.message_signing,
            self.startup_config.features.dead_letter_queue,
            self.startup_config.features.message_deduplication,
            self.startup_config.features.ordered_delivery,
            
            self.startup_config.deployment.environment,
            self.startup_config.deployment.config_watch_enabled,
            self.startup_config.deployment.config_watch_interval_ms,
            self.startup_config.deployment.graceful_shutdown_timeout_ms,
        )
    }
    
    /// Shutdown all background workers
    pub async fn shutdown(&mut self) {
        println!("Shutting down background workers...");
        
        if let Some(mut worker) = self.metrics_flush_worker.take() {
            worker.stop().await;
            println!("✅ Metrics flush worker stopped");
        }
        
        if let Some(mut worker) = self.config_reload_worker.take() {
            worker.stop().await;
            println!("✅ Config reload worker stopped");
        }
        
        if let Some(mut worker) = self.backup_worker.take() {
            worker.stop().await;
            println!("✅ Backup worker stopped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_config_manager_creation() {
        // This test would require a valid startup.json file
        // Skipping for now as it requires file system setup
    }
}
