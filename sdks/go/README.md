# FastDataBroker Go SDK

Go SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

**Version:** 0.1.15

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
- ⚡ **Context Support** - Full context.Context integration

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

### Go Version Requirements

- Go 1.19 or higher

## Quick Start

### 1. Basic Client Setup

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
		log.Fatal("Connection failed:", err)
	}
	fmt.Println("Connected to FastDataBroker")

	// Your code here

	client.Disconnect(ctx)
}
```

### 2. Send a Simple Message

```go
message := &fastdatabroker.Message{
	SenderID:     "app-001",
	RecipientIDs: []string{"user-123", "user-456"},
	Subject:      "Notification",
	Content:      []byte("Hello from FastDataBroker!"),
	Priority:     fastdatabroker.PriorityHigh,
}

result, err := client.SendMessage(ctx, message)
if err != nil {
	log.Fatal("Send failed:", err)
}
fmt.Printf("✓ Message sent: %s\n", result.MessageID)
fmt.Printf("  Status: %s\n", result.Status)
fmt.Printf("  Delivered channels: %d\n", result.DeliveredChannels)
```

### 3. Send Asynchronously

```go
asyncResult, err := client.SendMessageAsync(ctx, message)
if err != nil {
	log.Fatal("Async send failed:", err)
}
fmt.Printf("✓ Async message sent: %s\n", asyncResult.MessageID)
```

## Complete Examples

### Example 1: Priority-Based Messaging

```go
package main

import (
	"context"
	"fmt"
	"log"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		log.Fatal(err)
	}

	// Critical priority message
	criticalMsg := &fastdatabroker.Message{
		SenderID:     "system",
		RecipientIDs: []string{"admin"},
		Subject:      "CRITICAL: System Alert",
		Content:      []byte("Immediate action required"),
		Priority:     fastdatabroker.PriorityCritical, // 255
	}

	result1, err := client.SendMessageAsync(ctx, criticalMsg)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Critical message sent: %s\n", result1.MessageID)

	// Normal priority message
	normalMsg := &fastdatabroker.Message{
		SenderID:     "system",
		RecipientIDs: []string{"user"},
		Subject:      "Regular Update",
		Content:      []byte("Routine notification"),
		Priority:     fastdatabroker.PriorityNormal, // 100
	}

	result2, err := client.SendMessageAsync(ctx, normalMsg)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Normal message sent: %s\n", result2.MessageID)

	// Deferred priority message
	deferredMsg := &fastdatabroker.Message{
		SenderID:     "system",
		RecipientIDs: []string{"background-worker"},
		Subject:      "Background Task",
		Content:      []byte("Can be processed later"),
		Priority:     fastdatabroker.PriorityDeferred, // 50
	}

	result3, err := client.SendMessageAsync(ctx, deferredMsg)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Deferred message sent: %s\n", result3.MessageID)

	client.Disconnect(ctx)
}
```

### Example 2: Batch Message Sending with TTL

```go
package main

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		log.Fatal(err)
	}

	messages := make([]*fastdatabroker.Message, 5)
	for i := 0; i < 5; i++ {
		ttl := int64(3600 * (i + 1)) // 1, 2, 3, 4, 5 hours
		messages[i] = &fastdatabroker.Message{
			SenderID:       "batch-sender",
			RecipientIDs:   []string{fmt.Sprintf("recipient-%d", i)},
			Subject:        fmt.Sprintf("Batch Message %d", i+1),
			Content:        []byte(fmt.Sprintf("Content for message %d", i+1)),
			Priority:       fastdatabroker.PriorityHigh,
			TTLSeconds:     &ttl,
			RequireConfirm: true,
		}
	}

	// Send all messages
	for _, msg := range messages {
		result, err := client.SendMessageAsync(ctx, msg)
		if err != nil {
			fmt.Printf("✗ Failed: %v\n", err)
			continue
		}
		fmt.Printf("✓ Message sent - ID: %s, TTL: %ds\n", result.MessageID, *msg.TTLSeconds)
	}

	client.Disconnect(ctx)
}
```

### Example 3: Tagged Messages for Organization

```go
package main

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		log.Fatal(err)
	}

	message := &fastdatabroker.Message{
		SenderID:     "order-service",
		RecipientIDs: []string{"customer-789"},
		Subject:      "Order Confirmation",
		Content:      []byte("Your order has been confirmed"),
		Priority:     fastdatabroker.PriorityHigh,
		Tags: map[string]string{
			"order-id": "ORD-2024-001234",
			"region":   "us-west-2",
			"category": "order-notification",
			"version":  "v2",
			"timestamp": time.Now().UTC().Format(time.RFC3339),
		},
	}

	result, err := client.SendMessageAsync(ctx, message)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("✓ Tagged message sent: %s\n", result.MessageID)
	fmt.Println("  Tags:")
	for k, v := range message.Tags {
		fmt.Printf("    - %s: %s\n", k, v)
	}

	client.Disconnect(ctx)
}
```

### Example 4: WebSocket Integration

```go
package main

