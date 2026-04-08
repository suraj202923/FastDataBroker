# FastDataBroker Go SDK

Go SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

## Version
0.1.12

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

## Installation

### Using Go Modules

```bash
go get github.com/suraj202923/FastDataBroker-go
```

### From Source

```bash
cd sdks/go
go mod download
go build
```

## Quick Start

### Basic Usage

```go
package main

import (
	"context"
	"fmt"
	"log"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	// Create client
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	// Connect to FastDataBroker
	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		log.Fatal(err)
	}

	// Create a message
	message := &fastdatabroker.Message{
		SenderId:     "user-1",
		RecipientIds: []string{"user-2", "user-3"},
		Subject:      "Hello from Go",
		Content:      []byte("This is a test message"),
		Priority:     fastdatabroker.PriorityHigh,
	}

	// Send message
	result, err := client.SendMessage(ctx, message)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Message sent: %s\n", result.MessageId)

	// Send async
	asyncResult, err := client.SendMessageAsync(ctx, message)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Async message sent: %s\n", asyncResult.MessageId)
}
```

### Priority Levels

```go
// Set message priority
message.Priority = fastdatabroker.PriorityCritical // 255
// Other options:
// - PriorityUrgent (200)
// - PriorityHigh (150)
// - PriorityNormal (100)
// - PriorityDeferred (50)
```

### Message TTL (Time-To-Live)

```go
// Message expires after 3600 seconds (1 hour)
message.TTLSeconds = 3600
```

### Message Tags

```go
message.Tags = map[string]string{
	"region":   "us-east",
	"category": "notification",
	"version":  "v1",
}
```

### WebSocket Integration

```go
// Connect via WebSocket
wsClient := fastdatabroker.NewWebSocketClient("ws://localhost:6001")
if err := wsClient.Connect(ctx); err != nil {
	log.Fatal(err)
}

// Send message via WebSocket
result, err := wsClient.SendMessage(ctx, message)
if err != nil {
	log.Fatal(err)
}

// Listen for messages
ch := wsClient.Subscribe(ctx, "user-1")
for msg := range ch {
	fmt.Printf("Received: %s\n", msg.Subject)
}
```

### Error Handling

```go
import "github.com/suraj202923/FastDataBroker-go/errors"

result, err := client.SendMessage(ctx, message)
if err != nil {
	switch err.(type) {
	case *errors.ConnectionError:
		log.Print("Connection failed, retrying...")
	case *errors.ValidationError:
		log.Print("Invalid message parameters")
	case *errors.TimeoutError:
		log.Print("Message delivery timeout")
	default:
		log.Fatal(err)
	}
}
```

### Batch Operations

```go
// Send multiple messages in batch
messages := []*fastdatabroker.Message{
	{SenderId: "user-1", RecipientIds: []string{"user-2"}},
	{SenderId: "user-1", RecipientIds: []string{"user-3"}},
	{SenderId: "user-1", RecipientIds: []string{"user-4"}},
}

results, err := client.SendBatch(ctx, messages)
if err != nil {
	log.Fatal(err)
}

for _, result := range results {
	fmt.Printf("Message ID: %s, Status: %v\n", result.MessageId, result.Status)
}
```

### Clustering

```go
// Connect to cluster
clusterClient := fastdatabroker.NewClusterClient(
	[]string{"node1:6000", "node2:6000", "node3:6000"},
)
defer clusterClient.Close()

// Automatic failover and load balancing
result, err := clusterClient.SendMessage(ctx, message)
if err != nil {
	log.Fatal(err)
}
```

### QUIC Protocol Support

```go
// Use QUIC for faster, multiplexed connections
quicClient := fastdatabroker.NewQUICClient("localhost", 6002)
if err := quicClient.Connect(ctx); err != nil {
	log.Fatal(err)
}

result, err := quicClient.SendMessage(ctx, message)
if err != nil {
	log.Fatal(err)
}
```

## Configuration

### Client Options

```go
client := fastdatabroker.NewClient("localhost", 6000,
	fastdatabroker.WithTimeout(30*time.Second),
	fastdatabroker.WithRetries(3),
	fastdatabroker.WithCompression(true),
	fastdatabroker.WithEncryption(true),
)
```

### Logging

```go
import "log"

client := fastdatabroker.NewClient("localhost", 6000,
	fastdatabroker.WithLogger(log.New(os.Stdout, "FDB: ", log.LstdFlags)),
)
```

## API Reference

### Message Structure

```go
type Message struct {
	MessageId     string            // Unique message identifier
	SenderId      string            // Sender identifier
	RecipientIds  []string          // List of recipient identifiers
	Subject       string            // Message subject
	Content       []byte            // Message body
	Priority      Priority          // Message priority (0-255)
	TTLSeconds    int64             // Time to live in seconds
	Tags          map[string]string // Custom message tags
	Confirmation  bool              // Request delivery confirmation
	Timestamp     int64             // Message timestamp
}
```

### Client Methods

#### SendMessage
Send a message synchronously.
```go
result, err := client.SendMessage(ctx, message)
```

#### SendMessageAsync
Send a message asynchronously.
```go
result, err := client.SendMessageAsync(ctx, message)
```

#### SendBatch
Send multiple messages in one request.
```go
results, err := client.SendBatch(ctx, messages)
```

#### Subscribe
Subscribe to messages for a user.
```go
ch := client.Subscribe(ctx, userId)
```

#### GetMessageStatus
Get the status of a sent message.
```go
status, err := client.GetMessageStatus(ctx, messageId)
```

## Examples

- [Basic Example](examples/basic.go)
- [WebSocket Example](examples/websocket.go)
- [Batch Operations](examples/batch.go)
- [Clustering](examples/cluster.go)
- [Error Handling](examples/error_handling.go)

## Error Codes

- **1000**: Connection error
- **1001**: Validation error
- **1002**: Timeout error
- **1003**: Message not found
- **1004**: Authentication failed
- **1005**: Rate limit exceeded

## Testing

```bash
go test ./...
go test -race ./...        # Test with race detector
go test -bench ./...       # Run benchmarks
go test -cover ./...       # Check coverage
```

## Benchmarks

Performance comparison on standard hardware:

| Operation | Throughput | Latency |
|-----------|-----------|---------|
| Send Message | 100k msgs/sec | <1ms |
| Batch Send (100) | 250k msgs/sec | <50ms |
| WebSocket | 50k msgs/sec | <2ms |

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Support

- 📖 [Documentation](https://github.com/suraj202923/FastDataBroker)
- 🐛 [Issue Tracker](https://github.com/suraj202923/FastDataBroker/issues)
- 💬 [Discussions](https://github.com/suraj202923/FastDataBroker/discussions)

## Changelog

### Version 0.1.12
- Initial Go SDK release (v0.1.12)
- Synchronous and asynchronous message sending
- Multi-channel delivery support
- WebSocket integration
- Clustering and failover support
- QUIC protocol implementation
