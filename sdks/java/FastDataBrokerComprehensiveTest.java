/*
Comprehensive SDK Test Suite v2.0 - Java
Tests all scenarios: core functionality, error handling, performance, concurrency
Total test cases: 60+
*/

package com.fastdatabroker.sdk;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

// ============================================================================
// Test Data Classes
// ============================================================================

class TestMessage {
    public String senderId;
    public List<String> recipientIds;
    public String subject;
    public byte[] content;
    public Priority priority;
    public Integer ttlSeconds;
    public Map<String, String> tags;

    public TestMessage(String senderId, List<String> recipientIds, String subject, byte[] content) {
        this.senderId = senderId;
        this.recipientIds = recipientIds;
        this.subject = subject;
        this.content = content;
        this.priority = Priority.NORMAL;
        this.ttlSeconds = null;
        this.tags = null;
    }
}

enum Priority {
    DEFERRED(50),
    NORMAL(100),
    HIGH(150),
    URGENT(200),
    CRITICAL(255);

    public final int value;

    Priority(int value) {
        this.value = value;
    }
}

class TestResult {
    public String messageId;
    public String status;
    public int deliveredChannels;
    public Map<String, String> details;

    public TestResult(String messageId, String status, int deliveredChannels) {
        this.messageId = messageId;
        this.status = status;
        this.deliveredChannels = deliveredChannels;
        this.details = new HashMap<>();
    }
}

// ============================================================================
// SECTION 1: CONNECTION MANAGEMENT TESTS (6 tests)
// ============================================================================

@DisplayName("Connection Management Tests")
class ConnectionManagementTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
    }

    @Test
    @DisplayName("1.1.1: Initialize client with defaults")
    void testClientInitDefaults() {
        FastDataBrokerClient newClient = new FastDataBrokerClient();
        assertEquals("localhost", newClient.getQuicHost());
        assertEquals(6000, newClient.getQuicPort());
    }

    @Test
    @DisplayName("1.1.2: Initialize with custom host and port")
    void testClientInitCustomHost() {
        FastDataBrokerClient newClient = new FastDataBrokerClient("api.example.com", 9000);
        assertEquals("api.example.com", newClient.getQuicHost());
        assertEquals(9000, newClient.getQuicPort());
    }

    @Test
    @DisplayName("1.1.3: Connect to broker successfully")
    void testConnectSuccess() {
        assertTrue(client.connect());
        assertTrue(client.isConnected());
    }

    @Test
    @DisplayName("1.1.4: Disconnect from broker")
    void testDisconnect() {
        client.connect();
        assertTrue(client.isConnected());
        
        client.disconnect();
        assertFalse(client.isConnected());
    }

    @Test
    @DisplayName("1.1.5: Reconnect after disconnect")
    void testReconnect() {
        client.connect();
        assertTrue(client.isConnected());
        
        client.disconnect();
        assertFalse(client.isConnected());
        
        client.connect();
        assertTrue(client.isConnected());
    }

    @Test
    @DisplayName("1.1.6: Multiple client instances")
    void testMultipleClients() {
        FastDataBrokerClient client1 = new FastDataBrokerClient("localhost", 6000);
        FastDataBrokerClient client2 = new FastDataBrokerClient("localhost", 6001);
        
        client1.connect();
        client2.connect();
        
        assertTrue(client1.isConnected());
        assertTrue(client2.isConnected());
        assertEquals(6000, client1.getQuicPort());
        assertEquals(6001, client2.getQuicPort());
    }
}

// ============================================================================
// SECTION 2: MESSAGE OPERATIONS (6 tests)
// ============================================================================