import (
	"context"
	"fmt"
	"log"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	// Connect via WebSocket
	wsClient := fastdatabroker.NewWebSocketClient("ws://localhost:6001")
	ctx := context.Background()

	if err := wsClient.Connect(ctx); err != nil {
		log.Fatal("WebSocket connection failed:", err)
	}
	defer wsClient.Close()

	fmt.Println("✓ WebSocket client connected")

	// Send message via WebSocket
	message := &fastdatabroker.Message{
		SenderID:     "app",
		RecipientIDs: []string{"user-1"},
		Subject:      "Real-time Update",
		Content:      []byte("WebSocket real-time notification"),
		Priority:     fastdatabroker.PriorityUrgent,
	}

	result, err := wsClient.SendMessage(ctx, message)
	if err != nil {
		log.Fatal("Send failed:", err)
	}
	fmt.Printf("✓ Message sent: %s\n", result.MessageID)

	// Listen for messages
	ch := wsClient.Subscribe(ctx, "user-1")
	for msg := range ch {
		fmt.Printf("Received: %s\n", msg.Subject)
	}
}
```

### Example 5: Error Handling with Custom Errors

```go
package main

import (
	"context"
	"fmt"
	"log"

	"github.com/suraj202923/FastDataBroker-go"
	"github.com/suraj202923/FastDataBroker-go/errors"
)

func main() {
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		log.Fatal(err)
	}

	message := &fastdatabroker.Message{
		SenderID:     "app",
		RecipientIDs: []string{"user-1"},
		Subject:      "Test",
		Content:      []byte("Test content"),
		Priority:     fastdatabroker.PriorityHigh,
	}

	result, err := client.SendMessage(ctx, message)
	if err != nil {
		switch err.(type) {
		case *errors.ConnectionError:
			fmt.Println("✗ Connection failed, retrying...")
		case *errors.ValidationError:
			fmt.Println("✗ Invalid message parameters")
		case *errors.TimeoutError:
			fmt.Println("✗ Message delivery timeout")
		default:
			log.Fatal("Error:", err)
		}
	} else {
		fmt.Printf("✓ Message sent: %s\n", result.MessageID)
	}

	client.Disconnect(ctx)
}
```

### Example 6: Batch Operations with Concurrency

```go
package main

import (
	"context"
	"fmt"
	"log"
	"sync"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		log.Fatal(err)
	}

	// Send multiple messages concurrently
	results, err := client.SendBatch(ctx, []*fastdatabroker.Message{
		{
			SenderID:     "user-1",
			RecipientIDs: []string{"user-2"},
			Subject:      "Message 1",
			Content:      []byte("Content 1"),
			Priority:     fastdatabroker.PriorityHigh,
		},
		{
			SenderID:     "user-1",
			RecipientIDs: []string{"user-3"},
			Subject:      "Message 2",
			Content:      []byte("Content 2"),
			Priority:     fastdatabroker.PriorityNormal,
		},
		{
			SenderID:     "user-1",
			RecipientIDs: []string{"user-4"},
			Subject:      "Message 3",
			Content:      []byte("Content 3"),
			Priority:     fastdatabroker.PriorityDeferred,
		},
	})
	if err != nil {
		log.Fatal(err)
	}

	for _, result := range results {
		fmt.Printf("Message ID: %s, Status: %s\n", result.MessageID, result.Status)
	}

	client.Disconnect(ctx)
}
```

### Example 7: Complete End-to-End Application

```go
package main

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/suraj202923/FastDataBroker-go"
)

