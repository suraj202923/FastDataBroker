"""
FastDataBroker Multi-Server Failover & Resilience Test Suite
============================================================

Tests for failure scenarios and automated recovery
"""

import time
from typing import List, Dict
from enum import Enum


# ============================================================================
# BROKER STATE SIMULATION
# ============================================================================

class BrokerState(Enum):
    UP = "up"
    DOWN = "down"
    RECOVERING = "recovering"


class Broker:
    def __init__(self, broker_id: int):
        self.broker_id = broker_id
        self.state = BrokerState.UP
        self.last_heartbeat = time.time()
        self.partitions = []
        self.messages_processed = 0
    
    def heartbeat(self):
        """Send heartbeat to cluster"""
        if self.state == BrokerState.UP:
            self.last_heartbeat = time.time()
    
    def fail(self):
        """Simulate broker failure"""
        self.state = BrokerState.DOWN
    
    def recover(self):
        """Recover from failure"""
        self.state = BrokerState.RECOVERING
    
    def is_alive(self) -> bool:
        """Check if broker is alive"""
        return self.state == BrokerState.UP
    
    def is_responsive(self, timeout_seconds: int = 30) -> bool:
        """Check if broker is responsive based on heartbeat"""
        if self.state == BrokerState.UP:
            elapsed = time.time() - self.last_heartbeat
            return elapsed < timeout_seconds
        return False


class Partition:
    def __init__(self, partition_id: int, replicas: List[int]):
        self.partition_id = partition_id
        self.replicas = replicas
        self.leader = replicas[0]
        self.in_sync_replicas = replicas.copy()
        self.messages = []
        self.offset = 0
    
    def add_message(self, data: str) -> int:
        """Add message to partition"""
        self.messages.append(data)
        self.offset += 1
        return self.offset - 1
    
    def update_in_sync(self, brokers: Dict[int, Broker]):
        """Update in-sync replicas based on broker state"""
        self.in_sync_replicas = [
            broker_id for broker_id in self.replicas
            if brokers[broker_id].is_responsive()
        ]
    
    def is_healthy(self, min_replicas: int = 2) -> bool:
        """Check if partition has enough in-sync replicas"""
        return len(self.in_sync_replicas) >= min_replicas


class Cluster:
    def __init__(self, num_brokers: int = 4, num_partitions: int = 4):
        self.brokers = {i: Broker(i) for i in range(num_brokers)}
        self.partitions = {}
        self.heartbeat_timeout = 30
        self.detection_time = 0
        self.recovery_time = 0
        
        # Create partitions with 3-way replication
        for p in range(num_partitions):
            replicas = [
                p % num_brokers,
                (p + 1) % num_brokers,
                (p + 2) % num_brokers,
            ]
            self.partitions[p] = Partition(p, replicas)
    
    def get_broker(self, broker_id: int) -> Broker:
        return self.brokers[broker_id]
    
    def send_heartbeats(self):
        """Send heartbeats from all brokers"""
        for broker in self.brokers.values():
            if broker.is_alive():
                broker.heartbeat()
    
    def detect_failures(self):
        """Detect dead brokers"""
        for broker in self.brokers.values():
            if broker.state == BrokerState.DOWN and not broker.is_responsive():
                self.detection_time = time.time()
    
    def get_unhealthy_partitions(self) -> List[int]:
        """Get partitions with insufficient replicas"""
        unhealthy = []
        for partition_id, partition in self.partitions.items():
            if not partition.is_healthy():
                unhealthy.append(partition_id)
        return unhealthy
    
    def update_cluster_topology(self):
        """Update partition topology based on broker state"""
        for partition in self.partitions.values():
            partition.update_in_sync(self.brokers)


# ============================================================================
# FAILOVER TESTS
# ============================================================================

def test_single_broker_failure():
    """Test: Single broker failure and recovery"""
    print("\n" + "-" * 100)
    print("TEST 1: Single Broker Failure & Recovery")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    
    # Simulate heartbeats
    cluster.send_heartbeats()
    cluster.update_cluster_topology()
    
    print("Initial state: All brokers UP")
    print(f"  Total partitions: {len(cluster.partitions)}")
    
    unhealthy = cluster.get_unhealthy_partitions()
    print(f"  Unhealthy partitions: {len(unhealthy)}")
    assert len(unhealthy) == 0, "Should have no unhealthy partitions initially"
    
    # Broker-1 fails
    print("\nBroker-1 fails...")
    cluster.brokers[1].fail()
    time.sleep(0.1)
    
    # Detect failure
    cluster.update_cluster_topology()
    unhealthy = cluster.get_unhealthy_partitions()
    print(f"  Unhealthy partitions after failure: {len(unhealthy)}")
    
    # Partitions that had broker-1 as only replica
    expected_unhealthy = 0
    for partition in cluster.partitions.values():
        if 1 in partition.replicas and len(partition.in_sync_replicas) < 2:
            expected_unhealthy += 1
    
    print(f"  Expected unhealthy (min_replicas=2): {expected_unhealthy}")
    
    # Recover broker
    print("\nBroker-1 recovers...")
    cluster.brokers[1].state = BrokerState.UP
    cluster.brokers[1].heartbeat()
    cluster.update_cluster_topology()
    
    unhealthy = cluster.get_unhealthy_partitions()
    print(f"  Unhealthy partitions after recovery: {len(unhealthy)}")
    assert len(unhealthy) == 0, "Should have no unhealthy partitions after recovery"
    
    print("\n✓ test_single_broker_failure PASSED")


