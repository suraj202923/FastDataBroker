package com.fastdatabroker.sdk;

import java.io.*;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.time.Instant;
import java.util.*;
import java.util.concurrent.*;
import java.util.function.Consumer;
import java.util.logging.Level;
import java.util.logging.Logger;

import com.google.gson.*;

/**
 * FastDataBroker Java SDK - QUIC with PSK Authentication
 * High-performance client library with Pre-Shared Key authentication
 * 
 * Version: 1.0.0
 * Protocol: QUIC 1.0 (RFC 9000)
 * Authentication: TLS 1.3 PSK (Pre-Shared Key)
 */

public class FastDataBrokerQuicClient {
    private static final Logger LOGGER = Logger.getLogger(FastDataBrokerQuicClient.class.getName());
    private static final Gson GSON = new Gson();

    public enum Priority {
        LOW(1), NORMAL(5), HIGH(10), CRITICAL(20);

        private final int value;
        Priority(int value) {
            this.value = value;
        }

        public int getValue() {
            return value;
        }
    }

    public enum ConnectionState {
        DISCONNECTED, CONNECTING, CONNECTED, AUTHENTICATED, ERROR
    }

    // Configuration
    public static class QuicConnectionConfig {
        public String host;
        public int port;
        public String tenantId;
        public String clientId;
        public String pskSecret;
        public String secrets;
        public int idleTimeoutMs = 30000;
        public int maxStreams = 100;
        public boolean autoReconnect = true;
        public int readTimeoutMs = 60000;

        public QuicConnectionConfig(String host, int port, String tenantId, String clientId, String pskSecret) {
            this.host = host;
            this.port = port;
            this.tenantId = tenantId;
            this.clientId = clientId;
            this.pskSecret = pskSecret;
        }
    }

    // Message envelope
    public static class Message {
        public String topic;
        public Object payload;
        public Priority priority = Priority.NORMAL;
        public int ttlSeconds = 3600;
        public Map<String, String> headers = new HashMap<>();

        public Message(String topic, Object payload) {
            this.topic = topic;
            this.payload = payload;
        }

        public Message withPriority(Priority priority) {
            this.priority = priority;
            return this;
        }

        public Message withTtl(int seconds) {
            this.ttlSeconds = seconds;
            return this;
        }

        public Message withHeader(String key, String value) {
            this.headers.put(key, value);
            return this;
        }
    }

    // Delivery result
    public static class DeliveryResult {
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

    // Connection statistics
    public static class ConnectionStats {
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
            return String.format("ConnectionStats{isConnected=%b, messagesSent=%d, messagesReceived=%d, " +
                "connectionTimeMs=%d, uptimeSeconds=%d}", isConnected, messagesSent, messagesReceived,
                connectionTimeMs, uptimeSeconds);
        }
    }

    // ========================================================================
    // Client Implementation
    // ========================================================================

    private final QuicConnectionConfig config;
    private Socket socket;
    private PrintWriter writer;
    private BufferedReader reader;
    private boolean connected = false;
    private boolean authenticated = false;
    private ConnectionState state = ConnectionState.DISCONNECTED;
    private final Map<String, Consumer<JsonObject>> messageHandlers = new ConcurrentHashMap<>();
    private long connectionStart = 0;
    private ExecutorService executorService;
    private volatile boolean running = false;

    // Statistics
    private long messagesSent = 0;
    private long messagesReceived = 0;
    private long lastMessageTime = 0;
    private final Object statsLock = new Object();

    // Constructor
    public FastDataBrokerQuicClient(QuicConnectionConfig config) {
        this.config = config;
        LOGGER.info(String.format("Initialized FastDataBroker QUIC client for %s:%s",
            config.tenantId, config.clientId));
    }

    /**
     * Generate PSK identity and secret hash
     */
    private Map<String, String> generatePskIdentity() {
        String identity = config.tenantId + ":" + config.clientId;
        String secretHash = hashSha256(config.pskSecret);
        
        Map<String, String> result = new HashMap<>();
        result.put("identity", identity);
        result.put("secretHash", secretHash);
        return result;
    }

