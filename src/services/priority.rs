//! Priority Manager - Phase 2
//!
//! Handles message prioritization with aging and starvation prevention

use crate::models::Envelope;
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// Priority statistics
#[derive(Debug, Clone)]
pub struct PriorityStats {
    pub total_processed: Arc<AtomicU64>,
    pub priority_adjusted: Arc<AtomicU64>,
    pub starvation_prevented: Arc<AtomicU64>,
}

impl Default for PriorityStats {
    fn default() -> Self {
        PriorityStats {
            total_processed: Arc::new(AtomicU64::new(0)),
            priority_adjusted: Arc::new(AtomicU64::new(0)),
            starvation_prevented: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Priority levels (0-255)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

impl Priority {
    /// Minimum priority
    pub const MIN: Priority = Priority(0);
    /// Deferred priority (background)
    pub const DEFERRED: Priority = Priority(50);
    /// Normal priority
    pub const NORMAL: Priority = Priority(100);
    /// High priority
    pub const HIGH: Priority = Priority(150);
    /// Urgent priority
    pub const URGENT: Priority = Priority(200);
    /// Critical priority
    pub const CRITICAL: Priority = Priority(255);

    /// Boost priority for aging messages
    pub fn boost(&mut self, increment: u8) {
        self.0 = self.0.saturating_add(increment);
    }

    /// Get priority name for logging
    pub fn name(&self) -> &'static str {
        match self.0 {
            0..=50 => "DEFERRED",
            51..=100 => "NORMAL",
            101..=150 => "HIGH",
            151..=200 => "URGENT",
            201..=255 => "CRITICAL",
        }
    }
}

impl From<u8> for Priority {
    fn from(val: u8) -> Self {
        Priority(val)
    }
}

/// Message aging information
#[derive(Debug, Clone)]
struct MessageAge {
    created_at: u64,
    last_processed: u64,
    processing_attempts: u64,
}

/// Priority Manager configuration
#[derive(Debug, Clone)]
pub struct PriorityManagerConfig {
    /// Seconds before boosting priority for aged messages
    pub aging_threshold_seconds: u64,
    /// Priority boost per aging period
    pub priority_boost: u8,
    /// Max attempts before critical boost
    pub max_attempts_before_critical: u64,
}

impl Default for PriorityManagerConfig {
    fn default() -> Self {
        PriorityManagerConfig {
            aging_threshold_seconds: 3600,  // 1 hour
            priority_boost: 10,
            max_attempts_before_critical: 3,
        }
    }
}

/// Priority Manager - handles message prioritization
pub struct PriorityManager {
    config: PriorityManagerConfig,
    stats: PriorityStats,
    message_ages: Arc<tokio::sync::RwLock<std::collections::HashMap<String, MessageAge>>>,
}

impl PriorityManager {
    /// Create a new priority manager
    pub fn new(config: PriorityManagerConfig) -> Self {
        PriorityManager {
            config,
            stats: PriorityStats::default(),
            message_ages: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Process priority for a message, applying aging and starvation prevention
    pub async fn process_priority(&self, envelope: &mut Envelope) -> Result<()> {
        self.stats.total_processed.fetch_add(1, Ordering::Relaxed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let message_id = envelope.id.to_string();

        // Get or create message age record
        let mut ages = self.message_ages.write().await;
        let age = ages
            .entry(message_id.clone())
            .or_insert_with(|| MessageAge {
                created_at: now,
                last_processed: now,
                processing_attempts: 0,
            });

        let message_age = now - age.created_at;
        let mut priority_boost = 0u8;

        // Apply aging boost
        if message_age > self.config.aging_threshold_seconds {
            let aging_periods =
                (message_age / self.config.aging_threshold_seconds) as u8;
            priority_boost = priority_boost.saturating_add(
                self.config.priority_boost.saturating_mul(aging_periods),
            );

            debug!(
                "Message {} aged {} seconds, boosting priority by {}",
                message_id, message_age, priority_boost
            );
        }

        // Starvation prevention
        if age.processing_attempts >= self.config.max_attempts_before_critical {
            envelope.priority = Priority::CRITICAL.0;
            self.stats.starvation_prevented.fetch_add(1, Ordering::Relaxed);

            info!(
                "⚠️  Message {} prevented from starvation, set to CRITICAL priority",
                message_id
            );
        } else if priority_boost > 0 {
            let new_priority = envelope.priority.saturating_add(priority_boost);
            envelope.priority = new_priority.min(255);  // Cap at 255
            self.stats.priority_adjusted.fetch_add(1, Ordering::Relaxed);

            debug!(
                "Message {} priority boosted: {} → {} (boost: {})",
                message_id,
                envelope.priority.saturating_sub(priority_boost),
                envelope.priority,
                priority_boost
            );
        }

        // Update age record
        age.last_processed = now;
        age.processing_attempts += 1;

        Ok(())
    }

    /// Validate priority level
    pub fn validate_priority(&self, priority: u8) -> Result<()> {
        // All u8 values (0-255) are valid
        Ok(())
    }

    /// Get priority information
    pub fn get_priority_info(&self, priority: u8) -> &'static str {
        Priority::from(priority).name()
    }

    /// Clean up old message age records
    pub async fn cleanup_old_messages(&self, max_age_seconds: u64) -> Result<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut ages = self.message_ages.write().await;
        let initial_len = ages.len();

        ages.retain(|_, age| now - age.created_at <= max_age_seconds);

        let removed = initial_len - ages.len();
        if removed > 0 {
            info!(
                "🗑️  Cleaned up {} old message priority records",
                removed
            );
        }

        Ok(removed as u64)
    }

    /// Get priority statistics
    pub fn stats(&self) -> PriorityStats {
        self.stats.clone()
    }
}

impl Default for PriorityManager {
    fn default() -> Self {
        Self::new(PriorityManagerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::CRITICAL > Priority::URGENT);
        assert!(Priority::URGENT > Priority::HIGH);
        assert!(Priority::HIGH > Priority::NORMAL);
        assert!(Priority::NORMAL > Priority::DEFERRED);
        assert!(Priority::DEFERRED > Priority::MIN);
    }

    #[test]
    fn test_priority_name() {
        assert_eq!(Priority::DEFERRED.name(), "DEFERRED");
        assert_eq!(Priority::NORMAL.name(), "NORMAL");
        assert_eq!(Priority::HIGH.name(), "HIGH");
        assert_eq!(Priority::URGENT.name(), "URGENT");
        assert_eq!(Priority::CRITICAL.name(), "CRITICAL");
    }

    #[tokio::test]
    async fn test_priority_boost() {
        let config = PriorityManagerConfig {
            aging_threshold_seconds: 1,
            priority_boost: 10,
            ..Default::default()
        };
        let manager = PriorityManager::new(config);

        let mut envelope = Envelope::new(
            "producer".to_string(),
            vec!["recipient".to_string()],
            "Test".to_string(),
            vec![],
        );
        envelope.priority = 50;

        // Simulate aging
        let mut ages = manager.message_ages.write().await;
        ages.insert(
            envelope.id.to_string(),
            MessageAge {
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - 10,
                last_processed: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                processing_attempts: 0,
            },
        );
        drop(ages);

        manager.process_priority(&mut envelope).await.unwrap();
        assert!(envelope.priority > 50);
    }

    #[tokio::test]
    async fn test_starvation_prevention() {
        let config = PriorityManagerConfig {
            max_attempts_before_critical: 1,
            ..Default::default()
        };
        let manager = PriorityManager::new(config);

        let mut envelope = Envelope::new(
            "producer".to_string(),
            vec!["recipient".to_string()],
            "Test".to_string(),
            vec![],
        );
        envelope.priority = 10;

        // Simulate multiple processing attempts
        for _ in 0..2 {
            manager.process_priority(&mut envelope).await.unwrap();
        }

        assert_eq!(envelope.priority, Priority::CRITICAL.0);
    }
}
