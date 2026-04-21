// ============================================================================
// In-Memory Cache System (v3.1) - Fast Access Caches
// ============================================================================
// Three-layer caching system:
// 1. Tenant Configuration Cache (50 MB) - loaded at startup
// 2. Metrics Counters (100 MB) - atomic operations, flushed every 100ms
// 3. PSK Verification Cache (1 MB, LRU) - fast authentication

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use chrono::Utc;
use dashmap::DashMap;
use anyhow::Result;

use crate::config::tenant_json::{TenantJsonConfig, MetricsSnapshot};

/// Layer 1: Tenant Configuration Cache (~50 MB for 1000 tenants)
pub struct TenantConfigCache {
    cache: Arc<RwLock<HashMap<String, TenantJsonConfig>>>,
}

impl TenantConfigCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        TenantConfigCache {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Load all tenant configurations into cache
    pub fn load_all(&self, configs: HashMap<String, TenantJsonConfig>) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        cache.extend(configs);
    }
    
    /// Get a tenant configuration
    pub fn get(&self, tenant_id: &str) -> Option<TenantJsonConfig> {
        let cache = self.cache.read().unwrap();
        cache.get(tenant_id).cloned()
    }
    
    /// Update a tenant configuration
    pub fn update(&self, tenant_id: String, config: TenantJsonConfig) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(tenant_id, config);
    }
    
    /// Get all tenant IDs
    pub fn get_tenant_ids(&self) -> Vec<String> {
        let cache = self.cache.read().unwrap();
        cache.keys().cloned().collect()
    }
    
    /// Get cache size (number of tenants)
    pub fn size(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }
    
    /// Get rate limit for tenant
    pub fn get_rate_limit(&self, tenant_id: &str) -> Option<u32> {
        let cache = self.cache.read().unwrap();
        cache.get(tenant_id).map(|c| c.configuration.rate_limit_rps)
    }
    
    /// Check if tenant is enabled
    pub fn is_tenant_enabled(&self, tenant_id: &str) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(tenant_id)
    }
}

impl Clone for TenantConfigCache {
    fn clone(&self) -> Self {
        TenantConfigCache {
            cache: Arc::clone(&self.cache),
        }
    }
}

impl Default for TenantConfigCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer 2: Metrics Counters (~100 MB for 1000 tenants)
/// Atomic operations for fast metrics updates without locks
pub struct MetricsCounters {
    // Per-tenant metrics
    metrics: DashMap<String, TenantMetricsCounter>,
}

pub struct TenantMetricsCounter {
    pub messages_received: AtomicU64,
    pub messages_sent: AtomicU64,
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,
    pub active_connections: AtomicU32,
    pub rate_limit_violations: AtomicU32,
    pub errors: AtomicU32,
}

impl MetricsCounters {
    /// Create a new metrics counter system
    pub fn new() -> Self {
        MetricsCounters {
            metrics: DashMap::new(),
        }
    }
    
    /// Increment messages received for a tenant
    pub fn record_message_received(&self, tenant_id: &str, byte_count: u64) {
        let counter = self.get_or_create(tenant_id);
        counter.messages_received.fetch_add(1, Ordering::Relaxed);
        counter.bytes_in.fetch_add(byte_count, Ordering::Relaxed);
    }
    
    /// Increment messages sent for a tenant
    pub fn record_message_sent(&self, tenant_id: &str, byte_count: u64) {
        let counter = self.get_or_create(tenant_id);
        counter.messages_sent.fetch_add(1, Ordering::Relaxed);
        counter.bytes_out.fetch_add(byte_count, Ordering::Relaxed);
    }
    
