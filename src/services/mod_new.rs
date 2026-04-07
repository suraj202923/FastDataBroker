//! Core Post Office Services - Phase 2
//!
//! Services for message ingestion, routing, storage, delivery, and priority management

pub mod ingestion;
pub mod routing;
pub mod storage;
pub mod priority;
pub mod delivery;

pub use ingestion::{IngestionService, IngestionServiceConfig, IngestionStats};
pub use routing::{RoutingService, RoutingDecision};
pub use storage::{StorageService, StorageStats, StorageTier};
pub use priority::{PriorityManager, PriorityManagerConfig, Priority};
pub use delivery::{DeliveryService, DeliveryServiceConfig, DeliveryStatus};

use crate::models::Envelope;
use anyhow::Result;

/// Post Office Broker - orchestrates all services
pub struct FastDataBroker {
    pub ingestion: IngestionService,
    pub routing: RoutingService,
    pub storage: StorageService,
    pub priority: PriorityManager,
    pub delivery: DeliveryService,
}

impl FastDataBroker {
    /// Create a new broker with default configurations
    pub fn new() -> Self {
        FastDataBroker {
            ingestion: IngestionService::new(IngestionServiceConfig::default()),
            routing: RoutingService::new(),
            storage: StorageService::new(),
            priority: PriorityManager::new(PriorityManagerConfig::default()),
            delivery: DeliveryService::new(DeliveryServiceConfig::default()),
        }
    }

    /// Process a message through the complete pipeline
    pub async fn process_message(&self, mut envelope: Envelope) -> Result<()> {
        // Step 1: Ingestion
        let message_id = self.ingestion.ingest(envelope.clone()).await?;
        envelope.id = message_id;

        // Step 2: Priority processing
        self.priority.process_priority(&mut envelope).await?;

        // Step 3: Routing
        let routing_decisions = self.routing.route(&envelope).await?;

        // Step 4: Storage
        self.storage.store(&envelope).await?;

        // Step 5: Delivery
        for decision in routing_decisions {
            if decision.should_deliver {
                let _ = self.delivery.deliver(&envelope, &decision.recipient_id).await;
            }
        }

        Ok(())
    }
}

impl Default for FastDataBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broker_full_pipeline() {
        let broker = FastDataBroker::new();
        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test Message".to_string(),
            b"Hello, World!".to_vec(),
        );

        let result = broker.process_message(envelope).await;
        assert!(result.is_ok());
    }
}
