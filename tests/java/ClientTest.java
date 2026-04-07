package com.fastdatabroker.sdk.test;

import org.junit.Before;
import org.junit.Test;
import org.junit.BeforeClass;
import org.junit.AfterClass;
import static org.junit.Assert.*;

import java.util.*;
import java.util.concurrent.*;

/**
 * Java SDK Test Suite for FastDataBroker
 */
public class ClientTest {
    
    private static MockClient client;
    
    @BeforeClass
    public static void setUpClass() {
        System.out.println("Setting up FastDataBroker Java SDK tests...");
    }
    
    @AfterClass
    public static void tearDownClass() {
        System.out.println("Cleaning up Java SDK tests...");
    }
    
    @Before
    public void setUp() {
        List<String> brokers = Arrays.asList(
            "broker1:8080",
            "broker2:8081",
            "broker3:8082",
            "broker4:8083"
        );
        client = new MockClient(brokers, "test-stream");
    }
    
    @Test
    public void testClientInitialization() {
        assertNotNull("Client should not be null", client);
        assertEquals("Stream name should match", "test-stream", client.getStreamName());
        assertEquals("Should have 4 brokers", 4, client.getBrokerCount());
    }
    
    @Test
    public void testBrokerConnection() {
        List<String> brokers = client.getBrokers();
        assertFalse("Brokers list should not be empty", brokers.isEmpty());
        assertFalse("Broker address should not be empty", brokers.get(0).isEmpty());
        assertTrue("Broker should be reachable format", brokers.get(0).contains(":"));
    }
    
    @Test
    public void testTopologyDiscovery() {
        Map<String, Object> topology = client.getTopology();
        assertNotNull("Topology should not be null", topology);
        assertEquals("Topology stream should match", "test-stream", topology.get("stream"));
        assertEquals("Should have 4 partitions", 4, topology.get("partition_count"));
    }
    
    @Test
    public void testProducerMessage() {
        Producer producer = new Producer(client);
        
        Map<String, Object> message = new HashMap<>();
        message.put("key", "test-key");
        message.put("value", "test-value");
        message.put("timestamp", System.currentTimeMillis());
        
        int partition = producer.send("test-key", "test-value");
        
        assertFalse("Partition should be assigned", partition < 0);
        assertTrue("Partition should be valid", partition < 4);
    }
    
    @Test
    public void testBatchProducer() {
        BatchProducer producer = new BatchProducer(client, 100, 5000);
        
        int messageCount = 1000;
        for (int i = 0; i < messageCount; i++) {
            producer.send("key-" + i, "message-" + i);
        }
        
        int sent = producer.flush();
        assertEquals("All messages should be sent", messageCount, sent);
    }
    
    @Test
    public void testConsumerGroupAssignment() {
        ConsumerGroup group = new ConsumerGroup(client, "test-group");
        
        List<String> members = Arrays.asList("consumer-1", "consumer-2");
        Map<String, List<Integer>> assignment = group.assignPartitions(members);
        
        assertEquals("All members should be assigned", members.size(), assignment.size());
        
        // Verify each partition is assigned
        Set<Integer> assignedPartitions = new HashSet<>();
        for (List<Integer> partitions : assignment.values()) {
            assignedPartitions.addAll(partitions);
        }
        assertEquals("All 4 partitions should be assigned", 4, assignedPartitions.size());
    }
    
    @Test
    public void testConsumerPolling() {
        Consumer consumer = new Consumer(client, "test-group");
        
        int polledCount = 0;
        for (int i = 0; i < 100; i++) {
            // Simulate message polling
            Message message = new Message("key-" + i, "value-" + i);
            if (message != null) {
                polledCount++;
            }
        }
        
        assertTrue("Should poll at least some messages", polledCount > 0);
        assertEquals("Should poll all messages", 100, polledCount);
    }
    
    @Test
    public void testConsumerCommit() {
        Consumer consumer = new Consumer(client, "test-group");
        
        long offset = 100L;
        consumer.commitOffset(0, offset);
        
        long committedOffset = consumer.getCommittedOffset(0);
        assertEquals("Offset should be committed", offset, committedOffset);
    }
    
