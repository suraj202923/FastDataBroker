# FastDataBroker C# SDK

C# SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

**Version:** 0.1.16

## ðŸ“‹ Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Complete Examples](#complete-examples)
- [API Reference](#api-reference)
- [Error Handling](#error-handling)
- [Advanced Features](#advanced-features)
- [Testing](#testing)

## Features

- ðŸš€ **Synchronous & Asynchronous APIs** - Both sync and async message sending
- ðŸ“¨ **Multi-Channel Delivery** - Email, WebSocket, Push Notifications, Webhooks
- ðŸŽ¯ **Priority Levels** - 5 priority levels: Deferred, Normal, High, Urgent, Critical
- ðŸ”„ **Message Confirmation** - Optional delivery confirmation
- ðŸ·ï¸ **Message Tagging** - Tag messages for categorization
- â±ï¸ **TTL Support** - Set time-to-live for messages
- ðŸ”Œ **WebSocket Support** - Real-time bidirectional communication
- ðŸª **Webhook Endpoints** - Integrate with external systems
- ðŸŒ **QUIC Protocol** - High-performance UDP-based protocol
- ðŸ” **Clustering Support** - Multi-region failover and load balancing

## Installation

### Via NuGet (Coming Soon)

```bash
dotnet add package FastDataBroker
```

### From Source

```bash
cd sdks/csharp
dotnet build
dotnet pack -c Release
```

### Project File Configuration

Add to your `.csproj`:

```xml
<ItemGroup>
    <PackageReference Include="FastDataBroker" Version="0.1.16" />
</ItemGroup>
```

## Quick Start

### 1. Basic Client Setup

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

class Program
{
    static async Task Main(string[] args)
    {
        // Create and connect client
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();
            Console.WriteLine("Connected to FastDataBroker");

            // Your code here

            client.Disconnect();
        }
    }
}
```

### 2. Send a Simple Message

```csharp
// Create a message
var message = new FastDataBrokerSDK.Message
{
    SenderId = "app-001",
    RecipientIds = new List<string> { "user-123", "user-456" },
    Subject = "Notification",
    Content = System.Text.Encoding.UTF8.GetBytes("Hello from FastDataBroker!"),
    Priority = FastDataBrokerSDK.Priority.High
};

// Send synchronously
var result = client.SendMessage(message);
Console.WriteLine($"âœ“ Message sent: {result.MessageId}");
Console.WriteLine($"  Status: {result.Status}");
Console.WriteLine($"  Delivered channels: {result.DeliveredChannels}");
```

### 3. Send Asynchronously

```csharp
try
{
    var asyncResult = await client.SendMessageAsync(message);
    Console.WriteLine($"âœ“ Async message sent: {asyncResult.MessageId}");
}
catch (Exception ex)
{
    Console.WriteLine($"âœ— Error: {ex.Message}");
}
```

## Complete Examples

### Example 1: Priority-Based Messaging

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

class PriorityExample
{
    static async Task Main(string[] args)
    {
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();

            // Critical priority message
            var criticalMsg = new FastDataBrokerSDK.Message
            {
                SenderId = "system",
                RecipientIds = new List<string> { "admin" },
                Subject = "CRITICAL: System Alert",
                Content = System.Text.Encoding.UTF8.GetBytes("Immediate action required"),
                Priority = FastDataBrokerSDK.Priority.Critical // 255
            };

            var result1 = await client.SendMessageAsync(criticalMsg);
            Console.WriteLine($"Critical message sent: {result1.MessageId}");

            // Normal priority message
            var normalMsg = new FastDataBrokerSDK.Message
            {
                SenderId = "system",
                RecipientIds = new List<string> { "user" },
                Subject = "Regular Update",
                Content = System.Text.Encoding.UTF8.GetBytes("Routine notification"),
                Priority = FastDataBrokerSDK.Priority.Normal // 100
            };

            var result2 = await client.SendMessageAsync(normalMsg);
            Console.WriteLine($"Normal message sent: {result2.MessageId}");

            // Deferred priority message
            var deferredMsg = new FastDataBrokerSDK.Message
            {
                SenderId = "system",
                RecipientIds = new List<string> { "background-worker" },
                Subject = "Background Task",
                Content = System.Text.Encoding.UTF8.GetBytes("Can be processed later"),
                Priority = FastDataBrokerSDK.Priority.Deferred // 50
            };

            var result3 = await client.SendMessageAsync(deferredMsg);
            Console.WriteLine($"Deferred message sent: {result3.MessageId}");

            client.Disconnect();
        }
    }
}
```

### Example 2: Batch Message Sending with TTL

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

class BatchMessagesExample
{
    static async Task Main(string[] args)
    {
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();

            var messages = new List<FastDataBrokerSDK.Message>();

            // Create 5 messages with different TTLs
            for (int i = 0; i < 5; i++)
            {
                var msg = new FastDataBrokerSDK.Message
                {
                    SenderId = "batch-sender",
                    RecipientIds = new List<string> { $"recipient-{i}" },
                    Subject = $"Batch Message {i + 1}",
                    Content = System.Text.Encoding.UTF8.GetBytes($"Content for message {i + 1}"),
                    Priority = FastDataBrokerSDK.Priority.High,
                    TTLSeconds = 3600 * (i + 1), // 1, 2, 3, 4, 5 hours
                    RequireConfirm = true
                };
                messages.Add(msg);
            }

            // Send all messages
            foreach (var msg in messages)
            {
                try
                {
                    var result = await client.SendMessageAsync(msg);
                    Console.WriteLine($"âœ“ Message sent - ID: {result.MessageId}, TTL: {msg.TTLSeconds}s");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"âœ— Failed to send message: {ex.Message}");
                }
            }

            client.Disconnect();
        }
    }
}
```

### Example 3: Tagged Messages for Organization

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

class TaggedMessagesExample
{
    static async Task Main(string[] args)
    {
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();

            // Message with metadata tags
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "order-service",
                RecipientIds = new List<string> { "customer-789" },
                Subject = "Order Confirmation",
                Content = System.Text.Encoding.UTF8.GetBytes("Your order has been confirmed"),
                Priority = FastDataBrokerSDK.Priority.High,
                Tags = new Dictionary<string, string>
                {
                    { "order-id", "ORD-2024-001234" },
                    { "region", "us-west-2" },
                    { "category", "order-notification" },
                    { "version", "v2" },
                    { "timestamp", DateTime.UtcNow.ToString("O") }
                }
            };

            var result = await client.SendMessageAsync(message);
            Console.WriteLine($"âœ“ Tagged message sent: {result.MessageId}");
            Console.WriteLine("  Tags:");
            foreach (var tag in message.Tags)
            {
                Console.WriteLine($"    - {tag.Key}: {tag.Value}");
            }

            client.Disconnect();
        }
    }
}
```

### Example 4: WebSocket Client Registration

```csharp
using FastDataBroker;
using System;
using System.Threading.Tasks;

class WebSocketExample
{
    static async Task Main(string[] args)
    {
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();

            // Register multiple WebSocket clients
            var clientIds = new[] { "client-001", "client-002", "client-003" };
            var userIds = new[] { "user-A", "user-B", "user-C" };

            for (int i = 0; i < clientIds.Length; i++)
            {
                bool registered = client.RegisterWebSocketClient(clientIds[i], userIds[i]);
                if (registered)
                {
                    Console.WriteLine($"âœ“ WebSocket client registered: {clientIds[i]} -> {userIds[i]}");
                }
                else
                {
                    Console.WriteLine($"âœ— Failed to register: {clientIds[i]}");
                }
            }

            // Simulate message sending to WebSocket clients
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "system",
                RecipientIds = new[] { "user-A", "user-B", "user-C" }.ToList(),
                Subject = "Real-time Update",
                Content = System.Text.Encoding.UTF8.GetBytes("WebSocket real-time notification"),
                Priority = FastDataBrokerSDK.Priority.Urgent
            };

            var result = await client.SendMessageAsync(message);
            Console.WriteLine($"\nâœ“ Message sent to WebSocket clients: {result.MessageId}");

            // Unregister clients
            foreach (var clientId in clientIds)
            {
                client.UnregisterWebSocketClient(clientId);
                Console.WriteLine($"âœ“ Unregistered: {clientId}");
            }

            client.Disconnect();
        }
    }
}
```

### Example 5: Webhook Integration

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

class WebhookExample
{
    static async Task Main(string[] args)
    {
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();

            // Configure webhook
            var webhookConfig = new FastDataBrokerSDK.WebhookConfig
            {
                Url = "https://api.example.com/webhooks/fastdatabroker",
                Retries = 5,
                TimeoutMs = 30000,
                VerifySSL = true,
                Headers = new Dictionary<string, string>
                {
                    { "Authorization", "Bearer secret-token-123" },
                    { "X-API-Key", "api-key-456" },
                    { "X-Custom-Header", "CustomValue" }
                }
            };

            // Register webhook
            bool registered = client.RegisterWebhook(
                FastDataBrokerSDK.NotificationChannel.Webhook,
                webhookConfig
            );

            if (registered)
            {
                Console.WriteLine("âœ“ Webhook registered successfully");
                Console.WriteLine($"  URL: {webhookConfig.Url}");
                Console.WriteLine($"  Retries: {webhookConfig.Retries}");
                Console.WriteLine($"  Timeout: {webhookConfig.TimeoutMs}ms");
            }

            // Send message that will use webhook
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "notification-service",
                RecipientIds = new List<string> { "external-service" },
                Subject = "Event Notification",
                Content = System.Text.Encoding.UTF8.GetBytes("Event occurred at the system"),
                Priority = FastDataBrokerSDK.Priority.High
            };

            var result = await client.SendMessageAsync(message);
            Console.WriteLine($"\nâœ“ Message sent with webhook: {result.MessageId}");

            client.Disconnect();
        }
    }
}
```

### Example 6: Complete End-to-End Application

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading.Tasks;

class CompleteApplicationExample
{
    static async Task Main(string[] args)
    {
        Console.WriteLine("=== FastDataBroker C# SDK Complete Example ===\n");

        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            try
            {
                // Connect
                Console.WriteLine("1. Connecting to FastDataBroker...");
                bool connected = await client.ConnectAsync();
                if (!connected)
                {
                    Console.WriteLine("âœ— Failed to connect");
                    return;
                }
                Console.WriteLine("âœ“ Connected successfully\n");

                // Register WebSocket client
                Console.WriteLine("2. Registering WebSocket client...");
                bool wsRegistered = client.RegisterWebSocketClient("app-client", "user-123");
                Console.WriteLine(wsRegistered ? "âœ“ WebSocket client registered\n" : "âœ— Registration failed\n");

                // Send critical message
                Console.WriteLine("3. Sending critical priority message...");
                var stopwatch = Stopwatch.StartNew();
                var criticalMsg = new FastDataBrokerSDK.Message
                {
                    SenderId = "app",
                    RecipientIds = new List<string> { "user-123" },
                    Subject = "Critical Alert",
                    Content = System.Text.Encoding.UTF8.GetBytes("This is critical"),
                    Priority = FastDataBrokerSDK.Priority.Critical,
                    Tags = new Dictionary<string, string>
                    {
                        { "severity", "critical" },
                        { "timestamp", DateTime.UtcNow.ToString("O") }
                    }
                };
                var result1 = await client.SendMessageAsync(criticalMsg);
                stopwatch.Stop();
                Console.WriteLine($"âœ“ Message sent: {result1.MessageId}");
                Console.WriteLine($"  Time taken: {stopwatch.ElapsedMilliseconds}ms\n");

                // Send batch messages
                Console.WriteLine("4. Sending batch messages...");
                for (int i = 0; i < 3; i++)
                {
                    var msg = new FastDataBrokerSDK.Message
                    {
                        SenderId = "batch-app",
                        RecipientIds = new List<string> { $"user-{i}" },
                        Subject = $"Batch message {i + 1}",
                        Content = System.Text.Encoding.UTF8.GetBytes($"Content {i + 1}"),
                        Priority = FastDataBrokerSDK.Priority.Normal,
                        TTLSeconds = 7200
                    };
                    var res = await client.SendMessageAsync(msg);
                    Console.WriteLine($"  âœ“ Message {i + 1}: {res.MessageId}");
                }
                Console.WriteLine();

                // Statistics
                Console.WriteLine("5. Final Statistics:");
                Console.WriteLine($"  âœ“ Client connected: {client.IsConnected}");
                Console.WriteLine($"  âœ“ Messages sent successfully");

                // Cleanup
                Console.WriteLine("\n6. Cleaning up...");
                client.UnregisterWebSocketClient("app-client");
                client.Disconnect();
                Console.WriteLine("âœ“ Disconnected\n");

                Console.WriteLine("=== Example completed successfully ===");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"âœ— Error occurred: {ex.Message}");
                Console.WriteLine($"  Stack trace: {ex.StackTrace}");
            }
        }
    }
}
```

