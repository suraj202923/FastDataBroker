// ============================================================================
// FastDataBroker Persistent Storage - Tenant & PSK Data Management
// File-based storage with optional database backend support
// ============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, debug};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::config::TenantConfig;
use crate::security::PreSharedKey;

/// Persistent storage for tenant and PSK data
#[derive(Debug, Clone)]
pub struct TenantStorage {
    /// Base directory for storage
    storage_dir: PathBuf,
    /// In-memory cache of tenants
    tenants_cache: Arc<RwLock<std::collections::HashMap<String, TenantConfig>>>,
    /// In-memory cache of PSKs
    psks_cache: Arc<RwLock<std::collections::HashMap<String, PreSharedKey>>>,
}

impl TenantStorage {
    /// Create new storage instance with given directory
    pub async fn new(storage_dir: &str) -> Result<Self> {
        let path = PathBuf::from(storage_dir);

        // Create directories if they don't exist
        tokio::fs::create_dir_all(&path).await?;
        tokio::fs::create_dir_all(path.join("tenants")).await?;
        tokio::fs::create_dir_all(path.join("psks")).await?;

        info!("TenantStorage initialized at: {}", storage_dir);

        Ok(TenantStorage {
            storage_dir: path,
            tenants_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            psks_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }

    /// Save tenant configuration to file
    pub async fn save_tenant(&self, tenant: &TenantConfig) -> Result<()> {
        let tenant_file = self.storage_dir.join("tenants").join(format!("{}.json", tenant.tenant_id));

        let json = serde_json::to_string_pretty(tenant)?;
        let mut file = File::create(tenant_file).await?;
        file.write_all(json.as_bytes()).await?;

        // Update cache
        let mut cache = self.tenants_cache.write().await;
        cache.insert(tenant.tenant_id.clone(), tenant.clone());

        info!("Tenant saved: {}", tenant.tenant_id);
        Ok(())
    }

    /// Load tenant configuration from file
    pub async fn load_tenant(&self, tenant_id: &str) -> Result<Option<TenantConfig>> {
        // Check cache first
        {
            let cache = self.tenants_cache.read().await;
            if let Some(tenant) = cache.get(tenant_id) {
                return Ok(Some(tenant.clone()));
            }
        }

        let tenant_file = self.storage_dir.join("tenants").join(format!("{}.json", tenant_id));

        if !tenant_file.exists() {
            debug!("Tenant file not found: {}", tenant_id);
            return Ok(None);
        }

        let mut file = File::open(tenant_file).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let tenant: TenantConfig = serde_json::from_str(&contents)?;

        // Update cache
        let mut cache = self.tenants_cache.write().await;
        cache.insert(tenant_id.to_string(), tenant.clone());

        Ok(Some(tenant))
    }

    /// List all tenants
    pub async fn list_tenants(&self) -> Result<Vec<TenantConfig>> {
        let tenants_dir = self.storage_dir.join("tenants");
        let mut tenants = Vec::new();

        let mut entries = tokio::fs::read_dir(tenants_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(Some(tenant)) = self.load_tenant_from_path(&path).await {
                    tenants.push(tenant);
                }
            }
        }

        Ok(tenants)
    }

    /// Load tenant from file path
    async fn load_tenant_from_path(&self, path: &Path) -> Result<Option<TenantConfig>> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(Some(serde_json::from_str(&contents)?))
    }

    /// Delete tenant
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<()> {
        let tenant_file = self.storage_dir.join("tenants").join(format!("{}.json", tenant_id));

        if tenant_file.exists() {
            tokio::fs::remove_file(tenant_file).await?;
        }

        // Remove from cache
        let mut cache = self.tenants_cache.write().await;
        cache.remove(tenant_id);

        info!("Tenant deleted: {}", tenant_id);
        Ok(())
    }

    /// Save PSK to file
    pub async fn save_psk(&self, psk: &PreSharedKey) -> Result<()> {
        let psk_file = self.storage_dir.join("psks").join(format!("{}.json", psk.psk_id));

        let json = serde_json::to_string_pretty(psk)?;
        let mut file = File::create(psk_file).await?;
        file.write_all(json.as_bytes()).await?;

        // Update cache
        let mut cache = self.psks_cache.write().await;
        cache.insert(psk.psk_id.clone(), psk.clone());

        debug!("PSK saved: {}", psk.psk_id);
        Ok(())
    }

