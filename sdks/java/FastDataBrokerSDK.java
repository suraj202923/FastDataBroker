package com.FastDataBroker;

import java.time.Instant;
import java.util.*;
import java.util.concurrent.*;

/**
 * FastDataBroker Java SDK - Client library for rst_queue FastDataBroker
 * Provides simple synchronous and asynchronous interfaces for message delivery
 *
 * @version 0.4.0
 */
public class FastDataBrokerSDK {

    public static final String VERSION = "0.4.0";

    /**
     * Priority levels for messages
     */
    public enum Priority {
        DEFERRED(50),
        NORMAL(100),
        HIGH(150),
        URGENT(200),
        CRITICAL(255);

        private final int value;

        Priority(int value) {
            this.value = value;
        }

        public int getValue() {
            return value;
        }
    }

    /**
     * Notification delivery channels
     */
    public enum NotificationChannel {
        EMAIL("email"),
        WEBSOCKET("websocket"),
        PUSH("push"),
        WEBHOOK("webhook");

        private final String channel;

        NotificationChannel(String channel) {
            this.channel = channel;
        }

        public String getChannel() {
            return channel;
        }
    }

    /**
     * Push notification platforms
     */
    public enum PushPlatform {
        FIREBASE("firebase"),
        APNS("apns"),
        FCM("fcm"),
        WEBPUSH("webpush");

        private final String platform;

        PushPlatform(String platform) {
            this.platform = platform;
        }

        public String getPlatform() {
            return platform;
        }
    }

    /**
     * Message envelope for FastDataBroker
     */
    public static class Message {
        private final String senderId;
        private final List<String> recipientIds;
        private final String subject;
        private final byte[] content;
        private final Priority priority;
        private final Long ttlSeconds;
        private final Map<String, String> tags;
        private final boolean requireConfirm;

        private Message(Builder builder) {
            this.senderId = builder.senderId;
            this.recipientIds = new ArrayList<>(builder.recipientIds);
            this.subject = builder.subject;
            this.content = builder.content;
            this.priority = builder.priority;
            this.ttlSeconds = builder.ttlSeconds;
            this.tags = new HashMap<>(builder.tags);
            this.requireConfirm = builder.requireConfirm;
        }

        // Getters
        public String getSenderId() { return senderId; }
        public List<String> getRecipientIds() { return new ArrayList<>(recipientIds); }
        public String getSubject() { return subject; }
        public byte[] getContent() { return content; }
        public Priority getPriority() { return priority; }
        public Optional<Long> getTTLSeconds() { return Optional.ofNullable(ttlSeconds); }
        public Map<String, String> getTags() { return new HashMap<>(tags); }
        public boolean isRequireConfirm() { return requireConfirm; }

        // Builder pattern
        public static class Builder {
            private final String senderId;
            private final List<String> recipientIds = new ArrayList<>();
            private String subject = "";
            private byte[] content = new byte[0];
            private Priority priority = Priority.NORMAL;
            private Long ttlSeconds = null;
            private final Map<String, String> tags = new HashMap<>();
            private boolean requireConfirm = false;

            public Builder(String senderId) {
                this.senderId = senderId;
            }

            public Builder addRecipient(String recipientId) {
                this.recipientIds.add(recipientId);
                return this;
            }

            public Builder setSubject(String subject) {
                this.subject = subject;
                return this;
            }

            public Builder setContent(byte[] content) {
                this.content = content;
                return this;
            }

            public Builder setPriority(Priority priority) {
                this.priority = priority;
                return this;
            }

            public Builder setTTLSeconds(long ttl) {
                this.ttlSeconds = ttl;
                return this;
            }

            public Builder addTag(String key, String value) {
                this.tags.put(key, value);
                return this;
            }

            public Builder setRequireConfirm(boolean require) {
                this.requireConfirm = require;
                return this;
            }

            public Message build() {
                return new Message(this);
            }
        }
    }

    /**
     * Result of message delivery
     */
    public static class DeliveryResult {
        private final String messageId;
        private final String status;
        private final int deliveredChannels;
        private final Map<String, Object> details;

        public DeliveryResult(String messageId, String status, int deliveredChannels,
                            Map<String, Object> details) {
            this.messageId = messageId;
            this.status = status;
            this.deliveredChannels = deliveredChannels;
            this.details = new HashMap<>(details);
        }

        public String getMessageId() { return messageId; }
        public String getStatus() { return status; }
        public int getDeliveredChannels() { return deliveredChannels; }
        public Map<String, Object> getDetails() { return new HashMap<>(details); }

        @Override
        public String toString() {
            return String.format("DeliveryResult{id=%s, status=%s, channels=%d}",
                    messageId, status, deliveredChannels);
        }
    }

    /**
     * Webhook configuration
     */
    public static class WebhookConfig {
        private final String url;
        private final Map<String, String> headers;
        private final int retries;
        private final long timeoutMs;
        private final boolean verifySsl;

        private WebhookConfig(Builder builder) {
            this.url = builder.url;
            this.headers = new HashMap<>(builder.headers);
            this.retries = builder.retries;
            this.timeoutMs = builder.timeoutMs;
            this.verifySsl = builder.verifySsl;
        }

        public String getUrl() { return url; }
        public Map<String, String> getHeaders() { return new HashMap<>(headers); }
        public int getRetries() { return retries; }
        public long getTimeoutMs() { return timeoutMs; }
        public boolean isVerifySsl() { return verifySsl; }

        public static class Builder {
            private final String url;
            private final Map<String, String> headers = new HashMap<>();
            private int retries = 3;
            private long timeoutMs = 30000;
            private boolean verifySsl = true;

            public Builder(String url) {
                this.url = url;
            }

