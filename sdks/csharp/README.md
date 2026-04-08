# FastDataBroker C# SDK

C# SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering.

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

## Installation

### Via NuGet (Coming Soon)

```bash
dotnet add package FastDataBroker
```

### From Source

```bash
cd sdks/csharp
dotnet build
```

## Quick Start

### Basic Usage

```csharp
using FastDataBroker;
using System.Collections.Generic;
using System.Threading.Tasks;

// Create client
using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
{
    // Connect to FastDataBroker
    await client.ConnectAsync();

    // Create a message
    var message = new FastDataBrokerSDK.Message
    {
        SenderId = "user-1",
        RecipientIds = new List<string> { "user-2", "user-3" },
        Subject = "Hello from C#",
        Content = System.Text.Encoding.UTF8.GetBytes("This is a test message"),
        Priority = FastDataBrokerSDK.Priority.High
    };

    // Send synchronously
    var result = client.SendMessage(message);
    Console.WriteLine($"Message sent: {result.MessageId}");

    // Or send asynchronously
    var asyncResult = await client.SendMessageAsync(message);
    Console.WriteLine($"Async message sent: {asyncResult.MessageId}");
}
```

### Priority Levels

```csharp
// Set message priority
message.Priority = FastDataBrokerSDK.Priority.Critical; // 255
// Other options:
// - Priority.Urgent (200)
// - Priority.High (150)
// - Priority.Normal (100)
// - Priority.Deferred (50)
```

### Message TTL (Time-To-Live)

```csharp
// Message expires after 3600 seconds (1 hour)
message.TTLSeconds = 3600;
```

### Message Tags

```csharp
message.Tags = new Dictionary<string, string>
{
    { "region", "us-east" },
    { "category", "notification" },
    { "version", "v1" }
};
```

### WebSocket Integration

```csharp
// Register WebSocket client
var clientId = "client-unique-id";
var userId = "user-123";

if (client.RegisterWebSocketClient(clientId, userId))
{
    Console.WriteLine("WebSocket client registered");
}

// Later, unregister
client.UnregisterWebSocketClient(clientId);
```

### Webhook Configuration

```csharp
var webhookConfig = new FastDataBrokerSDK.WebhookConfig
{
    Url = "https://your-domain.com/webhook",
    Retries = 5,
    TimeoutMs = 45000,
    VerifySSL = true,
    Headers = new Dictionary<string, string>
    {
        { "Authorization", "Bearer your-token" },
        { "X-Custom-Header", "value" }
    }
};

// Register webhook
client.RegisterWebhook(
    FastDataBrokerSDK.NotificationChannel.Webhook,
    webhookConfig
);
```

## API Reference

### Priority Enum

```csharp
public enum Priority : byte
{
    Deferred = 50,      // Low priority, can be delayed
    Normal = 100,       // Standard priority
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
    Push,               // Push notifications
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

### Message Class

```csharp
public class Message
{
    public string SenderId { get; set; }
    public List<string> RecipientIds { get; set; }
    public string Subject { get; set; }
    public byte[] Content { get; set; }
    public Priority Priority { get; set; }
    public long? TTLSeconds { get; set; }
    public Dictionary<string, string> Tags { get; set; }
    public bool RequireConfirm { get; set; }
}
```

### DeliveryResult Class

```csharp
public class DeliveryResult
{
    public string MessageId { get; set; }
    public string Status { get; set; }
    public int DeliveredChannels { get; set; }
    public Dictionary<string, object> Details { get; set; }
}
```

### Client Class

```csharp
public class Client : IDisposable
{
    // Constructor
    public Client(string host = "localhost", int port = 6000);

    // Async methods
    public Task<bool> ConnectAsync();
    public Task<DeliveryResult> SendMessageAsync(Message message);

    // Sync methods
    public DeliveryResult SendMessage(Message message);

    // WebSocket methods
    public bool RegisterWebSocketClient(string clientId, string userId);
    public bool UnregisterWebSocketClient(string clientId);

    // Webhook methods
    public bool RegisterWebhook(NotificationChannel channel, WebhookConfig config);

    // Connection management
    public void Disconnect();
    public bool IsConnected { get; }
}
```

## Target Frameworks

- .NET 6.0
- .NET 7.0
- .NET 8.0

## Testing

Run unit tests:

```bash
cd sdks/csharp
dotnet test
```

Test coverage includes:
- Message creation and properties
- Client connection and disconnection
- Synchronous and asynchronous message sending
- WebSocket client registration
- Webhook configuration
- Delivery result handling

## Building and Publishing

### Local Build

```bash
dotnet build
```

### Run Tests

```bash
dotnet test
```

### Create NuGet Package

```bash
dotnet pack -c Release
```

## Error Handling

```csharp
try
{
    await client.ConnectAsync();
    var result = await client.SendMessageAsync(message);
}
catch (InvalidOperationException ex)
{
    // Not connected to FastDataBroker
    Console.WriteLine($"Operation failed: {ex.Message}");
}
catch (Exception ex)
{
    // Other errors
    Console.WriteLine($"Error: {ex.Message}");
}
```

## Requirements

- .NET 6.0 or higher
- System.Net.Http (for QUIC support)

## License

MIT License - See LICENSE file in the repository

## Support

For issues and questions:
- GitHub Issues: https://github.com/suraj202923/FastDataBroker/issues
- Documentation: Check the docs/ folder

## Contributing

Contributions are welcome! Please follow the project's contribution guidelines.

## Changelog

### Version 0.4.0
- Initial C# SDK release
- Synchronous and asynchronous message APIs
- WebSocket and Webhook support
- Comprehensive unit tests
- Multi-framework support (.NET 6.0, 7.0, 8.0)