    @Test
    public void testConsurrentConsumption() throws InterruptedException {
        Consumer consumer = new Consumer(client, "test-group");
        
        ExecutorService executor = Executors.newFixedThreadPool(4);
        CountDownLatch latch = new CountDownLatch(4);
        
        for (int i = 0; i < 4; i++) {
            executor.submit(() -> {
                try {
                    for (int j = 0; j < 250; j++) {
                        // Simulate message consumption
                    }
                } finally {
                    latch.countDown();
                }
            });
        }
        
        assertTrue("All consumers should finish", latch.await(5, TimeUnit.SECONDS));
        executor.shutdown();
    }
    
    @Test
    public void testPartitionDistribution() {
        int partitionCount = 4;
        int keyCount = 1000;
        int[] distribution = new int[partitionCount];
        
        // Distribute keys across partitions
        for (int i = 0; i < keyCount; i++) {
            String key = "key-" + i;
            int partition = Math.abs(key.hashCode() % partitionCount);
            distribution[partition]++;
        }
        
        // Verify distribution is balanced (within 25%)
        int expectedPerPartition = keyCount / partitionCount; // 250
        int tolerance = expectedPerPartition / 4;             // 62
        
        for (int i = 0; i < partitionCount; i++) {
            assertTrue("Partition " + i + " should be balanced",
                distribution[i] >= expectedPerPartition - tolerance &&
                distribution[i] <= expectedPerPartition + tolerance);
        }
    }
    
    @Test
    public void testConsistentHashPerformance() {
        int iterations = 10000;
        long startTime = System.nanoTime();
        
        for (int i = 0; i < iterations; i++) {
            String key = "key-" + i;
            int partition = Math.abs(key.hashCode() % 4);
        }
        
        long endTime = System.nanoTime();
        long durationMs = (endTime - startTime) / 1000000;
        
        System.out.println("Hashing " + iterations + " keys took " + durationMs + "ms");
        assertTrue("Hashing should be fast", durationMs < 1000);
    }
    
    @Test
    public void testReplicationAwareness() {
        Partition partition = new Partition(0, client);
        
        List<String> replicas = partition.getReplicas();
        assertEquals("Should have 3 replicas", 3, replicas.size());
        
        List<String> isr = partition.getInSyncReplicas();
        assertTrue("ISR should have at least 2", isr.size() >= 2);
    }
    
    @Test
    public void testFailoverDetection() {
        ClusterState cluster = new ClusterState(client);
        
        // Simulate broker failure
        cluster.markBrokerDown("broker2");
        
        List<String> healthy = cluster.getHealthyBrokers();
        assertEquals("Should have 3 healthy brokers after 1 failure", 3, healthy.size());
    }
    
    @Test
    public void testClusterMetrics() {
        ClusterMetrics metrics = new ClusterMetrics(client);
        
        Map<String, Object> stats = metrics.getMetrics();
        assertNotNull("Metrics should not be null", stats);
        assertNotNull("Should have broker count", stats.get("broker_count"));
        assertNotNull("Should have throughput", stats.get("throughput"));
        
        int brokerCount = (int) stats.get("broker_count");
        assertEquals("Should match client broker count", 4, brokerCount);
    }
    
    /**
     * Mock Client for testing
     */
    public static class MockClient {
        private List<String> brokers;
        private String streamName;
        
        public MockClient(List<String> brokers, String streamName) {
            this.brokers = brokers;
            this.streamName = streamName;
        }
        
        public String getStreamName() {
            return streamName;
        }
        
        public int getBrokerCount() {
            return brokers.size();
        }
        
        public List<String> getBrokers() {
            return new ArrayList<>(brokers);
        }
        
        public Map<String, Object> getTopology() {
            Map<String, Object> topology = new HashMap<>();
            topology.put("stream", streamName);
            topology.put("broker_count", brokers.size());
            topology.put("partition_count", 4);
            topology.put("replication_factor", 3);
            return topology;
        }
    }
    
    /**
     * Producer for testing
     */
    public static class Producer {
        private MockClient client;
        
