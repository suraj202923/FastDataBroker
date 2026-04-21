/// Object Pool - Reusable Object Allocation (Phase 3)
///
/// Reduces garbage collection pressure and allocation overhead by maintaining
/// a pool of pre-allocated objects that can be borrowed and returned.
/// Significantly improves performance for high-allocation workloads.

use std::sync::Arc;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Generic object pool for reusing allocations
/// Maintains a stack of available objects and metrics
pub struct ObjectPool<T: Clone + Send + Sync> {
    /// Stack of available objects (thread-safe)
    available: Arc<DashMap<usize, T>>,
    /// Factory function to create new objects
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    /// Number of objects currently in pool
    available_count: Arc<AtomicUsize>,
    /// Total objects created
    total_created: Arc<AtomicUsize>,
    /// Maximum pool size (0 = unbounded)
    max_size: usize,
}

impl<T: Clone + Send + Sync + 'static> ObjectPool<T> {
    /// Create a new object pool
    pub fn new(factory: Arc<dyn Fn() -> T + Send + Sync>, max_size: usize) -> Self {
        ObjectPool {
            available: Arc::new(DashMap::new()),
            factory,
            available_count: Arc::new(AtomicUsize::new(0)),
            total_created: Arc::new(AtomicUsize::new(0)),
            max_size,
        }
    }

    /// Get object from pool or create new one
    pub fn acquire(&self) -> T {
        // Try to get from pool
        if let Some((_, obj)) = self.available.remove(&0) {
            self.available_count.fetch_sub(1, Ordering::Relaxed);
            return obj;
        }

        // Create new object if not available
        let obj = (self.factory)();
        self.total_created.fetch_add(1, Ordering::Relaxed);
        obj
    }

    /// Return object to pool
    pub fn release(&self, obj: T) {
        let count = self.available_count.load(Ordering::Relaxed);

        // Check max size limit
        if self.max_size > 0 && count >= self.max_size {
            return; // Drop object if pool is full
        }

        self.available.insert(count, obj);
        self.available_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Pre-allocate objects in the pool
    pub fn pre_allocate(&self, count: usize) {
        for i in 0..count {
            let obj = (self.factory)();
            self.available.insert(i, obj);
            self.total_created.fetch_add(1, Ordering::Relaxed);
        }
        self.available_count.store(count, Ordering::Relaxed);
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            available_count: self.available_count.load(Ordering::Relaxed),
            total_created: self.total_created.load(Ordering::Relaxed),
            max_size: self.max_size,
        }
    }

    /// Clear the pool (release all objects)
    pub fn clear(&self) {
        self.available.clear();
        self.available_count.store(0, Ordering::Relaxed);
    }
}

/// Statistics for object pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_count: usize,
    pub total_created: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_acquire_release() {
        let pool = ObjectPool::new(
            Arc::new(|| vec![0u8; 1024]),
            100,
        );

        // Pre-allocate
        pool.pre_allocate(10);
        assert_eq!(pool.stats().available_count, 10);

        // Acquire
        let obj = pool.acquire();
        assert_eq!(pool.stats().available_count, 9);
        assert_eq!(obj.len(), 1024);

        // Release
        pool.release(obj);
        assert_eq!(pool.stats().available_count, 10);
    }

    #[test]
    fn test_object_pool_max_size() {
        let pool = ObjectPool::new(Arc::new(|| 42), 5);
        pool.pre_allocate(5);

        // Try to add more than max
        pool.release(100);
        assert_eq!(pool.stats().available_count, 5); // Still 5
    }

    #[test]
    fn test_concurrent_acquire_release() {
        let pool = Arc::new(ObjectPool::new(
            Arc::new(|| String::from("test")),
            50,
        ));

        pool.pre_allocate(10);

        let mut handles = vec![];
        for _ in 0..10 {
            let p = Arc::clone(&pool);
            let handle = std::thread::spawn(move || {
                let obj = p.acquire();
                std::thread::sleep(std::time::Duration::from_millis(1));
                p.release(obj);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(pool.stats().available_count > 0);
    }
}
