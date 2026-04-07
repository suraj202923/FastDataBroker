package com.FastDataBroker.Test;

import com.FastDataBroker.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import org.mockito.Mock;

import java.time.Instant;
import java.util.*;
import java.util.concurrent.*;
import java.util.stream.IntStream;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Comprehensive test suite for FastDataBroker Java SDK
 * Tests message delivery, priority handling, notifications, and edge cases
 */
@DisplayName("FastDataBroker Java SDK Tests")
public class FastDataBrokerSDKTest {

    private FastDataBrokerSDK.Client client;
    private static final String HOST = "localhost";
    private static final int PORT = 6000;

    @BeforeEach
    public void setUp() {
        client = new FastDataBrokerSDK.Client(HOST, PORT);
    }

    // ============== Client Initialization Tests ==============

    @Test
    @DisplayName("Test client initialization with host and port")
    public void testClientInitialization() {
        assertEquals(HOST, client.getHost());
        assertEquals(PORT, client.getPort());
        assertFalse(client.isConnected());
    }

    @Test
    @DisplayName("Test client initialization with defaults")
    public void testClientDefaultInitialization() {
        FastDataBrokerSDK.Client defaultClient = new FastDataBrokerSDK.Client();
        
        assertEquals("localhost", defaultClient.getHost());
        assertEquals(6000, defaultClient.getPort());
    }

    @Test
    @DisplayName("Test client with custom configuration")
    public void testClientCustomConfig() {
        FastDataBrokerSDK.Client customClient = new FastDataBrokerSDK.Client("192.168.1.1", 7000);
        
        assertEquals("192.168.1.1", customClient.getHost());
        assertEquals(7000, customClient.getPort());
    }

    @Test
    @DisplayName("Test client connection")
    public void testClientConnection() {
        boolean connected = client.connect();
        assertTrue(connected);
    }

    @Test
    @DisplayName("Test client disconnection")
    public void testClientDisconnection() throws Exception {
        client.connect();
        boolean disconnected = client.disconnect();
        assertTrue(disconnected);
    }

    // ============== Message Tests ==============

    @Test
    @DisplayName("Test message creation")
    public void testMessageCreation() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender1")
                .recipientID("user1")
                .recipientID("user2")
                .subject("Test Subject")
                .content("Test content".getBytes())
                .priority(FastDataBrokerSDK.Priority.HIGH)
                .build();