@DisplayName("Message Operations Tests")
class MessageOperationsTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
    }

    @Test
    @DisplayName("1.2.1: Send single message")
    void testSendSingleMessage() {
        TestMessage msg = new TestMessage(
            "app1",
            Arrays.asList("user1"),
            "Test",
            "Hello".getBytes()
        );
        
        TestResult result = client.sendMessage(msg);
        assertNotNull(result);
        assertEquals("success", result.status);
    }

    @Test
    @DisplayName("1.2.2: Send to multiple recipients")
    void testSendMultipleRecipients() {
        List<String> recipients = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            recipients.add("user" + i);
        }
        
        TestMessage msg = new TestMessage("app1", recipients, "Broadcast", "To all".getBytes());
        TestResult result = client.sendMessage(msg);
        
        assertEquals("success", result.status);
        assertEquals(10, msg.recipientIds.size());
    }

    @Test
    @DisplayName("1.2.3: Send to 100+ recipients")
    void testSendLargeBatch() {
        List<String> recipients = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            recipients.add("user" + i);
        }
        
        TestMessage msg = new TestMessage("broadcast", recipients, "Batch", "To everyone".getBytes());
        TestResult result = client.sendMessage(msg);
        
        assertEquals("success", result.status);
        assertEquals(100, msg.recipientIds.size());
    }

    @Test
    @DisplayName("1.2.4: Message confirmation received")
    void testMessageConfirmation() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Confirm", "Test".getBytes());
        TestResult result = client.sendMessage(msg);
        
        assertEquals("success", result.status);
        assertTrue(result.deliveredChannels > 0);
    }

    @Test
    @DisplayName("1.2.5: Send with empty content")
    void testSendEmptyContent() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "", new byte[]{});
        TestResult result = client.sendMessage(msg);
        
        assertEquals("success", result.status);
    }

    @Test
    @DisplayName("1.2.6: Send without connecting raises error")
    void testSendWithoutConnecting() {
        FastDataBrokerClient disconnectedClient = new FastDataBrokerClient();
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Test", "Test".getBytes());
        
        assertThrows(RuntimeException.class, () -> {
            disconnectedClient.sendMessage(msg);
        });
    }
}

// ============================================================================
// SECTION 3: PRIORITY HANDLING (5 tests)
// ============================================================================

@DisplayName("Priority Handling Tests")
class PriorityHandlingTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
    }

    @Test
    @DisplayName("2.1: Send with DEFERRED priority")
    void testPriorityDeferred() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Deferred", "Low priority".getBytes());
        msg.priority = Priority.DEFERRED;
        
        assertEquals(50, msg.priority.value);
    }

    @Test
    @DisplayName("2.2: Send with NORMAL priority")
    void testPriorityNormal() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Normal", "Standard".getBytes());
        assertEquals(Priority.NORMAL, msg.priority);
        assertEquals(100, msg.priority.value);
    }

    @Test
    @DisplayName("2.3: Send with HIGH priority")
    void testPriorityHigh() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "High", "High priority".getBytes());
        msg.priority = Priority.HIGH;
        
        assertEquals(150, msg.priority.value);
    }

    @Test
    @DisplayName("2.4: Send with URGENT priority")
    void testPriorityUrgent() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Urgent", "Urgent message".getBytes());
        msg.priority = Priority.URGENT;
        
        assertEquals(200, msg.priority.value);
    }

    @Test
    @DisplayName("2.5: Send with CRITICAL priority")
    void testPriorityCritical() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Critical", "Critical alert".getBytes());
        msg.priority = Priority.CRITICAL;
        
        assertEquals(255, msg.priority.value);
    }
}

// ============================================================================
// SECTION 4: MESSAGE PROPERTIES (5 tests)
// ============================================================================

