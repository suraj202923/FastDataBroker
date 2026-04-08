# FastDataBroker Java SDK

Java SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

## Version
0.1.11

## Features

- 🚀 **Synchronous & Asynchronous APIs** - Both sync and async message sending
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

## Installation

### Maven

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>com.fastdatabroker</groupId>
    <artifactId>fastdatabroker-sdk</artifactId>
    <version>0.1.11</version>
</dependency>
```

### Gradle

```gradle
implementation 'com.fastdatabroker:fastdatabroker-sdk:0.1.11'
```

### From Source

```bash
cd sdks/java
mvn clean install
```

## Quick Start

### Basic Usage

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import com.fastdatabroker.sdk.models.Message;
import com.fastdatabroker.sdk.models.Priority;

public class QuickStart {
    public static void main(String[] args) throws Exception {
        // Create client
        try (FastDataBrokerSDK client = new FastDataBrokerSDK("localhost", 6000)) {
            // Connect to FastDataBroker
            client.connect();

            // Create a message
            Message message = Message.builder()
                .senderId("user-1")
                .addRecipient("user-2")
                .addRecipient("user-3")
                .subject("Hello from Java")
                .content("This is a test message".getBytes(StandardCharsets.UTF_8))
                .priority(Priority.HIGH)
                .build();

            // Send synchronously
            String messageId = client.sendMessage(message);
            System.out.println("Message sent: " + messageId);

            // Or send asynchronously
            client.sendMessageAsync(message)
                .thenAccept(result -> System.out.println("Async message sent: " + result))
                .get();
        }
    }
}
```

### Priority Levels

```java
// Set message priority
message.setPriority(Priority.CRITICAL);    // 255
// Other options:
// - Priority.URGENT (200)
// - Priority.HIGH (150)
// - Priority.NORMAL (100)
// - Priority.DEFERRED (50)
```

### Message TTL (Time-To-Live)

```java
// Message expires after 3600 seconds (1 hour)
message.setTtlSeconds(3600);
```

### Message Tags

```java
message.addTag("region", "us-east");
message.addTag("category", "notification");
message.addTag("version", "v1");
```

### Asynchronous Operations

```java
client.sendMessageAsync(message)
    .thenAccept(result -> {
        System.out.println("Message sent: " + result.getMessageId());
    })
    .exceptionally(ex -> {
        System.err.println("Failed to send message: " + ex.getMessage());
        return null;
    });

// Or with callbacks
client.sendMessageAsync(message, new AsyncCallback() {
    @Override
    public void onSuccess(SendResult result) {
        System.out.println("Message sent: " + result.getMessageId());
    }

    @Override
    public void onFailure(Throwable ex) {
        System.err.println("Failed: " + ex.getMessage());
    }
});
```

### WebSocket Integration

```java
WebSocketClient wsClient = new WebSocketClient("ws://localhost:6001");
wsClient.connect();

// Send message via WebSocket
wsClient.sendMessage(message)
    .thenAccept(result -> System.out.println("Sent: " + result))
    .get();

// Listen for messages
wsClient.subscribe("user-1", message -> {
    System.out.println("Received: " + message.getSubject());
});
```

### Error Handling

```java
try {
    String messageId = client.sendMessage(message);
} catch (ValidationException ex) {
    System.err.println("Invalid message: " + ex.getMessage());
} catch (ConnectionException ex) {
    System.err.println("Connection failed: " + ex.getMessage());
} catch (TimeoutException ex) {
    System.err.println("Request timeout");
} catch (Exception ex) {
    System.err.println("Unexpected error: " + ex.getMessage());
}
```

### Batch Operations

```java
List<Message> messages = Arrays.asList(
    Message.builder().senderId("user-1").addRecipient("user-2").subject("Msg1").build(),
    Message.builder().senderId("user-1").addRecipient("user-3").subject("Msg2").build(),
    Message.builder().senderId("user-1").addRecipient("user-4").subject("Msg3").build()
);

List<SendResult> results = client.sendBatch(messages);
results.forEach(result -> 
    System.out.println("Sent: " + result.getMessageId())
);
```

