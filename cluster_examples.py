"""
FastDataBroker Clustering - Practical Implementation Examples
=============================================================

This module provides ready-to-use examples for clustering setup,
cluster operations, and monitoring.
"""

import json
import requests
import time
from typing import List, Dict, Optional
from dataclasses import dataclass, asdict
from enum import Enum


# ============================================================================
# CONFIGURATION CLASSES
# ============================================================================

class BrokerStatus(str, Enum):
    UP = "up"
    DOWN = "down"
    STARTING = "starting"
    STOPPING = "stopping"


@dataclass
class BrokerConfig:
    """Single broker configuration"""
    broker_id: int
    listen_address: str
    listen_port: int
    metadata_servers: List[str]
    bootstrap_servers: List[str]
    
    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class ReplicationConfig:
    """Replication settings"""
    replication_factor: int = 3
    min_insync_replicas: int = 2
    default_partitions: int = 12
    retention_hours: int = 168


@dataclass
class HeartbeatConfig:
    """Heartbeat and failure detection"""
    interval_ms: int = 3000
    timeout_ms: int = 10000
    max_consecutive_failures: int = 3


@dataclass
class ClusteringConfig:
    """Complete clustering configuration"""
    enabled: bool = True
    mode: str = "distributed"
    broker_id: int = 1
    listen_address: str = "0.0.0.0"
    listen_port: int = 5000
    metadata_servers: List[str] = None
    bootstrap_servers: List[str] = None
    replication: ReplicationConfig = None
    heartbeat: HeartbeatConfig = None
    
    def __post_init__(self):
        if self.metadata_servers is None:
            self.metadata_servers = ["zk1:2181", "zk2:2181", "zk3:2181"]
        if self.bootstrap_servers is None:
            self.bootstrap_servers = ["broker-1:5000", "broker-2:5000", "broker-3:5000"]
        if self.replication is None:
            self.replication = ReplicationConfig()
        if self.heartbeat is None:
            self.heartbeat = HeartbeatConfig()
    
    def to_dict(self) -> dict:
        return {
            'enabled': self.enabled,
            'mode': self.mode,
            'broker_id': self.broker_id,
            'listen_address': self.listen_address,
            'listen_port': self.listen_port,
            'metadata_servers': self.metadata_servers,
            'bootstrap_servers': self.bootstrap_servers,
            'replication': asdict(self.replication),
            'heartbeat': asdict(self.heartbeat),
        }


# ============================================================================
# CLUSTER MANAGER
# ============================================================================

class ClusterManager:
    """Manage FastDataBroker cluster operations"""
    
    def __init__(self, broker_host: str = "localhost", broker_port: int = 5000):
        self.base_url = f"http://{broker_host}:{broker_port}"
    
    def get_cluster_status(self) -> dict:
        """Get current cluster status"""
        response = requests.get(f"{self.base_url}/api/cluster/status")
        response.raise_for_status()
        return response.json()
    
    def get_brokers(self) -> List[dict]:
        """Get list of all brokers in cluster"""
        status = self.get_cluster_status()
        return status.get('brokers', [])
    
    def get_broker_by_id(self, broker_id: int) -> Optional[dict]:
        """Get specific broker metadata"""
        brokers = self.get_brokers()
        for broker in brokers:
            if broker['broker_id'] == broker_id:
                return broker
        return None
    
    def is_broker_up(self, broker_id: int) -> bool:
        """Check if broker is up"""
        broker = self.get_broker_by_id(broker_id)
        return broker is not None and broker['status'] == BrokerStatus.UP.value
    
    def get_all_brokers_up(self) -> bool:
        """Check if all brokers are up"""
        brokers = self.get_brokers()
        return all(b['status'] == BrokerStatus.UP.value for b in brokers)
    
    def get_cluster_metrics(self) -> dict:
        """Get cluster performance metrics"""
        response = requests.get(f"{self.base_url}/api/cluster/metrics")
        response.raise_for_status()
        return response.json()
    
    def get_streams(self) -> List[dict]:
        """Get all streams in cluster"""
        status = self.get_cluster_status()
        return status.get('streams', [])
    
    def create_stream(self, stream_id: str, num_partitions: int = 12) -> dict:
        """Create a new stream"""
        payload = {
            'stream_id': stream_id,
            'num_partitions': num_partitions
        }
        response = requests.post(
            f"{self.base_url}/api/cluster/streams",
            json=payload
        )
        response.raise_for_status()
        return response.json()
    
    def rebalance_cluster(self) -> dict:
        """Trigger cluster rebalancing"""
        response = requests.post(f"{self.base_url}/api/cluster/rebalance/all")
        response.raise_for_status()
        return response.json()
    
    def drain_broker(self, broker_id: int) -> dict:
        """Drain all partitions from a broker"""
        payload = {'broker_id': broker_id}
        response = requests.post(
            f"{self.base_url}/api/cluster/drain",
            json=payload
        )
        response.raise_for_status()
        return response.json()
    
    def get_rebalance_status(self) -> dict:
        """Get rebalance operation status"""
        response = requests.get(f"{self.base_url}/api/cluster/rebalance/status")
        response.raise_for_status()
        return response.json()
    
    def wait_for_rebalance(self, timeout_seconds: int = 300) -> bool:
        """Wait for rebalance to complete"""
        start = time.time()
        while time.time() - start < timeout_seconds:
            status = self.get_rebalance_status()
            if status.get('status') != 'in_progress':
                return True
            print(f"Rebalancing... {status.get('progress', '?')} complete")
            time.sleep(5)
        return False