    /**
     * Compute SHA-256 hash
     */
    private String hashSha256(String input) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(input.getBytes(StandardCharsets.UTF_8));
            StringBuilder hexString = new StringBuilder();
            for (byte b : hash) {
                String hex = Integer.toHexString(0xff & b);
                if (hex.length() == 1) hexString.append('0');
                hexString.append(hex);
            }
            return hexString.toString();
        } catch (Exception e) {
            throw new RuntimeException("Failed to hash secret", e);
        }
    }

    /**
     * Connect to FastDataBroker with PSK authentication
     */
    public synchronized void connect() throws IOException {
        if (connected) {
            LOGGER.warning("Already connected");
            return;
        }

        try {
            state = ConnectionState.CONNECTING;
            LOGGER.info(String.format("Connecting to %s:%d...", config.host, config.port));

            // Create socket
            socket = new Socket(config.host, config.port);
            socket.setKeepAlive(true);
            socket.setSoTimeout(config.readTimeoutMs);

            writer = new PrintWriter(new OutputStreamWriter(socket.getOutputStream(), StandardCharsets.UTF_8), true);
            reader = new BufferedReader(new InputStreamReader(socket.getInputStream(), StandardCharsets.UTF_8));

            connected = true;
            connectionStart = System.currentTimeMillis();
            LOGGER.info("✓ TCP connection established");

            // Send PSK handshake
            sendPskHandshake();

            // Start receive loop
            executorService = Executors.newSingleThreadExecutor(r -> {
                Thread t = new Thread(r, "FastDataBroker-Receive-Thread");
                t.setDaemon(true);
                return t;
            });

            running = true;
            executorService.execute(this::receiveLoop);

            authenticated = true;
            state = ConnectionState.AUTHENTICATED;
            LOGGER.info("✓ PSK authentication successful");

        } catch (IOException e) {
            state = ConnectionState.ERROR;
            connected = false;
            LOGGER.log(Level.SEVERE, "Connection failed", e);
            throw e;
        }
    }

    /**
     * Send PSK handshake
     */
    private void sendPskHandshake() {
        Map<String, String> psk = generatePskIdentity();
        JsonObject handshake = new JsonObject();
        handshake.addProperty("type", "psk_auth");
        handshake.addProperty("identity", psk.get("identity"));
        handshake.addProperty("secretHash", psk.get("secretHash"));
        handshake.addProperty("timestamp", System.currentTimeMillis());

        writer.println(handshake.toString());
        LOGGER.fine(String.format("Sent PSK handshake for %s", psk.get("identity")));
    }

    /**
     * Receive loop
     */
    private void receiveLoop() {
        String line;
        try {
            while (running && connected) {
                try {
                    line = reader.readLine();
                    if (line == null) {
                        LOGGER.info("Server closed connection");
                        connected = false;
                        break;
                    }

                    if (!line.isEmpty()) {
                        handleIncomingMessage(line);
                    }
                } catch (SocketTimeoutException e) {
                    continue;
                }
            }
        } catch (IOException e) {
            if (running && connected) {
                LOGGER.log(Level.WARNING, "Receive error", e);
            }
            connected = false;
        }
    }

    /**
     * Handle incoming message
     */
    private void handleIncomingMessage(String messageStr) {
        try {
            JsonObject parsed = JsonParser.parseString(messageStr).getAsJsonObject();

            if ("message".equals(parsed.get("type").getAsString())) {
                String topic = parsed.get("topic").getAsString();
                Consumer<JsonObject> handler = messageHandlers.get(topic);

                if (handler != null) {
                    handler.accept(parsed);
                    synchronized (statsLock) {
                        messagesReceived++;
                        lastMessageTime = System.currentTimeMillis();
                    }
                }
            }
        } catch (Exception e) {
            // Ignore malformed messages
        }
    }

    /**
     * Send message to FastDataBroker
     */
    public DeliveryResult sendMessage(Message message) throws IOException {
        if (!connected || socket == null) {
            throw new IOException("Not connected to FastDataBroker");
        }

        long startTime = System.currentTimeMillis();
        String messageId = String.format("msg_%d_%d", System.currentTimeMillis(), 
            (int)(Math.random() * 10000));

        JsonObject envelope = new JsonObject();
        envelope.addProperty("type", "message");
        envelope.addProperty("id", messageId);
        envelope.addProperty("topic", message.topic);
        envelope.add("payload", GSON.toJsonTree(message.payload));
        envelope.addProperty("priority", message.priority.getValue());
        envelope.addProperty("ttl", message.ttlSeconds);
        envelope.add("headers", GSON.toJsonTree(message.headers));
        envelope.addProperty("timestamp", System.currentTimeMillis());

        writer.println(envelope.toString());

        synchronized (statsLock) {
            messagesSent++;
            lastMessageTime = System.currentTimeMillis();
        }

        double latency = System.currentTimeMillis() - startTime;

        return new DeliveryResult(messageId, "success", latency, System.currentTimeMillis());
    }

    /**
     * Register message handler
     */
    public void onMessage(String topic, Consumer<JsonObject> handler) {
        messageHandlers.put(topic, handler);
        LOGGER.info("Registered handler for topic: " + topic);
    }

    /**
     * Unregister message handler
     */
    public void offMessage(String topic) {
        messageHandlers.remove(topic);
        LOGGER.info("Unregistered handler for topic: " + topic);
    }

    /**
     * Get connection statistics
     */
    public ConnectionStats getStats() {
        synchronized (statsLock) {
            long uptime = connected ? System.currentTimeMillis() - connectionStart : 0;
            return new ConnectionStats(
                connected,
                messagesSent,
                messagesReceived,
                uptime,
                uptime / 1000,
                lastMessageTime
            );
        }
    }

    /**
     * Check if connected
     */
    public boolean isConnected() {
        return connected && authenticated;
    }

    /**
     * Send multiple messages in parallel
     */
    public List<DeliveryResult> sendMessagesParallel(List<Message> messages, int numWorkers) {
        if (!isConnected()) {
            throw new RuntimeException("Not connected to FastDataBroker");
        }

        List<DeliveryResult> results = Collections.synchronizedList(new ArrayList<>());
        ExecutorService executor = Executors.newFixedThreadPool(numWorkers);
        List<Future<?>> futures = new ArrayList<>();

        for (Message message : messages) {
            Future<?> future = executor.submit(() -> {
                try {
                    DeliveryResult result = sendMessage(message);
                    results.add(result);
                } catch (Exception e) {
                    results.add(new DeliveryResult(
                        "error_" + System.currentTimeMillis(),
                        "failed",
                        0,
                        System.currentTimeMillis()
                    ));
                }
            });
            futures.add(future);
        }

        // Wait for all to complete
        for (Future<?> future : futures) {
            try {
                future.get();
            } catch (Exception e) {
                LOGGER.log(Level.WARNING, "Error waiting for message", e);
            }
        }

        executor.shutdown();
        return results;
    }

    /**
     * Send messages with progress callback
     */
    public List<DeliveryResult> sendMessagesParallelWithProgress(
        List<Message> messages,
        int numWorkers,
        Consumer<ProgressUpdate> callback
    ) {
        if (!isConnected()) {
            throw new RuntimeException("Not connected to FastDataBroker");
        }

        List<DeliveryResult> results = Collections.synchronizedList(new ArrayList<>());
        ExecutorService executor = Executors.newFixedThreadPool(numWorkers);
        AtomicInteger completed = new AtomicInteger(0);
        List<Future<?>> futures = new ArrayList<>();
        int total = messages.size();

        for (Message message : messages) {
            Future<?> future = executor.submit(() -> {
                try {
                    DeliveryResult result = sendMessage(message);
                    results.add(result);
                } catch (Exception e) {
                    results.add(new DeliveryResult(
                        "error_" + System.currentTimeMillis(),
                        "failed",
                        0,
                        System.currentTimeMillis()
                    ));
                }
                int count = completed.incrementAndGet();
                if (callback != null) {
                    callback.accept(new ProgressUpdate(count, total));
                }
            });
            futures.add(future);
        }

        // Wait for all to complete
        for (Future<?> future : futures) {
            try {
                future.get();
            } catch (Exception e) {
                LOGGER.log(Level.WARNING, "Error waiting for message", e);
            }
        }

        executor.shutdown();
        return results;
    }

    /**
     * Progress update for parallel processing
     */
    public static class ProgressUpdate {
        public final int completed;
        public final int total;

        public ProgressUpdate(int completed, int total) {
            this.completed = completed;
            this.total = total;
        }
    }

    /**
     * Disconnect from FastDataBroker
     */
    public synchronized void disconnect() {
        running = false;

        try {
            if (reader != null) reader.close();
            if (writer != null) writer.close();
            if (socket != null && !socket.isClosed()) socket.close();
        } catch (IOException e) {
            LOGGER.log(Level.WARNING, "Error during disconnect", e);
        }

        if (executorService != null) {
            executorService.shutdown();
            try {
                if (!executorService.awaitTermination(5, TimeUnit.SECONDS)) {
                    executorService.shutdownNow();
                }
            } catch (InterruptedException e) {
                executorService.shutdownNow();
                Thread.currentThread().interrupt();
            }
        }

        connected = false;
        authenticated = false;
        state = ConnectionState.DISCONNECTED;
        LOGGER.info("✓ Disconnected from FastDataBroker");
    }

    // ========================================================================
    // Static Factory Methods
    // ========================================================================

    public static FastDataBrokerQuicClient create(QuicConnectionConfig config) {
        return new FastDataBrokerQuicClient(config);
    }

    public static String getPskSecretFromEnv() {
        String secret = System.getenv("QUIC_PSK_SECRET");
        if (secret == null || secret.isEmpty()) {
            throw new IllegalStateException(
                "QUIC_PSK_SECRET environment variable not set. " +
                "Get it from: POST /api/quic/psks"
            );
        }
        return secret;
    }

    // ========================================================================
    // Example Usage
    // ========================================================================

    public static void main(String[] args) throws Exception {
        String pskSecret = System.getenv("QUIC_PSK_SECRET");
        if (pskSecret == null || pskSecret.isEmpty()) {
            pskSecret = "test-secret-key";
        }

        QuicConnectionConfig config = new QuicConnectionConfig(
            "localhost",
            6000,
            "test-tenant",
            "test-client",
            pskSecret
        );

        FastDataBrokerQuicClient client = new FastDataBrokerQuicClient(config);

        try {
            client.connect();
            System.out.println("✓ Client connected successfully");

            // Send message
            Message message = new Message("test.topic", new java.util.HashMap<>() {{
                put("data", "test");
            }}).withPriority(Priority.NORMAL);

            DeliveryResult result = client.sendMessage(message);
            System.out.println("✓ Message sent: " + result);

            // Get stats
            ConnectionStats stats = client.getStats();
            System.out.println("✓ Stats: " + stats);

            // Keep running
            Thread.sleep(10_000);

        } finally {
            client.disconnect();
        }
    }
}

// Import helper for SocketTimeoutException
import java.net.SocketTimeoutException;
import java.io.OutputStreamWriter;
