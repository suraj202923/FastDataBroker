/// Basic connection pool implementation for Phase 1 optimization
/// This module provides efficient connection reuse and management

use std::sync::Arc;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Configuration for connection pool behavior
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum concurrent connections
    pub max_size: usize,
    /// Minimum idle connections to maintain
    pub min_idle: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection acquisition timeout
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            max_size: 100,
            min_idle: 10,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(5),
        }
    }
}

/// Wrapper for a pooled connection
#[derive(Debug, Clone)]
pub struct PooledConnection<T: Clone> {
    // Connection state
    pub connection: T,
    // Last used timestamp
    pub last_used: Instant,
    // Connection ID
    pub id: u64,
}

/// Connection pool for reusing connections
pub struct ConnectionPool<T: Clone + Send + Sync> {
    config: PoolConfig,
    // Available connections ready to reuse
    available: Arc<Mutex<VecDeque<PooledConnection<T>>>>,
    // Total connections created
    total_created: Arc<std::sync::atomic::AtomicU64>,
    // Currently in use
    in_use: Arc<std::sync::atomic::AtomicUsize>,
    // Factory function to create new connections
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + Send + Sync + 'static> ConnectionPool<T> {
    /// Create a new connection pool
    pub fn new(config: PoolConfig, factory: Arc<dyn Fn() -> T + Send + Sync>) -> Self {
        ConnectionPool {
            config,
            available: Arc::new(Mutex::new(VecDeque::new())),
            total_created: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            in_use: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            factory,
        }
    }

    /// Get a connection from the pool (or create if needed)
    pub fn acquire(&self) -> Option<PooledConnection<T>> {
        use std::sync::atomic::Ordering;

        // Try to get from available pool first
        {
            let mut available = self.available.lock().unwrap();
            if let Some(pooled) = available.pop_front() {
                // Check if connection is still valid (not timed out)
                let elapsed = pooled.last_used.elapsed();
                if elapsed < self.config.idle_timeout {
                    self.in_use.fetch_add(1, Ordering::Relaxed);
                    return Some(pooled);
                }
            }
        }

        // Check if we can create a new connection
        let total = self.total_created.load(Ordering::Relaxed) as usize;
        let in_use = self.in_use.load(Ordering::Relaxed);
        
        if total < self.config.max_size && (total - in_use) < self.config.max_size {
            // Create new connection
            let conn = (self.factory)();
            let id = self.total_created.fetch_add(1, Ordering::Relaxed);
            self.in_use.fetch_add(1, Ordering::Relaxed);
            
            return Some(PooledConnection {
                connection: conn,
                last_used: Instant::now(),
                id,
            });
        }

        None
    }

    /// Return a connection to the pool
    pub fn release(&self, pooled: PooledConnection<T>) {
        use std::sync::atomic::Ordering;
        
        let mut pooled = pooled;
        pooled.last_used = Instant::now();

        // Return to available pool if under limit
        let available = self.available.lock().unwrap();
        if available.len() < self.config.max_size / 2 {
            drop(available);
            self.available.lock().unwrap().push_back(pooled);
        }

        self.in_use.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        use std::sync::atomic::Ordering;
        
        let available = self.available.lock().unwrap();
        PoolStats {
            total_created: self.total_created.load(Ordering::Relaxed),
            available_count: available.len(),
            in_use_count: self.in_use.load(Ordering::Relaxed),
        }
    }

    /// Clear idle connections
    pub fn cleanup(&self) {
        let mut available = self.available.lock().unwrap();
        available.retain(|conn| conn.last_used.elapsed() < self.config.idle_timeout);
    }

    /// Reset the pool (useful for testing)
    pub fn reset(&self) {
        use std::sync::atomic::Ordering;
        
        {
            let mut available = self.available.lock().unwrap();
            available.clear();
        }
        self.total_created.store(0, Ordering::Relaxed);
        self.in_use.store(0, Ordering::Relaxed);
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_created: u64,
    pub available_count: usize,
    pub in_use_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation_and_reuse() {
        let config = PoolConfig {
            max_size: 10,
            min_idle: 2,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(5),
        };

        let factory = Arc::new(|| {
            "connection".to_string()
        });

        let pool = ConnectionPool::new(config, factory);

        // Acquire a connection
        let conn1 = pool.acquire().expect("Failed to acquire connection");
        assert_eq!(conn1.connection, "connection");

        // Stats should show 1 in use
        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.in_use_count, 1);

        // Release it
        pool.release(conn1);

        // Stats should show it's available now
        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.available_count, 1);

