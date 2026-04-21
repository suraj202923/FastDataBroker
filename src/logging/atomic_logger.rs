/// Atomic Logger - Lock-free logging using only atomic operations
/// Optimized for rate limiting and critical paths
/// Zero allocations, negligible overhead (~10-100 nanoseconds per operation)
/// 
/// Use Case: Rate limit violations, critical metrics, high-frequency events

use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Statistics collected via atomic counters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AtomicStats {
    pub total_events: u64,
    pub warnings: u64,
    pub errors: u64,
    pub infos: u64,
    pub last_timestamp: u64,  // Unix timestamp
}

/// Lock-free logger using only atomic operations
/// No locks, no allocations, minimal overhead
pub struct AtomicLogger {
    total_events: Arc<AtomicU64>,
    warning_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    info_count: Arc<AtomicU64>,
    last_timestamp: Arc<AtomicU64>,
}

impl AtomicLogger {
    /// Create new atomic logger
    pub fn new() -> Self {
        Self {
            total_events: Arc::new(AtomicU64::new(0)),
            warning_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            info_count: Arc::new(AtomicU64::new(0)),
            last_timestamp: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Log warning event (nanoseconds overhead)
    #[inline]
    pub fn warn(&self) {
        self.total_events.fetch_add(1, Ordering::Relaxed);
        self.warning_count.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }

    /// Log error event (nanoseconds overhead)
    #[inline]
    pub fn error(&self) {
        self.total_events.fetch_add(1, Ordering::Relaxed);
        self.error_count.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }

    /// Log info event (nanoseconds overhead)
    #[inline]
    pub fn info(&self) {
        self.total_events.fetch_add(1, Ordering::Relaxed);
        self.info_count.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }

    /// Update last seen timestamp
    #[inline]
    fn update_timestamp(&self) {
        let now = Utc::now().timestamp() as u64;
        self.last_timestamp.store(now, Ordering::Relaxed);
    }

    /// Get current statistics (no locks)
    pub fn stats(&self) -> AtomicStats {
        AtomicStats {
            total_events: self.total_events.load(Ordering::Relaxed),
            warnings: self.warning_count.load(Ordering::Relaxed),
            errors: self.error_count.load(Ordering::Relaxed),
            infos: self.info_count.load(Ordering::Relaxed),
            last_timestamp: self.last_timestamp.load(Ordering::Relaxed),
        }
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.total_events.store(0, Ordering::Relaxed);
        self.warning_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.info_count.store(0, Ordering::Relaxed);
    }
}

impl Clone for AtomicLogger {
    fn clone(&self) -> Self {
        Self {
            total_events: Arc::clone(&self.total_events),
            warning_count: Arc::clone(&self.warning_count),
            error_count: Arc::clone(&self.error_count),
            info_count: Arc::clone(&self.info_count),
            last_timestamp: Arc::clone(&self.last_timestamp),
        }
    }
}

impl Default for AtomicLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit event counter - specialized for rate limiting
#[derive(Debug)]
pub struct RateLimitCounter {
    violations: Arc<AtomicU64>,
    last_violation_timestamp: Arc<AtomicU64>,
    peak_violations_per_sec: Arc<AtomicU64>,
    current_second_violations: Arc<AtomicU32>,
    current_second_start: Arc<AtomicU64>,
}

impl RateLimitCounter {
    /// Create new rate limit counter
    pub fn new() -> Self {
        Self {
            violations: Arc::new(AtomicU64::new(0)),
            last_violation_timestamp: Arc::new(AtomicU64::new(0)),
            peak_violations_per_sec: Arc::new(AtomicU64::new(0)),
            current_second_violations: Arc::new(AtomicU32::new(0)),
            current_second_start: Arc::new(AtomicU64::new(Utc::now().timestamp() as u64)),
        }
    }

    /// Record a rate limit violation (nanoseconds overhead)
    #[inline]
    pub fn record_violation(&self) {
        // Increment total violations
        self.violations.fetch_add(1, Ordering::Relaxed);
        
        // Update timestamp
        let now = Utc::now().timestamp() as u64;
        self.last_violation_timestamp.store(now, Ordering::Relaxed);

        // Track violations per second (for peak detection)
        self.update_peak_rate();
    }

    /// Update peak violations per second
    fn update_peak_rate(&self) {
        let now = Utc::now().timestamp() as u64;
        let second_start = self.current_second_start.load(Ordering::Relaxed);

        if now > second_start {
            // New second started, record peak
            let violations_this_second = self.current_second_violations.load(Ordering::Relaxed) as u64;
            let peak = self.peak_violations_per_sec.load(Ordering::Relaxed);
            
            if violations_this_second > peak {
                self.peak_violations_per_sec.store(violations_this_second, Ordering::Relaxed);
            }

            // Reset for new second
            self.current_second_violations.store(0, Ordering::Relaxed);
            self.current_second_start.store(now, Ordering::Relaxed);
        } else {
            // Same second, increment counter
            self.current_second_violations.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get rate limit statistics
    pub fn stats(&self) -> RateLimitStats {
        RateLimitStats {
            total_violations: self.violations.load(Ordering::Relaxed),
            last_violation_timestamp: self.last_violation_timestamp.load(Ordering::Relaxed),
            peak_violations_per_sec: self.peak_violations_per_sec.load(Ordering::Relaxed),
            current_second_violations: self.current_second_violations.load(Ordering::Relaxed) as u64,
        }
    }

    /// Reset counter
    pub fn reset(&self) {
        self.violations.store(0, Ordering::Relaxed);
        self.peak_violations_per_sec.store(0, Ordering::Relaxed);
        self.current_second_violations.store(0, Ordering::Relaxed);
    }
}

impl Clone for RateLimitCounter {
    fn clone(&self) -> Self {
        Self {
            violations: Arc::clone(&self.violations),
            last_violation_timestamp: Arc::clone(&self.last_violation_timestamp),
            peak_violations_per_sec: Arc::clone(&self.peak_violations_per_sec),
            current_second_violations: Arc::clone(&self.current_second_violations),
            current_second_start: Arc::clone(&self.current_second_start),
        }
    }
}

impl Default for RateLimitCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from rate limit counter
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RateLimitStats {
    pub total_violations: u64,
    pub last_violation_timestamp: u64,
    pub peak_violations_per_sec: u64,
    pub current_second_violations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_atomic_logger_basic() {
        let logger = AtomicLogger::new();
        
        logger.info();
        logger.warn();
        logger.error();
        
        let stats = logger.stats();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.infos, 1);
        assert_eq!(stats.warnings, 1);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    fn test_rate_limit_counter() {
        let counter = RateLimitCounter::new();
        
        for _ in 0..100 {
            counter.record_violation();
        }
        
        let stats = counter.stats();
        assert_eq!(stats.total_violations, 100);
    }

    #[test]
    fn test_concurrent_atomic_logging() {
        let logger = AtomicLogger::new();
        let mut handles = vec![];

        for _ in 0..10 {
            let logger_clone = logger.clone();
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    logger_clone.info();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = logger.stats();
        assert_eq!(stats.total_events, 10000);
    }

    #[test]
    fn test_concurrent_rate_limit_counter() {
        let counter = RateLimitCounter::new();
        let mut handles = vec![];

        for _ in 0..5 {
            let counter_clone = counter.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter_clone.record_violation();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = counter.stats();
        assert_eq!(stats.total_violations, 500);
    }
}
