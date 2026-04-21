# FastDataBroker C# SDK - Multi-Tenant Documentation
**Version 0.1.16**

## Overview
The FastDataBroker C# SDK now includes comprehensive multi-tenant support, allowing applications to manage multiple tenants with isolated configurations, API keys, rate limiting, and feature flags.

## Key Features

### Multi-Tenant Configuration
- Load configuration from JSON files (appsettings.json)
- Environment-specific overrides (development, staging, production)
- Per-tenant settings including rate limiting and connection limits
- Tenant validation and isolation

### API Key Management
- Tenant-aware API key generation
- API key prefix validation
- Per-tenant key management

### Rate Limiting & Quotas
- Per-tenant rate limiting (RPS)
- Connection limit enforcement
- Message size limits
- Data retention policies

## Getting Started

### 1. Configuration File (appsettings.json)

```json
{
  "app": {
    "name": "FastDataBroker",
    "version": "0.1.16",
    "environment": "development"
  },
  "server": {
    "bind_address": "0.0.0.0",
    "port": 6379,
    "enable_tls": false,
    "cert_path": "./certs/cert.pem",
    "key_path": "./certs/key.pem"
  },
  "tenants": [
    {
      "tenant_id": "acme-corp",
      "tenant_name": "ACME Corporation",
      "api_key_prefix": "acme_",
      "rate_limit_rps": 1000,
      "max_connections": 100,
      "max_message_size": 1048576,
      "retention_days": 30,
      "enabled": true,
      "metadata": {
        "tier": "enterprise",
        "region": "us-east-1"
      }
    },
    {
      "tenant_id": "startup-xyz",
      "tenant_name": "Startup XYZ",
      "api_key_prefix": "xyz_",
      "rate_limit_rps": 100,
      "max_connections": 10,
      "max_message_size": 524288,
      "retention_days": 7,
      "enabled": true
    }
  ]
}
```

### 2. Creating a Client with Tenant Support

```csharp
using FastDataBroker;

// Method 1: Create client with tenant ID and API key
var client = new FastDataBrokerSDK.Client(
    tenantId: "acme-corp",
    apiKey: "acme_your_api_key_here",
    host: "localhost",
    port: 6379
);

// Method 2: Load from configuration file
var settings = FastDataBrokerSDK.AppSettings.LoadFromFile(
    "appsettings.json",
    environment: "development"
);

var client = new FastDataBrokerSDK.Client(settings, "acme-corp", "acme_your_api_key_here");

// Method 3: Factory method
var client = FastDataBrokerSDK.CreateClient(
    "appsettings.json",
    "acme-corp",
    "acme_your_api_key_here",
    "development"
);

// Connect to server
await client.ConnectAsync();
```

### 3. Sending Messages with Tenant Isolation

```csharp
var message = new FastDataBrokerSDK.Message(
    tenantId: "acme-corp",
    senderId: "user123",
    recipientIds: new List<string> { "user456", "user789" },
    subject: "Hello from ACME",
    content: Encoding.UTF8.GetBytes("Message content")
)
{
    Priority = FastDataBrokerSDK.Priority.High,
    TTLSeconds = 3600,
    Tags = new Dictionary<string, string>
    {
        { "category", "notification" },
        { "type", "welcome" }
    }
};

// Send message
var result = await client.SendMessageAsync(message);
Console.WriteLine($"Message ID: {result.MessageId}");
Console.WriteLine($"Status: {result.Status}");
Console.WriteLine($"Tenant: {result.TenantId}");
```

### 4. Generating Tenant-Specific API Keys

```csharp
// Generate new API key for a client
string newApiKey = client.GenerateApiKey("client-001");
Console.WriteLine($"Generated API Key: {newApiKey}");

// Validate API key belongs to tenant
var tenant = settings.GetTenantByApiKey(newApiKey);
if (tenant != null)
{
    Console.WriteLine($"Key belongs to tenant: {tenant.TenantId}");
}
```

### 5. WebSocket Support with Tenant Isolation

```csharp
// Register WebSocket client
bool registered = client.RegisterWebSocketClient("ws-client-001", "user123");

// Unregister WebSocket client
bool unregistered = client.UnregisterWebSocketClient("ws-client-001");
```