        // Acquire again - should reuse
        let conn2 = pool.acquire().expect("Failed to acquire connection");
        assert_eq!(conn2.connection, "connection");
        assert_eq!(conn2.id, 0);

        let stats = pool.stats();
        assert_eq!(stats.total_created, 1); // Still 1, reused
    }

    #[test]
    fn test_pool_max_size_enforcement() {
        let config = PoolConfig {
            max_size: 3,
            min_idle: 1,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(5),
        };

        let factory = Arc::new(|| "connection".to_string());
        let pool = ConnectionPool::new(config, factory);

        // Acquire up to max_size
        let conn1 = pool.acquire().expect("Should acquire conn 1");
        let conn2 = pool.acquire().expect("Should acquire conn 2");
        let conn3 = pool.acquire().expect("Should acquire conn 3");

        let stats = pool.stats();
        assert_eq!(stats.total_created, 3);
        assert_eq!(stats.in_use_count, 3);

        // Try to acquire one more - should fail since all are in use
        let conn4 = pool.acquire();
        assert!(conn4.is_none());

        // Release one
        pool.release(conn1);

        // Now should be able to acquire
        let conn5 = pool.acquire();
        assert!(conn5.is_some());

        pool.release(conn2);
        pool.release(conn3);
        pool.release(conn5.unwrap());
    }

    #[test]
    fn test_pool_cleanup_idle_timeout() {
        let config = PoolConfig {
            max_size: 10,
            min_idle: 2,
            idle_timeout: Duration::from_millis(100),
            acquire_timeout: Duration::from_secs(5),
        };

        let factory = Arc::new(|| "connection".to_string());
        let pool = ConnectionPool::new(config, factory);

        // Acquire and release a connection
        let conn = pool.acquire().expect("Failed to acquire");
        pool.release(conn);

        let stats_before = pool.stats();
        assert_eq!(stats_before.available_count, 1);

        // Wait for connection to timeout
        std::thread::sleep(Duration::from_millis(150));

        // Cleanup should remove idle connections
        pool.cleanup();

        let stats_after = pool.stats();
        // After cleanup, idle connection should be removed
        assert_eq!(stats_after.available_count, 0);
    }

    #[test]
    fn test_pool_reset_functionality() {
        let config = PoolConfig {
            max_size: 10,
            min_idle: 2,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(5),
        };

        let factory = Arc::new(|| "connection".to_string());
        let pool = ConnectionPool::new(config, factory);

        // Create some connections
        let conn1 = pool.acquire().expect("Failed to acquire");
        let conn2 = pool.acquire().expect("Failed to acquire");
        pool.release(conn1);
        pool.release(conn2);

        let stats_before = pool.stats();
        assert_eq!(stats_before.total_created, 2);
        assert_eq!(stats_before.available_count, 2);

        // Reset pool
        pool.reset();

        let stats_after = pool.stats();
        assert_eq!(stats_after.total_created, 0);
        assert_eq!(stats_after.available_count, 0);
        assert_eq!(stats_after.in_use_count, 0);
    }

    #[test]
    fn test_concurrent_pool_access() {
        use std::thread;
        use std::sync::Arc;

        let config = PoolConfig {
            max_size: 10,
            min_idle: 2,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(5),
        };

        let factory = Arc::new(|| "connection".to_string());
        let pool = Arc::new(ConnectionPool::new(config, factory));

        let mut handles = vec![];

        // Spawn 5 threads that each acquire and release connections
        for _ in 0..5 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    if let Some(conn) = pool_clone.acquire() {
                        // Simulate work
                        thread::sleep(Duration::from_millis(1));
                        pool_clone.release(conn);
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // All connections should be back in the pool or cleaned up
        let stats = pool.stats();
        assert!(stats.in_use_count == 0, "All connections should be released");
    }
}
