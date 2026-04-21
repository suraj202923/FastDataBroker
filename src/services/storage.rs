//! Storage Service - Phase 2
//!
//! Persists messages using AsyncPersistenceQueue
//! Manages hot, warm, and cold storage tiers

use crate::models::Envelope;
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_stored: Arc<AtomicU64>,
    pub hot_storage: Arc<AtomicU64>,
    pub warm_storage: Arc<AtomicU64>,
    pub cold_storage: Arc<AtomicU64>,
    pub deleted: Arc<AtomicU64>,
}

impl Default for StorageStats {
    fn default() -> Self {
        StorageStats {
            total_stored: Arc::new(AtomicU64::new(0)),
            hot_storage: Arc::new(AtomicU64::new(0)),
            warm_storage: Arc::new(AtomicU64::new(0)),
            cold_storage: Arc::new(AtomicU64::new(0)),
            deleted: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Storage tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTier {
    /// Recent messages (< 1 hour), in-memory
    Hot,
    /// Active messages (1-24 hours), SSD cache
    Warm,
    /// Archived messages (> 24 hours), persistent storage
    Cold,
}

impl StorageTier {
    /// Get tier for message age
    pub fn for_age_seconds(age_seconds: u64) -> Self {
        const ONE_HOUR: u64 = 3600;
        const ONE_DAY: u64 = 86400;

        if age_seconds < ONE_HOUR {
            StorageTier::Hot
        } else if age_seconds < ONE_DAY {
            StorageTier::Warm
        } else {
            StorageTier::Cold
        }
    }
}

/// Storage metadata
#[derive(Debug, Clone)]
struct StorageMetadata {
    stored_at: u64,
    expires_at: Option<u64>,
    tier: StorageTier,
}

/// Storage Service - manages message persistence
pub struct StorageService {
    stats: StorageStats,
    // In-memory storage (Phase 2: replace with actual AsyncPersistenceQueue)
    hot_store: Arc<RwLock<Vec<Envelope>>>,
    metadata: Arc<RwLock<std::collections::HashMap<String, StorageMetadata>>>,
}

impl StorageService {
    /// Create a new storage service
    pub fn new() -> Self {
        StorageService {
            stats: StorageStats::default(),
            hot_store: Arc::new(RwLock::new(Vec::new())),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Store a message with appropriate tier selection
    pub async fn store(&self, envelope: &Envelope) -> Result<()> {
        self.stats.total_stored.fetch_add(1, Ordering::Relaxed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Determine storage tier based on message age
        let tier = StorageTier::for_age_seconds(0);  // New messages go to Hot

        // Calculate expiration time
        let expires_at = envelope.ttl_seconds.map(|ttl| now + ttl);

        // Store in appropriate tier
        match tier {
            StorageTier::Hot => {
                self.store_hot(envelope).await?;
                self.stats.hot_storage.fetch_add(1, Ordering::Relaxed);
            }
            StorageTier::Warm => {
                // TODO: Implement warm storage (SSD cache)
                self.stats.warm_storage.fetch_add(1, Ordering::Relaxed);
            }
            StorageTier::Cold => {
                // TODO: Implement cold storage (persistent)
                self.stats.cold_storage.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Track metadata
        let mut metadata = self.metadata.write().await;
        metadata.insert(
            envelope.id.to_string(),
            StorageMetadata {
                stored_at: now,
                expires_at,
                tier,
            },
        );

        info!(
            "💾 Message {} stored in {:?} storage (TTL: {:?}s)",
            envelope.id, tier, envelope.ttl_seconds
        );

        Ok(())
    }

    /// Retrieve a message
    pub async fn retrieve(&self, message_id: &str) -> Result<Option<Envelope>> {
        let hot = self.hot_store.read().await;
        let envelope = hot.iter().find(|e| e.id.to_string() == message_id).cloned();

        if let Some(env) = envelope {
            debug!("📤 Message {} retrieved from hot storage", message_id);
            return Ok(Some(env));
        }

        // TODO: Check warm and cold storage

        Ok(None)
    }

    /// Store in hot (in-memory) storage
    async fn store_hot(&self, envelope: &Envelope) -> Result<()> {
        let mut hot = self.hot_store.write().await;
        hot.push(envelope.clone());
        Ok(())
    }

    /// Clean up expired messages
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut hot = self.hot_store.write().await;
        let mut metadata = self.metadata.write().await;

        let initial_len = hot.len();

        // Remove expired messages
        hot.retain(|env| {
            let meta = metadata.get(&env.id.to_string());
            if let Some(meta) = meta {
                if let Some(expires_at) = meta.expires_at {
                    if now > expires_at {
                        metadata.remove(&env.id.to_string());
                        self.stats.deleted.fetch_add(1, Ordering::Relaxed);
                        return false;
                    }
                }
            }
            true
        });

        let deleted = initial_len - hot.len();
        if deleted > 0 {
            info!("🗑️  Cleaned up {} expired messages", deleted);
        }

        Ok(deleted as u64)
    }

    /// Get storage statistics
    pub fn stats(&self) -> StorageStats {
        self.stats.clone()
    }

    /// Get total messages in storage
    pub async fn count(&self) -> usize {
        self.hot_store.read().await.len()
    }
}

impl Default for StorageService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let service = StorageService::new();
        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test".to_string(),
            b"Data".to_vec(),
        );

        let msg_id = envelope.id.to_string();
        service.store(&envelope).await.unwrap();

        let retrieved = service.retrieve(&msg_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, envelope.id);
    }

    #[tokio::test]
    async fn test_storage_tier_classification() {
        let one_hour = 3600;
        let one_day = 86400;

        assert_eq!(StorageTier::for_age_seconds(0), StorageTier::Hot);
        assert_eq!(StorageTier::for_age_seconds(one_hour - 1), StorageTier::Hot);
        assert_eq!(StorageTier::for_age_seconds(one_hour), StorageTier::Warm);
        assert_eq!(StorageTier::for_age_seconds(one_day), StorageTier::Cold);
    }

    #[tokio::test]
    async fn test_storage_count() {
        let service = StorageService::new();
        assert_eq!(service.count().await, 0);

        let envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test".to_string(),
            vec![],
        );

        service.store(&envelope).await.unwrap();
        assert_eq!(service.count().await, 1);

        // Store another message
        let envelope2 = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-2".to_string()],
            "Test 2".to_string(),
            vec![],
        );
        service.store(&envelope2).await.unwrap();
        assert_eq!(service.count().await, 2);
    }

    #[tokio::test]
    async fn test_storage_ttl_expiration() {
        let service = StorageService::new();
        
        // Create envelope with 1 second TTL
        let mut envelope = Envelope::new(
            "producer-1".to_string(),
            vec!["recipient-1".to_string()],
            "Test TTL".to_string(),
            vec![],
        );
        envelope.ttl_seconds = Some(1);
        
        let msg_id = envelope.id.to_string();
        service.store(&envelope).await.unwrap();
        
        // Should be retrievable immediately
        let retrieved = service.retrieve(&msg_id).await.unwrap();
        assert!(retrieved.is_some());
        
        // Wait for TTL to expire
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // Now it should be expired when we cleanup
        let deleted = service.cleanup_expired().await.unwrap();
        assert_eq!(deleted, 1);
        
        // Verify it's gone
        let retrieved = service.retrieve(&msg_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_storage_concurrent_operations() {
        use std::sync::Arc;
        
        let service = Arc::new(StorageService::new());
        let mut handles = vec![];
        
        // Spawn 5 threads that each store messages
        for i in 0..5 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let envelope = Envelope::new(
                        format!("producer-{}", i),
                        vec![format!("recipient-{}", i)],
                        format!("Message {}-{}", i, j),
                        vec![],
                    );
                    service_clone.store(&envelope).await.unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Should have 50 messages total (5 threads * 10 messages)
        assert_eq!(service.count().await, 50);
        
        let stats = service.stats();
        assert_eq!(stats.total_stored.load(std::sync::atomic::Ordering::Relaxed), 50);
    }

    #[tokio::test]
    async fn test_storage_multiple_retrieval() {
        let service = StorageService::new();
        
        // Store multiple messages
        let mut envelopes = vec![];
        for i in 0..5 {
            let envelope = Envelope::new(
                "producer-1".to_string(),
                vec!["recipient-1".to_string()],
                format!("Message {}", i),
                vec![],
            );
            envelopes.push(envelope);
        }
        
        for envelope in &envelopes {
            service.store(envelope).await.unwrap();
        }
        
        assert_eq!(service.count().await, 5);
        
        // Retrieve each one
        for envelope in &envelopes {
            let retrieved = service.retrieve(&envelope.id.to_string()).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().id, envelope.id);
        }
    }
}
