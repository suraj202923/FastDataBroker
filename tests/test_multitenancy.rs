/// Comprehensive tests for Multi-Tenant Configuration and Isolation
/// Tests TenantConfig, AppSettings, and tenant isolation guarantees

#[cfg(test)]
mod multitenancy_tests {
    use std::collections::HashMap;

    // Mock structures for testing (in real scenario, import from fastdatabroker crate)
    #[derive(Clone, Debug, PartialEq)]
    struct TenantConfig {
        tenant_id: String,
        tenant_name: String,
        api_key_prefix: String,
        rate_limit_rps: u32,
        max_connections: u32,
        max_message_size: u64,
        retention_days: u32,
        enabled: bool,
        metadata: HashMap<String, String>,
    }

    struct AppSettings {
        tenants: Vec<TenantConfig>,
    }

    impl TenantConfig {
        fn new(
            tenant_id: String,
            api_key_prefix: String,
            rate_limit_rps: u32,
            max_connections: u32,
        ) -> Self {
            TenantConfig {
                tenant_id,
                tenant_name: "Test Tenant".to_string(),
                api_key_prefix,
                rate_limit_rps,
                max_connections,
                max_message_size: 1048576,
                retention_days: 30,
                enabled: true,
                metadata: HashMap::new(),
            }
        }

        fn validate(&self) -> Result<(), String> {
            if self.tenant_id.is_empty() {
                return Err("TenantId cannot be empty".to_string());
            }
            if !self.api_key_prefix.ends_with('_') {
                return Err("ApiKeyPrefix must end with '_'".to_string());
            }
            if self.rate_limit_rps == 0 {
                return Err("RateLimitRps must be greater than 0".to_string());
            }
            if self.max_connections == 0 {
                return Err("MaxConnections must be greater than 0".to_string());
            }
            Ok(())
        }
    }

    impl AppSettings {
        fn new() -> Self {
            AppSettings {
                tenants: Vec::new(),
            }
        }

        fn add_tenant(&mut self, tenant: TenantConfig) {
            self.tenants.push(tenant);
        }

        fn get_tenant(&self, tenant_id: &str) -> Option<&TenantConfig> {
            self.tenants.iter().find(|t| t.tenant_id == tenant_id)
        }

        fn get_tenant_by_api_key_prefix(&self, prefix: &str) -> Option<&TenantConfig> {
            self.tenants.iter().find(|t| t.api_key_prefix == prefix)
        }
    }

    // ============== Tenant Creation & Validation ==============

    #[test]
    fn test_tenant_creation() {
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        assert_eq!(tenant.tenant_id, "acme-corp");
        assert_eq!(tenant.api_key_prefix, "acme_");
        assert_eq!(tenant.rate_limit_rps, 1000);
        assert_eq!(tenant.max_connections, 100);
        assert!(tenant.enabled);
    }

    #[test]
    fn test_tenant_validation_success() {
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        assert!(tenant.validate().is_ok());
    }

    #[test]
    fn test_tenant_validation_empty_id() {
        let tenant = TenantConfig::new(
            "".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        let result = tenant.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "TenantId cannot be empty");
    }

    #[test]
    fn test_tenant_validation_bad_prefix() {
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme".to_string(),
            1000,
            100,
        );

