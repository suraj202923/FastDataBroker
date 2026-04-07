# FastDataBroker Clustering

Multi-server distributed architecture for high availability and scalability.

## Cluster Architecture

### Basic 4-Node Cluster

```
┌────────────────────────────────────────────────┐
│          FastDataBroker Cluster                │
├────────────────────────────────────────────────┤
│                                                 │
│  Broker 0          Broker 1          Broker 2  │
│  (Leader-0)       (Leader-1)       (Leader-2) │
│    │ part-0         │ part-1         │ part-2  │
│    │ (rep: 1,2)     │ (rep: 2,3)     │ (rep:3,0)
│    │                │                │         │
│    └────────────────┼────────────────┘         │
│                     │                          │
│                Broker 3                        │
│              (Leader-3)                        │
│                │ part-3                        │
│              (rep: 0,1)                        │
│                                                 │
│  Replication: 3-way (1 leader + 2 replicas)   │
│  Failover: Automatic on leader failure        │
│  Consistency: Quorum write (min 2 replicas)   │
│                                                 │
└────────────────────────────────────────────────┘
```

## Broker Roles

### Leader Broker
- Primary replica for partition
- Handles all writes for that partition
- Sends replicas to follower brokers
- Coordinates quorum write confirmations
- Electable as new leader if current fails