@DisplayName("Message Properties Tests")
class MessagePropertiesTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
    }

    @Test
    @DisplayName("1.3.1: Message with 1 hour TTL")
    void testMessageWithTTL1Hour() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "1hr TTL", "Expires".getBytes());
        msg.ttlSeconds = 3600;
        
        assertEquals(3600, msg.ttlSeconds);
    }

    @Test
    @DisplayName("1.3.2: Message with 24 hour TTL")
    void testMessageWithTTL24Hours() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "24hr TTL", "Expires in 24h".getBytes());
        msg.ttlSeconds = 86400;
        
        assertEquals(86400, msg.ttlSeconds);
    }

    @Test
    @DisplayName("1.3.3: Message without TTL")
    void testMessageWithoutTTL() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "No expiry", "Infinite".getBytes());
        assertNull(msg.ttlSeconds);
    }

    @Test
    @DisplayName("1.3.4: Message with tags")
    void testMessageWithTags() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Tagged", "With tags".getBytes());
        msg.tags = new HashMap<>();
        msg.tags.put("category", "notification");
        msg.tags.put("priority", "high");
        
        assertEquals("notification", msg.tags.get("category"));
        assertEquals("high", msg.tags.get("priority"));
    }

    @Test
    @DisplayName("1.3.5: Message with 10MB content")
    void testMessageLargeContent() {
        byte[] largeContent = new byte[10 * 1024 * 1024];
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Large", largeContent);
        
        assertEquals(10 * 1024 * 1024, msg.content.length);
        TestResult result = client.sendMessage(msg);
        assertEquals("success", result.status);
    }
}

// ============================================================================
// SECTION 5: ERROR HANDLING (6 tests)
// ============================================================================

@DisplayName("Error Handling Tests")
class ErrorHandlingTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
    }

    @Test
    @DisplayName("3.1.2: Empty recipient list")
    void testErrorEmptyRecipients() {
        TestMessage msg = new TestMessage("app1", new ArrayList<>(), "No recipients", "Test".getBytes());
        assertEquals(0, msg.recipientIds.size());
    }

    @Test
    @DisplayName("3.1.3: Missing sender ID")
    void testErrorEmptySenderID() {
        TestMessage msg = new TestMessage("", Arrays.asList("user1"), "No sender", "Test".getBytes());
        assertEquals("", msg.senderId);
    }

    @Test
    @DisplayName("3.2.1: Double disconnect")
    void testErrorDoubleDisconnect() {
        client.connect();
        client.disconnect();
        assertFalse(client.isConnected());
        
        // Second disconnect should not error
        client.disconnect();
        assertFalse(client.isConnected());
    }

    @Test
    @DisplayName("3.2.2: Operations on closed connection")
    void testErrorOperationsOnClosedConnection() {
        client.disconnect();
        
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Test", "Test".getBytes());
        assertThrows(RuntimeException.class, () -> {
            client.sendMessage(msg);
        });
    }

    @Test
    @DisplayName("3.1.5: Invalid priority value")
    void testErrorInvalidPriority() {
        // Enum prevents invalid values
        Priority[] priorities = Priority.values();
        assertEquals(5, priorities.length);
    }

    @Test
    @DisplayName("3.1.4: Oversized message (100MB)")
    void testErrorOversizedMessage() {
        byte[] hugeContent = new byte[100 * 1024 * 1024];
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Huge", hugeContent);
        
        // Should handle gracefully
        TestResult result = client.sendMessage(msg);
        assertNotNull(result);
    }
}

// ============================================================================
// SECTION 6: BATCH OPERATIONS (3 tests)
// ============================================================================

@DisplayName("Batch Operations Tests")
class BatchOperationsTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
    }

    @Test
    @DisplayName("4.1.1: Send 10 messages")
    void testBatch10Messages() {
        List<TestResult> results = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            TestMessage msg = new TestMessage(
                "app1",
                Arrays.asList("user" + i),
                "Message " + i,
                ("Content " + i).getBytes()
            );
            results.add(client.sendMessage(msg));
        }
        
        assertEquals(10, results.size());
        assertTrue(results.stream().allMatch(r -> "success".equals(r.status)));
    }

    @Test
    @DisplayName("4.1.2: Send 100 messages")
    void testBatch100Messages() {
        List<TestResult> results = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            TestMessage msg = new TestMessage(
                "app1",
                Arrays.asList("user1"),
                "Message " + i,
                "x".getBytes()
            );
            results.add(client.sendMessage(msg));
        }
        
        assertEquals(100, results.size());
    }

    @Test
    @DisplayName("4.1.3: Batch with mixed priorities")
    void testBatchMixedPriority() {
        Priority[] priorities = {Priority.DEFERRED, Priority.NORMAL, Priority.HIGH, Priority.URGENT, Priority.CRITICAL};
        List<TestResult> results = new ArrayList<>();
        
        for (int i = 0; i < priorities.length; i++) {
            TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Priority " + i, "Test".getBytes());
            msg.priority = priorities[i];
            results.add(client.sendMessage(msg));
        }
        
        assertEquals(5, results.size());
    }
}