func main() {
	fmt.Println("=== FastDataBroker Go SDK Complete Example ===\n")

	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()

	// 1. Connection
	fmt.Println("1. Connecting to FastDataBroker...")
	if err := client.Connect(ctx); err != nil {
		log.Fatal("Connection failed:", err)
	}
	fmt.Println("✓ Connected successfully\n")

	// 2. Send critical message
	fmt.Println("2. Sending critical priority message...")
	start := time.Now()
	ttl := int64(3600)
	criticalMsg := &fastdatabroker.Message{
		SenderID:     "app",
		RecipientIDs: []string{"user-123"},
		Subject:      "Critical Alert",
		Content:      []byte("This is critical"),
		Priority:     fastdatabroker.PriorityCritical,
		TTLSeconds:   &ttl,
		Tags: map[string]string{
			"severity":  "critical",
			"timestamp": time.Now().UTC().Format(time.RFC3339),
		},
	}
	result, err := client.SendMessageAsync(ctx, criticalMsg)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("✓ Message sent: %s (took %dms)\n", result.MessageID, time.Since(start).Milliseconds())
	fmt.Println()

	// 3. Send batch messages
	fmt.Println("3. Sending batch messages...")
	for i := 0; i < 3; i++ {
		msg := &fastdatabroker.Message{
			SenderID:     "batch-app",
			RecipientIDs: []string{fmt.Sprintf("user-%d", i)},
			Subject:      fmt.Sprintf("Batch message %d", i+1),
			Content:      []byte(fmt.Sprintf("Content %d", i+1)),
			Priority:     fastdatabroker.PriorityNormal,
			TTLSeconds:   &ttl,
		}
		res, err := client.SendMessageAsync(ctx, msg)
		if err != nil {
			fmt.Printf("  ✗ Message %d failed: %v\n", i+1, err)
			continue
		}
		fmt.Printf("  ✓ Message %d: %s\n", i+1, res.MessageID)
	}
	fmt.Println()

	// 4. Get message status
	fmt.Println("4. Getting message status...")
	status, err := client.GetMessageStatus(ctx, result.MessageID)
	if err != nil {
		fmt.Printf("  ✗ Failed to get status: %v\n", err)
	} else {
		fmt.Printf("  ✓ Message status: %s\n", status)
	}
	fmt.Println()

	// 5. Cleanup
	fmt.Println("5. Cleaning up...")
	client.Disconnect(ctx)
	fmt.Println("✓ Disconnected\n")

	fmt.Println("=== Example completed successfully ===")
}
```

## API Reference

### Priority Constants

```go
const (
	PriorityDeferred Priority = 50      // Low priority, can be delayed
	PriorityNormal   Priority = 100     // Standard priority (default)
	PriorityHigh     Priority = 150     // Higher priority
	PriorityUrgent   Priority = 200     // Very high priority
	PriorityCritical Priority = 255     // Critical, process immediately
)
```

### NotificationChannel Constants

```go
const (
	ChannelEmail     NotificationChannel = "email"      // Email delivery
	ChannelWebSocket NotificationChannel = "websocket"  // WebSocket push
	ChannelPush      NotificationChannel = "push"       // Push notifications
	ChannelWebhook   NotificationChannel = "webhook"    // Webhook callback
)
```

### Message Structure

```go
type Message struct {
	SenderID       string            // Sender identifier
	RecipientIDs   []string          // List of recipient identifiers
	Subject        string            // Message subject line
	Content        []byte            // Message body (binary)
	Priority       Priority          // Message priority (default: PriorityNormal)
	TTLSeconds     *int64            // Time to live in seconds
	Tags           map[string]string // Custom metadata tags
	RequireConfirm bool              // Request delivery confirmation
	Timestamp      int64             // Message creation timestamp
}
```

### DeliveryResult Structure

```go
type DeliveryResult struct {
	MessageID         string                 // Unique message identifier
	Status            string                 // "success", "partial", "failed"
	DeliveredChannels int                    // Number of channels delivered
	Details           map[string]interface{} // Additional details
	Timestamp         int64                  // Delivery timestamp
}
```

### Client Methods

```go
type Client interface {
	// Connection management
	Connect(ctx context.Context) error
	Close() error
	Disconnect(ctx context.Context) error
	IsConnected() bool

	// Message sending
	SendMessage(ctx context.Context, message *Message) (*DeliveryResult, error)
	SendMessageAsync(ctx context.Context, message *Message) (*DeliveryResult, error)
	SendBatch(ctx context.Context, messages []*Message) ([]*DeliveryResult, error)

	// Message retrieval
	GetMessageStatus(ctx context.Context, messageID string) (string, error)

	// WebSocket operations
	Subscribe(ctx context.Context, userID string) <-chan *Message

	// WebSocket client registration
	RegisterWebSocketClient(clientID string, userID string) bool
	UnregisterWebSocketClient(clientID string) bool
}
```

## Error Handling

### Custom Error Types

```go
// Connection errors
type ConnectionError struct {
	Reason string
	Err    error
}