    /// Record rate limit violation
    pub fn record_rate_limit_violation(&self, tenant_id: &str) {
        let counter = self.get_or_create(tenant_id);
        counter.rate_limit_violations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record an error
    pub fn record_error(&self, tenant_id: &str) {
        let counter = self.get_or_create(tenant_id);
        counter.errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update active connections
    pub fn set_active_connections(&self, tenant_id: &str, count: u32) {
        let counter = self.get_or_create(tenant_id);
        counter.active_connections.store(count, Ordering::Relaxed);
    }
    
    /// Increment active connections
    pub fn increment_active_connections(&self, tenant_id: &str) {
        let counter = self.get_or_create(tenant_id);
        counter.active_connections.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Decrement active connections
    pub fn decrement_active_connections(&self, tenant_id: &str) {
        let counter = self.get_or_create(tenant_id);
        counter.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
    
    /// Get current metrics for a tenant
    pub fn get_metrics(&self, tenant_id: &str) -> Option<MetricsSnapshot> {
        self.metrics.get(tenant_id).map(|counter| MetricsSnapshot {
            timestamp: Utc::now().to_rfc3339(),
            messages_received: counter.messages_received.load(Ordering::Relaxed),
            messages_sent: counter.messages_sent.load(Ordering::Relaxed),
            bytes_in: counter.bytes_in.load(Ordering::Relaxed),
            bytes_out: counter.bytes_out.load(Ordering::Relaxed),
            active_connections: counter.active_connections.load(Ordering::Relaxed),
            rate_limit_violations: counter.rate_limit_violations.load(Ordering::Relaxed),
            average_latency_ms: 0.0, // Would be calculated separately
            p99_latency_ms: 0.0,     // Would be calculated separately
            errors: counter.errors.load(Ordering::Relaxed),
        })
    }
    
    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, MetricsSnapshot> {
        let mut result = HashMap::new();
        
        for entry in self.metrics.iter() {
            let tenant_id = entry.key().clone();
            let counter = entry.value();
            
            result.insert(
                tenant_id,
                MetricsSnapshot {
                    timestamp: Utc::now().to_rfc3339(),
                    messages_received: counter.messages_received.load(Ordering::Relaxed),
                    messages_sent: counter.messages_sent.load(Ordering::Relaxed),
                    bytes_in: counter.bytes_in.load(Ordering::Relaxed),
                    bytes_out: counter.bytes_out.load(Ordering::Relaxed),
                    active_connections: counter.active_connections.load(Ordering::Relaxed),
                    rate_limit_violations: counter.rate_limit_violations.load(Ordering::Relaxed),
                    average_latency_ms: 0.0,
                    p99_latency_ms: 0.0,
                    errors: counter.errors.load(Ordering::Relaxed),
                },
            );
        }
        
        result
    }
    
    /// Reset metrics for a tenant
    pub fn reset_metrics(&self, tenant_id: &str) {
        self.metrics.remove(tenant_id);
    }
    
    /// Get or create a counter for a tenant
    fn get_or_create(&self, tenant_id: &str) -> dashmap::mapref::one::RefMut<'_, String, TenantMetricsCounter> {
        self.metrics
            .entry(tenant_id.to_string())
            .or_insert_with(TenantMetricsCounter::new)
    }
}

impl Clone for MetricsCounters {
    fn clone(&self) -> Self {
        MetricsCounters {
            metrics: DashMap::new(),
        }
    }
}

impl Default for MetricsCounters {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantMetricsCounter {
    fn new() -> Self {
        TenantMetricsCounter {
            messages_received: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
            rate_limit_violations: AtomicU32::new(0),
            errors: AtomicU32::new(0),
        }
    }
}

/// Layer 3: PSK Verification Cache (LRU, ~1 MB)
pub struct PskVerificationCache {
    cache: DashMap<String, String>, // PSK -> TenantID
    ttl_seconds: u32,
}

impl PskVerificationCache {
    /// Create a new PSK cache with TTL
    pub fn new(ttl_seconds: u32) -> Self {
        PskVerificationCache {
            cache: DashMap::new(),
            ttl_seconds,
        }
    }
    
    /// Get tenant ID from PSK
    pub fn get(&self, psk: &str) -> Option<String> {
        self.cache.get(psk).map(|v| v.clone())
    }
    
    /// Store PSK -> TenantID mapping
    pub fn insert(&self, psk: String, tenant_id: String) {
        self.cache.insert(psk, tenant_id);
        // Note: TTL expiration would be handled by a background worker
    }
    
    /// Remove a PSK from cache
    pub fn remove(&self, psk: &str) {
        self.cache.remove(psk);
    }
    
    /// Clear all entries
    pub fn clear(&self) {
        self.cache.clear();
    }
    
    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

impl Clone for PskVerificationCache {
    fn clone(&self) -> Self {
        PskVerificationCache {
            cache: DashMap::new(),
            ttl_seconds: self.ttl_seconds,
        }
    }
}

/// Combined cache system
pub struct CacheSystem {
    pub tenant_config: TenantConfigCache,
    pub metrics: MetricsCounters,
    pub psk_verification: PskVerificationCache,
}

impl CacheSystem {
    /// Create a new cache system
    pub fn new(psk_ttl_seconds: u32) -> Self {
        CacheSystem {
            tenant_config: TenantConfigCache::new(),
            metrics: MetricsCounters::new(),
            psk_verification: PskVerificationCache::new(psk_ttl_seconds),
        }
    }
    
    /// Get tenant rate limit
    pub fn get_rate_limit(&self, tenant_id: &str) -> Option<u32> {
        self.tenant_config.get_rate_limit(tenant_id)
    }
    
    /// Get all metrics for flushing
    pub fn drain_metrics(&self) -> HashMap<String, MetricsSnapshot> {
        self.metrics.get_all_metrics()
    }
    
    /// Check if tenant context is valid
    pub fn is_valid_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_config.is_tenant_enabled(tenant_id)
    }
}

impl Clone for CacheSystem {
    fn clone(&self) -> Self {
        CacheSystem {
            tenant_config: self.tenant_config.clone(),
            metrics: self.metrics.clone(),
            psk_verification: self.psk_verification.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_config_cache() {
        let cache = TenantConfigCache::new();
        
        let mut configs = HashMap::new();
        let mut config = TenantJsonConfig::example();
        config.tenant.id = "t_test".to_string();
        configs.insert("t_test".to_string(), config);
        
        cache.load_all(configs);
        assert_eq!(cache.size(), 1);
        
        let retrieved = cache.get("t_test");
        assert!(retrieved.is_some());
    }
    
    #[test]
    fn test_metrics_counters() {
        let metrics = MetricsCounters::new();
        
        metrics.record_message_received("t_test", 1024);
        metrics.record_message_sent("t_test", 2048);
        metrics.increment_active_connections("t_test");
        
        let snapshot = metrics.get_metrics("t_test").unwrap();
        assert_eq!(snapshot.messages_received, 1);
        assert_eq!(snapshot.messages_sent, 1);
        assert_eq!(snapshot.bytes_in, 1024);
        assert_eq!(snapshot.bytes_out, 2048);
        assert_eq!(snapshot.active_connections, 1);
    }
    
    #[test]
    fn test_psk_cache() {
        let cache = PskVerificationCache::new(3600);
        
        cache.insert("test_psk_123".to_string(), "t_test".to_string());
        assert_eq!(cache.get("test_psk_123"), Some("t_test".to_string()));
        
        cache.remove("test_psk_123");
        assert_eq!(cache.get("test_psk_123"), None);
    }
}
