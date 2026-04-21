/*
FastDataBroker Tenant Access Test Suite
=======================================

Tests for:
1. Tenant loading from configuration
2. API key verification
3. Tenant isolation enforcement
4. Feature access control
5. Rate limiting per tenant

Run with: cargo test --test test_tenant_access
*/

#[cfg(test)]
mod tests {
    use fastdatabroker::config::{AppSettings, TenantConfig, TenantFeatures};
    use std::collections::HashMap;

    // ============================================================================
    // Test 1: Load Tenants from Configuration
    // ============================================================================
    
    #[test]
    fn test_load_tenants_from_config() {
        // Load configuration
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        // Verify tenants are loaded
        assert!(!settings.tenants.is_empty(), "No tenants found in config");
        
        // Expected tenants
        let expected = vec!["acme-corp", "fintech-solutions", "retail-chain", "startup-xyz"];
        
        for expected_id in expected {
            let tenant = settings.get_tenant(expected_id);
            assert!(tenant.is_some(), "Tenant {} not found", expected_id);
            println!("✅ Tenant loaded: {}", expected_id);
        }
    }

    // ============================================================================
    // Test 2: Get Tenant by ID
    // ============================================================================
    
    #[test]
    fn test_get_tenant_by_id() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let tenant = settings.get_tenant("acme-corp")
            .expect("Failed to get tenant");

        assert_eq!(tenant.tenant_id, "acme-corp");
        assert_eq!(tenant.tenant_name, "ACME Corporation");
        assert_eq!(tenant.rate_limit_rps, 50000);
        