## API Reference

### Priority Enum

```csharp
public enum Priority : byte
{
    Deferred = 50,      // Low priority, can be delayed
    Normal = 100,       // Standard priority (default)
    High = 150,         // Higher priority
    Urgent = 200,       // Very high priority
    Critical = 255      // Critical, process immediately
}
```

### NotificationChannel Enum

```csharp
public enum NotificationChannel
{
    Email,              // Email delivery
    WebSocket,          // WebSocket push
    Push,               // Push notifications (Firebase, APNs)
    Webhook             // Webhook callback
}
```

### PushPlatform Enum

```csharp
public enum PushPlatform
{
    Firebase,           // Google Firebase Cloud Messaging
    APNs,              // Apple Push Notification service
    FCM,               // Firebase Cloud Messaging
    WebPush            // Web Push API
}
```

### Message Class Properties

```csharp
public class Message
{
    public string SenderId { get; set; }              // Sender identifier
    public List<string> RecipientIds { get; set; }   // Recipient identifiers
    public string Subject { get; set; }              // Message subject
    public byte[] Content { get; set; }              // Message body (binary)
    public Priority Priority { get; set; }           // Message priority (default: Normal)
    public long? TTLSeconds { get; set; }            // Time-to-live in seconds
    public Dictionary<string, string> Tags { get; set; } // Metadata tags
    public bool RequireConfirm { get; set; }         // Request delivery confirmation
}
```

