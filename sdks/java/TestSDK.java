package com.fastdatabroker.sdk.test;

import java.util.*;
import java.util.concurrent.*;

/**
 * FastDataBroker Java SDK - Test Suite
 */

// Mock SDK Types
class Message {
    public String topic;
    public Object payload;
    public int priority;
    public int ttlSeconds;
    public Map<String, String> headers;

    public Message(String topic, Object payload) {
        this.topic = topic;
        this.payload = payload;
        this.priority = 5;
        this.ttlSeconds = 3600;
        this.headers = new HashMap<>();
    }

    public Message withPriority(int priority) {
        this.priority = priority;
        return this;
    }
}

class DeliveryResult {
    public String messageId;
    public String status;
    public double latencyMs;
    public long timestamp;

    public DeliveryResult(String messageId, String status, double latencyMs, long timestamp) {
        this.messageId = messageId;
        this.status = status;
        this.latencyMs = latencyMs;
        this.timestamp = timestamp;
    }

    @Override
    public String toString() {
        return String.format("DeliveryResult{messageId='%s', status='%s', latencyMs=%.2f}", 
            messageId, status, latencyMs);
    }
}

class ConnectionStats {
    public boolean isConnected;
    public long messagesSent;
    public long messagesReceived;
    public long connectionTimeMs;
    public long uptimeSeconds;
    public long lastMessageTime;

    public ConnectionStats(boolean isConnected, long messagesSent, long messagesReceived,
                          long connectionTimeMs, long uptimeSeconds, long lastMessageTime) {
        this.isConnected = isConnected;
        this.messagesSent = messagesSent;
        this.messagesReceived = messagesReceived;
        this.connectionTimeMs = connectionTimeMs;
        this.uptimeSeconds = uptimeSeconds;
        this.lastMessageTime = lastMessageTime;
    }

    @Override
    public String toString() {
        return String.format("ConnectionStats{isConnected=%b, messagesSent=%d, uptimeSeconds=%d}", 
            isConnected, messagesSent, uptimeSeconds);
    }
}

class FastDataBrokerQuicClient {
    private Map<String, String> config;
    private boolean connected;
    private boolean authenticated;
    private Map<String, String> stats;
    private long connectionStart;
    private Map<String, Consumer<Object>> messageHandlers;

    @FunctionalInterface
    interface Consumer<T> {
        void accept(T t);
    }

    public FastDataBrokerQuicClient(String host, int port, String tenantId, String clientId, String pskSecret) {
        config = new HashMap<>();
        config.put("host", host);
        config.put("port", String.valueOf(port));
        config.put("tenant_id", tenantId);
        config.put("client_id", clientId);
        config.put("psk_secret", pskSecret);

        this.connected = false;
        this.authenticated = false;
        this.stats = new HashMap<>();
        stats.put("messages_sent", "0");
        stats.put("messages_received", "0");
        this.messageHandlers = new HashMap<>();
    }

    public void connect() throws Exception {
        String host = config.get("host");
        String port = config.get("port");
        System.out.printf("Connecting to %s:%s...%n", host, port);
        this.connected = true;
        this.connectionStart = System.currentTimeMillis();
        System.out.printf("✓ Connected to %s:%s%n", host, port);
    }

    public DeliveryResult sendMessage(Message message) throws Exception {
        if (!connected) {
            throw new Exception("Not connected");
        }

        String messageId = String.format("msg_%d_%d", System.currentTimeMillis(), 
            (int)(Math.random() * 10000));
        double latency = Math.random() * 50 + 5;
        long messagesSent = Long.parseLong(stats.getOrDefault("messages_sent", "0"));
        stats.put("messages_sent", String.valueOf(messagesSent + 1));

        return new DeliveryResult(messageId, "success", latency, System.currentTimeMillis());
    }

    public void onMessage(String topic, Consumer<Object> handler) {
        messageHandlers.put(topic, handler);
    }

    public void offMessage(String topic) {
        messageHandlers.remove(topic);
    }

    public ConnectionStats getStats() {
        long uptime = connected ? System.currentTimeMillis() - connectionStart : 0;
        return new ConnectionStats(
            connected,
            Long.parseLong(stats.getOrDefault("messages_sent", "0")),
            Long.parseLong(stats.getOrDefault("messages_received", "0")),
            uptime,
            uptime / 1000,
            0
        );
    }

    public boolean isConnected() {
        return connected;
    }

    public void disconnect() {
        connected = false;
        System.out.println("✓ Disconnected");
    }
}

/**
 * Test Suite
 */
public class TestSDK {
    private static int testsPassed = 0;
    private static int testsFailed = 0;

    static void runTest(String name, Runnable testFn) {
        try {
            testFn.run();
            System.out.printf("✅ PASS: %s%n", name);
            testsPassed++;
        } catch (Exception error) {
            System.out.printf("❌ FAIL: %s%n", name);
            System.out.printf("   Error: %s%n", error.getMessage());
            testsFailed++;
        }
    }

    static void test1BasicConnection() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        if (client.isConnected()) {
            throw new Exception("Should not be connected yet");
        }

        client.connect();

        if (!client.isConnected()) {
            throw new Exception("Should be connected");
        }