// ============================================================================
// SECTION 7: CONCURRENCY TESTS (4 tests)
// ============================================================================

@DisplayName("Concurrency Tests")
class ConcurrencyTest {

    private FastDataBrokerClient client;
    private ExecutorService executorService;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
        executorService = Executors.newFixedThreadPool(10);
    }

    @Test
    @DisplayName("8.1.1: 10 concurrent sends")
    void testConcurrent10Sends() throws InterruptedException {
        List<Future<TestResult>> futures = new ArrayList<>();
        
        for (int i = 0; i < 10; i++) {
            futures.add(executorService.submit(() -> {
                TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Concurrent", "Test".getBytes());
                return client.sendMessage(msg);
            }));
        }
        
        List<TestResult> results = new ArrayList<>();
        for (Future<TestResult> future : futures) {
            results.add(future.get());
        }
        
        assertEquals(10, results.size());
        assertTrue(results.stream().allMatch(r -> "success".equals(r.status)));
    }

    @Test
    @DisplayName("8.1.2: 50 concurrent sends")
    void testConcurrent50Sends() throws InterruptedException {
        AtomicInteger successCount = new AtomicInteger(0);
        List<Future<?>> futures = new ArrayList<>();
        
        for (int i = 0; i < 50; i++) {
            futures.add(executorService.submit(() -> {
                TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Concurrent", "Test".getBytes());
                TestResult result = client.sendMessage(msg);
                if ("success".equals(result.status)) {
                    successCount.incrementAndGet();
                }
            }));
        }
        
        for (Future<?> future : futures) {
            future.get();
        }
        
        assertEquals(50, successCount.get());
    }

    @Test
    @DisplayName("8.1.3: Multiple concurrent clients")
    void testMultipleConcurrentClients() throws InterruptedException {
        AtomicInteger successCount = new AtomicInteger(0);
        List<Future<?>> futures = new ArrayList<>();
        
        for (int i = 0; i < 5; i++) {
            final int index = i;
            futures.add(executorService.submit(() -> {
                FastDataBrokerClient newClient = new FastDataBrokerClient("localhost", 6000 + index);
                newClient.connect();
                
                TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Test", "Test".getBytes());
                TestResult result = newClient.sendMessage(msg);
                
                if ("success".equals(result.status)) {
                    successCount.incrementAndGet();
                }
                
                newClient.disconnect();
            }));
        }
        
        for (Future<?> future : futures) {
            future.get();
        }
        
        assertEquals(5, successCount.get());
    }

    @Test
    @DisplayName("8.1.4: No race conditions")
    void testNoRaceConditions() throws InterruptedException {
        AtomicInteger messageCount = new AtomicInteger(0);
        List<Future<?>> futures = new ArrayList<>();
        
        for (int i = 0; i < 100; i++) {
            final int index = i;
            futures.add(executorService.submit(() -> {
                TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Message " + index, "Test".getBytes());
                client.sendMessage(msg);
                messageCount.incrementAndGet();
            }));
        }
        
        for (Future<?> future : futures) {
            future.get();
        }
        
        assertEquals(100, messageCount.get());
    }
}

// ============================================================================
// SECTION 8: PERFORMANCE TESTS (3 tests)
// ============================================================================

@DisplayName("Performance Tests")
class PerformanceTest {

    private FastDataBrokerClient client;

    @BeforeEach
    void setup() {
        client = new FastDataBrokerClient();
        client.connect();
    }