### DeliveryResult Class Properties

```csharp
public class DeliveryResult
{
    public string MessageId { get; set; }            // Unique message identifier
    public string Status { get; set; }               // "success", "partial", "failed"
    public int DeliveredChannels { get; set; }       // Number of channels delivered to
    public Dictionary<string, object> Details { get; set; } // Additional details
}
```

### Client Class Methods

```csharp
public class Client : IDisposable
{
    // Constructor
    public Client(string host = "localhost", int port = 6000);

    // Connection methods
    public Task<bool> ConnectAsync();
    public void Disconnect();
    public bool IsConnected { get; }

    // Message sending
    public DeliveryResult SendMessage(Message message);
    public Task<DeliveryResult> SendMessageAsync(Message message);

    // WebSocket management
    public bool RegisterWebSocketClient(string clientId, string userId);
    public bool UnregisterWebSocketClient(string clientId);

    // Webhook management
    public bool RegisterWebhook(NotificationChannel channel, WebhookConfig config);

    // Resource management
    public void Dispose();
}
```

## Error Handling

### Try-Catch Pattern

```csharp
try
{
    await client.ConnectAsync();
    var result = await client.SendMessageAsync(message);
    Console.WriteLine($"Success: {result.MessageId}");
}
catch (InvalidOperationException ex)
{
    Console.WriteLine($"Invalid operation: {ex.Message}");
}
catch (TimeoutException ex)
{
    Console.WriteLine($"Timeout occurred: {ex.Message}");
}
catch (Exception ex)
{
    Console.WriteLine($"Unexpected error: {ex.Message}");
}
```

