// ============================================================================
// Metrics Flushing Worker (v3.1) - Background Async Metrics Flusher
// ============================================================================
// Flushes in-memory metrics to JSON files every 100ms asynchronously

use tokio::task::JoinHandle;
use std::time::Duration;
use std::sync::Arc;
use std::path::Path;
use anyhow::Result;
use chrono::Timelike;

use crate::config::in_memory_cache::{CacheSystem, MetricsCounters};
use crate::config::tenant_json::TenantJsonConfig;

/// Metrics flushing worker
pub struct MetricsFlushWorker {
    handle: Option<JoinHandle<()>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl MetricsFlushWorker {
    /// Create and start a new metrics flush worker
    pub fn start(
        cache_system: CacheSystem,
        tenants_path: String,
        flush_interval_ms: u32,
        max_metrics_per_tenant: usize,
    ) -> Self {
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);
        
        let handle = tokio::spawn(async move {
            metrics_flush_loop(
                cache_system,
                tenants_path,
                flush_interval_ms,
                max_metrics_per_tenant,
                is_running_clone,
            )
            .await;
        });
        
        MetricsFlushWorker {
            handle: Some(handle),
            is_running,
        }
    }
    
    /// Stop the worker gracefully
    pub async fn stop(&mut self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

/// Main metrics flushing loop (runs every 100ms)
async fn metrics_flush_loop(
    cache_system: CacheSystem,
    tenants_path: String,
    flush_interval_ms: u32,
    max_metrics_per_tenant: usize,
    is_running: Arc<std::sync::atomic::AtomicBool>,
) {
    let flush_interval = Duration::from_millis(flush_interval_ms as u64);
    
    loop {
        if !is_running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        
        // Sleep first to maintain consistent intervals
        tokio::time::sleep(flush_interval).await;
        
        if let Err(e) = flush_metrics_to_files(
            &cache_system,
            &tenants_path,
            max_metrics_per_tenant,
        )
        .await
        {
            eprintln!("Error flushing metrics: {}", e);
            // Continue on error, don't panic
        }
    }
}

/// Flush all metrics from cache to JSON files
async fn flush_metrics_to_files(
    cache_system: &CacheSystem,
    tenants_path: &str,
    max_metrics_per_tenant: usize,
) -> Result<()> {
    // Get all metrics from the in-memory counters
    let all_metrics = cache_system.drain_metrics();
    
    // For each tenant with metrics, update their JSON file
    for (tenant_id, metrics_snapshot) in all_metrics {
        // Get the current tenant configuration
        if let Some(mut tenant_config) = cache_system.tenant_config.get(&tenant_id) {
            // Add the new metrics snapshot
            tenant_config.add_metrics_snapshot(metrics_snapshot, max_metrics_per_tenant);
            
            // Update the tenant's updated_at timestamp
            tenant_config.tenant.updated_at = chrono::Utc::now().to_rfc3339();
            
            // Save to disk asynchronously
            let path = format!("{}/{}.json", tenants_path, tenant_id);
            if let Err(e) = tenant_config.to_file(&path) {
                eprintln!("Warning: Failed to flush metrics for tenant {}: {}", tenant_id, e);
                // Continue with other tenants on error
            }
            
            // Update cache with the new configuration
            cache_system.tenant_config.update(tenant_id, tenant_config);
        }
    }
    
    Ok(())
}

/// Configuration reloader worker
pub struct ConfigReloadWorker {
    handle: Option<JoinHandle<()>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl ConfigReloadWorker {
    /// Create and start a new config reload worker
    pub fn start(
        cache_system: CacheSystem,
        tenants_path: String,
        reload_interval_seconds: u32,
    ) -> Self {
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);
        
        let handle = tokio::spawn(async move {
            config_reload_loop(
                cache_system,
                tenants_path,
                reload_interval_seconds,
                is_running_clone,
            )
            .await;
        });
        
        ConfigReloadWorker {
            handle: Some(handle),
            is_running,
        }
    }
    
    /// Stop the worker gracefully
    pub async fn stop(&mut self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

/// Main config reload loop
async fn config_reload_loop(
    cache_system: CacheSystem,
    tenants_path: String,
    reload_interval_seconds: u32,
    is_running: Arc<std::sync::atomic::AtomicBool>,
) {
    let reload_interval = Duration::from_secs(reload_interval_seconds as u64);
    
    loop {
        if !is_running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        
        tokio::time::sleep(reload_interval).await;
        
        if let Err(e) = reload_tenant_configs(&cache_system, &tenants_path).await {
            eprintln!("Error reloading tenant configs: {}", e);
            // Continue on error
        }
    }
}

/// Reload all tenant configurations from disk
async fn reload_tenant_configs(
    cache_system: &CacheSystem,
    tenants_path: &str,
) -> Result<()> {
    // Load all tenant configs from disk
    let configs = TenantJsonConfig::load_all(tenants_path)?;
    
    // Update cache with new configs
    cache_system.tenant_config.load_all(configs);
    
    Ok(())
}

/// Backup worker for daily backups
pub struct BackupWorker {
    handle: Option<JoinHandle<()>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl BackupWorker {
    /// Create and start a new backup worker
    pub fn start(
        tenants_path: String,
        backup_path: String,
        backup_hour_utc: u32,
        backup_minute_utc: u32,
    ) -> Self {
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);
        
        let handle = tokio::spawn(async move {
            backup_worker_loop(
                tenants_path,
                backup_path,
                backup_hour_utc,
                backup_minute_utc,
                is_running_clone,
            )
            .await;
        });
        
        BackupWorker {
            handle: Some(handle),
            is_running,
        }
    }
    
    /// Stop the worker gracefully
    pub async fn stop(&mut self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

/// Backup worker loop - creates daily tar.gz backups at specified UTC time
async fn backup_worker_loop(
    tenants_path: String,
    backup_path: String,
    backup_hour_utc: u32,
    backup_minute_utc: u32,
    is_running: Arc<std::sync::atomic::AtomicBool>,
) {
    loop {
        if !is_running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        
        // Calculate time until next backup
        let now = chrono::Utc::now();
        let today_backup: Option<chrono::DateTime<chrono::Utc>> =
            now.with_hour(backup_hour_utc).and_then(|t| t.with_minute(backup_minute_utc));
        
        let next_backup = if let Some(backup_time) = today_backup {
            if backup_time > now {
                backup_time
            } else {
                // Schedule for tomorrow at the same time
                backup_time + chrono::Duration::days(1)
            }
        } else {
            // Invalid time, wait an hour
            now + chrono::Duration::hours(1)
        };
        
        let wait_duration = (next_backup - now)
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(3600));
        
        tokio::time::sleep(wait_duration).await;
        
        if !is_running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
        
        // Perform backup
        if let Err(e) = perform_backup(&tenants_path, &backup_path).await {
            eprintln!("Error performing backup: {}", e);
        }
    }
}

/// Perform a single backup operation
async fn perform_backup(tenants_path: &str, backup_path: &str) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_file = format!("{}/tenants_backup_{}.tar.gz", backup_path, timestamp);
    
    // Create backup directory if it doesn't exist
    std::fs::create_dir_all(backup_path)?;
    
    // Use tar to create a compressed backup
    // In production, you'd use a proper tar library
    let output = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&backup_file)
        .arg("-C")
        .arg(std::path::Path::new(tenants_path).parent().unwrap_or(std::path::Path::new("/")))
        .arg("tenants")
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to create backup: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("Backup created: {}", backup_file);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_flush_worker_creation() {
        let cache_system = CacheSystem::new(86400);
        
        let worker = MetricsFlushWorker::start(
            cache_system,
            "/tmp/tenants".to_string(),
            100,
            1000,
        );
        
        // Worker should be created successfully
        assert!(worker.handle.is_some());
    }
    
    #[tokio::test]
    async fn test_config_reload_worker_creation() {
        let cache_system = CacheSystem::new(86400);
        
        let worker = ConfigReloadWorker::start(
            cache_system,
            "/tmp/tenants".to_string(),
            3600,
        );
        
        assert!(worker.handle.is_some());
    }
}