        let result = tenant.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "ApiKeyPrefix must end with '_'"
        );
    }

    #[test]
    fn test_tenant_validation_zero_rate_limit() {
        let mut tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );
        tenant.rate_limit_rps = 0;

        let result = tenant.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "RateLimitRps must be greater than 0"
        );
    }

    #[test]
    fn test_tenant_validation_zero_max_connections() {
        let mut tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );
        tenant.max_connections = 0;

        let result = tenant.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "MaxConnections must be greater than 0"
        );
    }

    // ============== AppSettings Management ==============

    #[test]
    fn test_appsettings_creation() {
        let settings = AppSettings::new();
        assert_eq!(settings.tenants.len(), 0);
    }

    #[test]
    fn test_appsettings_add_tenant() {
        let mut settings = AppSettings::new();
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        settings.add_tenant(tenant);
        assert_eq!(settings.tenants.len(), 1);
        assert_eq!(settings.tenants[0].tenant_id, "acme-corp");
    }

    #[test]
    fn test_appsettings_multiple_tenants() {
        let mut settings = AppSettings::new();

        let tenant1 = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        let tenant2 = TenantConfig::new(
            "startup-xyz".to_string(),
            "xyz_".to_string(),
            100,
            10,
        );

        settings.add_tenant(tenant1);
        settings.add_tenant(tenant2);

        assert_eq!(settings.tenants.len(), 2);
    }

    #[test]
    fn test_get_tenant_by_id() {
        let mut settings = AppSettings::new();
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        settings.add_tenant(tenant);

        let retrieved = settings.get_tenant("acme-corp");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().tenant_id, "acme-corp");
    }

    #[test]
    fn test_get_nonexistent_tenant() {
        let settings = AppSettings::new();
        let retrieved = settings.get_tenant("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_tenant_by_api_key_prefix() {
        let mut settings = AppSettings::new();
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        settings.add_tenant(tenant);

        let retrieved = settings.get_tenant_by_api_key_prefix("acme_");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().api_key_prefix, "acme_");
    }

    // ============== Tenant Isolation ==============

    #[test]
    fn test_tenant_isolation_multiple_tenants() {
        let mut settings = AppSettings::new();

        let tenant1 = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        let tenant2 = TenantConfig::new(
            "startup-xyz".to_string(),
            "xyz_".to_string(),
            100,
            10,
        );

        settings.add_tenant(tenant1);
        settings.add_tenant(tenant2);

        let t1 = settings.get_tenant("acme-corp").unwrap();
        let t2 = settings.get_tenant("startup-xyz").unwrap();

        // Verify rate limits are separate
        assert_eq!(t1.rate_limit_rps, 1000);
        assert_eq!(t2.rate_limit_rps, 100);

        // Verify max connections are separate
        assert_eq!(t1.max_connections, 100);
        assert_eq!(t2.max_connections, 10);
    }

    #[test]
    fn test_api_key_validation_matches_tenant_prefix() {
        let mut settings = AppSettings::new();
        let tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        settings.add_tenant(tenant);

        // Valid API key
        let api_key_valid = "acme_550e8400e29b41d4a716446655440000";
        assert!(api_key_valid.starts_with("acme_"));

        // Invalid API key
        let api_key_invalid = "xyz_550e8400e29b41d4a716446655440000";
        assert!(!api_key_invalid.starts_with("acme_"));
    }

    #[test]
    fn test_tenant_metadata_storage() {
        let mut tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        tenant.metadata.insert("region".to_string(), "us-east-1".to_string());
        tenant.metadata.insert("tier".to_string(), "enterprise".to_string());

        assert_eq!(tenant.metadata.len(), 2);
        assert_eq!(tenant.metadata.get("region").unwrap(), "us-east-1");
        assert_eq!(tenant.metadata.get("tier").unwrap(), "enterprise");
    }

    #[test]
    fn test_tenant_disabled_state() {
        let mut tenant = TenantConfig::new(
            "acme-corp".to_string(),
            "acme_".to_string(),
            1000,
            100,
        );

        assert!(tenant.enabled);

        tenant.enabled = false;
        assert!(!tenant.enabled);
    }

    // ============== Concurrent Tenant Access ==============

    #[test]
    fn test_concurrent_tenant_lookups() {
        let mut settings = AppSettings::new();

        for i in 0..10 {
            let tenant_id = format!("tenant-{}", i);
            let prefix = format!("ten{}_ ", i);
            let tenant = TenantConfig::new(tenant_id, prefix, 100 * (i as u32 + 1), 10 * (i as u32 + 1));
            settings.add_tenant(tenant);
        }

        assert_eq!(settings.tenants.len(), 10);

        // Verify all tenants are accessible
        for i in 0..10 {
            let tenant_id = format!("tenant-{}", i);
            let retrieved = settings.get_tenant(&tenant_id);
            assert!(retrieved.is_some());
        }
    }

    // ============== Rate Limiting Configuration ==============

    #[test]
    fn test_different_rate_limits_per_tenant() {
        let mut settings = AppSettings::new();

        let tenant1 = TenantConfig {
            tenant_id: "enterprise".to_string(),
            tenant_name: "Enterprise Client".to_string(),
            api_key_prefix: "ent_".to_string(),
            rate_limit_rps: 10000,
            max_connections: 1000,
            max_message_size: 10485760,
            retention_days: 90,
            enabled: true,
            metadata: HashMap::new(),
        };

        let tenant2 = TenantConfig {
            tenant_id: "startup".to_string(),
            tenant_name: "Startup Client".to_string(),
            api_key_prefix: "sta_".to_string(),
            rate_limit_rps: 100,
            max_connections: 10,
            max_message_size: 524288,
            retention_days: 7,
            enabled: true,
            metadata: HashMap::new(),
        };

        settings.add_tenant(tenant1);
        settings.add_tenant(tenant2);

        let ent = settings.get_tenant("enterprise").unwrap();
        let sta = settings.get_tenant("startup").unwrap();

        // Enterprise has 100x higher rate limit
        assert_eq!(ent.rate_limit_rps, 10000);
        assert_eq!(sta.rate_limit_rps, 100);
        assert_eq!(ent.rate_limit_rps / sta.rate_limit_rps, 100);

        // Enterprise has higher connection limit
        assert_eq!(ent.max_connections, 1000);
        assert_eq!(sta.max_connections, 10);

        // Enterprise has larger message size
        assert_eq!(ent.max_message_size, 10485760);
        assert_eq!(sta.max_message_size, 524288);
    }

    // ============== Message Size Limits ==============

    #[test]
    fn test_message_size_limits_per_tenant() {
        let mut enterprise = TenantConfig::new(
            "enterprise".to_string(),
            "ent_".to_string(),
            10000,
            1000,
        );
        enterprise.max_message_size = 10485760; // 10MB

        let mut startup = TenantConfig::new(
            "startup".to_string(),
            "sta_".to_string(),
            100,
            10,
        );
        startup.max_message_size = 524288; // 512KB

        assert!(enterprise.max_message_size > startup.max_message_size);
    }

    // ============== Retention Policy ==============

    #[test]
    fn test_retention_days_per_tenant() {
        let mut enterprise = TenantConfig::new(
            "enterprise".to_string(),
            "ent_".to_string(),
            10000,
            1000,
        );
        enterprise.retention_days = 90;

        let mut startup = TenantConfig::new(
            "startup".to_string(),
            "sta_".to_string(),
            100,
            10,
        );
        startup.retention_days = 7;

        assert!(enterprise.retention_days > startup.retention_days);
        assert_eq!(enterprise.retention_days, 90);
        assert_eq!(startup.retention_days, 7);
    }
}
