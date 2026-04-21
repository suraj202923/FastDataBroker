use lru::LruCache;
use serde_json::Value;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// In-memory LRU cache for JSON data
pub struct JsonCache {
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    data: Value,
    timestamp: std::time::SystemTime,
}

impl JsonCache {
    /// Create new cache with max capacity
    pub fn new(capacity: usize) -> Self {
        let cache_size = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }

    /// Get value from cache if hit
    pub fn get(&self, key: &str) -> Option<Value> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(key).map(|entry| entry.data.clone())
    }

    /// Store value in cache
    pub fn put(&self, key: String, value: Value) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(
            key,
            CacheEntry {
                data: value,
                timestamp: std::time::SystemTime::now(),
            },
        );
    }

    /// Remove entry from cache
    pub fn remove(&self, key: &str) -> Option<Value> {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(key).map(|entry| entry.data)
    }

    /// Clear all cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        CacheStats {
            len: cache.len(),
            cap: cache.cap().get(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub len: usize,
    pub cap: usize,
}

impl Clone for JsonCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let cache = JsonCache::new(10);
        let key = "test_key".to_string();
        let value = serde_json::json!({"test": "value"});

        // Test put and get
        cache.put(key.clone(), value.clone());
        assert_eq!(cache.get(&key), Some(value.clone()));

        // Test remove
        cache.remove(&key);
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = JsonCache::new(2);

        cache.put("key1".to_string(), serde_json::json!(1));
        cache.put("key2".to_string(), serde_json::json!(2));
        cache.put("key3".to_string(), serde_json::json!(3)); // Evicts key1

        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some(serde_json::json!(2)));
        assert_eq!(cache.get("key3"), Some(serde_json::json!(3)));
    }
}
