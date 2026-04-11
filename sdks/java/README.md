# FastDataBroker Java SDK

Java SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

**Version:** 0.1.14

## 📋 Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Complete Examples](#complete-examples)
- [API Reference](#api-reference)
- [Error Handling](#error-handling)
- [Advanced Features](#advanced-features)
- [Testing](#testing)

## Features

- 🚀 **Synchronous & Asynchronous APIs** - CompletableFuture-based async support
- 📨 **Multi-Channel Delivery** - Email, WebSocket, Push Notifications, Webhooks
- 🎯 **Priority Levels** - 5 priority levels: Deferred, Normal, High, Urgent, Critical
- 🔄 **Message Confirmation** - Optional delivery confirmation
- 🏷️ **Message Tagging** - Tag messages for categorization
- ⏱️ **TTL Support** - Set time-to-live for messages
- 🔌 **WebSocket Support** - Real-time bidirectional communication
- 🪝 **Webhook Endpoints** - Integrate with external systems
- 🌐 **QUIC Protocol** - High-performance UDP-based protocol
- 🔐 **Clustering Support** - Multi-region failover and load balancing
- 📊 **Reactive Streams** - RxJava integration for reactive programming
- 🏗️ **Builder Pattern** - Fluent API for message construction

## Installation

### Maven

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>com.fastdatabroker</groupId>
    <artifactId>fastdatabroker-sdk</artifactId>
    <version>0.1.14</version>
</dependency>
```

### Gradle

```gradle
implementation 'com.fastdatabroker:fastdatabroker-sdk:0.1.14'
```

### From Source

```bash
cd sdks/java
mvn clean install
```

## Quick Start

### 1. Basic Client Setup

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;

public class QuickStart {
    public static void main(String[] args) throws Exception {
        // Create and connect client
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();
            System.out.println("Connected to FastDataBroker");

            // Your code here

            client.disconnect();
        }
    }
}
```

### 2. Send a Simple Message

```java
// Create a message
Message message = Message.builder()
    .senderId("app-001")
    .addRecipient("user-123")
    .addRecipient("user-456")
    .subject("Notification")
    .content("Hello from FastDataBroker!".getBytes(StandardCharsets.UTF_8))
    .priority(Priority.HIGH)
    .build();

// Send synchronously
String messageId = client.sendMessage(message);
System.out.println("✓ Message sent: " + messageId);
```

### 3. Send Asynchronously

```java
// Send asynchronously
client.sendMessageAsync(message)
    .thenAccept(messageId -> System.out.println("✓ Async message sent: " + messageId))
    .exceptionally(ex -> {
        System.err.println("✗ Error: " + ex.getMessage());
        return null;
    });
```

## Complete Examples

### Example 1: Priority-Based Messaging

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import java.nio.charset.StandardCharsets;

public class PriorityExample {
    public static void main(String[] args) throws Exception {
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();

            // Critical priority message
            Message criticalMsg = Message.builder()
                .senderId("system")
                .addRecipient("admin")
                .subject("CRITICAL: System Alert")
                .content("Immediate action required".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.CRITICAL)  // 255
                .build();

            String id1 = client.sendMessage(criticalMsg);
            System.out.println("✓ Critical message sent: " + id1);

            // Urgent priority message
            Message urgentMsg = Message.builder()
                .senderId("system")
                .addRecipient("manager")
                .subject("URGENT: Important Update")
                .content("Requires immediate attention".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.URGENT)  // 200
                .build();

            String id2 = client.sendMessage(urgentMsg);
            System.out.println("✓ Urgent message sent: " + id2);

            // Normal priority message
            Message normalMsg = Message.builder()
                .senderId("system")
                .addRecipient("user")
                .subject("Regular Update")
                .content("Routine notification".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.NORMAL)  // 100
                .build();

            String id3 = client.sendMessage(normalMsg);
            System.out.println("✓ Normal message sent: " + id3);

            // Deferred priority message
            Message deferredMsg = Message.builder()
                .senderId("system")
                .addRecipient("background-worker")
                .subject("Background Task")
                .content("Can be processed later".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.DEFERRED)  // 50
                .build();

            String id4 = client.sendMessage(deferredMsg);
            System.out.println("✓ Deferred message sent: " + id4);
        }
    }
}
```

### Example 2: Batch Message Sending with TTL

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

public class BatchMessagesExample {
    public static void main(String[] args) throws Exception {
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();

            List<Message> messages = new ArrayList<>();

            // Create 5 messages with different TTLs
            for (int i = 0; i < 5; i++) {
                Message msg = Message.builder()
                    .senderId("batch-sender")
                    .addRecipient("recipient-" + i)
                    .subject("Batch Message " + (i + 1))
                    .content(("Content for message " + (i + 1)).getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.HIGH)
                    .ttlSeconds(3600L * (i + 1))  // 1, 2, 3, 4, 5 hours
                    .requireConfirmation(true)
                    .build();
                messages.add(msg);
            }

            // Send all messages
            for (Message msg : messages) {
                try {
                    String messageId = client.sendMessage(msg);
                    System.out.printf("✓ Message sent - ID: %s, TTL: %ds%n", 
                        messageId, msg.getTtlSeconds());
                } catch (Exception ex) {
                    System.err.println("✗ Failed: " + ex.getMessage());
                }
            }
        }
    }
}
```

### Example 3: Tagged Messages for Organization

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import java.nio.charset.StandardCharsets;
import java.time.Instant;

public class TaggedMessagesExample {
    public static void main(String[] args) throws Exception {
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();

            Message message = Message.builder()
                .senderId("order-service")
                .addRecipient("customer-789")
                .subject("Order Confirmation")
                .content("Your order has been confirmed".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.HIGH)
                .addTag("order-id", "ORD-2024-001234")
                .addTag("region", "us-west-2")
                .addTag("category", "order-notification")
                .addTag("version", "v2")
                .addTag("timestamp", Instant.now().toString())
                .build();

            String messageId = client.sendMessage(message);
            System.out.println("✓ Tagged message sent: " + messageId);
            System.out.println("  Tags:");
            message.getTags().forEach((k, v) -> System.out.printf("    - %s: %s%n", k, v));
        }
    }
}
```

### Example 4: WebSocket Integration

```java
import com.fastdatabroker.sdk.websocket.WebSocketClient;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import java.nio.charset.StandardCharsets;

public class WebSocketExample {
    public static void main(String[] args) throws Exception {
        WebSocketClient wsClient = new WebSocketClient("ws://localhost:6001");
        wsClient.connect();
        System.out.println("✓ WebSocket client connected");

        // Register multiple WebSocket clients
        String[] clientIds = {"client-001", "client-002", "client-003"};
        String[] userIds = {"user-A", "user-B", "user-C"};

        for (int i = 0; i < clientIds.length; i++) {
            wsClient.registerClient(clientIds[i], userIds[i]);
            System.out.println("✓ WebSocket registered: " + clientIds[i] + 
                " -> " + userIds[i]);
        }

        // Send message via WebSocket
        Message message = Message.builder()
            .senderId("system")
            .addRecipient("user-A")
            .addRecipient("user-B")
            .addRecipient("user-C")
            .subject("Real-time Update")
            .content("WebSocket real-time notification".getBytes(StandardCharsets.UTF_8))
            .priority(Priority.URGENT)
            .build();

        String messageId = wsClient.sendMessage(message);
        System.out.println("✓ Message sent to WebSocket clients: " + messageId);

        // Listen for messages
        wsClient.subscribe("user-A", msg -> {
            System.out.println("✓ Received: " + msg.getSubject());
        });

        // Cleanup
        for (String clientId : clientIds) {
            wsClient.unregisterClient(clientId);
            System.out.println("✓ Unregistered: " + clientId);
        }

        wsClient.disconnect();
    }
}
```

### Example 5: Reactive Programming with RxJava

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import io.reactivex.rxjava3.core.Observable;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;

public class ReactiveExample {
    public static void main(String[] args) throws Exception {
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();

            // Create multiple messages
            List<Message> messages = Arrays.asList(
                Message.builder()
                    .senderId("reactive-app")
                    .addRecipient("user-1")
                    .subject("Message 1")
                    .content("Content 1".getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.HIGH)
                    .build(),
                Message.builder()
                    .senderId("reactive-app")
                    .addRecipient("user-2")
                    .subject("Message 2")
                    .content("Content 2".getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.NORMAL)
                    .build(),
                Message.builder()
                    .senderId("reactive-app")
                    .addRecipient("user-3")
                    .subject("Message 3")
                    .content("Content 3".getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.DEFERRED)
                    .build()
            );

            // Send reactively
            Observable.fromIterable(messages)
                .map(msg -> {
                    try {
                        return client.sendMessage(msg);
                    } catch (Exception e) {
                        throw new RuntimeException(e);
                    }
                })
                .subscribe(
                    messageId -> System.out.println("✓ Sent: " + messageId),
                    error -> System.err.println("✗ Error: " + error),
                    () -> System.out.println("✓ Completed all messages")
                );
        }
    }
}
```

### Example 6: Batch Operations with Futures

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;

public class AsyncBatchExample {
    public static void main(String[] args) throws Exception {
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();

            List<CompletableFuture<String>> futures = new ArrayList<>();

            // Send messages asynchronously
            for (int i = 0; i < 5; i++) {
                Message msg = Message.builder()
                    .senderId("async-app")
                    .addRecipient("user-" + i)
                    .subject("Async Message " + i)
                    .content(("Content " + i).getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.HIGH)
                    .build();

                CompletableFuture<String> future = client.sendMessageAsync(msg);
                futures.add(future);
            }

            // Wait for all to complete
            CompletableFuture.allOf(futures.toArray(new CompletableFuture[0]))
                .thenRun(() -> {
                    System.out.println("✓ All messages sent:");
                    futures.forEach(f -> {
                        try {
                            System.out.println("  - " + f.get());
                        } catch (Exception e) {
                            System.err.println("  ✗ Error: " + e.getMessage());
                        }
                    });
                })
                .exceptionally(ex -> {
                    System.err.println("✗ Failed: " + ex.getMessage());
                    return null;
                })
                .get();  // Wait for completion
        }
    }
}
```

### Example 7: Complete End-to-End Application

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.time.temporal.ChronoUnit;

public class CompleteApplicationExample {
    public static void main(String[] args) throws Exception {
        System.out.println("=== FastDataBroker Java SDK Complete Example ===\n");

        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            // 1. Connection
            System.out.println("1. Connecting to FastDataBroker...");
            client.connect();
            System.out.println("✓ Connected successfully\n");

            // 2. Send critical message with TTL
            System.out.println("2. Sending critical priority message...");
            Instant startTime = Instant.now();

            Message criticalMsg = Message.builder()
                .senderId("app")
                .addRecipient("user-123")
                .subject("Critical Alert")
                .content("This is critical".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.CRITICAL)
                .ttlSeconds(3600L)
                .addTag("severity", "critical")
                .addTag("timestamp", Instant.now().toString())
                .requireConfirmation(true)
                .build();

            String messageId = client.sendMessage(criticalMsg);
            long duration = ChronoUnit.MILLIS.between(startTime, Instant.now());
            System.out.printf("✓ Message sent: %s (took %dms)%n", messageId, duration);
            System.out.println();

            // 3. Send batch messages
            System.out.println("3. Sending batch messages...");
            for (int i = 0; i < 3; i++) {
                Message msg = Message.builder()
                    .senderId("batch-app")
                    .addRecipient("user-" + i)
                    .subject("Batch message " + (i + 1))
                    .content(("Content " + (i + 1)).getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.NORMAL)
                    .ttlSeconds(7200L)
                    .build();

                String id = client.sendMessage(msg);
                System.out.printf("  ✓ Message %d: %s%n", i + 1, id);
            }
            System.out.println();

            // 4. Send async messages
            System.out.println("4. Sending async messages...");
            for (int i = 0; i < 2; i++) {
                Message msg = Message.builder()
                    .senderId("async-app")
                    .addRecipient("user-async-" + i)
                    .subject("Async message " + (i + 1))
                    .content(("Async content " + (i + 1)).getBytes(StandardCharsets.UTF_8))
                    .priority(Priority.HIGH)
                    .build();

                client.sendMessageAsync(msg)
                    .thenAccept(id -> System.out.printf("  ✓ Async message sent: %s%n", id))
                    .exceptionally(ex -> {
                        System.err.printf("  ✗ Failed: %s%n", ex.getMessage());
                        return null;
                    })
                    .get();  // Wait for this message
            }
            System.out.println();

            // 5. Statistics
            System.out.println("5. Statistics:");
            System.out.println("  ✓ All messages sent successfully");
            System.out.println("  ✓ Client connected: true");

            // 6. Cleanup
            System.out.println("\n6. Cleaning up...");
            client.disconnect();
            System.out.println("✓ Disconnected\n");

            System.out.println("=== Example completed successfully ===");
        } catch (Exception ex) {
            System.err.println("✗ Error: " + ex.getMessage());
            ex.printStackTrace();
        }
    }
}
```

## API Reference

### Priority Enum

```java
public enum Priority {
    DEFERRED(50),       // Low priority, can be delayed
    NORMAL(100),        // Standard priority (default)
    HIGH(150),          // Higher priority
    URGENT(200),        // Very high priority
    CRITICAL(255);      // Critical, process immediately

    private final int value;
    Priority(int value) { this.value = value; }
    public int getValue() { return value; }
}
```

### NotificationChannel Enum

```java
public enum NotificationChannel {
    EMAIL("email"),           // Email delivery
    WEBSOCKET("websocket"),   // WebSocket push
    PUSH("push"),             // Push notifications
    WEBHOOK("webhook");       // Webhook callback

    private final String channel;
    NotificationChannel(String channel) { this.channel = channel; }
    public String getChannel() { return channel; }
}
```

### Message Class

```java
public class Message {
    // Builder pattern for construction
    public static Builder builder() { return new Builder(); }
    
    // Getters
    public String getSenderId();
    public List<String> getRecipientIds();
    public String getSubject();
    public byte[] getContent();
    public Priority getPriority();
    public Long getTtlSeconds();
    public Map<String, String> getTags();
    public boolean isRequireConfirmation();
    
    // Builder
    public static class Builder {
        public Builder senderId(String senderId);
        public Builder addRecipient(String recipientId);
        public Builder subject(String subject);
        public Builder content(byte[] content);
        public Builder priority(Priority priority);
        public Builder ttlSeconds(Long ttlSeconds);
        public Builder addTag(String key, String value);
        public Builder requireConfirmation(boolean require);
        public Message build();
    }
}
```

### Client Methods

```java
public class FastDataBrokerSDK implements AutoCloseable {
    // Constructor
    public FastDataBrokerSDK(String host, int port);

    // Connection methods
    public void connect() throws Exception;
    public void disconnect() throws Exception;
    public boolean isConnected();

    // Message sending (sync)
    public String sendMessage(Message message) throws Exception;

    // Message sending (async)
    public CompletableFuture<String> sendMessageAsync(Message message);

    // Batch operations
    public List<String> sendBatch(List<Message> messages) throws Exception;

    // Resource management
    public void close() throws Exception;
}
```

## Error Handling

### Custom Exceptions

```java
// ValidationException - Invalid message parameters
try {
    client.sendMessage(invalidMessage);
} catch (ValidationException ex) {
    System.err.println("Invalid message: " + ex.getMessage());
}

// ConnectionException - Connection failed
try {
    client.connect();
} catch (ConnectionException ex) {
    System.err.println("Connection failed: " + ex.getMessage());
}

// TimeoutException - Request timeout
try {
    client.sendMessage(message);
} catch (TimeoutException ex) {
    System.err.println("Request timed out");
}
```

### Async Error Handling

```java
client.sendMessageAsync(message)
    .thenAccept(messageId -> System.out.println("Sent: " + messageId))
    .exceptionally(ex -> {
        if (ex instanceof ValidationException) {
            System.err.println("Validation error");
        } else if (ex instanceof ConnectionException) {
            System.err.println("Connection error");
        } else {
            System.err.println("Unexpected error: " + ex);
        }
        return null;
    });
```

## Advanced Features

### Configuration

```java
FastDataBrokerSDK client = new FastDataBrokerSDK.Builder()
    .host("localhost")
    .port(6000)
    .timeout(Duration.ofSeconds(30))
    .retries(3)
    .compression(true)
    .encryption(true)
    .connectionPoolSize(10)
    .build();
```

### Logging

```java
import java.util.logging.Logger;

Logger logger = Logger.getLogger("FastDataBrokerSDK");
client.setLogger(logger);
```

## Testing

### Run Tests

```bash
cd sdks/java
mvn test
mvn test -Dtest=TestClassName
```

### Unit Test Example

```java
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class FastDataBrokerTest {
    @Test
    public void testMessageSending() throws Exception {
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            client.connect();

            Message message = Message.builder()
                .senderId("test")
                .addRecipient("test-recipient")
                .subject("Test")
                .content("Test content".getBytes())
                .priority(Priority.HIGH)
                .build();

            String messageId = client.sendMessage(message);
            assertNotNull(messageId);
            assertTrue(messageId.length() > 0);
        }
    }
}
```

## Requirements

- Java 11 or higher
- Maven 3.6+ or Gradle 6.0+

## Building and Publishing

### Build

```bash
mvn clean package
```

### Run Tests

```bash
mvn test
```

### Deploy to Maven Central

```bash
mvn clean deploy
```

## License

MIT License - See LICENSE file in the repository

## Support

- GitHub Issues: https://github.com/suraj202923/FastDataBroker/issues
- Documentation: https://github.com/suraj202923/FastDataBroker/tree/main/docs
- FastDataBroker Docs: https://fastdatabroker.io/docs

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Changelog

### Version 0.1.13 (Latest)
- Enhanced documentation with complete examples
- WebSocket and reactive programming examples
- Comprehensive error handling patterns
- Batch async operations with CompletableFuture
- RxJava integration examples
- Builder pattern for message construction

### Version 0.1.12
- Initial Java SDK release
- Synchronous and asynchronous APIs
- Priority-based message routing
- TTL and tagging support
- QUIC protocol integration