### Connection Validation

```csharp
using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
{
    if (!await client.ConnectAsync())
    {
        Console.WriteLine("Connection failed");
        return;
    }

    if (!client.IsConnected)
    {
        Console.WriteLine("Client is not connected");
        return;
    }

    // Safe to send messages
    var result = await client.SendMessageAsync(message);
}
```

## Testing

### Unit Tests

```bash
dotnet test
```

### Comprehensive SDK Test Suite

This SDK is part of the comprehensive FastDataBroker test suite with **260+ test cases** across 4 languages.

**SDK Test Coverage**: 260+ tests across Python, Go, Java, and JavaScript SDKs
- âœ“ Connection management (6 tests per SDK)
- âœ“ Message operations (6 tests per SDK)
- âœ“ Priority handling (5 tests per SDK)
- âœ“ Error handling (8+ tests per SDK)
- âœ“ Concurrency testing (5+ tests per SDK)
- âœ“ Performance benchmarks (4+ tests per SDK)
- âœ“ Integration scenarios (4+ tests per SDK)

**Run all SDK tests**:
```bash
# From workspace root - Run all 260+ tests across Python, Go, Java, JavaScript
python run_all_sdk_tests.py

# C# SDK note: Comprehensive test suite for C# coming soon
# Current: Unit tests available with: dotnet test
```