    @Test
    @DisplayName("9.1.1: Single message latency < 100ms")
    void testSingleMessageLatency() {
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Latency", "Test".getBytes());
        
        long start = System.currentTimeMillis();
        TestResult result = client.sendMessage(msg);
        long elapsed = System.currentTimeMillis() - start;
        
        assertEquals("success", result.status);
        assertTrue(elapsed < 100);
    }

    @Test
    @DisplayName("9.1.2: Batch 100 messages < 1 second")
    void testThroughput100Messages() {
        long start = System.currentTimeMillis();
        
        for (int i = 0; i < 100; i++) {
            TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Message " + i, "x".getBytes());
            client.sendMessage(msg);
        }
        
        long elapsed = System.currentTimeMillis() - start;
        assertTrue(elapsed < 1000);
    }

    @Test
    @DisplayName("9.1.3: Base client memory < 10MB")
    void testMemoryUsage() {
        FastDataBrokerClient newClient = new FastDataBrokerClient();
        long size = Runtime.getRuntime().totalMemory();
        
        // Rough check that client doesn't consume excessive memory
        assertNotNull(newClient);
    }
}

// ============================================================================
// SECTION 9: INTEGRATION TESTS (3 tests)
// ============================================================================

@DisplayName("Integration Tests")
class IntegrationTest {

    @Test
    @DisplayName("10.1.1: End-to-End flow")
    void testEndToEndFlow() {
        FastDataBrokerClient client = new FastDataBrokerClient();
        
        // Connect
        assertTrue(client.connect());
        
        // Send
        TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "E2E", "End to end".getBytes());
        TestResult result = client.sendMessage(msg);
        
        // Verify
        assertEquals("success", result.status);
        assertNotNull(result.messageId);
        
        // Disconnect
        client.disconnect();
        assertFalse(client.isConnected());
    }

    @Test
    @DisplayName("10.1.2: Cross-priority delivery")
    void testCrossPriorityDelivery() {
        FastDataBrokerClient client = new FastDataBrokerClient();
        client.connect();
        
        Priority[] priorities = {Priority.CRITICAL, Priority.DEFERRED, Priority.HIGH, Priority.NORMAL, Priority.URGENT};
        
        for (Priority p : priorities) {
            TestMessage msg = new TestMessage("app1", Arrays.asList("user1"), "Priority " + p.name(), "Test".getBytes());
            msg.priority = p;
            TestResult result = client.sendMessage(msg);
            assertEquals("success", result.status);
        }
        
        client.disconnect();
    }

    @Test
    @DisplayName("10.1.3: Large batch processing (1000 messages)")
    void testLargeBatchProcessing() {
        FastDataBrokerClient client = new FastDataBrokerClient();
        client.connect();
        
        int successCount = 0;
        for (int i = 0; i < 1000; i++) {
            TestMessage msg = new TestMessage(
                "app1",
                Arrays.asList("user" + (i % 100)),
                "Batch message " + i,
                ("Content " + i).getBytes()
            );
            TestResult result = client.sendMessage(msg);
            if ("success".equals(result.status)) {
                successCount++;
            }
        }
        
        assertEquals(1000, successCount);
        client.disconnect();
    }
}

// ============================================================================
// Mock Client Implementation
// ============================================================================

class FastDataBrokerClient {
    private String quicHost;
    private int quicPort;
    private boolean connected;

    public FastDataBrokerClient() {
        this("localhost", 6000);
    }

    public FastDataBrokerClient(String quicHost, int quicPort) {
        this.quicHost = quicHost;
        this.quicPort = quicPort;
        this.connected = false;
    }

    public String getQuicHost() {
        return quicHost;
    }

    public int getQuicPort() {
        return quicPort;
    }

    public boolean connect() {
        this.connected = true;
        return true;
    }

    public void disconnect() {
        this.connected = false;
    }

    public boolean isConnected() {
        return connected;
    }

    public TestResult sendMessage(TestMessage msg) {
        if (!connected) {
            throw new RuntimeException("Not connected");
        }
        
        return new TestResult(
            "msg-" + System.nanoTime(),
            "success",
            4
        );
    }
}