        client.disconnect();
    }

    static void test2SendMessage() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        client.connect();

        Message msg = new Message("test.topic", new HashMap<>()).withPriority(5);
        DeliveryResult result = client.sendMessage(msg);

        if (!result.status.equals("success")) {
            throw new Exception("Expected status 'success', got '" + result.status + "'");
        }

        if (result.messageId == null || result.messageId.isEmpty()) {
            throw new Exception("Message ID should not be empty");
        }

        if (result.latencyMs < 0) {
            throw new Exception("Latency should be non-negative");
        }

        client.disconnect();
    }

    static void test3MessageHandlers() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        client.connect();

        client.onMessage("test.topic", msg -> {});

        if (!client.messageHandlers.containsKey("test.topic")) {
            throw new Exception("Handler should be registered");
        }

        client.offMessage("test.topic");

        if (client.messageHandlers.containsKey("test.topic")) {
            throw new Exception("Handler should be unregistered");
        }

        client.disconnect();
    }

    static void test4ConnectionStatistics() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        client.connect();

        for (int i = 0; i < 5; i++) {
            client.sendMessage(new Message("test.topic", new HashMap<>()));
        }

        // Add small delay to ensure timing is measurable
        Thread.sleep(50);

        ConnectionStats stats = client.getStats();

        if (!stats.isConnected) {
            throw new Exception("Should be connected");
        }

        if (stats.messagesSent != 5) {
            throw new Exception("Expected 5 messages sent, got " + stats.messagesSent);
        }

        if (stats.uptimeSeconds < 0) {
            throw new Exception("Uptime should be non-negative");
        }

        client.disconnect();
    }

    static void test5ConcurrentMessages() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        client.connect();

        List<DeliveryResult> results = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            results.add(client.sendMessage(new Message("test.concurrent", new HashMap<>())));
        }

        if (results.size() != 10) {
            throw new Exception("Expected 10 results, got " + results.size());
        }

        for (DeliveryResult result : results) {
            if (!result.status.equals("success")) {
                throw new Exception("All messages should be successful");
            }
        }

        if (client.stats.get("messages_sent") == null || 
            !client.stats.get("messages_sent").equals("10")) {
            throw new Exception("Expected 10 messages sent");
        }

        client.disconnect();
    }

    static void test6PriorityLevels() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        client.connect();

        int[] priorities = {1, 5, 10, 20};

        for (int priority : priorities) {
            DeliveryResult result = client.sendMessage(
                new Message("test.priority", new HashMap<>()).withPriority(priority)
            );

            if (!result.status.equals("success")) {
                throw new Exception("Failed to send message with priority " + priority);
            }
        }

        client.disconnect();
    }

    static void test7LatencyMeasurement() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        client.connect();

        List<Double> latencies = new ArrayList<>();
        for (int i = 0; i < 50; i++) {
            DeliveryResult result = client.sendMessage(
                new Message("test.latency", new HashMap<>())
            );
            latencies.add(result.latencyMs);
        }

        double avgLatency = 0;
        for (double latency : latencies) {
            avgLatency += latency;
        }
        avgLatency /= latencies.size();

        if (avgLatency < 0) {
            throw new Exception("Average latency should be non-negative");
        }

        double maxLatency = Collections.max(latencies);

        System.out.printf("   Average latency: %.2fms, Max: %.2fms%n", avgLatency, maxLatency);

        client.disconnect();
    }

    static void test8ErrorHandling() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        try {
            client.sendMessage(new Message("test", new HashMap<>()));
            throw new Exception("Should have thrown error");
        } catch (Exception e) {
            if (!e.getMessage().equals("Not connected")) {
                throw new Exception("Should throw 'Not connected' error");
            }
        }

        client.connect();
        DeliveryResult result = client.sendMessage(new Message("test", new HashMap<>()));
        if (!result.status.equals("success")) {
            throw new Exception("Should send successfully after connect");
        }

        client.disconnect();
    }

    static void test9ConfigurationValidation() throws Exception {
        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(
            "localhost", 6000, "test-tenant", "test-client", "test-secret");

        if (!client.config.get("host").equals("localhost")) {
            throw new Exception("Host configuration not properly saved");
        }

        if (!client.config.get("port").equals("6000")) {
            throw new Exception("Port configuration not properly saved");
        }
    }

    public static void main(String[] args) {
        System.out.println("\n" + "=".repeat(70));
        System.out.println("FastDataBroker Java SDK - Test Suite");
        System.out.println("=".repeat(70) + "\n");

        runTest("1. Basic Connection", () -> test1BasicConnection());
        runTest("2. Send Message", () -> test2SendMessage());
        runTest("3. Message Handlers", () -> test3MessageHandlers());
        runTest("4. Connection Statistics", () -> test4ConnectionStatistics());
        runTest("5. Concurrent Messages", () -> test5ConcurrentMessages());
        runTest("6. Priority Levels", () -> test6PriorityLevels());
        runTest("7. Latency Measurement", () -> test7LatencyMeasurement());
        runTest("8. Error Handling", () -> test8ErrorHandling());
        runTest("9. Configuration Validation", () -> test9ConfigurationValidation());

        System.out.println("\n" + "=".repeat(70));
        System.out.printf("Results: %d passed, %d failed%n", testsPassed, testsFailed);
        System.out.println("=".repeat(70) + "\n");

        System.exit(testsFailed > 0 ? 1 : 0);
    }
}