    /// Load PSK from file
    pub async fn load_psk(&self, psk_id: &str) -> Result<Option<PreSharedKey>> {
        // Check cache first
        {
            let cache = self.psks_cache.read().await;
            if let Some(psk) = cache.get(psk_id) {
                return Ok(Some(psk.clone()));
            }
        }

        let psk_file = self.storage_dir.join("psks").join(format!("{}.json", psk_id));

        if !psk_file.exists() {
            debug!("PSK file not found: {}", psk_id);
            return Ok(None);
        }

        let mut file = File::open(psk_file).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let psk: PreSharedKey = serde_json::from_str(&contents)?;

        // Update cache
        let mut cache = self.psks_cache.write().await;
        cache.insert(psk_id.to_string(), psk.clone());

        Ok(Some(psk))
    }

    /// List all PSKs for a tenant
    pub async fn list_tenant_psks(&self, tenant_id: &str) -> Result<Vec<PreSharedKey>> {
        let psks_dir = self.storage_dir.join("psks");
        let mut psks = Vec::new();

        let mut entries = tokio::fs::read_dir(psks_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(Some(psk)) = self.load_psk_from_path(&path).await {
                    if psk.tenant_id == tenant_id {
                        psks.push(psk);
                    }
                }
            }
        }

        Ok(psks)
    }

    /// Load PSK from file path
    async fn load_psk_from_path(&self, path: &Path) -> Result<Option<PreSharedKey>> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(Some(serde_json::from_str(&contents)?))
    }

    /// Delete PSK
    pub async fn delete_psk(&self, psk_id: &str) -> Result<()> {
        let psk_file = self.storage_dir.join("psks").join(format!("{}.json", psk_id));

        if psk_file.exists() {
            tokio::fs::remove_file(psk_file).await?;
        }

        // Remove from cache
        let mut cache = self.psks_cache.write().await;
        cache.remove(psk_id);

        info!("PSK deleted: {}", psk_id);
        Ok(())
    }

    /// Load all data into cache on startup
    pub async fn load_all(&self) -> Result<()> {
        info!("Loading all tenants and PSKs from storage...");

        // Load all tenants
        let tenants = self.list_tenants().await?;
        let mut tenant_cache = self.tenants_cache.write().await;
        for tenant in tenants {
            tenant_cache.insert(tenant.tenant_id.clone(), tenant);
        }

        // Load all PSKs
        let psks_dir = self.storage_dir.join("psks");
        if psks_dir.exists() {
            let mut psk_cache = self.psks_cache.write().await;
            let mut entries = tokio::fs::read_dir(psks_dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(Some(psk)) = self.load_psk_from_path(&path).await {
                        psk_cache.insert(psk.psk_id.clone(), psk);
                    }
                }
            }
        }

        info!("Storage loaded successfully");
        Ok(())
    }

    /// Get tenant from cache
    pub async fn get_tenant_cached(&self, tenant_id: &str) -> Option<TenantConfig> {
        let cache = self.tenants_cache.read().await;
        cache.get(tenant_id).cloned()
    }

    /// Get PSK from cache
    pub async fn get_psk_cached(&self, psk_id: &str) -> Option<PreSharedKey> {
        let cache = self.psks_cache.read().await;
        cache.get(psk_id).cloned()
    }
}

/// Storage events for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub entity_id: String,
    pub entity_type: String,
    pub details: Option<String>,
}

impl StorageEvent {
    pub fn new(event_type: &str, entity_id: &str, entity_type: &str) -> Self {
        StorageEvent {
            timestamp: chrono::Utc::now(),
            event_type: event_type.to_string(),
            entity_id: entity_id.to_string(),
            entity_type: entity_type.to_string(),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_storage() {
        let storage = TenantStorage::new("./test_storage").await.unwrap();
        let tenant = TenantConfig::new("test_tenant", "Test Tenant", "sk_test_");

        storage.save_tenant(&tenant).await.unwrap();
        let loaded = storage.load_tenant("test_tenant").await.unwrap();

        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().tenant_id, "test_tenant");

        // Cleanup
        let _ = tokio::fs::remove_dir_all("./test_storage").await;
    }
}