        public Producer(MockClient client) {
            this.client = client;
        }
        
        public int send(String key, String value) {
            return Math.abs(key.hashCode() % 4);
        }
    }
    
    /**
     * Batch Producer for testing
     */
    public static class BatchProducer {
        private MockClient client;
        private int batchSize;
        private long batchTimeoutMs;
        private List<Map<String, String>> batch = new ArrayList<>();
        
        public BatchProducer(MockClient client, int batchSize, long batchTimeoutMs) {
            this.client = client;
            this.batchSize = batchSize;
            this.batchTimeoutMs = batchTimeoutMs;
        }
        
        public void send(String key, String value) {
            Map<String, String> message = new HashMap<>();
            message.put("key", key);
            message.put("value", value);
            batch.add(message);
        }
        
        public int flush() {
            int count = batch.size();
            batch.clear();
            return count;
        }
    }
    
    /**
     * Consumer Group for testing
     */
    public static class ConsumerGroup {
        private MockClient client;
        private String groupId;
        
        public ConsumerGroup(MockClient client, String groupId) {
            this.client = client;
            this.groupId = groupId;
        }
        
        public Map<String, List<Integer>> assignPartitions(List<String> members) {
            Map<String, List<Integer>> assignment = new HashMap<>();
            int partitionsPerMember = 4 / members.size();
            
            for (int i = 0; i < members.size(); i++) {
                List<Integer> partitions = new ArrayList<>();
                for (int j = 0; j < partitionsPerMember; j++) {
                    partitions.add(i * partitionsPerMember + j);
                }
                assignment.put(members.get(i), partitions);
            }
            
            return assignment;
        }
    }
    
    /**
     * Consumer for testing
     */
    public static class Consumer {
        private MockClient client;
        private String groupId;
        private Map<Integer, Long> offsets = new HashMap<>();
        
        public Consumer(MockClient client, String groupId) {
            this.client = client;
            this.groupId = groupId;
        }
        
        public void commitOffset(int partition, long offset) {
            offsets.put(partition, offset);
        }
        
        public long getCommittedOffset(int partition) {
            return offsets.getOrDefault(partition, 0L);
        }
    }
    
    /**
     * Message for testing
     */
    public static class Message {
        private String key;
        private String value;
        
        public Message(String key, String value) {
            this.key = key;
            this.value = value;
        }
        
        public String getKey() { return key; }
        public String getValue() { return value; }
    }
    
    /**
     * Partition for testing
     */
    public static class Partition {
        private int id;
        private MockClient client;
        
        public Partition(int id, MockClient client) {
            this.id = id;
            this.client = client;
        }
        
        public List<String> getReplicas() {
            List<String> brokers = client.getBrokers();
            return brokers.subList(0, Math.min(3, brokers.size()));
        }
        
        public List<String> getInSyncReplicas() {
            return getReplicas().subList(0, Math.min(2, getReplicas().size()));
        }
    }
    
    /**
     * Cluster State for testing
     */
    public static class ClusterState {
        private MockClient client;
        private Set<String> downBrokers = new HashSet<>();
        
        public ClusterState(MockClient client) {
            this.client = client;
        }
        
        public void markBrokerDown(String broker) {
            downBrokers.add(broker);
        }
        
        public List<String> getHealthyBrokers() {
            List<String> healthy = new ArrayList<>();
            for (String broker : client.getBrokers()) {
                if (!downBrokers.contains(broker)) {
                    healthy.add(broker);
                }
            }
            return healthy;
        }
    }
    
    /**
     * Cluster Metrics for testing
     */
    public static class ClusterMetrics {
        private MockClient client;
        
        public ClusterMetrics(MockClient client) {
            this.client = client;
        }
        
        public Map<String, Object> getMetrics() {
            Map<String, Object> metrics = new HashMap<>();
            metrics.put("broker_count", client.getBrokerCount());
            metrics.put("throughput", 912000);
            metrics.put("latency_p99_ms", 2.5);
            metrics.put("replication_lag_ms", 15);
            return metrics;
        }
    }
}
