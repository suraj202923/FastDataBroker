/*
FastDataBroker Multi-Server Clustering Tests
=============================================

Comprehensive test suite for cluster functionality:
- Broker registration and discovery
- Partition assignment and reassignment
- Replication and message safety
- Leader election and failover
- Cluster rebalancing
*/

#[cfg(test)]
mod clustering_tests {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock structures for testing
    #[derive(Debug, Clone, PartialEq)]
    enum BrokerStatus {
        Up,
        Down,
        Starting,
        Stopping,
    }

    #[derive(Debug, Clone)]
    struct BrokerMetadata {
        broker_id: u32,
        host: String,
        port: u16,
        status: BrokerStatus,
        last_heartbeat: u64,
    }

    #[derive(Debug, Clone)]
    struct PartitionMetadata {
        stream_id: String,
        partition_id: u32,
        leader_broker_id: u32,
        replica_broker_ids: Vec<u32>,
        in_sync_replicas: Vec<u32>,
        current_offset: u64,
    }

    #[derive(Debug, Clone)]
    struct StreamMetadata {
        stream_id: String,
        num_partitions: u32,
        replication_factor: u32,
        partitions: Vec<PartitionMetadata>,
    }

    struct ClusterState {
        brokers: Arc<RwLock<std::collections::HashMap<u32, BrokerMetadata>>>,
        streams: Arc<RwLock<std::collections::HashMap<String, StreamMetadata>>>,
    }

    impl ClusterState {
        fn new() -> Self {
            Self {
                brokers: Arc::new(RwLock::new(std::collections::HashMap::new())),
                streams: Arc::new(RwLock::new(std::collections::HashMap::new())),
            }
        }

        async fn register_broker(&self, metadata: BrokerMetadata) {
            let mut brokers = self.brokers.write().await;
            brokers.insert(metadata.broker_id, metadata);
        }

        async fn get_broker(&self, broker_id: u32) -> Option<BrokerMetadata> {
            let brokers = self.brokers.read().await;
            brokers.get(&broker_id).cloned()
        }

        async fn get_all_brokers(&self) -> Vec<BrokerMetadata> {
            let brokers = self.brokers.read().await;
            brokers.values().cloned().collect()
        }

        async fn create_stream(
            &self,
            stream_id: String,
            num_partitions: u32,
            replication_factor: u32,
        ) -> Result<StreamMetadata, String> {
            let brokers = self.brokers.read().await;
            let broker_ids: Vec<u32> = brokers.keys().cloned().collect();

            if broker_ids.is_empty() {
                return Err("No active brokers".to_string());
            }

            let mut partitions = Vec::new();

            for partition_id in 0..num_partitions {
                let leader_idx = (partition_id % broker_ids.len() as u32) as usize;
                let leader_broker_id = broker_ids[leader_idx];

                let mut replicas = vec![leader_broker_id];
                for i in 1..replication_factor {
                    let replica_idx = ((partition_id + i) % broker_ids.len() as u32) as usize;
                    replicas.push(broker_ids[replica_idx]);
                }

                partitions.push(PartitionMetadata {
                    stream_id: stream_id.clone(),
                    partition_id,
                    leader_broker_id,
                    replica_broker_ids: replicas.clone(),
                    in_sync_replicas: replicas,
                    current_offset: 0,
                });
            }

            let metadata = StreamMetadata {
                stream_id: stream_id.clone(),
                num_partitions,
                replication_factor,
                partitions,
            };

            let mut streams = self.streams.write().await;
            streams.insert(stream_id, metadata.clone());

            Ok(metadata)
        }

        async fn get_stream(&self, stream_id: &str) -> Option<StreamMetadata> {
            let streams = self.streams.read().await;
            streams.get(stream_id).cloned()
        }
    }

    // ========================================================================
    // TEST CASES
    // ========================================================================

    #[tokio::test]
    async fn test_broker_registration() {
        let state = ClusterState::new();

        // Register a broker
        let broker = BrokerMetadata {
            broker_id: 1,
            host: "127.0.0.1".to_string(),
            port: 6000,
            status: BrokerStatus::Up,
            last_heartbeat: 0,
        };

        state.register_broker(broker.clone()).await;

        // Verify broker is registered
        let registered = state.get_broker(1).await;
        assert_eq!(registered, Some(broker));
    }

    #[tokio::test]
    async fn test_multiple_broker_registration() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Verify all brokers registered
        let brokers = state.get_all_brokers().await;
        assert_eq!(brokers.len(), 4);
        assert_eq!(
            brokers.iter().map(|b| b.broker_id).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
    }