### 6. Webhook Configuration per Tenant

```csharp
var webhookConfig = new FastDataBrokerSDK.WebhookConfig
{
    Url = "https://example.com/webhooks/fastdatabroker",
    Headers = new Dictionary<string, string>
    {
        { "Authorization", "Bearer your-token" },
        { "X-Custom-Header", "value" }
    },
    Retries = 3,
    TimeoutMs = 30000,
    VerifySSL = true
};

client.RegisterWebhook(
    FastDataBrokerSDK.NotificationChannel.Webhook,
    webhookConfig
);
```

## Tenant Configuration Structure

### TenantConfig Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `TenantId` | string | Yes | Unique identifier for the tenant |
| `TenantName` | string | Yes | Human-readable name |
| `ApiKeyPrefix` | string | Yes | Prefix for API keys (must end with `_`) |
| `RateLimitRps` | uint | Yes | Rate limit in requests per second |
| `MaxConnections` | uint | Yes | Maximum concurrent connections |
| `MaxMessageSize` | ulong | No | Maximum message size in bytes |
| `RetentionDays` | uint | No | Message retention period |
| `Enabled` | bool | No | Whether tenant is active (default: true) |
| `Metadata` | Dictionary | No | Custom metadata key-value pairs |

### Validation Rules

- TenantId cannot be empty
- ApiKeyPrefix must end with `_`
- RateLimitRps must be > 0
- MaxConnections must be > 0

## Environment-Specific Configuration

### Development (appsettings.development.json)
```json
{
  "app": {
    "environment": "development"
  },
  "server": {
    "enable_tls": false
  },
  "tenants": [
    {
      "tenant_id": "test-tenant",
      "api_key_prefix": "test_",
      "rate_limit_rps": 100,
      "max_connections": 5
    }
  ]
}
```

### Production (appsettings.production.json)
```json
{
  "app": {
    "environment": "production"
  },
  "server": {
    "enable_tls": true,
    "cert_path": "/etc/certs/cert.pem",
    "key_path": "/etc/certs/key.pem"
  },
  "tenants": [
    {
      "tenant_id": "acme-corp",
      "api_key_prefix": "acme_",
      "rate_limit_rps": 5000,
      "max_connections": 500
    }
  ]
}
```

## Tenant Isolation Guarantees

1. **API Key Isolation**: API keys are validated against tenant prefix
2. **Message Isolation**: Messages can only be sent to the same tenant
3. **Connection Isolation**: Each client connects with explicit tenant context
4. **Rate Limiting**: Per-tenant rate limits are enforced
5. **Configuration Isolation**: Each tenant has independent settings

## Error Handling

```csharp
try
{
    await client.ConnectAsync();
    
    var result = await client.SendMessageAsync(message);
}
catch (ArgumentException ex)
{
    // Configuration or validation error
    Console.WriteLine($"Configuration error: {ex.Message}");
}
catch (InvalidOperationException ex)
{
    // Connection or tenant isolation error
    Console.WriteLine($"Operational error: {ex.Message}");
}
catch (Exception ex)
{
    // Unexpected error
    Console.WriteLine($"Error: {ex.Message}");
}
finally
{
    client.Disconnect();
}
```

## Migration from Previous SDK Version

### Before (v0.1.13)
```csharp
var client = new FastDataBrokerSDK.Client("localhost", 6379);
await client.ConnectAsync();
```

### After (v0.1.16)
```csharp
var client = new FastDataBrokerSDK.Client(
    tenantId: "acme-corp",
    apiKey: "acme_key_xxx",
    host: "localhost",
    port: 6379
);
await client.ConnectAsync();
```

## Version History

### v0.1.16
- âœ… Added multi-tenant support
- âœ… Added AppSettings configuration loader
- âœ… Added TenantConfig with validation
- âœ… Added tenant-aware API key generation
- âœ… Added environment-specific configuration overrides
- âœ… Added tenant isolation guarantees

### v0.1.13
- Initial release with basic messaging

## Support

For issues, questions, or contributions, please refer to the main FastDataBroker documentation.

