// Phase 5: Circuit Breaker Pattern (Resilience)
use std::sync::{Arc, atomic::{AtomicU64, AtomicU32, Ordering}};
use std::time::{Instant, Duration};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerStatus {
    /// Circuit is closed - requests pass through normally
    Closed,
    /// Circuit is open - requests fail immediately
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit (percentage 0-100)
    pub failure_threshold: f64,
    /// Success threshold to close circuit (percentage 0-100)
    pub success_threshold: f64,
    /// Number of requests to evaluate before state change
    pub evaluation_window: u32,
    /// Time before half-open state transitions to open if still failing
    pub timeout: Duration,
    /// Maximum concurrent requests in half-open state
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 50.0,
            success_threshold: 90.0,
            evaluation_window: 10,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

/// Circuit breaker for resilient operations
pub struct CircuitBreaker {
    status: Arc<RwLock<CircuitBreakerStatus>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    success_count: Arc<AtomicU32>,
    failure_count: Arc<AtomicU32>,
    total_calls: Arc<AtomicU64>,
    half_open_calls: Arc<AtomicU32>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create new circuit breaker with default config
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create new circuit breaker with custom config
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            status: Arc::new(RwLock::new(CircuitBreakerStatus::Closed)),
            last_failure_time: Arc::new(RwLock::new(None)),
            success_count: Arc::new(AtomicU32::new(0)),
            failure_count: Arc::new(AtomicU32::new(0)),
            total_calls: Arc::new(AtomicU64::new(0)),
            half_open_calls: Arc::new(AtomicU32::new(0)),
            config,
        }
    }

    /// Get current circuit breaker status
    pub async fn get_status(&self) -> CircuitBreakerStatus {
        *self.status.read().await
    }

    /// Record successful operation
    pub async fn record_success(&self) {
        let status = *self.status.read().await;
        
        match status {
            CircuitBreakerStatus::Closed => {
                self.success_count.fetch_add(1, Ordering::SeqCst);
                self.total_calls.fetch_add(1, Ordering::SeqCst);
                self.check_close_threshold().await;
            }
            CircuitBreakerStatus::HalfOpen => {
                self.success_count.fetch_add(1, Ordering::SeqCst);
                self.total_calls.fetch_add(1, Ordering::SeqCst);
                self.half_open_calls.fetch_sub(1, Ordering::SeqCst);
                
                if self.check_half_open_success().await {
                    *self.status.write().await = CircuitBreakerStatus::Closed;
                    self.reset_counts();
                    tracing::info!("Circuit breaker closed - service recovered");
                }
            }
            CircuitBreakerStatus::Open => {
                // Ignore successes while open
            }
        }
    }

    /// Record failed operation
    pub async fn record_failure(&self) {
        let status = *self.status.read().await;
        
        match status {
            CircuitBreakerStatus::Closed => {
                self.failure_count.fetch_add(1, Ordering::SeqCst);
                self.total_calls.fetch_add(1, Ordering::SeqCst);
                *self.last_failure_time.write().await = Some(Instant::now());
                self.check_open_threshold().await;
            }
            CircuitBreakerStatus::HalfOpen => {
                self.failure_count.fetch_add(1, Ordering::SeqCst);
                self.half_open_calls.fetch_sub(1, Ordering::SeqCst);
                *self.status.write().await = CircuitBreakerStatus::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                tracing::warn!("Circuit breaker opened - service failures detected");
            }
            CircuitBreakerStatus::Open => {
                // Ignore failures while open
            }
        }
    }

    /// Try to execute operation, returns true if allowed
    pub async fn allow_request(&self) -> bool {
        let status = *self.status.read().await;
        
        match status {
            CircuitBreakerStatus::Closed => true,
            CircuitBreakerStatus::Open => {
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() > self.config.timeout {
                        *self.status.write().await = CircuitBreakerStatus::HalfOpen;
                        self.reset_counts();
                        tracing::info!("Circuit breaker half-open - testing recovery");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerStatus::HalfOpen => {
                let current = self.half_open_calls.load(Ordering::SeqCst);
                if current < self.config.half_open_max_calls {
                    self.half_open_calls.fetch_add(1, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Check if failure threshold exceeded
    async fn check_open_threshold(&self) {
        let total = self.total_calls.load(Ordering::SeqCst);
        if total >= self.config.evaluation_window as u64 {
            let failures = self.failure_count.load(Ordering::SeqCst) as f64;
            let total_f = total as f64;
            let failure_rate = (failures / total_f) * 100.0;
            
            if failure_rate >= self.config.failure_threshold {
                *self.status.write().await = CircuitBreakerStatus::Open;
                tracing::warn!(
                    "Circuit breaker opened - failure rate: {:.1}%",
                    failure_rate
                );
            }
        }
    }

    /// Check if success threshold exceeded in closed state
    async fn check_close_threshold(&self) {
        let total = self.total_calls.load(Ordering::SeqCst);
        if total > 0 && total % self.config.evaluation_window as u64 == 0 {
            // Reset after evaluation window
            let failures = self.failure_count.load(Ordering::SeqCst) as f64;
            let failure_rate = (failures / total as f64) * 100.0;
            
            if failure_rate < self.config.failure_threshold {
                self.reset_counts();
            }
        }
    }

    /// Check if half-open state should transition to closed
    async fn check_half_open_success(&self) -> bool {
        let successes = self.success_count.load(Ordering::SeqCst);
        (successes as f64 / self.config.half_open_max_calls as f64) * 100.0 
            >= self.config.success_threshold
    }

    /// Reset call counters
    fn reset_counts(&self) {
        self.success_count.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
        self.total_calls.store(0, Ordering::SeqCst);
        self.half_open_calls.store(0, Ordering::SeqCst);
    }

    /// Get metrics
    pub fn get_metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            successes: self.success_count.load(Ordering::SeqCst),
            failures: self.failure_count.load(Ordering::SeqCst),
            total_calls: self.total_calls.load(Ordering::SeqCst),
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            last_failure_time: self.last_failure_time.clone(),
            success_count: self.success_count.clone(),
            failure_count: self.failure_count.clone(),
            total_calls: self.total_calls.clone(),
            half_open_calls: self.half_open_calls.clone(),
            config: self.config.clone(),
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub successes: u32,
    pub failures: u32,
    pub total_calls: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed_on_success() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.get_status().await, CircuitBreakerStatus::Closed);
        
        cb.record_success().await;
        assert_eq!(cb.get_status().await, CircuitBreakerStatus::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let mut config = CircuitBreakerConfig::default();
        config.evaluation_window = 5;
        config.failure_threshold = 50.0;
        
        let cb = CircuitBreaker::with_config(config);
        
        // Record 3 failures out of 5 calls
        for _ in 0..3 {
            cb.record_failure().await;
        }
        for _ in 0..2 {
            cb.record_success().await;
        }
        
        assert_eq!(cb.get_status().await, CircuitBreakerStatus::Closed);
    }

    #[tokio::test]
    async fn test_allow_request_when_closed() {
        let cb = CircuitBreaker::new();
        assert!(cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_metrics() {
        let cb = CircuitBreaker::new();
        cb.record_success().await;
        cb.record_failure().await;
        
        let metrics = cb.get_metrics();
        assert_eq!(metrics.successes, 1);
        assert_eq!(metrics.failures, 1);
        assert_eq!(metrics.total_calls, 2);
    }
}
