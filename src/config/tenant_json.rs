// ============================================================================
// Tenant JSON Configurations (v3.1) - Per-Tenant Configuration Files
// ============================================================================
// Each tenant has a single JSON file containing:
// - Tenant metadata
// - Configuration settings (rate limits, max connections, etc.)
// - Credentials/PSKs
// - Login history (recent entries)
// - Metrics (last 1000 entries)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

/// Complete tenant configuration stored in t_XXX.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantJsonConfig {
    pub tenant: TenantMetadata,
    pub configuration: TenantSettings,
    pub credentials: Vec<Credential>,
    pub login_history: Vec<LoginHistoryEntry>,
    #[serde(default)]
    pub metrics: Vec<MetricsSnapshot>,
}

/// Tenant metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMetadata {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Per-tenant configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub rate_limit_rps: u32,
    pub max_connections: u32,
    pub max_queue_size: u64,
    pub compression_enabled: bool,
    pub retention_days: u32,
    pub priority_queue_enabled: bool,
    pub ordered_delivery: bool,
    pub message_deduplication: bool,
    pub dead_letter_queue_enabled: bool,
    #[serde(default)]
    pub custom_settings: HashMap<String, String>,
}

impl Default for TenantSettings {
    fn default() -> Self {
        TenantSettings {
            rate_limit_rps: 1000,
            max_connections: 100,
            max_queue_size: 1_000_000,
            compression_enabled: true,
            retention_days: 30,
            priority_queue_enabled: true,
            ordered_delivery: true,
            message_deduplication: true,
            dead_letter_queue_enabled: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// Credential stored per tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub username: String,
    pub psk: String, // Pre-shared key
    pub password_hash: Option<String>,
    pub created_at: String,
    pub last_rotated_at: Option<String>,
    pub enabled: bool,
}

/// Login history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginHistoryEntry {
    pub username: String,
    pub login_timestamp: String,
    pub ip_address: String,
    pub success: bool,
    pub session_id: String,
}

/// Metrics snapshot (in-memory metrics pushed to JSON every 100ms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: String,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub active_connections: u32,
    pub rate_limit_violations: u32,
    pub average_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub errors: u32,
}

impl TenantJsonConfig {
    /// Load tenant configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        let content = fs::read_to_string(path)
            .context(format!("Failed to read tenant config from {:?}", path))?;
        