// Validation errors
type ValidationError struct {
	Field  string
	Reason string
}

// Timeout errors
type TimeoutError struct {
	Duration time.Duration
	Err      error
}
```

### Error Handling Pattern

```go
result, err := client.SendMessage(ctx, message)
if err != nil {
	switch e := err.(type) {
	case *errors.ConnectionError:
		fmt.Println("Connection issue:", e.Reason)
	case *errors.ValidationError:
		fmt.Printf("Field %s: %s\n", e.Field, e.Reason)
	case *errors.TimeoutError:
		fmt.Println("Timeout after:", e.Duration)
	default:
		fmt.Println("Unknown error:", err)
	}
}
```

## Advanced Features

### Configuration

```go
client := fastdatabroker.NewClient("localhost", 6000,
	fastdatabroker.WithTimeout(30*time.Second),
	fastdatabroker.WithRetries(3),
	fastdatabroker.WithCompression(true),
	fastdatabroker.WithEncryption(true),
)
```

### Concurrency Control

```go
// Send messages with limited concurrency
semaphore := make(chan struct{}, 10) // Max 10 concurrent sends
for _, msg := range messages {
	semaphore <- struct{}{}
	go func(m *fastdatabroker.Message) {
		defer func() { <-semaphore }()
		client.SendMessage(ctx, m)
	}(msg)
}
```

### Timeout Handling

```go
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()

result, err := client.SendMessage(ctx, message)
if err == context.DeadlineExceeded {
	fmt.Println("Operation timeout")
}
```

## Testing

### Run Tests

```bash
cd sdks/go
go test -v
go test -cover   # With coverage
go test -race    # Race condition detection
```

### Unit Test Example

```go
func TestMessageSending(t *testing.T) {
	client := fastdatabroker.NewClient("localhost", 6000)
	defer client.Close()

	ctx := context.Background()
	if err := client.Connect(ctx); err != nil {
		t.Fatal(err)
	}

	message := &fastdatabroker.Message{
		SenderID:     "test",
		RecipientIDs: []string{"test-recipient"},
		Subject:      "Test",
		Content:      []byte("Test content"),
		Priority:     fastdatabroker.PriorityHigh,
	}

	result, err := client.SendMessageAsync(ctx, message)
	if err != nil {
		t.Fatal(err)
	}

	if result.MessageID == "" {
		t.Error("Expected non-empty message ID")
	}
}
```

## Requirements

- Go 1.19 or higher
- Standard library only (no external dependencies for core)

## Building and Publishing

### Build

```bash
go build
```

### Create Module

```bash
go mod init github.com/yourname/fastdatabroker-go
go mod tidy
```

### Publish to GitHub

```bash
git tag v0.1.13
git push origin v0.1.13
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
- Enhanced SDK with complete documentation
- WebSocket and streaming integration
- Comprehensive error types
- Batch operations with concurrency
- Context.Context integration throughout

### Version 0.1.12
- Initial Go SDK release
- Basic synchronous and asynchronous APIs
- Priority-based message routing
- QUIC protocol support
