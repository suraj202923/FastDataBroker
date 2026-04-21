/// Intelligent Caching System (Phase 3)
///
/// LRU (Least Recently Used) cache with TTL support for frequently accessed data.
/// Reduces repeated lookups and computationally expensive operations.
/// Particularly effective for metric queries and tenant data lookups.

use std::sync::Arc;
use dashmap::DashMap;
use std::time::{SystemTime, Duration};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Cache entry with TTL support
#[derive(Clone, Debug)]
struct CacheEntry<V> {
    value: V,
    created_at: SystemTime,
    accessed_at: SystemTime,
    access_count: usize,
}

impl<V: Clone> CacheEntry<V> {
    fn new(value: V) -> Self {
        let now = SystemTime::now();
        CacheEntry {
            value,
            created_at: now,
            accessed_at: now,
            access_count: 1,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed().unwrap_or(ttl) > ttl
    }

    fn touch(&mut self) {
        self.accessed_at = SystemTime::now();
        self.access_count += 1;
    }
}

/// LRU Cache with TTL support (Phase 3 Optimization)
///
/// # Features
/// - O(1) get/set operations (DashMap backed)
/// - TTL (Time To Live) for automatic expiration
/// - LRU eviction when capacity is reached
/// - Thread-safe concurrent access
/// - Access statistics for monitoring
pub struct LruCache<K: Clone + std::hash::Hash + Eq + Send + Sync, V: Clone + Send + Sync> {
    /// Main cache storage
    data: Arc<DashMap<K, CacheEntry<V>>>,
    /// Capacity limit
    capacity: usize,
    /// Time to live for entries
    ttl: Duration,
    /// Statistics
    hits: Arc<AtomicUsize>,
    misses: Arc<AtomicUsize>,
    evictions: Arc<AtomicUsize>,
}

impl<K: Clone + std::hash::Hash + Eq + Send + Sync + 'static, V: Clone + Send + Sync + 'static>
    LruCache<K, V>
{
    /// Create a new LRU cache
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        LruCache {
            data: Arc::new(DashMap::new()),
            capacity,
            ttl,
            hits: Arc::new(AtomicUsize::new(0)),
            misses: Arc::new(AtomicUsize::new(0)),
            evictions: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get value from cache
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(mut entry) = self.data.get_mut(key) {
            // Check expiration
            if entry.is_expired(self.ttl) {
                drop(entry);
                self.data.remove(key);
                self.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }

            // Update access time
            entry.touch();
            let value = entry.value.clone();
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(value)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert value into cache
    pub fn insert(&self, key: K, value: V) {
        // Check if we need to evict
        if self.data.len() >= self.capacity && !self.data.contains_key(&key) {
            self.evict_lru();
        }

        self.data.insert(key, CacheEntry::new(value));
    }

    /// Evict least recently used entry
    fn evict_lru(&self) {
        const MAX_DURATION: Duration = Duration::from_secs(u64::MAX);
        let mut lru_entry = None;
        let mut max_time = Duration::from_secs(0);

        for entry in self.data.iter() {
            let elapsed = entry
                .value()
                .accessed_at
                .elapsed()
                .unwrap_or(MAX_DURATION);
            if elapsed > max_time {
                max_time = elapsed;
                lru_entry = Some(entry.key().clone());
            }
        }

        if let Some(key) = lru_entry {
            self.data.remove(&key);
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Invalidate entry
    pub fn invalidate(&self, key: &K) {
        self.data.remove(key);
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.data.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        CacheStats {
            size: self.data.len(),
            capacity: self.capacity,
            hits,
            misses,
            evictions: self.evictions.load(Ordering::Relaxed),
            hit_rate: if total > 0 {
                (hits as f64 / total as f64)
            } else {
                0.0
            },
        }
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&self) -> usize {
        let mut removed = 0;
        self.data.retain(|_, entry| {
            if entry.is_expired(self.ttl) {
                removed += 1;
                false
            } else {
                true
            }
        });
        removed
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_get_set() {
        let cache = LruCache::new(10, Duration::from_secs(60));

        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_lru_cache_ttl_expiration() {
        let cache = LruCache::new(10, Duration::from_millis(100));

        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));

        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_lru_cache_capacity_eviction() {
        let cache = LruCache::new(3, Duration::from_secs(60));

        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.insert("key3", 3);

        // Access key1 to mark it as recently used
        let _ = cache.get(&"key1");

        // Add new entry, should evict least recently used (key2)
        cache.insert("key4", 4);

        assert_eq!(cache.get(&"key4"), Some(4));
        // One of the original keys must have been evicted when capacity was exceeded.
        let remaining = ["key1", "key2", "key3"]
            .iter()
            .filter(|k| cache.get(k).is_some())
            .count();
        assert!(remaining <= 2);

        let stats = cache.stats();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let cache = LruCache::new(10, Duration::from_secs(60));

        cache.insert("key", "value");
        let _ = cache.get(&"key");
        let _ = cache.get(&"key");
        let _ = cache.get(&"missing");

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        // hit_rate is 2/3 ≈ 0.667, multiply by 100 to get 66.7%
        assert_eq!((stats.hit_rate * 100.0).round() as u32, 67); // ~66.7%
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = LruCache::new(10, Duration::from_millis(50));

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        std::thread::sleep(Duration::from_millis(100));

        let removed = cache.cleanup_expired();
        assert_eq!(removed, 2);
        assert_eq!(cache.stats().size, 0);
    }
}