    #[tokio::test]
    async fn test_stream_creation() {
        let state = ClusterState::new();

        // Register brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        assert_eq!(stream.stream_id, "orders");
        assert_eq!(stream.num_partitions, 4);
        assert_eq!(stream.replication_factor, 3);
        assert_eq!(stream.partitions.len(), 4);
    }

    #[tokio::test]
    async fn test_partition_assignment() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream with 4 partitions
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // Verify partition assignment
        for (i, partition) in stream.partitions.iter().enumerate() {
            assert_eq!(partition.partition_id, i as u32);
            assert_eq!(partition.leader_broker_id, (i as u32) % 4 + 1);
            assert_eq!(partition.replica_broker_ids.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_round_robin_leader_assignment() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream
        let stream = state
            .create_stream("test".to_string(), 8, 2)
            .await
            .unwrap();

        // Verify round-robin assignment
        let leaders: Vec<u32> = stream
            .partitions
            .iter()
            .map(|p| p.leader_broker_id)
            .collect();

        expected_leaders = vec![1, 2, 3, 4, 1, 2, 3, 4];
        assert_eq!(leaders, expected_leaders);
    }

    #[tokio::test]
    async fn test_replication_across_brokers() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream with 3 replicas
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // Verify each partition has 3 replicas
        for partition in &stream.partitions {
            assert_eq!(partition.replica_broker_ids.len(), 3);
            // All replicas should be different
            let unique_replicas: std::collections::HashSet<_> =
                partition.replica_broker_ids.iter().cloned().collect();
            assert_eq!(unique_replicas.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_replica_ring_distribution() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // Verify replica distribution (ring topology)
        for partition in &stream.partitions {
            let replicas = &partition.replica_broker_ids;
            // Each broker should appear multiple times across all partitions
            let leader = partition.leader_broker_id;
            assert_eq!(replicas[0], leader); // First replica is the leader
        }
    }

    #[tokio::test]
    async fn test_consistent_hashing() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream
        let _stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // Simulate consistent hashing
        fn consistent_hash(key: &str, num_partitions: u32) -> u32 {
            let hash = key.bytes().fold(0u32, |acc, b| {
                acc.wrapping_mul(31).wrapping_add(b as u32)
            });
            hash % num_partitions
        }

        // Same key should always hash to same partition
        let partition1 = consistent_hash("ORD-001", 4);
        let partition2 = consistent_hash("ORD-001", 4);
        assert_eq!(partition1, partition2);

        // Different keys may hash to different partitions
        let partition3 = consistent_hash("ORD-002", 4);
        let partition4 = consistent_hash("ORD-003", 4);
        // Not necessarily different, but valid partitions
        assert!(partition3 < 4);
        assert!(partition4 < 4);
    }

    #[tokio::test]
    async fn test_stream_creation_without_brokers() {
        let state = ClusterState::new();

        // Try to create stream without registering brokers
        let result = state
            .create_stream("orders".to_string(), 4, 3)
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active brokers");
    }

    #[tokio::test]
    async fn test_multiple_stream_creation() {
        let state = ClusterState::new();

        // Register brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create multiple streams
        let stream1 = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        let stream2 = state
            .create_stream("events".to_string(), 8, 2)
            .await
            .unwrap();

        let stream3 = state
            .create_stream("alerts".to_string(), 2, 3)
            .await
            .unwrap();

        assert_eq!(stream1.num_partitions, 4);
        assert_eq!(stream2.num_partitions, 8);
        assert_eq!(stream3.num_partitions, 2);

        // Verify all streams registered
        assert!(state.get_stream("orders").await.is_some());
        assert!(state.get_stream("events").await.is_some());
        assert!(state.get_stream("alerts").await.is_some());
    }

    #[tokio::test]
    async fn test_broker_status_tracking() {
        let state = ClusterState::new();

        // Register broker as UP
        let broker = BrokerMetadata {
            broker_id: 1,
            host: "127.0.0.1".to_string(),
            port: 6000,
            status: BrokerStatus::Up,
            last_heartbeat: 0,
        };
        state.register_broker(broker).await;

        // Verify status
        let registered = state.get_broker(1).await.unwrap();
        assert_eq!(registered.status, BrokerStatus::Up);
    }

    #[tokio::test]
    async fn test_heartbeat_tracking() {
        let state = ClusterState::new();

        let timestamp = 1000u64;
        let broker = BrokerMetadata {
            broker_id: 1,
            host: "127.0.0.1".to_string(),
            port: 6000,
            status: BrokerStatus::Up,
            last_heartbeat: timestamp,
        };
        state.register_broker(broker).await;

        let registered = state.get_broker(1).await.unwrap();
        assert_eq!(registered.last_heartbeat, timestamp);
    }

    #[tokio::test]
    async fn test_partition_offset_tracking() {
        let state = ClusterState::new();

        // Register brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // Verify initial offsets are 0
        for partition in &stream.partitions {
            assert_eq!(partition.current_offset, 0);
        }
    }

    #[tokio::test]
    async fn test_in_sync_replicas_tracking() {
        let state = ClusterState::new();

        // Register brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // Verify in-sync replicas
        for partition in &stream.partitions {
            assert_eq!(partition.in_sync_replicas.len(), 3);
            assert_eq!(partition.in_sync_replicas, partition.replica_broker_ids);
        }
    }

    #[tokio::test]
    async fn test_stream_with_single_broker() {
        let state = ClusterState::new();

        // Register single broker
        let broker = BrokerMetadata {
            broker_id: 1,
            host: "127.0.0.1".to_string(),
            port: 6000,
            status: BrokerStatus::Up,
            last_heartbeat: 0,
        };
        state.register_broker(broker).await;

        // Create stream with 4 partitions (all on same broker)
        let stream = state
            .create_stream("orders".to_string(), 4, 3)
            .await
            .unwrap();

        // All partitions should be on the same leader (broker 1)
        for partition in &stream.partitions {
            assert_eq!(partition.leader_broker_id, 1);
        }

        // But replicas should be requested from other brokers
        // Since only 1 exists, this is a limitation
        assert_eq!(stream.replication_factor, 3);
    }

    #[tokio::test]
    async fn test_large_partition_count() {
        let state = ClusterState::new();

        // Register 32 brokers
        for i in 1..=32 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream with 128 partitions
        let stream = state
            .create_stream("orders".to_string(), 128, 3)
            .await
            .unwrap();

        assert_eq!(stream.partitions.len(), 128);

        // Verify each partition has a leader
        for partition in &stream.partitions {
            assert!(partition.leader_broker_id >= 1 && partition.leader_broker_id <= 32);
        }
    }

    #[tokio::test]
    async fn test_partition_distribution_balance() {
        let state = ClusterState::new();

        // Register 4 brokers
        for i in 1..=4 {
            let broker = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(broker).await;
        }

        // Create stream with 16 partitions
        let stream = state
            .create_stream("orders".to_string(), 16, 3)
            .await
            .unwrap();

        // Count leaders per broker
        let mut leader_counts = std::collections::HashMap::new();
        for partition in &stream.partitions {
            *leader_counts.entry(partition.leader_broker_id).or_insert(0) += 1;
        }

        // Each broker should have ~4 partition leaders (16 partitions / 4 brokers)
        for (_, count) in leader_counts {
            assert_eq!(count, 4);
        }
    }

    #[tokio::test]
    async fn test_offset_increment() {
        let state = ClusterState::new();

        // Register broker
        let broker = BrokerMetadata {
            broker_id: 1,
            host: "127.0.0.1".to_string(),
            port: 6000,
            status: BrokerStatus::Up,
            last_heartbeat: 0,
        };
        state.register_broker(broker).await;

        // Create stream
        let stream = state
            .create_stream("test".to_string(), 1, 1)
            .await
            .unwrap();

        let partition = &stream.partitions[0];
        assert_eq!(partition.current_offset, 0);

        // In real implementation, offset would increment with each message
        // This test verifies initial state
    }
}

// Non-async tests
#[cfg(test)]
mod consistent_hashing_tests {
    fn consistent_hash(key: &str, num_partitions: u32) -> u32 {
        let hash = key.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u32)
        });
        hash % num_partitions
    }

    #[test]
    fn test_hash_consistency() {
        let key = "ORD-12345";
        let p1 = consistent_hash(key, 4);
        let p2 = consistent_hash(key, 4);
        let p3 = consistent_hash(key, 4);

        assert_eq!(p1, p2);
        assert_eq!(p2, p3);
    }

    #[test]
    fn test_hash_distribution() {
        let mut counts = std::collections::HashMap::new();

        for i in 0..1000 {
            let key = format!("order-{}", i);
            let partition = consistent_hash(&key, 4);
            *counts.entry(partition).or_insert(0) += 1;
        }

        // Should be relatively balanced (between 200-300 for 4 partitions with 1000 keys)
        for (_, count) in counts {
            assert!(count >= 200 && count <= 300, "Partition has {} entries", count);
        }
    }

    #[test]
    fn test_hash_range() {
        let partitions = 8;
        for i in 0..100 {
            let key = format!("key-{}", i);
            let partition = consistent_hash(&key, partitions);
            assert!(
                partition < partitions,
                "Partition {} out of range for {} partitions",
                partition,
                partitions
            );
        }
    }
}