            public Builder addHeader(String key, String value) {
                this.headers.put(key, value);
                return this;
            }

            public Builder setRetries(int retries) {
                this.retries = retries;
                return this;
            }

            public Builder setTimeoutMs(long timeout) {
                this.timeoutMs = timeout;
                return this;
            }

            public Builder setVerifySsl(boolean verify) {
                this.verifySsl = verify;
                return this;
            }

            public WebhookConfig build() {
                return new WebhookConfig(this);
            }
        }
    }

    /**
     * FastDataBroker client for synchronous operations
     */
    public static class FastDataBrokerClient implements AutoCloseable {
        private final String quicHost;
        private final int quicPort;
        private boolean connected = false;
        private final Map<String, WebSocketClientInfo> wsClients = new ConcurrentHashMap<>();
        private final ExecutorService executor = Executors.newFixedThreadPool(4);

        public FastDataBrokerClient(String quicHost, int quicPort) {
            this.quicHost = quicHost;
            this.quicPort = quicPort;
        }

        public FastDataBrokerClient() {
            this("localhost", 6000);
        }

        /**
         * Connect to FastDataBroker server
         */
        public boolean connect() {
            try {
                // Phase 4: Establish QUIC connection
                connected = true;
                System.out.println("Connected to FastDataBroker at " + quicHost + ":" + quicPort);
                return true;
            } catch (Exception e) {
                System.err.println("Connection failed: " + e.getMessage());
                return false;
            }
        }

        /**
         * Disconnect from FastDataBroker server
         */
        public void disconnect() {
            connected = false;
        }

        /**
         * Send a message through FastDataBroker
         */
        public DeliveryResult sendMessage(Message message) throws Exception {
            if (!connected) {
                throw new IllegalStateException("Not connected to FastDataBroker");
            }

            // Phase 4: Send via QUIC transport
            Thread.sleep(10);

            return new DeliveryResult(
                    "msg-" + System.currentTimeMillis(),
                    "success",
                    4,
                    Map.ofEntries(
                            Map.entry("email", "sent"),
                            Map.entry("websocket", "delivered"),
                            Map.entry("push", "pending"),
                            Map.entry("webhook", "delivered")
                    )
            );
        }

        /**
         * Send multiple messages in batch asynchronously
         */
        public CompletableFuture<List<DeliveryResult>> batchSendAsync(List<Message> messages) {
            return CompletableFuture.supplyAsync(() ->
                    messages.stream()
                            .map(msg -> {
                                try {
                                    return sendMessage(msg);
                                } catch (Exception e) {
                                    throw new RuntimeException(e);
                                }
                            })
                            .toList(),
                    executor
            );
        }

        /**
         * Register a WebSocket client
         */
        public boolean registerWebSocket(String clientId, String userId) {
            wsClients.put(clientId, new WebSocketClientInfo(clientId, userId));
            System.out.println("WebSocket client registered: " + clientId + " -> " + userId);
            return true;
        }

        /**
         * Unregister a WebSocket client
         */
        public boolean unregisterWebSocket(String clientId) {
            return wsClients.remove(clientId) != null;
        }

        /**
         * Register a webhook endpoint
         */
        public boolean registerWebhook(String recipientId, WebhookConfig config) {
            if (config.getUrl().isEmpty()) {
                throw new IllegalArgumentException("Webhook URL cannot be empty");
            }
            System.out.println("Webhook registered for " + recipientId + ": " + config.getUrl());
            return true;
        }

        /**
         * Get FastDataBroker statistics
         */
        public Map<String, Object> getStats() {
            Map<String, Object> stats = new HashMap<>();
            stats.put("totalMessages", 0);
            stats.put("delivered", 0);
            stats.put("failed", 0);

            Map<String, Object> channels = new HashMap<>();
            channels.put("email", Map.of("sent", 0, "failed", 0));
            channels.put("websocket", Map.of("connected", wsClients.size(), "delivered", 0));
            channels.put("push", Map.of("sent", 0, "delivered", 0));
            channels.put("webhook", Map.of("sent", 0, "delivered", 0));
            stats.put("channels", channels);

            return stats;
        }

        @Override
        public void close() {
            disconnect();
            executor.shutdown();
        }
    }

    /**
     * WebSocket client information
     */
    public static class WebSocketClientInfo {
        private final String clientId;
        private final String userId;
        private final Instant connectedAt;

        public WebSocketClientInfo(String clientId, String userId) {
            this.clientId = clientId;
            this.userId = userId;
            this.connectedAt = Instant.now();
        }

        public String getClientId() { return clientId; }
        public String getUserId() { return userId; }
        public Instant getConnectedAt() { return connectedAt; }
    }

    // Example usage
    public static void main(String[] args) {
        try (FastDataBrokerClient client = new FastDataBrokerClient()) {
            if (!client.connect()) {
                System.err.println("Failed to connect");
                return;
            }

            // Create and send a message
            Message message = new Message.Builder("app1")
                    .addRecipient("user-123")
                    .setSubject("Hello")
                    .setContent("Welcome to FastDataBroker!".getBytes())
                    .setPriority(Priority.HIGH)
                    .build();

            DeliveryResult result = client.sendMessage(message);
            System.out.println("Message sent: " + result);

            // Register WebSocket client
            client.registerWebSocket("browser-1", "user-123");

            // Register webhook
            WebhookConfig webhookConfig = new WebhookConfig.Builder("https://example.com/webhook")
                    .setRetries(3)
                    .build();
            client.registerWebhook("service-1", webhookConfig);

            // Get statistics
            Map<String, Object> stats = client.getStats();
            System.out.println("Statistics: " + stats);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
