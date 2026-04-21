// ============================================================================
// AppSettings - Application Configuration (like appsettings.json in .NET)
// Centralized configuration management for FastDataBroker
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::config::tenant::TenantConfig;

/// Main application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Application name and version
    pub app: AppConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// List of tenants
    pub tenants: Vec<TenantConfig>,
    /// Feature flags
    #[serde(default)]
    pub features: FeatureFlags,
}

/// Application metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            name: "FastDataBroker".to_string(),
            version: "0.1.16".to_string(),
            environment: "development".to_string(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind address
    pub bind_address: String,
    /// Bind port
    pub port: u16,
    /// TLS certificate path
    pub cert_path: String,
    /// TLS private key path
    pub key_path: String,
    /// Enable TLS
    pub enable_tls: bool,
    /// Global max connections
    pub max_connections: usize,
    /// Global max streams per connection
    pub max_streams: u64,
    /// Idle timeout in milliseconds
    pub idle_timeout_ms: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Whether to enable metrics
    pub enable_metrics: bool,
    /// Metrics export interval in seconds
    pub metrics_interval_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 6379,
            cert_path: "./certs/cert.pem".to_string(),
            key_path: "./certs/key.pem".to_string(),
            enable_tls: false,
            max_connections: 10000,
            max_streams: 1000000,
            idle_timeout_ms: 30000,
            request_timeout_secs: 30,
            enable_metrics: true,
            metrics_interval_secs: 60,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, text)
    pub format: String,
    /// Log output (console, file, both)
    pub output: String,
    /// Log file path (if output includes file)
    pub file_path: Option<String>,
    /// Include target in logs
    pub include_target: bool,
    /// Include thread IDs in logs
    pub include_thread_ids: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            format: "text".to_string(),
            output: "console".to_string(),
            file_path: None,
            include_target: true,
            include_thread_ids: true,
        }
    }
}

/// Global feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable multi-tenancy
    pub multi_tenancy: bool,
    /// Enable authentication
    pub authentication: bool,
    /// Enable rate limiting
    pub rate_limiting: bool,
    /// Enable persistence
    pub persistence: bool,
    /// Enable clustering
    pub clustering: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        FeatureFlags {
            multi_tenancy: true,
            authentication: true,
            rate_limiting: true,
            persistence: true,
            clustering: false,
        }
    }
}

impl AppSettings {
    /// Create default settings
    pub fn default() -> Self {
        AppSettings {
            app: AppConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            tenants: vec![],
            features: FeatureFlags::default(),
        }
    }

    /// Load settings from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        settings.validate()?;
        Ok(settings)
    }

    /// Load settings with environment-specific overrides
    pub fn from_env(base_path: &str, environment: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load base configuration
        let mut settings = AppSettings::from_file(base_path)?;

        // Load environment-specific configuration if it exists
        let env_file = format!(
            "{}settings.{}.json",
            base_path.trim_end_matches("appsettings.json"),
            environment
        );

        if Path::new(&env_file).exists() {
            let env_content = fs::read_to_string(&env_file)?;
            let env_settings: AppSettings = serde_json::from_str(&env_content)?;
            
            // Merge environment-specific settings over base
            settings.app = env_settings.app;
            settings.server = env_settings.server;
            settings.logging = env_settings.logging;
            settings.features = env_settings.features;
            
            // Merge tenants
            for tenant in env_settings.tenants {
                if !settings.tenants.iter().any(|t| t.tenant_id == tenant.tenant_id) {
                    settings.tenants.push(tenant);
                }
            }

            settings.validate()?;
        }

        Ok(settings)
    }

    /// Save settings to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Validate all settings
    pub fn validate(&self) -> Result<(), String> {
        if self.app.name.is_empty() {
            return Err("app.name cannot be empty".to_string());
        }

        if self.server.port == 0 {
            return Err("server.port must be greater than 0".to_string());
        }

        // Validate all tenants
        for tenant in &self.tenants {
            tenant.validate()?;
        }

        Ok(())
    }

    /// Get tenant by ID
    pub fn get_tenant(&self, tenant_id: &str) -> Option<&TenantConfig> {
        self.tenants.iter().find(|t| t.tenant_id == tenant_id)
    }

    /// Get tenant by API key prefix
    pub fn get_tenant_by_prefix(&self, prefix: &str) -> Option<&TenantConfig> {
        self.tenants
            .iter()
            .find(|t| prefix.starts_with(&t.api_key_prefix))
    }

    /// Add a new tenant
    pub fn add_tenant(&mut self, tenant: TenantConfig) -> Result<(), String> {
        tenant.validate()?;

        if self.tenants.iter().any(|t| t.tenant_id == tenant.tenant_id) {
            return Err(format!("Tenant {} already exists", tenant.tenant_id));
        }

        self.tenants.push(tenant);
        Ok(())
    }

    /// Remove a tenant
    pub fn remove_tenant(&mut self, tenant_id: &str) -> bool {
        let initial_count = self.tenants.len();
        self.tenants.retain(|t| t.tenant_id != tenant_id);
        self.tenants.len() < initial_count
    }

    /// Get number of active tenants
    pub fn active_tenant_count(&self) -> usize {
        self.tenants.iter().filter(|t| t.enabled).count()
    }

    /// Get server URL
    pub fn server_url(&self) -> String {
        format!("{}://{}:{}", 
            if self.server.enable_tls { "quics" } else { "quic" },
            self.server.bind_address,
            self.server.port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.app.name, "FastDataBroker");
        assert_eq!(settings.server.port, 6379);
        assert!(settings.features.multi_tenancy);
    }

    #[test]
    fn test_validate_empty_name() {
        let mut settings = AppSettings::default();
        settings.app.name = String::new();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_add_tenant() {
        let mut settings = AppSettings::default();
        let tenant = TenantConfig::new("tenant1", "Tenant One", "sk_prod_t1_");
        
        assert!(settings.add_tenant(tenant).is_ok());
        assert_eq!(settings.tenants.len(), 1);
        assert_eq!(settings.active_tenant_count(), 1);
    }

    #[test]
    fn test_get_tenant() {
        let mut settings = AppSettings::default();
        let tenant = TenantConfig::new("tenant1", "Tenant One", "sk_prod_t1_");
        settings.add_tenant(tenant).unwrap();

        let found = settings.get_tenant("tenant1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().tenant_name, "Tenant One");
    }

    #[test]
    fn test_get_tenant_by_prefix() {
        let mut settings = AppSettings::default();
        let tenant = TenantConfig::new("tenant1", "Tenant One", "sk_prod_t1_");
        settings.add_tenant(tenant).unwrap();

        let found = settings.get_tenant_by_prefix("sk_prod_t1_abc123");
        assert!(found.is_some());
    }

    #[test]
    fn test_remove_tenant() {
        let mut settings = AppSettings::default();
        let tenant = TenantConfig::new("tenant1", "Tenant One", "sk_prod_t1_");
        settings.add_tenant(tenant).unwrap();

        assert!(settings.remove_tenant("tenant1"));
        assert_eq!(settings.tenants.len(), 0);
    }

    #[test]
    fn test_server_url() {
        let settings = AppSettings::default();
        assert_eq!(settings.server_url(), "quic://0.0.0.0:6379");
    }

    #[test]
    fn test_server_url_with_tls() {
        let mut settings = AppSettings::default();
        settings.server.enable_tls = true;
        assert_eq!(settings.server_url(), "quics://0.0.0.0:6379");
    }
}