        assertEquals("sender1", msg.getSenderID());
        assertEquals(2, msg.getRecipientIDs().size());
        assertEquals("Test Subject", msg.getSubject());
        assertEquals(FastDataBrokerSDK.Priority.HIGH, msg.getPriority());
    }

    @Test
    @DisplayName("Test message with default priority")
    public void testMessageDefaultPriority() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content("content".getBytes())
                .build();

        assertEquals(FastDataBrokerSDK.Priority.NORMAL, msg.getPriority());
    }

    @Test
    @DisplayName("Test message with TTL")
    public void testMessageWithTTL() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content("content".getBytes())
                .ttlSeconds(3600)
                .build();

        assertEquals(3600L, msg.getTtlSeconds().longValue());
    }

    @Test
    @DisplayName("Test message with tags")
    public void testMessageWithTags() {
        Map<String, String> tags = new HashMap<>();
        tags.put("category", "notification");
        tags.put("source", "api");

        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content("content".getBytes())
                .tags(tags)
                .build();

        assertEquals(2, msg.getTags().size());
        assertEquals("notification", msg.getTags().get("category"));
    }

    @Test
    @DisplayName("Test message with empty content")
    public void testMessageEmptyContent() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content(new byte[0])
                .build();

        assertEquals(0, msg.getContent().length);
    }

    @Test
    @DisplayName("Test message with large content")
    public void testMessageLargeContent() {
        byte[] largeContent = new byte[10 * 1024 * 1024]; // 10MB
        
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content(largeContent)
                .build();

        assertEquals(10 * 1024 * 1024, msg.getContent().length);
    }

    @Test
    @DisplayName("Test message with multiple recipients")
    public void testMessageMultipleRecipients() {
        FastDataBrokerSDK.Message.MessageBuilder builder = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .subject("Subject")
                .content("content".getBytes());

        for (int i = 0; i < 100; i++) {
            builder.recipientID("user" + i);
        }

        FastDataBrokerSDK.Message msg = builder.build();

        assertEquals(100, msg.getRecipientIDs().size());
    }

    @Test
    @DisplayName("Test message with no recipients")
    public void testMessageNoRecipients() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .subject("Subject")
                .content("content".getBytes())
                .build();

        assertEquals(0, msg.getRecipientIDs().size());
    }

    // ============== Priority Tests ==============

    @Test
    @DisplayName("Test all priority levels")
    public void testPriorityLevels() {
        assertEquals(50, FastDataBrokerSDK.Priority.DEFERRED.getValue());
        assertEquals(100, FastDataBrokerSDK.Priority.NORMAL.getValue());
        assertEquals(150, FastDataBrokerSDK.Priority.HIGH.getValue());
        assertEquals(200, FastDataBrokerSDK.Priority.URGENT.getValue());
        assertEquals(255, FastDataBrokerSDK.Priority.CRITICAL.getValue());
    }

    @Test
    @DisplayName("Test priority ordering")
    public void testPriorityOrdering() {
        List<FastDataBrokerSDK.Priority> priorities = Arrays.asList(
                FastDataBrokerSDK.Priority.DEFERRED,
                FastDataBrokerSDK.Priority.CRITICAL,
                FastDataBrokerSDK.Priority.NORMAL,
                FastDataBrokerSDK.Priority.HIGH,
                FastDataBrokerSDK.Priority.URGENT
        );

        priorities.sort((p1, p2) -> Integer.compare(p1.getValue(), p2.getValue()));

        assertEquals(FastDataBrokerSDK.Priority.DEFERRED, priorities.get(0));
        assertEquals(FastDataBrokerSDK.Priority.CRITICAL, priorities.get(4));
    }

    // ============== Notification Tests ==============

    @Test
    @DisplayName("Test email notification")
    public void testEmailNotification() {
        FastDataBrokerSDK.EmailNotification email = 
                new FastDataBrokerSDK.EmailNotification(
                        "user@example.com",
                        "Subject",
                        "Body"
                );

        assertEquals("user@example.com", email.getRecipientEmail());
        assertEquals("Subject", email.getSubject());
        assertEquals("Body", email.getBody());
    }

    @Test
    @DisplayName("Test push notification")
    public void testPushNotification() {
        FastDataBrokerSDK.PushNotification push = 
                new FastDataBrokerSDK.PushNotification(
                        "device123",
                        "Title",
                        "Body",
                        FastDataBrokerSDK.PushPlatform.FIREBASE
                );

        assertEquals("device123", push.getDeviceToken());
        assertEquals("Title", push.getTitle());
        assertEquals("Body", push.getBody());
        assertEquals(FastDataBrokerSDK.PushPlatform.FIREBASE, push.getPlatform());
    }

    @Test
    @DisplayName("Test webhook notification")
    public void testWebhookNotification() {
        Map<String, Object> payload = new HashMap<>();
        payload.put("key", "value");
        payload.put("status", "success");

        FastDataBrokerSDK.WebhookNotification webhook = 
                new FastDataBrokerSDK.WebhookNotification(
                        "https://example.com/webhook",
                        payload
                );

        assertEquals("https://example.com/webhook", webhook.getWebhookUrl());
        assertEquals(2, webhook.getPayload().size());
    }

    @Test
    @DisplayName("Test websocket notification")
    public void testWebSocketNotification() {
        FastDataBrokerSDK.WebSocketNotification ws = 
                new FastDataBrokerSDK.WebSocketNotification(
                        "client123",
                        "Test message"
                );

        assertEquals("client123", ws.getClientID());
        assertEquals("Test message", ws.getMessage());
    }

    // ============== Notification Channel Tests ==============

    @Test
    @DisplayName("Test notification channels")
    public void testNotificationChannels() {
        assertEquals("email", FastDataBrokerSDK.NotificationChannel.EMAIL.getChannel());
        assertEquals("websocket", FastDataBrokerSDK.NotificationChannel.WEBSOCKET.getChannel());
        assertEquals("push", FastDataBrokerSDK.NotificationChannel.PUSH.getChannel());
        assertEquals("webhook", FastDataBrokerSDK.NotificationChannel.WEBHOOK.getChannel());
    }

    @Test
    @DisplayName("Test push platforms")
    public void testPushPlatforms() {
        assertEquals("firebase", FastDataBrokerSDK.PushPlatform.FIREBASE.getPlatform());
        assertEquals("apns", FastDataBrokerSDK.PushPlatform.APNS.getPlatform());
        assertEquals("fcm", FastDataBrokerSDK.PushPlatform.FCM.getPlatform());
        assertEquals("webpush", FastDataBrokerSDK.PushPlatform.WEBPUSH.getPlatform());
    }

    // ============== Delivery Result Tests ==============

    @Test
    @DisplayName("Test delivery result")
    public void testDeliveryResult() {
        Map<String, Object> details = new HashMap<>();
        details.put("email", "sent");
        details.put("push", "sent");

        FastDataBrokerSDK.DeliveryResult result = 
                new FastDataBrokerSDK.DeliveryResult(
                        "msg123",
                        "delivered",
                        2,
                        details
                );

        assertEquals("msg123", result.getMessageID());
        assertEquals("delivered", result.getStatus());
        assertEquals(2, result.getDeliveredChannels());
        assertEquals(2, result.getDetails().size());
    }

    // ============== Send Message Tests ==============

    @Test
    @DisplayName("Test sending message")
    public void testSendMessage() {
        client.connect();
        
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content("content".getBytes())
                .build();

        FastDataBrokerSDK.DeliveryResult result = client.sendMessage(msg);
        
        assertNotNull(result);
    }

    // ============== Concurrency Tests ==============

    @Test
    @DisplayName("Test multiple clients")
    public void testMultipleClients() {
        List<FastDataBrokerSDK.Client> clients = new ArrayList<>();
        
        for (int i = 0; i < 5; i++) {
            clients.add(new FastDataBrokerSDK.Client(HOST, 6000 + i));
        }
        
        assertEquals(5, clients.size());
        assertTrue(clients.stream().allMatch(Objects::nonNull));
    }

    @Test
    @DisplayName("Test concurrent message sending")
    public void testConcurrentMessageSending() throws InterruptedException, ExecutionException {
        client.connect();
        
        ExecutorService executor = Executors.newFixedThreadPool(4);
        List<Future<FastDataBrokerSDK.DeliveryResult>> futures = new ArrayList<>();

        for (int i = 0; i < 10; i++) {
            final int index = i;
            futures.add(executor.submit(() -> {
                FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                        .senderID("sender")
                        .recipientID("user")
                        .subject("Subject " + index)
                        .content(("content " + index).getBytes())
                        .build();
                
                return client.sendMessage(msg);
            }));
        }

        int completedCount = 0;
        for (Future<FastDataBrokerSDK.DeliveryResult> future : futures) {
            if (future.isDone()) {
                completedCount++;
            }
        }

        executor.shutdown();
        assertTrue(executor.awaitTermination(5, TimeUnit.SECONDS));
    }

    // ============== Error Handling Tests ==============

    @Test
    @DisplayName("Test sending message when not connected")
    public void testSendMessageNotConnected() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content("content".getBytes())
                .build();

        // Should handle gracefully
        assertDoesNotThrow(() -> client.sendMessage(msg));
    }

    @Test
    @DisplayName("Test invalid email format")
    public void testInvalidEmailFormat() {
        FastDataBrokerSDK.EmailNotification email = 
                new FastDataBrokerSDK.EmailNotification(
                        "invalid-email",
                        "Subject",
                        "Body"
                );

        assertEquals("invalid-email", email.getRecipientEmail());
    }

    @Test
    @DisplayName("Test register webhook with invalid URL")
    public void testRegisterWebhookInvalidURL() {
        client.connect();
        
        // Should handle invalid URLs
        boolean result = client.registerWebhook("invalid-url");
        
        assertFalse(result);
    }

    // ============== Edge Case Tests ==============

    @Test
    @DisplayName("Test message with special characters")
    public void testMessageSpecialCharacters() {
        String special = "Hello 你好 🚀 مرحبا";
        byte[] content = special.getBytes();

        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Special")
                .content(content)
                .build();

        assertEquals(special, new String(msg.getContent()));
    }

    @Test
    @DisplayName("Test message with null tag values")
    public void testMessageNullTagValues() {
        Map<String, String> tags = new HashMap<>();
        tags.put("key", null);

        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject("Subject")
                .content("content".getBytes())
                .tags(tags)
                .build();

        assertTrue(msg.getTags().containsKey("key"));
    }

    @Test
    @DisplayName("Test extremely long subject")
    public void testExtremelyLongSubject() {
        StringBuilder longSubject = new StringBuilder();
        for (int i = 0; i < 10000; i++) {
            longSubject.append("x");
        }

        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user")
                .subject(longSubject.toString())
                .content("content".getBytes())
                .build();

        assertEquals(10000, msg.getSubject().length());
    }

    @Test
    @DisplayName("Test message builder fluent API")
    public void testMessageBuilderFluent() {
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("sender")
                .recipientID("user1")
                .recipientID("user2")
                .recipientID("user3")
                .subject("Subject")
                .content("content".getBytes())
                .priority(FastDataBrokerSDK.Priority.HIGH)
                .ttlSeconds(3600)
                .requireConfirmation(true)
                .build();

        assertEquals(3, msg.getRecipientIDs().size());
        assertEquals(FastDataBrokerSDK.Priority.HIGH, msg.getPriority());
        assertTrue(msg.isRequireConfirmation());
    }

    @Test
    @DisplayName("Test integration - full message workflow")
    public void testFullMessageWorkflow() {
        client.connect();

        // Create message
        FastDataBrokerSDK.Message msg = FastDataBrokerSDK.Message.builder()
                .senderID("system")
                .recipientID("user1")
                .recipientID("user2")
                .subject("Important Notification")
                .content("This is important".getBytes())
                .priority(FastDataBrokerSDK.Priority.HIGH)
                .tags(Collections.singletonMap("type", "notification"))
                .build();

        // Create notifications
        List<FastDataBrokerSDK.EmailNotification> notifications = new ArrayList<>();
        notifications.add(new FastDataBrokerSDK.EmailNotification("user1@example.com", "Subject", "Body"));
        notifications.add(new FastDataBrokerSDK.EmailNotification("user2@example.com", "Subject", "Body"));

        assertEquals("Important Notification", msg.getSubject());
        assertEquals(2, notifications.size());
    }
}
