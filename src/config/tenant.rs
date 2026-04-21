// ============================================================================
// Tenant Configuration - Multi-Tenant Support
// Each tenant has isolated queues, API keys, and rate limits
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Unique tenant identifier (e.g., "acme-corp", "startup-xyz")
    pub tenant_id: String,
    /// Human-readable tenant name
    pub tenant_name: String,
    /// Tenant description
    pub description: Option<String>,
    /// API key prefix for tenant (e.g., "sk_prod_tenant1_")
    pub api_key_prefix: String,
    /// Rate limit per tenant (requests/second)
    pub rate_limit_rps: u32,
    /// Max concurrent connections for this tenant
    pub max_connections: usize,
    /// Max message size in bytes (e.g., 1MB = 1_048_576)
    pub max_message_size: u64,
    /// Data retention period in days
    pub retention_days: u32,
    /// Enabled features for this tenant
    pub features: TenantFeatures,
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Is tenant active/enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Feature flags for tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantFeatures {
    /// Allow priority queue usage
    pub priority_queue: bool,
    /// Allow scheduled/delayed messages
    pub scheduled_messages: bool,
    /// Allow message routing/topics
    pub routing: bool,
    /// Allow webhooks for delivery
    pub webhooks: bool,
    /// Allow clustering/distribution
    pub clustering: bool,
    /// Allow metrics export
    pub metrics: bool,
    /// Allow persistence
    pub persistence: bool,
}

impl Default for TenantFeatures {
    fn default() -> Self {
        TenantFeatures {
            priority_queue: true,
            scheduled_messages: true,
            routing: true,
            webhooks: true,
            clustering: false,
            metrics: true,
            persistence: true,
        }
    }
}

impl TenantConfig {
    /// Create a new tenant configuration
    pub fn new(tenant_id: &str, tenant_name: &str, api_key_prefix: &str) -> Self {
        TenantConfig {
            tenant_id: tenant_id.to_string(),
            tenant_name: tenant_name.to_string(),
            description: None,
            api_key_prefix: api_key_prefix.to_string(),
            rate_limit_rps: 1000,
            max_connections: 100,
            max_message_size: 1_048_576, // 1MB default
            retention_days: 7,
            features: TenantFeatures::default(),
            metadata: HashMap::new(),
            enabled: true,
        }
    }

    /// Validate tenant configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.tenant_id.is_empty() {
            return Err("tenant_id cannot be empty".to_string());
        }

        if self.tenant_name.is_empty() {
            return Err("tenant_name cannot be empty".to_string());
        }

        if self.api_key_prefix.is_empty() {
            return Err("api_key_prefix cannot be empty".to_string());
        }

        if !self.api_key_prefix.ends_with('_') {
            return Err("api_key_prefix should end with '_'".to_string());
        }

        if self.rate_limit_rps == 0 {
            return Err("rate_limit_rps must be greater than 0".to_string());
        }

        if self.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }

        if self.max_message_size == 0 {
            return Err("max_message_size must be greater than 0".to_string());
        }

        if self.retention_days == 0 {
            return Err("retention_days must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "priority_queue" => self.features.priority_queue,
            "scheduled_messages" => self.features.scheduled_messages,
            "routing" => self.features.routing,
            "webhooks" => self.features.webhooks,
            "clustering" => self.features.clustering,
            "metrics" => self.features.metrics,
            "persistence" => self.features.persistence,
            _ => false,
        }
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_new() {
        let tenant = TenantConfig::new("acme", "ACME Corp", "sk_prod_acme_");
        assert_eq!(tenant.tenant_id, "acme");
        assert_eq!(tenant.tenant_name, "ACME Corp");
        assert_eq!(tenant.api_key_prefix, "sk_prod_acme_");
        assert!(tenant.enabled);
    }

    #[test]
    fn test_tenant_validate_success() {
        let tenant = TenantConfig::new("acme", "ACME Corp", "sk_prod_acme_");
        assert!(tenant.validate().is_ok());
    }

    #[test]
    fn test_tenant_validate_empty_id() {
        let mut tenant = TenantConfig::new("acme", "ACME Corp", "sk_prod_acme_");
        tenant.tenant_id = String::new();
        assert!(tenant.validate().is_err());
    }

    #[test]
    fn test_tenant_validate_invalid_prefix() {
        let mut tenant = TenantConfig::new("acme", "ACME Corp", "sk_prod_acme_");
        tenant.api_key_prefix = "invalid".to_string();
        assert!(tenant.validate().is_err());
    }

    #[test]
    fn test_feature_check() {
        let tenant = TenantConfig::new("acme", "ACME Corp", "sk_prod_acme_");
        assert!(tenant.is_feature_enabled("priority_queue"));
        assert!(tenant.is_feature_enabled("routing"));
        assert!(!tenant.is_feature_enabled("clustering"));
    }

    #[test]
    fn test_metadata() {
        let mut tenant = TenantConfig::new("acme", "ACME Corp", "sk_prod_acme_");
        tenant.metadata.insert("department".to_string(), "engineering".to_string());
        assert_eq!(
            tenant.get_metadata("department"),
            Some(&"engineering".to_string())
        );
    }
}
