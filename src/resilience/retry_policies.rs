// Phase 5: Advanced Retry Policies
use std::time::Duration;
use rand::Rng;
use anyhow::Result;

/// Backoff strategies for retry logic
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed(Duration),
    /// Exponential backoff: delay = base * multiplier^attempt
    Exponential {
        base: Duration,
        multiplier: f64,
        max_delay: Duration,
    },
    /// Linear backoff: delay = base * attempt
    Linear {
        base: Duration,
        max_delay: Duration,
    },
    /// Exponential backoff with randomized jitter
    ExponentialWithJitter {
        base: Duration,
        multiplier: f64,
        max_delay: Duration,
        jitter_factor: f64, // 0.0-1.0
    },
}

impl BackoffStrategy {
    /// Calculate delay for given attempt number (0-indexed)
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            BackoffStrategy::Fixed(delay) => *delay,
            
            BackoffStrategy::Exponential {
                base,
                multiplier,
                max_delay,
            } => {
                let delay = base.as_millis() as f64
                    * multiplier.powi(attempt as i32);
                let delay = Duration::from_millis(delay as u64);
                delay.min(*max_delay)
            }
            
            BackoffStrategy::Linear {
                base,
                max_delay,
            } => {
                let delay = base.as_millis() * (attempt as u128 + 1);
                let delay = Duration::from_millis(delay as u64);
                delay.min(*max_delay)
            }
            
            BackoffStrategy::ExponentialWithJitter {
                base,
                multiplier,
                max_delay,
                jitter_factor,
            } => {
                let base_delay = base.as_millis() as f64
                    * multiplier.powi(attempt as i32);
                
                // Add jitter: random value between base_delay * (1 - jitter_factor) and base_delay
                let mut rng = rand::thread_rng();
                let jitter = rng.gen_range(0.0..(*jitter_factor)) * base_delay;
                let delay = (base_delay - (jitter_factor * base_delay / 2.0) + jitter).max(0.0);
                
                let delay = Duration::from_millis(delay as u64);
                delay.min(*max_delay)
            }
        }
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        BackoffStrategy::ExponentialWithJitter {
            base: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_secs(30),
            jitter_factor: 0.1,
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Backoff strategy to use
    pub backoff: BackoffStrategy,
    /// Whether to retry on specific error types
    pub retriable_errors: Vec<String>,
    /// Whether to log each retry attempt
    pub log_retries: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::default(),
            retriable_errors: vec![
                "Timeout".to_string(),
                "ConnectionReset".to_string(),
                "ServiceUnavailable".to_string(),
            ],
            log_retries: true,
        }
    }
}

/// Retry policy executor
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    /// Create new retry policy with default config
    pub fn new() -> Self {
        Self {
            config: RetryConfig::default(),
        }
    }

    /// Create new retry policy with custom config
    pub fn with_config(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute operation with retries
    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + Send + 'static,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_retries {
                        if self.config.log_retries {
                            tracing::error!(
                                "Operation failed after {} attempts: {}",
                                attempt + 1,
                                e
                            );
                        }
                        return Err(e);
                    }

                    let is_retriable = self.config.retriable_errors.is_empty()
                        || self.config.retriable_errors.iter().any(|err_type| {
                            e.to_string().contains(err_type)
                        });

                    if !is_retriable {
                        if self.config.log_retries {
                            tracing::info!("Non-retriable error: {}", e);
                        }
                        return Err(e);
                    }

                    let delay = self.config.backoff.calculate_delay(attempt);
                    if self.config.log_retries {
                        tracing::warn!(
                            "Attempt {} failed: {}. Retrying in {:?}",
                            attempt + 1,
                            e,
                            delay
                        );
                    }

                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Execute sync operation with retries
    pub fn execute_sync<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display + Send + 'static,
    {
        let mut attempt = 0;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_retries {
                        if self.config.log_retries {
                            tracing::error!(
                                "Operation failed after {} attempts: {}",
                                attempt + 1,
                                e
                            );
                        }
                        return Err(e);
                    }

                    let is_retriable = self.config.retriable_errors.is_empty()
                        || self.config.retriable_errors.iter().any(|err_type| {
                            e.to_string().contains(err_type)
                        });

                    if !is_retriable {
                        if self.config.log_retries {
                            tracing::info!("Non-retriable error: {}", e);
                        }
                        return Err(e);
                    }

                    let delay = self.config.backoff.calculate_delay(attempt);
                    if self.config.log_retries {
                        tracing::warn!(
                            "Attempt {} failed: {}. Retrying in {:?}",
                            attempt + 1,
                            e,
                            delay
                        );
                    }

                    std::thread::sleep(delay);
                    attempt += 1;
                }
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_backoff() {
        let strategy = BackoffStrategy::Fixed(Duration::from_millis(100));
        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(strategy.calculate_delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_exponential_backoff() {
        let strategy = BackoffStrategy::Exponential {
            base: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_secs(30),
        };
        
        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(400));
    }

    #[test]
    fn test_exponential_backoff_max_delay() {
        let strategy = BackoffStrategy::Exponential {
            base: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_millis(500),
        };
        
        assert_eq!(strategy.calculate_delay(3), Duration::from_millis(500));
        assert_eq!(strategy.calculate_delay(10), Duration::from_millis(500));
    }

    #[test]
    fn test_linear_backoff() {
        let strategy = BackoffStrategy::Linear {
            base: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        };
        
        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(strategy.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(strategy.calculate_delay(2), Duration::from_millis(300));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.log_retries);
        assert!(!config.retriable_errors.is_empty());
    }

    #[tokio::test]
    async fn test_retry_policy_success_first_attempt() {
        let policy = RetryPolicy::new();
        let result = policy.execute(|| {
            Box::pin(async { Ok::<_, &str>(42) })
        }).await;
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_policy_sync() {
        let policy = RetryPolicy::new();
        let mut attempt = 0;
        let result = policy.execute_sync(|| {
            attempt += 1;
            if attempt < 2 {
                Err::<_, &str>("Timeout")
            } else {
                Ok(42)
            }
        });
        assert_eq!(result, Ok(42));
    }
}
