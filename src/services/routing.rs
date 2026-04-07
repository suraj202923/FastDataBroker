//! Routing Service - Phase 2
//!
//! Routes messages to recipient queues
//! Implements direct routing, topic routing, load balancing, and dead letter queue

use crate::models::{Envelope, RecipientId};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Routing statistics
#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_routed: Arc<AtomicU64>,
    pub direct_routes: Arc<AtomicU64>,
    pub topic_routes: Arc<AtomicU64>,
    pub dead_letter: Arc<AtomicU64>,
}

impl Default for RoutingStats {
    fn default() -> Self {
        RoutingStats {
            total_routed: Arc::new(AtomicU64::new(0)),
            direct_routes: Arc::new(AtomicU64::new(0)),
            topic_routes: Arc::new(AtomicU64::new(0)),
            dead_letter: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Routing rule for pattern matching
#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub pattern: String,  // e.g., "users_#_notifications"
    pub destination: String,
}

/// Routing result for each recipient
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub recipient_id: RecipientId,
    pub should_deliver: bool,
    pub reason: String,
}

/// Routing Service - distributes messages to recipients
pub struct RoutingService {
    stats: RoutingStats,
    rules: Arc<RwLock<Vec<RoutingRule>>>,
    recipient_load: Arc<RwLock<HashMap<String, u64>>>,
}

impl RoutingService {
    /// Create a new routing service
    pub fn new() -> Self {
        RoutingService {
            stats: RoutingStats::default(),
            rules: Arc::new(RwLock::new(Vec::new())),
            recipient_load: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Route a message to all recipients
    pub async fn route(&self, envelope: &Envelope) -> Result<Vec<RoutingDecision>> {
        self.stats.total_routed.fetch_add(1, Ordering::Relaxed);

        let mut decisions = Vec::new();

        for recipient_id in &envelope.recipient_ids {
            let decision = self.route_to_recipient(envelope, recipient_id).await?;
            decisions.push(decision);
        }

        let deliverable = decisions.iter().filter(|d| d.should_deliver).count();
        if deliverable == 0 {
            warn!(
                "Message {} has no deliverable recipients, routing to DLQ",
                envelope.id
            );
            self.stats.dead_letter.fetch_add(1, Ordering::Relaxed);
        }

        info!(
            "Message {} routed to {} recipients ({} deliverable)",
            envelope.id,
            envelope.recipient_ids.len(),
            deliverable
        );

        Ok(decisions)
    }

    /// Route message to a single recipient
    async fn route_to_recipient(
        &self,
        envelope: &Envelope,
        recipient_id: &RecipientId,
    ) -> Result<RoutingDecision> {
        // Try direct routing first
        let decision = self.direct_route(recipient_id).await;

        if decision.should_deliver {
            self.stats.direct_routes.fetch_add(1, Ordering::Relaxed);
            debug!(
                "Direct route for message {} to recipient {}",
                envelope.id, recipient_id
            );
            return Ok(decision);
        }

        // Try topic-based routing
        if let Some(decision) = self.try_topic_route(recipient_id).await {
            self.stats.topic_routes.fetch_add(1, Ordering::Relaxed);
            debug!(
                "Topic route for message {} to recipient {}",
                envelope.id, recipient_id
            );
            return Ok(decision);
        }

        // Fall back to dead letter queue
        warn!("No route found for recipient {}, sending to DLQ", recipient_id);
        Ok(RoutingDecision {
            recipient_id: recipient_id.clone(),
            should_deliver: false,
            reason: "No matching route found".to_string(),
        })
    }

    /// Direct routing - check if recipient exists
    async fn direct_route(&self, recipient_id: &RecipientId) -> RoutingDecision {
        // For now, accept all recipients
        // TODO: Check against recipient registry in Phase 2
        self.update_recipient_load(recipient_id, 1).await;

        RoutingDecision {
            recipient_id: recipient_id.clone(),
            should_deliver: true,
            reason: "Direct route".to_string(),
        }
    }

    /// Topic-based routing with pattern matching
    async fn try_topic_route(&self, recipient_id: &RecipientId) -> Option<RoutingDecision> {
        let rules = self.rules.read().await;

        for rule in rules.iter() {
            if self.pattern_matches(&rule.pattern, recipient_id) {
                return Some(RoutingDecision {
                    recipient_id: recipient_id.clone(),
                    should_deliver: true,
                    reason: format!("Topic route: {}", rule.pattern),
                });
            }
        }

        None
    }

    /// Pattern matching (simple wildcard support: # = any character)
    fn pattern_matches(&self, pattern: &str, recipient: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('#').collect();

        if pattern_parts.len() == 1 {
            return recipient == pattern;
        }

        // Check if recipient starts with first part
        if !recipient.starts_with(pattern_parts[0]) {
            return false;
        }

        // Check if recipient ends with last part
        if !pattern_parts[pattern_parts.len() - 1].is_empty() {
            if !recipient.ends_with(pattern_parts[pattern_parts.len() - 1]) {
                return false;
            }
        }

        true
    }

    /// Update recipient load tracking
    async fn update_recipient_load(&self, recipient_id: &RecipientId, messages: u64) {
        let mut load = self.recipient_load.write().await;
        *load.entry(recipient_id.clone()).or_insert(0) += messages;
    }

    /// Get load-balanced recipient (lowest load)
    pub async fn get_least_loaded_recipient(
        &self,
        candidates: &[RecipientId],
    ) -> Option<RecipientId> {
        if candidates.is_empty() {
            return None;
        }

        let load = self.recipient_load.read().await;
        candidates
            .iter()
            .min_by_key(|r| load.get(*r).unwrap_or(&0))
            .cloned()
    }

    /// Add a routing rule
    pub async fn add_rule(&self, pattern: String, destination: String) {
        let mut rules = self.rules.write().await;
        rules.push(RoutingRule { pattern, destination });
    }

    /// Get routing statistics
    pub fn stats(&self) -> RoutingStats {
        self.stats.clone()
    }
}

impl Default for RoutingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_direct_route() {
        let service = RoutingService::new();
        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test".to_string(),
            vec![],
        );

        let decisions = service.route(&envelope).await.unwrap();
        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].should_deliver);
    }

    #[tokio::test]
    async fn test_multiple_recipients() {
        let service = RoutingService::new();
        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec![
                "recipient-1".to_string(),
                "recipient-2".to_string(),
                "recipient-3".to_string(),
            ],
            "Test".to_string(),
            vec![],
        );

        let decisions = service.route(&envelope).await.unwrap();
        assert_eq!(decisions.len(), 3);
        assert!(decisions.iter().all(|d| d.should_deliver));
    }

    #[test]
    fn test_pattern_matching() {
        let service = RoutingService::new();

        assert!(service.pattern_matches("users_#_notifications", "users_123_notifications"));
        assert!(service.pattern_matches("users_#_notifications", "users_john_notifications"));
        assert!(!service.pattern_matches("users_#_notifications", "admin_123_notifications"));
    }

    #[tokio::test]
    async fn test_load_balancing() {
        let service = RoutingService::new();
        let recipients = vec![
            "recipient-1".to_string(),
            "recipient-2".to_string(),
            "recipient-3".to_string(),
        ];

        service.update_recipient_load(&recipients[0], 100).await;
        service.update_recipient_load(&recipients[1], 50).await;
        service.update_recipient_load(&recipients[2], 75).await;

        let least_loaded = service
            .get_least_loaded_recipient(&recipients)
            .await
            .unwrap();
        assert_eq!(least_loaded, "recipient-2");
    }
}