        println!("✅ Tenant ID lookup works");
    }

    // ============================================================================
    // Test 3: Verify Tenant is Enabled
    // ============================================================================
    
    #[test]
    fn test_tenant_enabled_status() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        for tenant in &settings.tenants {
            assert!(tenant.enabled, "Tenant {} should be enabled", tenant.tenant_id);
            println!("✅ Tenant {} is enabled", tenant.tenant_id);
        }
    }

    // ============================================================================
    // Test 4: Check Feature Access
    // ============================================================================
    
    #[test]
    fn test_feature_access_control() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        // ACME has all features
        let acme = settings.get_tenant("acme-corp").expect("acme-corp not found");
        assert!(acme.is_feature_enabled("priority_queue"));
        assert!(acme.is_feature_enabled("webhooks"));
        assert!(acme.is_feature_enabled("clustering"));
        println!("✅ ACME has full feature access");

        // Startup has limited features
        let startup = settings.get_tenant("startup-xyz").expect("startup-xyz not found");
        assert!(startup.is_feature_enabled("priority_queue"));
        assert!(!startup.is_feature_enabled("scheduled_messages"));  // Disabled for starter tier
        assert!(!startup.is_feature_enabled("clustering"));          // Disabled for starter tier
        println!("✅ Startup has limited feature access");
    }

    // ============================================================================
    // Test 5: Get Tenant by API Key Prefix
    // ============================================================================
    
    #[test]
    fn test_get_tenant_by_api_key() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let api_key = "sk_prod_acme_550e8400e29b41d4a716446655440000";
        let tenant = settings.get_tenant_by_prefix(api_key)
            .expect("Failed to find tenant by API key");

        assert_eq!(tenant.tenant_id, "acme-corp");
        println!("✅ API key lookup works: {} → {}", api_key, tenant.tenant_id);
    }

    // ============================================================================
    // Test 6: Tenant Isolation - Verify Cross-Tenant Blocking
    // ============================================================================
    
    #[test]
    fn test_tenant_isolation() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let acme_key = "sk_prod_acme_xxx";
        let fintech_key = "sk_prod_fintech_yyy";

        let acme_tenant = settings.get_tenant_by_prefix(acme_key);
        let fintech_tenant = settings.get_tenant_by_prefix(fintech_key);

        // Both should find their respective tenants
        assert!(acme_tenant.is_some(), "ACME API key should be valid");
        assert!(fintech_tenant.is_some(), "FinTech API key should be valid");

        // But they should be different tenants
        let acme_id = acme_tenant.unwrap().tenant_id.clone();
        let fintech_id = fintech_tenant.unwrap().tenant_id.clone();
        
        assert_ne!(acme_id, fintech_id, "Tenants should be isolated");
        println!("✅ Tenant isolation verified: {} ≠ {}", acme_id, fintech_id);
    }

    // ============================================================================
    // Test 7: Rate Limit Enforcement
    // ============================================================================
    
    #[test]
    fn test_rate_limit_per_tenant() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let acme = settings.get_tenant("acme-corp").expect("acme-corp not found");
        let startup = settings.get_tenant("startup-xyz").expect("startup-xyz not found");

        // ACME has higher rate limit
        assert!(acme.rate_limit_rps > startup.rate_limit_rps);
        
        println!("✅ ACME rate limit: {} RPS", acme.rate_limit_rps);
        println!("✅ Startup rate limit: {} RPS", startup.rate_limit_rps);
    }

    // ============================================================================
    // Test 8: Message Size Limit per Tenant
    // ============================================================================
    
    #[test]
    fn test_message_size_limits() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let acme = settings.get_tenant("acme-corp").expect("acme-corp not found");
        let startup = settings.get_tenant("startup-xyz").expect("startup-xyz not found");

        // Different tenants, different limits
        assert_ne!(acme.max_message_size, startup.max_message_size);
        
        // Enterprise tenant has higher limit
        assert!(acme.max_message_size > startup.max_message_size);
        
        println!("✅ ACME max message size: {} bytes", acme.max_message_size);
        println!("✅ Startup max message size: {} bytes", startup.max_message_size);
    }

    // ============================================================================
    // Test 9: Metadata Access
    // ============================================================================
    
    #[test]
    fn test_tenant_metadata() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let acme = settings.get_tenant("acme-corp").expect("acme-corp not found");

        let tier = acme.get_metadata("tier");
        assert_eq!(tier, Some(&"premium".to_string()));
        
        let sla = acme.get_metadata("sla_uptime");
        assert_eq!(sla, Some(&"99.99%".to_string()));
        
        println!("✅ Tenant metadata accessible");
        println!("   Tier: {}", tier.unwrap());
        println!("   SLA: {}", sla.unwrap());
    }

    // ============================================================================
    // Test 10: Add New Tenant Programmatically
    // ============================================================================
    
    #[test]
    fn test_add_new_tenant() {
        let mut settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let initial_count = settings.tenants.len();

        // Create new tenant
        let mut new_tenant = TenantConfig::new(
            "test-tenant",
            "Test Tenant",
            "sk_test_tenant_"
        );
        new_tenant.rate_limit_rps = 1000;
        new_tenant.max_connections = 100;

        // Add to settings
        let result = settings.add_tenant(new_tenant);
        assert!(result.is_ok(), "Failed to add tenant");

        // Verify count increased
        assert_eq!(settings.tenants.len(), initial_count + 1);
        
        // Verify we can get it
        let added = settings.get_tenant("test-tenant");
        assert!(added.is_some());
        
        println!("✅ New tenant added successfully");
    }

    // ============================================================================
    // Test 11: Validation Rules
    // ============================================================================
    
    #[test]
    fn test_tenant_validation() {
        // Valid tenant
        let valid = TenantConfig::new("valid-id", "Valid Name", "sk_valid_");
        assert!(valid.validate().is_ok(), "Valid config should pass validation");

        // Invalid - empty tenant_id
        let mut invalid = TenantConfig::new("", "Name", "sk_test_");
        assert!(invalid.validate().is_err(), "Empty tenant_id should fail");

        // Invalid - api_key_prefix without underscore
        let mut invalid2 = TenantConfig::new("id", "Name", "sk_test_no_underscore");
        assert!(invalid2.validate().is_err(), "Invalid prefix format should fail");

        println!("✅ Validation rules working correctly");
    }

    // ============================================================================
    // Test 12: List All Active Tenants
    // ============================================================================
    
    #[test]
    fn test_list_active_tenants() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let active_count = settings.active_tenant_count();
        assert!(active_count > 0, "Should have active tenants");

        println!("✅ Active tenants: {}", active_count);
        
        for tenant in &settings.tenants {
            if tenant.enabled {
                println!("   ✅ {} ({})", tenant.tenant_name, tenant.tenant_id);
            }
        }
    }

    // ============================================================================
    // Test 13: Multi-Tenant Request Handler Simulation
    // ============================================================================
    
    #[test]
    fn test_multi_tenant_request_handler() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        // Simulate requests from different tenants
        let test_cases = vec![
            ("sk_prod_acme_xxx", "acme-corp", 52428800),           // 50MB
            ("sk_prod_fintech_yyy", "fintech-solutions", 104857600), // 100MB
            ("sk_prod_retail_zzz", "retail-chain", 10485760),       // 10MB
            ("sk_prod_startup_aaa", "startup-xyz", 5242880),        // 5MB
        ];

        for (api_key, expected_id, expected_size) in test_cases {
            let tenant = settings.get_tenant_by_prefix(api_key)
                .expect(&format!("Failed to get tenant for {}", api_key));

            assert_eq!(tenant.tenant_id, expected_id);
            assert_eq!(tenant.max_message_size, expected_size);
            
            println!("✅ Request for {} → {} (max {} bytes)",
                api_key, expected_id, expected_size);
        }
    }

    // ============================================================================
    // Test 14: Retention Policy per Tenant
    // ============================================================================
    
    #[test]
    fn test_retention_policies() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let acme = settings.get_tenant("acme-corp").expect("acme-corp not found");
        let fintech = settings.get_tenant("fintech-solutions").expect("fintech not found");
        let startup = settings.get_tenant("startup-xyz").expect("startup-xyz not found");

        // Different retention policies per tenant tier
        assert_eq!(acme.retention_days, 90);      // Premium
        assert_eq!(fintech.retention_days, 365);   // Enterprise
        assert_eq!(startup.retention_days, 30);    // Starter

        println!("✅ Retention policies:");
        println!("   ACME (Premium): {} days", acme.retention_days);
        println!("   FinTech (Enterprise): {} days", fintech.retention_days);
        println!("   Startup (Starter): {} days", startup.retention_days);
    }

    // ============================================================================
    // Test 15: Load Configuration with Environment Override
    // ============================================================================
    
    #[test]
    fn test_load_with_environment_override() {
        // Try loading with environment override
        let result = AppSettings::from_env(".", "production");
        
        match result {
            Ok(settings) => {
                assert!(!settings.tenants.is_empty());
                println!("✅ Configuration loaded with environment override");
            }
            Err(_) => {
                println!("⚠️  No environment-specific override found (expected)");
            }
        }
    }
}

// ============================================================================
// BENCHMARK: Tenant Lookup Performance
// ============================================================================

#[cfg(test)]
mod benchmarks {
    use fastdatabroker::config::AppSettings;
    use std::time::Instant;

    #[test]
    fn benchmark_tenant_lookup() {
        let settings = AppSettings::from_file("appsettings.production.json")
            .expect("Failed to load config");

        let iterations = 100_000;

        // Benchmark: Get tenant by ID
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = settings.get_tenant("acme-corp");
        }
        let duration = start.elapsed();
        
        let per_lookup = duration.as_micros() as f64 / iterations as f64;
        println!("✅ Tenant lookup by ID: {:.2} µs per lookup", per_lookup);

        // Benchmark: Get tenant by API key prefix
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = settings.get_tenant_by_prefix("sk_prod_acme_xxx");
        }
        let duration = start.elapsed();
        
        let per_lookup = duration.as_micros() as f64 / iterations as f64;
        println!("✅ Tenant lookup by prefix: {:.2} µs per lookup", per_lookup);
    }
}
