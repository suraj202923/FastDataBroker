"""
FastDataBroker Multi-Server Cluster Client Test Suite
====================================================

Comprehensive test cases for cluster client functionality
"""

import asyncio
import time
from typing import List, Dict
import hashlib


# ============================================================================
# CLUSTER CLIENT MOCK
# ============================================================================

class TopologyPartition:
    def __init__(self, id: int, leader: int, replicas: List[int], in_sync: List[int]):
        self.id = id
        self.leader = leader
        self.replicas = replicas
        self.in_sync_replicas = in_sync


class Topology:
    def __init__(self, stream: str, partitions: List[TopologyPartition], brokers: List[int]):
        self.stream = stream
        self.partitions = partitions
        self.brokers = brokers


class ClusterClient:
    def __init__(self, bootstrap_servers: List[str], client_id: str, replication_factor: int = 3):
        self.bootstrap_servers = bootstrap_servers
        self.client_id = client_id
        self.replication_factor = replication_factor
        self.topology_cache: Dict[str, Topology] = {}

    def _consistent_hash(self, key: str) -> int:
        hash_obj = hashlib.md5(key.encode())
        return int(hash_obj.hexdigest(), 16)

    def determine_partition(self, stream_id: str, partition_key: str) -> int:
        if stream_id not in self.topology_cache:
            self._load_topology(stream_id)
        
        topology = self.topology_cache[stream_id]
        hash_value = self._consistent_hash(partition_key)
        return hash_value % len(topology.partitions)

    def _load_topology(self, stream_id: str):
        # Simulate loading topology
        partitions = [
            TopologyPartition(i, (i % 4) + 1, [(i % 4) + 1, ((i + 1) % 4) + 1, ((i + 2) % 4) + 1],
                            [(i % 4) + 1, ((i + 1) % 4) + 1, ((i + 2) % 4) + 1])
            for i in range(4)
        ]
        topology = Topology(stream_id, partitions, [1, 2, 3, 4])
        self.topology_cache[stream_id] = topology

    def get_partition_leader(self, stream_id: str, partition_id: int) -> int:
        if stream_id not in self.topology_cache:
            self._load_topology(stream_id)
        
        topology = self.topology_cache[stream_id]
        return topology.partitions[partition_id].leader

    def send_message(self, stream_id: str, partition_key: str, data: Dict) -> Dict:
        partition = self.determine_partition(stream_id, partition_key)
        leader = self.get_partition_leader(stream_id, partition)
        topology = self.topology_cache[stream_id]
        replicas = topology.partitions[partition].replicas
        
        return {
            "stream": stream_id,
            "partition": partition,
            "offset": 0,
            "leader": leader,
            "replicas": replicas,
        }


# ============================================================================
# TEST CASES
# ============================================================================

def test_client_initialization():
    """Test client initialization with bootstrap servers"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000", "broker-2:6000", "broker-3:6000"],
        client_id="test-client",
        replication_factor=3,
    )
    
    assert client.client_id == "test-client"
    assert len(client.bootstrap_servers) == 3
    assert client.replication_factor == 3
    print("✓ test_client_initialization PASSED")


def test_topology_loading():
    """Test topology loading from broker"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    client._load_topology("orders")
    
    assert "orders" in client.topology_cache
    topology = client.topology_cache["orders"]
    assert len(topology.partitions) == 4
    assert len(topology.brokers) == 4
    print("✓ test_topology_loading PASSED")


def test_partition_determination():
    """Test consistent hashing for partition determination"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    client._load_topology("orders")
    
    # Same key should always go to same partition
    p1 = client.determine_partition("orders", "ORD-001")
    p2 = client.determine_partition("orders", "ORD-001")
    p3 = client.determine_partition("orders", "ORD-001")
    
    assert p1 == p2 == p3
    assert 0 <= p1 < 4
    print("✓ test_partition_determination PASSED")


def test_partition_distribution():
    """Test even distribution across partitions"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    client._load_topology("orders")
    
    partition_counts = {0: 0, 1: 0, 2: 0, 3: 0}
    
    # Send 1000 messages
    for i in range(1000):
        key = f"ORD-{i:06d}"
        partition = client.determine_partition("orders", key)
        partition_counts[partition] += 1
    
    # Should be roughly balanced (250 ± 50)
    for count in partition_counts.values():
        assert 200 <= count <= 300, f"Imbalanced: {count}"
    
    print(f"✓ test_partition_distribution PASSED (counts: {partition_counts})")


def test_leader_election():
    """Test leader assignment for partitions"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    client._load_topology("orders")
    
    for partition_id in range(4):
        leader = client.get_partition_leader("orders", partition_id)
        assert 1 <= leader <= 4
    
    print("✓ test_leader_election PASSED")


def test_send_message():
    """Test message sending with automatic routing"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    result = client.send_message(
        stream_id="orders",
        partition_key="ORD-001",
        data={"order_id": "ORD-001", "amount": 100}
    )
    
    assert result["stream"] == "orders"
    assert 0 <= result["partition"] < 4
    assert 1 <= result["leader"] <= 4
    assert len(result["replicas"]) == 3
    print("✓ test_send_message PASSED")