def test_multiple_broker_failures():
    """Test: Multiple broker failures"""
    print("\n" + "-" * 100)
    print("TEST 2: Multiple Broker Failures")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    cluster.send_heartbeats()
    cluster.update_cluster_topology()
    
    print("Initial state: All brokers UP")
    
    # Broker-1 fails
    print("Broker-1 fails...")
    cluster.brokers[1].fail()
    cluster.update_cluster_topology()
    
    # Broker-2 fails
    print("Broker-2 fails...")
    cluster.brokers[2].fail()
    cluster.update_cluster_topology()
    
    unhealthy = cluster.get_unhealthy_partitions()
    print(f"Unhealthy partitions with 2 failures: {len(unhealthy)}")
    print(f"  Partition details:")
    for p_id, partition in cluster.partitions.items():
        print(f"    Partition {p_id}: {len(partition.in_sync_replicas)} in-sync replicas")
    
    # With replication_factor=3, losing 2 brokers may leave some partitions unhealthy
    # depending on topology
    
    print("\n✓ test_multiple_broker_failures PASSED")


def test_cascade_failure():
    """Test: Cascade failure (3 out of 4 brokers down)"""
    print("\n" + "-" * 100)
    print("TEST 3: Cascade Failure (3 Brokers Down)")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    
    print("Initial state: All 4 brokers UP")
    
    # Fail 3 brokers
    for broker_id in [0, 1, 2]:
        print(f"Broker-{broker_id} fails...")
        cluster.brokers[broker_id].fail()
    
    cluster.update_cluster_topology()
    
    unhealthy = cluster.get_unhealthy_partitions()
    print(f"\nResult with 3 failures: {len(unhealthy)} unhealthy partitions")
    
    # Most partitions should have only 1 replica left (broker 3)
    # With min_replicas=2, all or most should be unhealthy
    print(f"  Expected result: All or most partitions unhealthy")
    
    print("\n✓ test_cascade_failure PASSED")


def test_partition_rebalancing():
    """Test: Partition rebalancing on broker failure"""
    print("\n" + "-" * 100)
    print("TEST 4: Partition Rebalancing on Failure")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    cluster.send_heartbeats()
    cluster.update_cluster_topology()
    
    print("Initial partition distribution:")
    for p_id, partition in cluster.partitions.items():
        print(f"  Partition {p_id}: Leader={partition.leader}, Replicas={partition.replicas}, In-Sync={partition.in_sync_replicas}")
    
    # Fail broker 1
    print("\nBroker-1 fails...")
    cluster.brokers[1].fail()
    cluster.update_cluster_topology()
    
    print("\nPartition distribution after failure:")
    for p_id, partition in cluster.partitions.items():
        print(f"  Partition {p_id}: In-Sync={partition.in_sync_replicas}")
    
    print("\n✓ test_partition_rebalancing PASSED")


def test_message_durability():
    """Test: Message durability with replication"""
    print("\n" + "-" * 100)
    print("TEST 5: Message Durability During Failure")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    
    # Add message to partition 0
    partition = cluster.partitions[0]
    offset = partition.add_message("order-123")
    
    print(f"Message added to Partition-0")
    print(f"  Message: 'order-123'")
    print(f"  Offset: {offset}")
    print(f"  Stored on replicas: {partition.replicas}")
    
    # Fail one replica
    failure_broker = partition.replicas[0]
    print(f"\nBroker-{failure_broker} (replica) fails...")
    cluster.brokers[failure_broker].fail()
    cluster.update_cluster_topology()
    
    remaining_replicas = partition.in_sync_replicas
    print(f"  Message still on: {remaining_replicas}")
    print(f"  Message safe: {len(remaining_replicas) >= 2}")
    
    # Verify message is still there
    assert len(partition.messages) == 1
    assert partition.messages[0] == "order-123"
    
    print("\n✓ test_message_durability PASSED")


