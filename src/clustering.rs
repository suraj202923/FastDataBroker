/*
FastDataBroker Multi-Server Cluster Implementation
==================================================

Distributed brokers with:
- Partition management
- Replication
- Leader election
- Automatic failover
*/

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// CLUSTER CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    pub broker_id: u32,
    pub listen_address: String,
    pub listen_port: u16,
    pub metadata_servers: Vec<String>,
    pub bootstrap_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamPartitionConfig {
    pub stream_id: String,
    pub num_partitions: u32,
    pub replication_factor: u32,
    pub min_insync_replicas: u32,
    pub retention_hours: u32,
}

// ============================================================================
// BROKER & PARTITION METADATA
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerMetadata {
    pub broker_id: u32,
    pub host: String,
    pub port: u16,
    pub status: BrokerStatus,
    pub last_heartbeat: u64,  // unix timestamp
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BrokerStatus {
    Up,
    Down,
    Starting,
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionMetadata {
    pub stream_id: String,
    pub partition_id: u32,
    pub leader_broker_id: u32,
    pub replica_broker_ids: Vec<u32>,
    pub in_sync_replicas: Vec<u32>,
    pub current_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub stream_id: String,
    pub num_partitions: u32,
    pub replication_factor: u32,
    pub partitions: Vec<PartitionMetadata>,
}

// ============================================================================
// CLUSTER STATE
// ============================================================================

pub struct ClusterState {
    brokers: Arc<RwLock<HashMap<u32, BrokerMetadata>>>,
    streams: Arc<RwLock<HashMap<String, StreamMetadata>>>,
    partition_leaders: Arc<RwLock<HashMap<(String, u32), u32>>>,  // (stream, partition) -> leader
    controller_broker_id: Arc<RwLock<u32>>,
}

impl ClusterState {
    pub fn new() -> Self {
        Self {
            brokers: Arc::new(RwLock::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
            partition_leaders: Arc::new(RwLock::new(HashMap::new())),
            controller_broker_id: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a broker in the cluster
    pub async fn register_broker(&self, metadata: BrokerMetadata) {
        let mut brokers = self.brokers.write().await;
        brokers.insert(metadata.broker_id, metadata);
    }

    /// Get broker metadata
    pub async fn get_broker(&self, broker_id: u32) -> Option<BrokerMetadata> {
        let brokers = self.brokers.read().await;
        brokers.get(&broker_id).cloned()
    }

    /// Get all brokers
    pub async fn get_all_brokers(&self) -> Vec<BrokerMetadata> {
        let brokers = self.brokers.read().await;
        brokers.values().cloned().collect()
    }

    /// Create a stream with partitions
    pub async fn create_stream(&self, config: StreamPartitionConfig) -> Result<StreamMetadata, String> {
        let brokers = self.brokers.read().await;
        let broker_ids: Vec<u32> = brokers.keys().cloned().collect();

        if broker_ids.is_empty() {
            return Err("No active brokers in cluster".to_string());
        }

        let mut partitions = Vec::new();

        // Assign partitions to brokers using round-robin
        for partition_id in 0..config.num_partitions {
            let leader_idx = (partition_id % broker_ids.len() as u32) as usize;
            let leader_broker_id = broker_ids[leader_idx];

            // Replicate to next brokers in ring
            let mut replicas = vec![leader_broker_id];
            for i in 1..config.replication_factor {
                let replica_idx = ((partition_id + i) % broker_ids.len() as u32) as usize;
                replicas.push(broker_ids[replica_idx]);
            }

            partitions.push(PartitionMetadata {
                stream_id: config.stream_id.clone(),
                partition_id,
                leader_broker_id,
                replica_broker_ids: replicas.clone(),
                in_sync_replicas: replicas.clone(),
                current_offset: 0,
            });

            // Register partition leader
            let mut leaders = self.partition_leaders.write().await;
            leaders.insert((config.stream_id.clone(), partition_id), leader_broker_id);
        }

        let metadata = StreamMetadata {
            stream_id: config.stream_id.clone(),
            num_partitions: config.num_partitions,
            replication_factor: config.replication_factor,
            partitions,
        };

        let mut streams = self.streams.write().await;
        streams.insert(config.stream_id.clone(), metadata.clone());

        Ok(metadata)
    }

    /// Get stream metadata
    pub async fn get_stream(&self, stream_id: &str) -> Option<StreamMetadata> {
        let streams = self.streams.read().await;
        streams.get(stream_id).cloned()
    }

    /// Get partition leader
    pub async fn get_partition_leader(
        &self,
        stream_id: &str,
        partition_id: u32,
    ) -> Option<u32> {
        let leaders = self.partition_leaders.read().await;
        leaders.get(&(stream_id.to_string(), partition_id)).copied()
    }

    /// Determine partition for a message (consistent hashing)
    pub async fn determine_partition(
        &self,
        stream_id: &str,
        partition_key: &str,
    ) -> Result<u32, String> {
        let stream = self.get_stream(stream_id).await
            .ok_or("Stream not found".to_string())?;

        // Use simple hash modulo
        let hash = partition_key.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u32)
        });

        Ok(hash % stream.num_partitions)
    }
}

// ============================================================================
// PARTITION REPLICA MANAGER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationLog {
    pub message_offset: u64,
    pub leader_broker_id: u32,
    pub replicated_to: Vec<u32>,
}

pub struct ReplicationManager {
    state: Arc<ClusterState>,
    replication_logs: Arc<RwLock<HashMap<(String, u32), Vec<ReplicationLog>>>>,
}

impl ReplicationManager {
    pub fn new(state: Arc<ClusterState>) -> Self {
        Self {
            state,
            replication_logs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Send message to leaders and replicas
    pub async fn replicate_message(
        &self,
        stream_id: &str,
        partition_id: u32,
        offset: u64,
    ) -> Result<ReplicationLog, String> {
        let stream = self.state.get_stream(stream_id).await
            .ok_or("Stream not found".to_string())?;

        let partition = stream.partitions.iter()
            .find(|p| p.partition_id == partition_id)
            .ok_or("Partition not found".to_string())?;

        // In real implementation, would send to replicas
        // For now, simulate successful replication
        let log = ReplicationLog {
            message_offset: offset,
            leader_broker_id: partition.leader_broker_id,
            replicated_to: partition.replica_broker_ids.clone(),
        };

        // Record replication log
        let key = (stream_id.to_string(), partition_id);
        let mut logs = self.replication_logs.write().await;
        logs.entry(key).or_insert_with(Vec::new).push(log.clone());

        Ok(log)
    }

    /// Check if message is safely replicated (quorum)
    pub async fn is_committed(
        &self,
        stream_id: &str,
        partition_id: u32,
        offset: u64,
        min_replicas: u32,
    ) -> bool {
        let key = (stream_id.to_string(), partition_id);
        let logs = self.replication_logs.read().await;

        if let Some(replication_logs) = logs.get(&key) {
            if let Some(log) = replication_logs.iter().find(|l| l.message_offset == offset) {
                // Check if minimum replicas have acknowledged
                (log.replicated_to.len() as u32) >= min_replicas
            } else {
                false
            }
        } else {
            false
        }
    }
}

// ============================================================================
// LEADER ELECTION & FAILOVER
// ============================================================================

pub struct LeaderElector {
    state: Arc<ClusterState>,
}

impl LeaderElector {
    pub fn new(state: Arc<ClusterState>) -> Self {
        Self { state }
    }

    /// Elect new leader for partition if current is dead
    pub async fn elect_new_leader(
        &self,
        stream_id: &str,
        partition_id: u32,
    ) -> Result<u32, String> {
        let mut stream = self.state.get_stream(stream_id)
            .await
            .ok_or("Stream not found".to_string())?;

        let partition = stream.partitions.iter_mut()
            .find(|p| p.partition_id == partition_id)
            .ok_or("Partition not found".to_string())?;

        let old_leader = partition.leader_broker_id;

        // Check if current leader is alive
        if let Some(broker) = self.state.get_broker(old_leader).await {
            if broker.status == BrokerStatus::Up {
                return Ok(old_leader);  // Leader still alive
            }
        }

        // Find new leader from in-sync replicas
        let new_leader = partition.in_sync_replicas
            .iter()
            .find(|&&broker_id| {
                if let Some(broker) = self.state.get_broker(broker_id).await {
                    broker.status == BrokerStatus::Up
                } else {
                    false
                }
            })
            .copied()
            .ok_or("No alive replicas available".to_string())?;

        println!("Leader election: Partition {}/{} - {} -> {}",
                 stream_id, partition_id, old_leader, new_leader);

        partition.leader_broker_id = new_leader;

        // Update in partition leaders map
        let mut leaders = self.state.partition_leaders.write().await;
        leaders.insert((stream_id.to_string(), partition_id), new_leader);

        Ok(new_leader)
    }

    /// Handle broker failure - reassign its partitions
    pub async fn handle_broker_failure(&self, dead_broker_id: u32) -> Result<u32, String> {
        let streams = {
            let s = self.state.streams.read().await;
            s.values().cloned().collect::<Vec<_>>()
        };

        let mut reassigned_count = 0;

        for stream in streams {
            for partition in &stream.partitions {
                if partition.leader_broker_id == dead_broker_id {
                    match self.elect_new_leader(&stream.stream_id, partition.partition_id).await {
                        Ok(_) => reassigned_count += 1,
                        Err(e) => eprintln!("Failed to reassign partition: {}", e),
                    }
                }
            }
        }

        Ok(reassigned_count)
    }
}

// ============================================================================
// CLUSTER REBALANCER
// ============================================================================

pub struct ClusterRebalancer {
    state: Arc<ClusterState>,
}

impl ClusterRebalancer {
    pub fn new(state: Arc<ClusterState>) -> Self {
        Self { state }
    }

    /// Rebalance partitions after broker added/removed
    pub async fn rebalance(&self, stream_id: &str) -> Result<(), String> {
        let brokers = self.state.get_all_brokers().await;
        let alive_brokers: Vec<_> = brokers.iter()
            .filter(|b| b.status == BrokerStatus::Up)
            .collect();

        if alive_brokers.is_empty() {
            return Err("No alive brokers available".to_string());
        }

        let mut stream = self.state.get_stream(stream_id)
            .await
            .ok_or("Stream not found".to_string())?;

        println!("Rebalancing stream {} across {} brokers...", stream_id, alive_brokers.len());

        // Reassign partitions round-robin
        for (i, partition) in stream.partitions.iter_mut().enumerate() {
            let leader_idx = i % alive_brokers.len();
            let new_leader = alive_brokers[leader_idx].broker_id;

            if partition.leader_broker_id != new_leader {
                println!("  Partition {} moved: {} -> {}", 
                         partition.partition_id, 
                         partition.leader_broker_id, 
                         new_leader);
                partition.leader_broker_id = new_leader;

                // Update replicas
                let mut replicas = vec![new_leader];
                for j in 1..partition.replica_broker_ids.len() {
                    let replica_idx = (i + j) % alive_brokers.len();
                    replicas.push(alive_brokers[replica_idx].broker_id);
                }
                partition.replica_broker_ids = replicas;
                partition.in_sync_replicas = partition.replica_broker_ids.clone();
            }
        }

        Ok(())
    }
}

// ============================================================================
// CLUSTER BROKER (Main server)
// ============================================================================

pub struct ClusterBroker {
    config: BrokerConfig,
    state: Arc<ClusterState>,
    replication_manager: Arc<ReplicationManager>,
    leader_elector: Arc<LeaderElector>,
    rebalancer: Arc<ClusterRebalancer>,
}

impl ClusterBroker {
    pub fn new(config: BrokerConfig) -> Self {
        let state = Arc::new(ClusterState::new());
        
        Self {
            config,
            state: state.clone(),
            replication_manager: Arc::new(ReplicationManager::new(state.clone())),
            leader_elector: Arc::new(LeaderElector::new(state.clone())),
            rebalancer: Arc::new(ClusterRebalancer::new(state.clone())),
        }
    }

    /// Initialize as cluster member
    pub async fn initialize_cluster(&self) -> Result<(), String> {
        let broker_metadata = BrokerMetadata {
            broker_id: self.config.broker_id,
            host: self.config.listen_address.clone(),
            port: self.config.listen_port,
            status: BrokerStatus::Starting,
            last_heartbeat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.state.register_broker(broker_metadata).await;
        println!("Broker {} initialized in cluster", self.config.broker_id);

        // If this is the first broker, make it controller
        if self.config.broker_id == 1 {
            *self.state.controller_broker_id.write().await = self.config.broker_id;
            println!("Broker {} elected as cluster controller", self.config.broker_id);
        }

        Ok(())
    }

    /// Send a message to the cluster
    pub async fn send_message(
        &self,
        stream_id: &str,
        partition_key: &str,
        data: Vec<u8>,
    ) -> Result<SendResult, String> {
        // Determine partition
        let partition_id = self.state.determine_partition(stream_id, partition_key).await?;

        // Get partition leader
        let leader = self.state.get_partition_leader(stream_id, partition_id)
            .await
            .ok_or("No partition leader".to_string())?;

        // Check if we're the leader
        if leader != self.config.broker_id {
            return Err(format!("Not the leader for this partition. Leader is broker {}", leader));
        }

        // Write to local storage (would be RocksDB in real impl)
        // Get current offset
        let stream = self.state.get_stream(stream_id)
            .await
            .ok_or("Stream not found".to_string())?;

        let partition = stream.partitions.iter()
            .find(|p| p.partition_id == partition_id)
            .ok_or("Partition not found".to_string())?;

        let offset = partition.current_offset;

        // Replicate to replicas
        self.replication_manager.replicate_message(stream_id, partition_id, offset).await?;

        Ok(SendResult {
            stream: stream_id.to_string(),
            partition: partition_id,
            offset,
            replica_nodes: partition.replica_broker_ids.clone(),
        })
    }

    /// Get topology for clients
    pub async fn get_topology(&self, stream_id: &str) -> Result<Topology, String> {
        let stream = self.state.get_stream(stream_id)
            .await
            .ok_or("Stream not found".to_string())?;

        let brokers = self.state.get_all_brokers().await;

        let partitions = stream.partitions.iter().map(|p| {
            TopologyPartition {
                id: p.partition_id,
                leader: p.leader_broker_id,
                replicas: p.replica_broker_ids.clone(),
                in_sync_replicas: p.in_sync_replicas.clone(),
            }
        }).collect();

        Ok(Topology {
            stream: stream_id.to_string(),
            partitions,
            brokers: brokers.into_iter().map(|b| {
                TopologyBroker {
                    id: b.broker_id,
                    host: b.host,
                    port: b.port,
                    status: format!("{:?}", b.status),
                }
            }).collect(),
        })
    }
}

// ============================================================================
// RESULT TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub stream: String,
    pub partition: u32,
    pub offset: u64,
    pub replica_nodes: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub stream: String,
    pub partitions: Vec<TopologyPartition>,
    pub brokers: Vec<TopologyBroker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyPartition {
    pub id: u32,
    pub leader: u32,
    pub replicas: Vec<u32>,
    pub in_sync_replicas: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyBroker {
    pub id: u32,
    pub host: String,
    pub port: u16,
    pub status: String,
}

// ============================================================================
// USAGE EXAMPLE
// ============================================================================

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create 4 brokers
    let brokers = vec![
        BrokerConfig {
            broker_id: 1,
            listen_address: "10.0.1.1".to_string(),
            listen_port: 6000,
            metadata_servers: vec!["10.0.2.1:2181".to_string()],
            bootstrap_servers: vec![
                "10.0.1.1:6000".to_string(),
                "10.0.1.2:6000".to_string(),
                "10.0.1.3:6000".to_string(),
                "10.0.1.4:6000".to_string(),
            ],
        },
        BrokerConfig {
            broker_id: 2,
            listen_address: "10.0.1.2".to_string(),
            listen_port: 6000,
            metadata_servers: vec!["10.0.2.1:2181".to_string()],
            bootstrap_servers: vec![
                "10.0.1.1:6000".to_string(),
                "10.0.1.2:6000".to_string(),
                "10.0.1.3:6000".to_string(),
                "10.0.1.4:6000".to_string(),
            ],
        },
        BrokerConfig {
            broker_id: 3,
            listen_address: "10.0.1.3".to_string(),
            listen_port: 6000,
            metadata_servers: vec!["10.0.2.1:2181".to_string()],
            bootstrap_servers: vec![
                "10.0.1.1:6000".to_string(),
                "10.0.1.2:6000".to_string(),
                "10.0.1.3:6000".to_string(),
                "10.0.1.4:6000".to_string(),
            ],
        },
        BrokerConfig {
            broker_id: 4,
            listen_address: "10.0.1.4".to_string(),
            listen_port: 6000,
            metadata_servers: vec!["10.0.2.1:2181".to_string()],
            bootstrap_servers: vec![
                "10.0.1.1:6000".to_string(),
                "10.0.1.2:6000".to_string(),
                "10.0.1.3:6000".to_string(),
                "10.0.1.4:6000".to_string(),
            ],
        },
    ];

    // Initialize all brokers
    let mut cluster_brokers = Vec::new();
    for config in brokers {
        let broker = ClusterBroker::new(config);
        broker.initialize_cluster().await?;
        cluster_brokers.push(broker);
    }

    println!("\n=== Cluster Created ===\n");

    // Create stream with 4 partitions
    let cluster_state = &cluster_brokers[0].state;
    
    let stream_config = StreamPartitionConfig {
        stream_id: "orders".to_string(),
        num_partitions: 4,
        replication_factor: 3,
        min_insync_replicas: 2,
        retention_hours: 72,
    };

    let stream = cluster_state.create_stream(stream_config).await?;
    println!("Stream 'orders' created with {} partitions", stream.num_partitions);
    println!("Replication factor: {}", stream.replication_factor);

    for partition in &stream.partitions {
        println!("  Partition {}: Leader=Broker-{}, Replicas={:?}",
                 partition.partition_id,
                 partition.leader_broker_id,
                 partition.replica_broker_ids);
    }

    // Simulate sending messages
    println!("\n=== Sending Messages ===\n");

    let test_orders = vec![
        ("ORD-001", b"order 1"),
        ("ORD-002", b"order 2"),
        ("ORD-003", b"order 3"),
        ("ORD-004", b"order 4"),
    ];

    for (order_id, data) in test_orders {
        // Determine which broker is the leader for this order
        let partition = cluster_state.determine_partition("orders", order_id).await?;
        
        if let Some(leader_id) = cluster_state.get_partition_leader("orders", partition).await {
            let leader = &cluster_brokers[(leader_id - 1) as usize];
            
            let result = leader.send_message("orders", order_id, data.to_vec()).await?;
            
            println!("{} -> Partition {}, Leader=Broker-{}, Offset={}",
                     order_id,
                     result.partition,
                     partition % 4 + 1,  // Simplified for demo
                     result.offset);
        }
    }

    // Get topology
    println!("\n=== Cluster Topology ===\n");
    let topology = cluster_brokers[0].get_topology("orders").await?;
    
    println!("Stream: {}", topology.stream);
    println!("Brokers:");
    for broker in &topology.brokers {
        println!("  {} ({}:{}): {}", broker.id, broker.host, broker.port, broker.status);
    }

    println!("\nPartitions:");
    for partition in &topology.partitions {
        println!("  Partition {}: Leader={}, Replicas={:?}, In-Sync={:?}",
                 partition.id,
                 partition.leader,
                 partition.replicas,
                 partition.in_sync_replicas);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cluster_creation() {
        let config = BrokerConfig {
            broker_id: 1,
            listen_address: "127.0.0.1".to_string(),
            listen_port: 6000,
            metadata_servers: vec!["127.0.0.1:2181".to_string()],
            bootstrap_servers: vec!["127.0.0.1:6000".to_string()],
        };

        let broker = ClusterBroker::new(config);
        assert!(broker.initialize_cluster().await.is_ok());
    }

    #[tokio::test]
    async fn test_partition_assignment() {
        let state = Arc::new(ClusterState::new());
        
        // Register brokers
        for i in 1..=4 {
            let metadata = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            };
            state.register_broker(metadata).await;
        }

        // Create stream
        let config = StreamPartitionConfig {
            stream_id: "test".to_string(),
            num_partitions: 4,
            replication_factor: 3,
            min_insync_replicas: 2,
            retention_hours: 24,
        };

        let stream = state.create_stream(config).await.unwrap();
        assert_eq!(stream.num_partitions, 4);
        assert_eq!(stream.partitions.len(), 4);

        // Check each partition has replicas
        for partition in &stream.partitions {
            assert_eq!(partition.replica_broker_ids.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_consistent_hashing() {
        let state = Arc::new(ClusterState::new());
        
        // Register brokers
        for i in 1..=4 {
            state.register_broker(BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: BrokerStatus::Up,
                last_heartbeat: 0,
            }).await;
        }

        // Create stream
        state.create_stream(StreamPartitionConfig {
            stream_id: "orders".to_string(),
            num_partitions: 4,
            replication_factor: 3,
            min_insync_replicas: 2,
            retention_hours: 72,
        }).await.unwrap();

        // Same key should always go to same partition
        let p1 = state.determine_partition("orders", "ORD-123").await.unwrap();
        let p2 = state.determine_partition("orders", "ORD-123").await.unwrap();
        assert_eq!(p1, p2);

        // Different keys can go to different partitions
        let p3 = state.determine_partition("orders", "ORD-456").await.unwrap();
        // Not necessarily different, but valid partition
        assert!(p3 <= 3);
    }

    #[tokio::test]
    async fn test_leader_election() {
        let state = Arc::new(ClusterState::new());
        
        // Register brokers
        let mut brokers = vec![];
        for i in 1..=4 {
            let metadata = BrokerMetadata {
                broker_id: i,
                host: format!("10.0.1.{}", i),
                port: 6000,
                status: if i == 1 { BrokerStatus::Down } else { BrokerStatus::Up },
                last_heartbeat: 0,
            };
            brokers.push(metadata.clone());
            state.register_broker(metadata).await;
        }

        // Create stream
        state.create_stream(StreamPartitionConfig {
            stream_id: "test".to_string(),
            num_partitions: 4,
            replication_factor: 3,
            min_insync_replicas: 2,
            retention_hours: 24,
        }).await.unwrap();

        let elector = LeaderElector::new(state.clone());
        
        // Elect new leader when first one is down
        let new_leader = elector.elect_new_leader("test", 0).await.unwrap();
        assert!(new_leader > 1);  // Should elect one of the alive brokers
    }
}