### Reactive Programming with RxJava

```java
import io.reactivex.rxjava3.core.Observable;

client.sendMessageReactive(message)
    .subscribe(
        result -> System.out.println("Sent: " + result),
        error -> System.err.println("Error: " + error),
        () -> System.out.println("Completed")
    );
```

### Clustering

```java
List<String> nodes = Arrays.asList("node1:6000", "node2:6000", "node3:6000");
ClusterClient clusterClient = new ClusterClient(nodes);
clusterClient.connect();

// Automatic failover and load balancing
String messageId = clusterClient.sendMessage(message);
```

### Message Builder Pattern

```java
Message message = Message.builder()
    .senderId("user-1")
    .addRecipient("user-2")
    .addRecipient("user-3")
    .subject("Important Notification")
    .content("This is the message body".getBytes())
    .priority(Priority.HIGH)
    .ttlSeconds(7200)
    .addTag("source", "system")
    .addTag("type", "notification")
    .requireConfirmation(true)
    .build();
```

## Configuration

### Client Builder

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

client.setLogger(Logger.getLogger("FastDataBroker"));
```

## API Reference

### Message Class

```java
public class Message {
    String messageId;           // Unique identifier
    String senderId;            // Sender ID
    List<String> recipientIds;  // List of recipients
    String subject;             // Message subject
    byte[] content;             // Message body
    Priority priority;          // Priority level
    long ttlSeconds;            // Time to live
    Map<String, String> tags;   // Custom tags
    boolean requireConfirmation;// Request confirmation
    long timestamp;             // Creation timestamp
}
```

### Client Methods

#### sendMessage
Send a message synchronously.
```java
String messageId = client.sendMessage(message);
```

#### sendMessageAsync
Send a message asynchronously.
```java
CompletableFuture<SendResult> future = client.sendMessageAsync(message);
```

#### sendBatch
Send multiple messages.
```java
List<SendResult> results = client.sendBatch(messages);
```

#### subscribe
Subscribe to messages.
```java
client.subscribe("user-1", message -> {
    // Handle message
});
```

#### getMessageStatus
Get message delivery status.
```java
MessageStatus status = client.getMessageStatus(messageId);
```

## Examples

- [Basic Example](examples/BasicExample.java)
- [Async Example](examples/AsyncExample.java)
- [WebSocket Example](examples/WebSocketExample.java)
- [Batch Operations](examples/BatchExample.java)
- [Clustering](examples/ClusteringExample.java)
- [Reactive Example](examples/ReactiveExample.java)

## Error Codes

- **1000**: Connection error
- **1001**: Validation error
- **1002**: Timeout error
- **1003**: Message not found
- **1004**: Authentication failed
- **1005**: Rate limit exceeded

## Testing

```bash
mvn test
mvn test -Dgroups=integration   # Run integration tests
mvn test -Dgroups=performance  # Run performance tests
mvn clean test jacoco:report   # Generate coverage report
```

## Benchmarks

Performance on standard hardware:

| Operation | Throughput | Latency |
|-----------|-----------|---------|
| Send Message | 80k msgs/sec | <2ms |
| Batch Send (100) | 200k msgs/sec | <100ms |
| WebSocket | 40k msgs/sec | <3ms |
| Reactive | 100k msgs/sec | <1ms |

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Support

- 📖 [Documentation](https://github.com/suraj202923/FastDataBroker)
- 🐛 [Issue Tracker](https://github.com/suraj202923/FastDataBroker/issues)
- 💬 [Discussions](https://github.com/suraj202923/FastDataBroker/discussions)

## Changelog

### Version 0.1.11
- Initial Java SDK release
- Synchronous and asynchronous message sending
- Multi-channel delivery support
- WebSocket integration
- RxJava reactive streaming
- Clustering and failover support
- Builder pattern for message creation