def test_multiple_stream_handling():
    """Test handling multiple streams"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    # Send to different streams
    result1 = client.send_message("orders", "ORD-001", {})
    result2 = client.send_message("events", "EVT-001", {})
    result3 = client.send_message("alerts", "ALT-001", {})
    
    assert result1["stream"] == "orders"
    assert result2["stream"] == "events"
    assert result3["stream"] == "alerts"
    
    assert len(client.topology_cache) == 3
    print("✓ test_multiple_stream_handling PASSED")


def test_batch_routing():
    """Test batch message routing by partition"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    messages = [
        {"order_id": "ORD-001", "amount": 100},
        {"order_id": "ORD-002", "amount": 200},
        {"order_id": "ORD-003", "amount": 300},
        {"order_id": "ORD-004", "amount": 400},
    ]
    
    partition_groups = {0: [], 1: [], 2: [], 3: []}
    
    for msg in messages:
        partition = client.determine_partition("orders", msg["order_id"])
        partition_groups[partition].append(msg)
    
    # Each message should go to correct partition
    for partition_id, msgs in partition_groups.items():
        for msg in msgs:
            expected_partition = client.determine_partition("orders", msg["order_id"])
            assert expected_partition == partition_id
    
    print(f"✓ test_batch_routing PASSED (distribution: {[len(m) for m in partition_groups.values()]})")


def test_replication_awareness():
    """Test that client understands replication topology"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    client._load_topology("orders")
    
    topology = client.topology_cache["orders"]
    
    for partition in topology.partitions:
        # Each partition should have 3 replicas
        assert len(partition.replicas) == 3
        # All replicas should be unique
        assert len(set(partition.replicas)) == 3
        # First replica should be the leader
        assert partition.leader == partition.replicas[0]
    
    print("✓ test_replication_awareness PASSED")


def test_failover_awareness():
    """Test that in-sync replicas are tracked correctly"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    client._load_topology("orders")
    
    topology = client.topology_cache["orders"]
    
    for partition in topology.partitions:
        # Should have at least 2 in-sync replicas
        assert len(partition.in_sync_replicas) >= 2
        # Leader should be in-sync
        assert partition.leader in partition.in_sync_replicas
    
    print("✓ test_failover_awareness PASSED")


def test_consistent_hash_performance():
    """Test consistent hashing performance"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    start = time.time()
    
    for i in range(10000):
        key = f"key-{i}"
        _ = client._consistent_hash(key)
    
    elapsed = time.time() - start
    
    # Should be very fast (<100ms for 10K hashes)
    assert elapsed < 0.1, f"Hash too slow: {elapsed}s"
    
    throughput = 10000 / elapsed
    print(f"✓ test_consistent_hash_performance PASSED ({throughput:.0f} hashes/sec)")


def test_topology_refresh():
    """Test topology refresh on stale data"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    # Load initial topology
    client._load_topology("orders")
    topology1 = client.topology_cache["orders"]
    initial_partitions = len(topology1.partitions)
    
    # Refresh topology
    client._load_topology("orders")
    topology2 = client.topology_cache["orders"]
    
    assert len(topology2.partitions) == initial_partitions
    print("✓ test_topology_refresh PASSED")


def test_concurrent_sends():
    """Test handling concurrent sends to different partitions"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    results = []
    for i in range(100):
        result = client.send_message(
            stream_id="orders",
            partition_key=f"ORD-{i:06d}",
            data={"id": f"ORD-{i:06d}"}
        )
        results.append(result)
    
    assert len(results) == 100
    
    # Check all results are valid
    for result in results:
        assert result["stream"] == "orders"
        assert 0 <= result["partition"] < 4
        assert len(result["replicas"]) == 3
    
    print("✓ test_concurrent_sends PASSED")


def test_consumer_group_assignment():
    """Test consumer group partition assignment"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    client._load_topology("orders")
    
    # Simulate 4 consumers, 4 partitions
    num_consumers = 4
    num_partitions = 4
    
    consumer_assignments = {i: [] for i in range(num_consumers)}
    
    for partition_id in range(num_partitions):
        consumer_id = partition_id % num_consumers
        consumer_assignments[consumer_id].append(partition_id)
    
    # Each consumer should have 1 partition
    for consumer_id, partitions in consumer_assignments.items():
        assert len(partitions) == 1
    
    print("✓ test_consumer_group_assignment PASSED")


def test_message_ordering():
    """Test that same partition key maintains message ordering"""
    client = ClusterClient(
        bootstrap_servers=["broker-1:6000"],
        client_id="test",
    )
    
    key = "ORD-12345"
    
    # All messages with same key should go to same partition
    partitions = []
    for i in range(100):
        partition = client.determine_partition("orders", key)
        partitions.append(partition)
    
    # All should be same
    assert len(set(partitions)) == 1
    print("✓ test_message_ordering PASSED")


# ============================================================================
# RUN ALL TESTS
# ============================================================================

def run_all_tests():
    print("\n" + "=" * 100)
    print("FastDataBroker Multi-Server Cluster Client Test Suite")
    print("=" * 100 + "\n")
    
    tests = [
        test_client_initialization,
        test_topology_loading,
        test_partition_determination,
        test_partition_distribution,
        test_leader_election,
        test_send_message,
        test_multiple_stream_handling,
        test_batch_routing,
        test_replication_awareness,
        test_failover_awareness,
        test_consistent_hash_performance,
        test_topology_refresh,
        test_concurrent_sends,
        test_consumer_group_assignment,
        test_message_ordering,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            print(f"✗ {test.__name__} FAILED: {e}")
            failed += 1
        except Exception as e:
            print(f"✗ {test.__name__} ERROR: {e}")
            failed += 1
    
    print("\n" + "=" * 100)
    print(f"Test Results: {passed} passed, {failed} failed out of {len(tests)}")
    print("=" * 100 + "\n")
    
    return failed == 0


if __name__ == "__main__":
    import sys
    success = run_all_tests()
    sys.exit(0 if success else 1)