        let config: TenantJsonConfig = serde_json::from_str(&content)
            .context(format!(
                "Failed to parse tenant config from {:?}. Invalid JSON format",
                path
            ))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save tenant configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create tenant config directory")?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize tenant config to JSON")?;
        
        fs::write(path, json)
            .context(format!("Failed to write tenant config to {:?}", path))?;
        
        Ok(())
    }
    
    /// Load all tenant configurations from directory
    pub fn load_all<P: AsRef<Path>>(dir: P) -> Result<HashMap<String, Self>> {
        let dir = dir.as_ref();
        
        if !dir.exists() {
            anyhow::bail!("Tenants directory does not exist: {:?}", dir);
        }
        
        let mut tenants = HashMap::new();
        
        for entry in fs::read_dir(dir)
            .context(format!("Failed to read tenants directory: {:?}", dir))?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // Only process .json files
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                match Self::from_file(&path) {
                    Ok(config) => {
                        tenants.insert(config.tenant.id.clone(), config);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load tenant config from {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(tenants)
    }
    
    /// Validate the tenant configuration
    fn validate(&self) -> Result<()> {
        // Validate tenant ID is not empty
        if self.tenant.id.is_empty() {
            anyhow::bail!("Tenant ID cannot be empty");
        }
        
        // Validate rate limit
        if self.configuration.rate_limit_rps == 0 {
            anyhow::bail!("Rate limit must be greater than 0");
        }
        
        // Validate max connections
        if self.configuration.max_connections == 0 {
            anyhow::bail!("Max connections must be greater than 0");
        }
        
        // Validate at least one credential exists
        if self.credentials.is_empty() {
            anyhow::bail!("At least one credential must be configured");
        }
        
        // Validate PSK length
        for cred in &self.credentials {
            if cred.psk.len() < 32 {
                anyhow::bail!("PSK must be at least 32 characters for credential: {}", cred.id);
            }
        }
        
        Ok(())
    }
    
    /// Get enabled credentials
    pub fn get_enabled_credentials(&self) -> Vec<&Credential> {
        self.credentials.iter().filter(|c| c.enabled).collect()
    }
    
    /// Find credential by PSK
    pub fn find_credential_by_psk(&self, psk: &str) -> Option<&Credential> {
        self.credentials.iter().find(|c| c.psk == psk && c.enabled)
    }
    
    /// Find credential by username
    pub fn find_credential_by_username(&self, username: &str) -> Option<&Credential> {
        self.credentials.iter().find(|c| c.username == username && c.enabled)
    }
    
    /// Add login history entry (keep only last N entries)
    pub fn add_login_history(
        &mut self,
        username: String,
        ip_address: String,
        success: bool,
        max_entries: usize,
    ) {
        let entry = LoginHistoryEntry {
            username,
            login_timestamp: Utc::now().to_rfc3339(),
            ip_address,
            success,
            session_id: uuid::Uuid::new_v4().to_string(),
        };
        
        self.login_history.push(entry);
        
        // Keep only last N entries
        if self.login_history.len() > max_entries {
            self.login_history = self.login_history
                .drain(self.login_history.len() - max_entries..)
                .collect();
        }
    }
    
    /// Add metrics snapshot (keep only last N entries, typically 1000)
    pub fn add_metrics_snapshot(&mut self, metrics: MetricsSnapshot, max_entries: usize) {
        self.metrics.push(metrics);
        
        // Keep only last N entries (sliding window)
        if self.metrics.len() > max_entries {
            self.metrics = self.metrics
                .drain(self.metrics.len() - max_entries..)
                .collect();
        }
    }
    
    /// Create a new tenant configuration with defaults
    pub fn new(tenant_id: String, tenant_name: String) -> Self {
        let now = Utc::now().to_rfc3339();
        
        TenantJsonConfig {
            tenant: TenantMetadata {
                id: tenant_id,
                name: tenant_name,
                created_at: now.clone(),
                updated_at: now,
            },
            configuration: TenantSettings::default(),
            credentials: vec![],
            login_history: vec![],
            metrics: vec![],
        }
    }
    
    /// Add a credential
    pub fn add_credential(&mut self, credential: Credential) {
        self.credentials.push(credential);
    }
    
    /// Create a sample tenant configuration
    pub fn example() -> Self {
        let now = Utc::now().to_rfc3339();
        
        TenantJsonConfig {
            tenant: TenantMetadata {
                id: "t_acme".to_string(),
                name: "acme-corp".to_string(),
                created_at: now.clone(),
                updated_at: now.clone(),
            },
            configuration: TenantSettings {
                rate_limit_rps: 1000,
                max_connections: 100,
                max_queue_size: 1_000_000,
                compression_enabled: true,
                retention_days: 30,
                priority_queue_enabled: true,
                ordered_delivery: true,
                message_deduplication: true,
                dead_letter_queue_enabled: true,
                custom_settings: HashMap::new(),
            },
            credentials: vec![
                Credential {
                    id: "c001".to_string(),
                    username: "api_user_1".to_string(),
                    psk: "0123456789abcdef0123456789abcdef01234567".to_string(),
                    password_hash: Some("hash".to_string()),
                    created_at: now.clone(),
                    last_rotated_at: None,
                    enabled: true,
                },
                Credential {
                    id: "c002".to_string(),
                    username: "api_user_2".to_string(),
                    psk: "fedcba9876543210fedcba9876543210fedcba98".to_string(),
                    password_hash: Some("hash".to_string()),
                    created_at: now,
                    last_rotated_at: None,
                    enabled: true,
                },
            ],
            login_history: vec![],
            metrics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_tenant_config() {
        let mut config = TenantJsonConfig::new("t_test".to_string(), "test-tenant".to_string());
        
        assert_eq!(config.tenant.id, "t_test");
        assert_eq!(config.tenant.name, "test-tenant");
        
        let cred = Credential {
            id: "c1".to_string(),
            username: "user1".to_string(),
            psk: "01234567890123456789012345678901".to_string(),
            password_hash: None,
            created_at: Utc::now().to_rfc3339(),
            last_rotated_at: None,
            enabled: true,
        };
        
        config.add_credential(cred.clone());
        assert_eq!(config.credentials.len(), 1);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_example_config() {
        let config = TenantJsonConfig::example();
        assert_eq!(config.tenant.id, "t_acme");
        assert_eq!(config.credentials.len(), 2);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_metrics_sliding_window() {
        let mut config = TenantJsonConfig::example();
        
        // Add more than 1000 metrics snapshots
        for i in 0..1005 {
            config.add_metrics_snapshot(
                MetricsSnapshot {
                    timestamp: format!("2026-04-12T10:00:{:02}", i % 60),
                    messages_received: i as u64,
                    messages_sent: i as u64,
                    bytes_in: (i * 1024) as u64,
                    bytes_out: (i * 1024) as u64,
                    active_connections: 10,
                    rate_limit_violations: 0,
                    average_latency_ms: 5.0,
                    p99_latency_ms: 20.0,
                    errors: 0,
                },
                1000,
            );
        }
        
        // Should only keep 1000 entries
        assert_eq!(config.metrics.len(), 1000);
    }
}