### Follower Broker
- Secondary replica for partition
- Receives replication stream from leader
- Acknowledges writes (but doesn't serve reads)
- Can become new leader on leader failure
- Tracks in-sync status

### Observer (Optional)
- Monitors cluster state
- Provides read-only access
- Never becomes leader
- Useful for analytics/backup

## Partition Replication

### 3-Way Replication Setup

```
Partition 0: replicated to brokers [0, 1, 2]
  ├─ Broker 0: Leader (primary)
  ├─ Broker 1: In-Sync Replica (ISR)
  └─ Broker 2: In-Sync Replica (ISR)

Traffic flow:
  Producer → Leader (Broker 0)
               ├─→ Disk write (Broker 0)
               ├─→ Send to ISR 1
               ├─→ Send to ISR 2
               ├─ Wait for ACK from min_insync_replicas (2)
               └─→ Producer ACK
```

### Replication Parameters

```yaml
replication_factor: 3           # Total replicas (1 leader + 2 followers)
min_insync_replicas: 2          # Minimum replicas before ACK
replica_lag_time_ms: 10000      # Time before replica marked out-of-sync
replica_socket_receive_buffer: 65536
```

### Replication Guarantees

```
With replication_factor=3 and min_insync_replicas=2:
├─ 0 failures: Normal operation ✓
├─ 1 failure: Can still quorum write ✓
├─ 2 failures: Read-only mode (safe degradation)
└─ 3 failures: Complete loss of availability
```

## Failover Mechanism

### Broker Failure Detection

```
Timeline:
  T+0s:   Broker crashes or becomes unreachable
  T+1s:   Heartbeat timeout detected
  T+1.5s: Other brokers mark broker as DOWN
  T+2s:   Leader election triggered
  T+2.5s: New leader elected and topology updated
  T+3s:   Clients notified via topology refresh
  T+3.5s: Message flow resumes
```

### Leader Election

```
When leader Broker A fails:

1. Remaining brokers detect Broker A down
2. Election candidates: In-sync replicas of Broker A
3. Zookeeper-based election (deterministic)
4. New leader elected by majority vote
5. Followers connect to new leader
6. Message flow resumes
```

### Selection Logic

```
New leader chosen from:
  - In-sync replicas (preferred - has all messages)
  - Out-of-sync replicas (last resort - may lose messages)
  - Cannot elect broker not in replica set

Priority:
  1. ISR with lowest ID (for determinism)
  2. Any ISR (if above unavailable)
  3. Latest OOSync replica (dangerous, use only if needed)
```

## Distributed Consistency

### Offsets & Message Ordering

```
Consumer reads from partition:
  Get partition offset: 0
  Read message at offset 0: Message A
  Commit offset to consumer group: 1
  Read message at offset 1: Message B
  Commit offset to consumer group: 2
  ...

Result: Exact ordering preserved ✓
```

### Quorum Writes

```
Write flow with quorum:
  1. Producer sends to leader: "store message X"
  2. Leader writes to local storage
  3. Leader sends to all ISR replicas
  4. Replicas write and acknowledge
  5. Leader waits for min_insync_replicas ACKs
  6. Leader sends ACK to producer
  7. Producer guaranteed message on min_insync_replicas brokers
```

### Consistency Levels

#### Strong Consistency (min_insync_replicas=2, replication_factor=3)
```
Guarantee: Message on 2+ brokers before ACK
Tolerance: Lose 1 broker (keep 2)
Use case: Financial transactions, critical data
Cost: Slight latency increase
```

#### Eventual Consistency (min_insync_replicas=1, replication_factor=3)
```
Guarantee: Message on 2+ brokers eventually
Tolerance: Lose 2 brokers
Use case: Analytics, non-critical data
Cost: Faster writes, eventual consistency
```

## Cluster Coordination

### Zookeeper Integration

```
Zookeeper stores:
├─ /brokers/ids/{id}              → Broker metadata
├─ /brokers/topics/{name}         → Topic configuration
├─ /brokers/topics/{name}/partitions/{id}/state → Partition leader
├─ /consumers/{group}/offsets/{topic}/{partition} → Consumer offset
└─ /config/brokers/{id}           → Runtime broker config

Updates trigger watches:
├─ Broker join/leave → Update topology
├─ Partition leader change → Notify clients
├─ Consumer group change → Rebalance
└─ Config change → Apply to running broker
```

### Broker Discovery

```
Client startup:
  1. Connect to any bootstrap broker
  2. Request topology: "Give me all brokers"
  3. Receive: [broker1:8080, broker2:8081, broker3:8082, broker4:8083]
  4. Cache topology locally
  5. Connect to multiple brokers for load distribution
  6. Subscribe to topology change notifications
  7. Refresh topology when notified
```

## Partition Assignment Strategy

### Round-Robin Assignment

```
Brokers: [b0, b1, b2, b3]
Stream: "orders" with 16 partitions
Replication factor: 3

Assignment:
  p0:  leaders=[b0], replicas=[b1, b2]
  p1:  leaders=[b1], replicas=[b2, b3]
  p2:  leaders=[b2], replicas=[b3, b0]
  p3:  leaders=[b3], replicas=[b0, b1]
  p4:  leaders=[b0], replicas=[b1, b2]
  p5:  leaders=[b1], replicas=[b2, b3]
  ...

Result: Balanced leadership + replicas across brokers
```

### Load Distribution

```
Final distribution:
  Broker 0: 4 leaders, 8 replicas
  Broker 1: 4 leaders, 8 replicas
  Broker 2: 4 leaders, 8 replicas
  Broker 3: 4 leaders, 8 replicas

Traffic distribution: Equal across all brokers ✓
```

## Scaling the Cluster

### Adding a Broker

```
Before:
  ┌─────────────────────────────┐
  │  Broker 0  Broker 1  Broker 2
  │    (75%)    (75%)    (75%)
  └─────────────────────────────┘

Add Broker 3:
  1. Start Broker 3
  2. Trigger rebalance
  3. Redistribute partitions

After:
  ┌─────────────────────────────┐
  │ Broker 0  Broker 1  Broker 2  Broker 3
  │   (56%)    (56%)    (56%)    (56%)
  └─────────────────────────────┘
```

### Removing a Broker

```
Before removing Broker 2:
  1. Drain partitions from Broker 2
  2. Reassign to other brokers
  3. Wait for replicas to catch up
  4. Verify all data copied
  5. Mark Broker 2 as offline
  6. Shut down Broker 2
```

## Multi-Region Clustering

### Geo-Replication

```
Region A (Primary):
  ┌──────────────────┐
  │ Broker A1, A2, A3 │
  └──────────────────┘
         │
    [WAN Link]
         │
Region B (Secondary):
  ┌──────────────────┐
  │ Broker B1, B2, B3 │
  └──────────────────┘

Replication:
  - Write locally to Region A (fast, ~2ms)
  - Async replicate across regions (100-500ms)
  - Both regions can serve reads
```

### Active-Active Setup

```
Both regions receive writes:
  Client → Region A → Write + replicate to Region B
  Client → Region B → Write + replicate to Region A

Conflict resolution:
  - Vector clocks for causality detection
  - Application-specific conflict resolution
  - Guaranteed eventual consistency
```

## Monitoring & Health

### Health Checks

```
Broker health metrics:
  ├─ Heartbeat status: HEALTHY/UNHEALTHY
  ├─ Disk usage: % full
  ├─ Memory usage: % available
  ├─ Active connections: count
  ├─ Message throughput: msg/sec
  ├─ Replication lag: milliseconds
  └─ Leader count: number of partitions leading
```

### Cluster Metrics

```
prometheus_metrics = {
  "broker.uptime": seconds,
  "broker.cpu": percent,
  "broker.memory": percent,
  "broker.disk": percent,
  "partition.lag": milliseconds,
  "replication.ack_rate": percent,
  "cluster.leader_elections": count,
  "cluster.unbalanced_partitions": count,
}
```

## Troubleshooting

### Common Issues

#### Issue 1: Slow Replication
```
Symptoms: High replication lag, messages not syncing
Causes:
  - Network congestion
  - Slow disk I/O
  - Too many partitions per broker
  - Replica catching up from far behind

Solutions:
  1. Check network: ping across brokers
  2. Monitor disk latency: iostat
  3. Reduce partitions per broker
  4. Increase replica socket buffer size
```

#### Issue 2: Leader Stuck
```
Symptoms: Partition has no leader, messages blocked
Causes:
  - All replicas down
  - Broker process crashed
  - Network partition

Solutions:
  1. Check broker status: are all replicas down?
  2. Verify network connectivity
  3. Restart brokers if crashed
  4. Use emergency leader election if necessary
```

#### Issue 3: Unbalanced Partitions
```
Symptoms: Some brokers hot (high load), others cold
Causes:
  - Uneven partition distribution
  - Skewed partition size
  - Consumer group rebalancing issue

Solutions:
  1. Rebalance partitions across brokers
  2. Increase partition count
  3. Verify consumer group assignment
  4. Check partition sizes
```

## Best Practices

### For Production

1. **Always use 3-way replication** for critical data
2. **Set min_insync_replicas=2** for consistency
3. **Monitor replication lag** - alert if >5 seconds
4. **Use at least 3 brokers** for high availability
5. **Separate read/write workloads** if possible
6. **Regular backup** of critical data
7. **Test failover scenarios** before production
8. **Use Zookeeper quorum** of 3+ nodes
9. **Network: <10ms latency** between brokers
10. **Disk: Use SSD** for performance

### Configuration Tuning

```yaml
# For latency-sensitive workloads
broker:
  batch_size: 10
  batch_timeout_ms: 10
  compression: none

# For throughput-sensitive workloads
broker:
  batch_size: 1000
  batch_timeout_ms: 100
  compression: snappy

# For cost-conscious deployments
broker:
  replication_factor: 2
  min_insync_replicas: 1
  retention_ms: 86400000  # 1 day instead of 3
```

---

**Last Updated**: Phase 7 - Complete clustering validation