# ============================================================================
# CLUSTER SETUP
# ============================================================================

class ClusterSetup:
    """Helper for initial cluster setup"""
    
    @staticmethod
    def generate_broker_config(
        broker_id: int,
        hostname: str,
        num_brokers: int,
        zk_servers: List[str] = None
    ) -> ClusteringConfig:
        """Generate configuration for a broker"""
        
        if zk_servers is None:
            zk_servers = ["zk1:2181", "zk2:2181", "zk3:2181"]
        
        bootstrap_servers = [
            f"broker-{i}:5000" for i in range(1, num_brokers + 1)
        ]
        
        return ClusteringConfig(
            enabled=True,
            broker_id=broker_id,
            listen_address=hostname,
            listen_port=5000,
            metadata_servers=zk_servers,
            bootstrap_servers=bootstrap_servers
        )
    
    @staticmethod
    def save_config_to_file(config: ClusteringConfig, filepath: str):
        """Save clustering configuration to file"""
        full_config = {
            'clustering': config.to_dict()
        }
        with open(filepath, 'w') as f:
            json.dump(full_config, f, indent=2)
        print(f"✓ Configuration saved to {filepath}")
    
    @staticmethod
    def generate_all_broker_configs(
        num_brokers: int = 3,
        output_dir: str = "./configs"
    ) -> List[ClusteringConfig]:
        """Generate configurations for all brokers in cluster"""
        
        import os
        os.makedirs(output_dir, exist_ok=True)
        
        configs = []
        for i in range(1, num_brokers + 1):
            config = ClusterSetup.generate_broker_config(
                broker_id=i,
                hostname=f"broker-{i}",
                num_brokers=num_brokers
            )
            configs.append(config)
            
            # Save to file
            filepath = os.path.join(output_dir, f"broker-{i}.json")
            ClusterSetup.save_config_to_file(config, filepath)
        
        return configs


# ============================================================================
# CLUSTER HEALTH MONITORING
# ============================================================================

class ClusterHealthMonitor:
    """Monitor cluster health and perform actions"""
    
    def __init__(self, cluster_manager: ClusterManager):
        self.manager = cluster_manager
    
    def check_broker_health(self, broker_id: int) -> Dict[str, any]:
        """Check individual broker health"""
        broker = self.manager.get_broker_by_id(broker_id)
        
        if not broker:
            return {'broker_id': broker_id, 'status': 'NOT_FOUND'}
        
        metrics = self.manager.get_cluster_metrics()
        broker_metrics = metrics.get('brokers', {}).get(str(broker_id), {})
        
        return {
            'broker_id': broker_id,
            'status': broker['status'],
            'uptime_seconds': broker_metrics.get('uptime_seconds', 0),
            'partitions_led': broker_metrics.get('partitions_led', 0),
            'partitions_replicated': broker_metrics.get('partitions_replicated', 0),
            'message_throughput': broker_metrics.get('message_throughput', 0),
            'replication_lag_ms': broker_metrics.get('replication_lag_ms', 0),
        }
    
    def check_cluster_health(self) -> Dict[str, any]:
        """Check overall cluster health"""
        status = self.manager.get_cluster_status()
        metrics = self.manager.get_cluster_metrics()
        
        brokers = status.get('brokers', [])
        up_brokers = sum(1 for b in brokers if b['status'] == 'up')
        total_brokers = len(brokers)
        
        cluster_metrics = metrics.get('cluster', {})
        
        return {
            'total_brokers': total_brokers,
            'up_brokers': up_brokers,
            'broker_availability': f"{100 * up_brokers / total_brokers:.1f}%",
            'total_throughput': cluster_metrics.get('total_throughput', 0),
            'replication_factor': cluster_metrics.get('replication_factor', 0),
            'min_insync_replicas': cluster_metrics.get('min_insync_replicas', 0),
            'data_loss_risk': cluster_metrics.get('data_loss_risk', 0),
        }
    
    def print_cluster_status(self):
        """Print formatted cluster status"""
        health = self.check_cluster_health()
        
        print("\n" + "="*60)
        print("CLUSTER STATUS")
        print("="*60)
        print(f"Total Brokers: {health['total_brokers']}")
        print(f"Up Brokers: {health['up_brokers']}")
        print(f"Availability: {health['broker_availability']}")
        print(f"Throughput: {health['total_throughput']:,} msg/sec")
        print(f"Replication Factor: {health['replication_factor']}")
        print(f"Min In-Sync Replicas: {health['min_insync_replicas']}")
        print(f"Data Loss Risk: {health['data_loss_risk']}%")
        print("="*60 + "\n")
    
    def print_broker_status(self, broker_id: int):
        """Print formatted broker status"""
        health = self.check_broker_health(broker_id)
        
        print(f"\nBroker {broker_id}:")
        print(f"  Status: {health.get('status')}")
        print(f"  Uptime: {health.get('uptime_seconds')} seconds")
        print(f"  Partitions Led: {health.get('partitions_led')}")
        print(f"  Partitions Replicated: {health.get('partitions_replicated')}")
        print(f"  Throughput: {health.get('message_throughput'):,} msg/sec")
        print(f"  Replication Lag: {health.get('replication_lag_ms'):.1f}ms")