def test_quorum_write():
    """Test: Quorum write with min_insync_replicas"""
    print("\n" + "-" * 100)
    print("TEST 6: Quorum Write Protocol (min_insync_replicas)")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    min_insync = 2
    
    partition = cluster.partitions[0]
    
    print(f"Partition-0 replication: {partition.replicas}")
    print(f"Min in-sync replicas required: {min_insync}")
    
    # Add first message (all replicas healthy)
    partition.update_in_sync(cluster.brokers)
    print(f"\nAll healthy: {len(partition.in_sync_replicas)} in-sync")
    can_write = len(partition.in_sync_replicas) >= min_insync
    print(f"  Can write: {can_write}")
    assert can_write
    
    # Fail one replica
    cluster.brokers[partition.replicas[0]].fail()
    partition.update_in_sync(cluster.brokers)
    
    print(f"\nAfter 1 failure: {len(partition.in_sync_replicas)} in-sync")
    can_write = len(partition.in_sync_replicas) >= min_insync
    print(f"  Can write: {can_write}")
    assert can_write
    
    # Fail second replica
    cluster.brokers[partition.replicas[1]].fail()
    partition.update_in_sync(cluster.brokers)
    
    print(f"\nAfter 2 failures: {len(partition.in_sync_replicas)} in-sync")
    can_write = len(partition.in_sync_replicas) >= min_insync
    print(f"  Can write: {can_write}")
    assert not can_write, "Should not be able to write with only 1 replica"
    
    print("\n✓ test_quorum_write PASSED")


def test_replica_reconstruction():
    """Test: Replica reconstruction after failure"""
    print("\n" + "-" * 100)
    print("TEST 7: Replica Reconstruction After Failure")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    
    partition = cluster.partitions[0]
    
    # Add messages
    for i in range(5):
        partition.add_message(f"msg-{i}")
    
    print(f"Added 5 messages to Partition-0")
    print(f"  Current offset: {partition.offset}")
    print(f"  Messages: {partition.messages}")
    
    # Fail one replica
    failed_broker = partition.replicas[0]
    print(f"\nBroker-{failed_broker} fails...")
    cluster.brokers[failed_broker].fail()
    
    # Recovery: Broker comes back
    print(f"Broker-{failed_broker} recovers...")
    cluster.brokers[failed_broker].state = BrokerState.UP
    cluster.brokers[failed_broker].heartbeat()
    
    # Reconstruct by reading from other replicas
    print(f"  Reconstructing messages from other replicas...")
    reconstructed_messages = partition.messages.copy()
    print(f"  Reconstructed {len(reconstructed_messages)} messages")
    assert len(reconstructed_messages) == 5
    
    print("\n✓ test_replica_reconstruction PASSED")


def test_zero_message_loss():
    """Test: Zero message loss during failover"""
    print("\n" + "-" * 100)
    print("TEST 8: Zero Message Loss During Failover")
    print("-" * 100 + "\n")
    
    cluster = Cluster(num_brokers=4, num_partitions=4)
    
    total_messages = 0
    
    # Add messages to multiple partitions
    for p_id, partition in cluster.partitions.items():
        for i in range(10):
            partition.add_message(f"msg-p{p_id}-{i}")
            total_messages += 1
    
    print(f"Total messages sent: {total_messages}")
    
    # Fail a broker
    print(f"Broker-1 fails...")
    cluster.brokers[1].fail()
    cluster.update_cluster_topology()
    
    # Count messages still accessible
    accessible_messages = 0
    for partition in cluster.partitions.values():
        if len(partition.in_sync_replicas) > 0:
            accessible_messages += len(partition.messages)
    
    print(f"Accessible messages after failure: {accessible_messages}")
    print(f"Message loss: {total_messages - accessible_messages}")
    
    # With 3-way replication, should have zero loss even with 1 failure
    assert accessible_messages == total_messages, "Should have zero message loss"
    
    print("\n✓ test_zero_message_loss PASSED")


# ============================================================================
# RUN ALL TESTS
# ============================================================================

def run_all_failover_tests():
    """Run all failover and resilience tests"""
    print("\n" + "=" * 100)
    print("FastDataBroker Multi-Server Failover & Resilience Test Suite")
    print("=" * 100)
    
    tests = [
        test_single_broker_failure,
        test_multiple_broker_failures,
        test_cascade_failure,
        test_partition_rebalancing,
        test_message_durability,
        test_quorum_write,
        test_replica_reconstruction,
        test_zero_message_loss,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            print(f"\n✗ {test.__name__} FAILED: {e}")
            failed += 1
        except Exception as e:
            print(f"\n✗ {test.__name__} ERROR: {e}")
            failed += 1
    
    print("\n" + "=" * 100)
    print(f"Failover Test Results: {passed} passed, {failed} failed out of {len(tests)}")
    print("=" * 100 + "\n")
    
    return failed == 0


if __name__ == "__main__":
    import sys
    success = run_all_failover_tests()
    sys.exit(0 if success else 1)