ðŸ“– See [TEST_RUNNER_GUIDE.md](../../TEST_RUNNER_GUIDE.md) for detailed testing instructions
ðŸ“„ See [SDK_TESTING_COMPLETE_v2.0.md](../../SDK_TESTING_COMPLETE_v2.0.md) for full test suite overview

## Advanced Features

### Batch Processing

```csharp
var messages = new List<FastDataBrokerSDK.Message>();
for (int i = 0; i < 100; i++)
{
    messages.Add(new FastDataBrokerSDK.Message
    {
        SenderId = "batch-processor",
        RecipientIds = new List<string> { $"user-{i}" },
        Subject = $"Message {i}",
        Content = System.Text.Encoding.UTF8.GetBytes($"Content {i}"),
        Priority = FastDataBrokerSDK.Priority.Normal
    });
}

int successCount = 0;
foreach (var msg in messages)
{
    try
    {
        var result = await client.SendMessageAsync(msg);
        successCount++;
    }
    catch { }
}
Console.WriteLine($"Sent {successCount}/{messages.Count} messages");
```

## Testing

### Run Tests

```bash
cd sdks/csharp
dotnet test
dotnet test --logger "html"  // Generate HTML report
```

### Unit Test Example

```csharp
[TestClass]
public class FastDataBrokerTests
{
    [TestMethod]
    public async Task TestMessageSending()
    {
        using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
        {
            await client.ConnectAsync();
            
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "test",
                RecipientIds = new List<string> { "test-recipient" },
                Subject = "Test",
                Content = System.Text.Encoding.UTF8.GetBytes("Test content"),
                Priority = FastDataBrokerSDK.Priority.High
            };

            var result = await client.SendMessageAsync(message);
            Assert.IsNotNull(result.MessageId);
            Assert.AreEqual("success", result.Status);
        }
    }
}
```

## Target Frameworks

- .NET 6.0
- .NET 7.0
- .NET 8.0

## Requirements

- .NET 6.0 or higher
- System.Net.Http (for QUIC support)
- System.Runtime.Serialization

## Building and Publishing

### Build

```bash
dotnet build -c Release
```

### Run Tests

```bash
dotnet test -c Release
```

### Create Package

```bash
dotnet pack -c Release
```

### Publish to NuGet

```bash
dotnet nuget push bin/Release/FastDataBroker.*.nupkg
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
4. Submit a pull request

## Changelog

### Version 0.1.13 (Latest)
- Enhanced SDK with complete documentation
- WebSocket and Webhook integration examples
- Comprehensive error handling
- Batch message processing support
- Multi-framework support (.NET 6.0, 7.0, 8.0)

### Version 0.1.12
- Initial C# SDK release
- Basic synchronous and asynchronous message APIs
- Priority-based message routing
- TTL and tagging support