# ============================================================================
# CLUSTER OPERATIONS
# ============================================================================

class ClusterOperations:
    """Handle common cluster operations"""
    
    def __init__(self, cluster_manager: ClusterManager):
        self.manager = cluster_manager
    
    def add_broker(self, new_broker_id: int) -> bool:
        """Add new broker to cluster"""
        print(f"\n→ Adding broker {new_broker_id} to cluster...")
        
        # Verify cluster is healthy before adding
        if not self.manager.get_all_brokers_up():
            print("✗ Cluster not fully healthy. Abort add broker.")
            return False
        
        print(f"✓ Cluster healthy. Starting broker {new_broker_id}...")
        print("  (Start the broker with: ./fastdatabroker --broker-id {})".format(new_broker_id))
        
        # Wait for broker to join
        time.sleep(5)
        
        # Verify broker joined
        if self.manager.is_broker_up(new_broker_id):
            print(f"✓ Broker {new_broker_id} joined cluster")
            
            # Trigger rebalance
            print("→ Rebalancing partitions...")
            self.manager.rebalance_cluster()
            
            if self.manager.wait_for_rebalance():
                print("✓ Rebalance complete")
                return True
        else:
            print(f"✗ Broker {new_broker_id} failed to join")
            return False
    
    def remove_broker(self, broker_id: int) -> bool:
        """Remove broker from cluster"""
        print(f"\n→ Removing broker {broker_id} from cluster...")
        
        # Verify broker exists
        if not self.manager.is_broker_up(broker_id):
            print(f"✗ Broker {broker_id} is not up")
            return False
        
        # Drain broker
        print(f"→ Draining broker {broker_id}...")
        self.manager.drain_broker(broker_id)
        
        if self.manager.wait_for_rebalance():
            print(f"✓ Broker {broker_id} drained")
            print(f"→ Shutting down broker {broker_id}...")
            print("  (Stop the broker with: curl -X POST http://broker-{}/api/shutdown)".format(broker_id))
            return True
        else:
            print("✗ Drain timeout")
            return False
    
    def rebalance(self) -> bool:
        """Rebalance all partitions"""
        print("\n→ Rebalancing cluster...")
        self.manager.rebalance_cluster()
        
        if self.manager.wait_for_rebalance():
            print("✓ Rebalance complete")
            return True
        else:
            print("✗ Rebalance timeout")
            return False


# ============================================================================
# EXAMPLE USAGE
# ============================================================================

def example_generate_configs():
    """Example: Generate configs for 3-broker cluster"""
    print("Generating configuration for 3-broker cluster...")
    
    configs = ClusterSetup.generate_all_broker_configs(
        num_brokers=3,
        output_dir="./cluster_configs"
    )
    
    print(f"✓ Generated {len(configs)} broker configurations")
    for config in configs:
        print(f"  - Broker {config.broker_id}: {config.listen_address}")


def example_monitor_cluster():
    """Example: Monitor cluster health"""
    manager = ClusterManager("broker-1", 5000)
    monitor = ClusterHealthMonitor(manager)
    
    # Check cluster health
    monitor.print_cluster_status()
    
    # Check individual brokers
    for broker_id in [1, 2, 3]:
        monitor.print_broker_status(broker_id)


def example_add_broker():
    """Example: Add new broker to cluster"""
    manager = ClusterManager("broker-1", 5000)
    ops = ClusterOperations(manager)
    ops.add_broker(new_broker_id=4)


def example_rebalance():
    """Example: Rebalance cluster"""
    manager = ClusterManager("broker-1", 5000)
    ops = ClusterOperations(manager)
    ops.rebalance()


if __name__ == "__main__":
    # Run examples
    print("FastDataBroker Clustering Examples\n")
    
    # Uncomment to run:
    # example_generate_configs()
    # example_monitor_cluster()
    # example_add_broker()
    # example_rebalance()
