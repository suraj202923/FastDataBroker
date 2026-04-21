# FastDataBroker Java SDK - Multi-Tenant Documentation
**Version 0.1.16**

## Overview
The FastDataBroker Java SDK provides comprehensive multi-tenant support with configuration management, tenant isolation, API key management, and per-tenant rate limiting.

## Key Features

- **Multi-Tenant Configuration**: JSON-based configuration with Jackson
- **Environment Support**: Development, staging, and production environments
- **API Key Management**: Tenant-aware API key generation and validation
- **Tenant Isolation**: Guaranteed message and connection isolation
- **Configuration Loading**: Automatic environment-specific overrides

## Quick Start

### 1. Maven Dependency

```xml
<dependency>
    <groupId>com.fastdatabroker</groupId>
    <artifactId>fastdatabroker-sdk</artifactId>
    <version>0.1.16</version>
</dependency>

<!-- Required dependencies -->
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
    <version>2.15.0</version>
</dependency>
```

### 2. Configuration File (appsettings.json)

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

### 3. Create and Use Client

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import java.io.IOException;

public class Main {
    public static void main(String[] args) throws IOException {
        // Method 1: Direct client creation
        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            "acme-corp",
            "acme_your_api_key_here",
            "localhost",
            6379
        );

        // Method 2: Load from configuration
        FastDataBrokerSDK.AppSettings settings = 
            FastDataBrokerSDK.AppSettings.loadFromFile(
                "appsettings.json",
                "development"
            );

        client = new FastDataBrokerSDK.Client(settings, "acme-corp", "acme_your_api_key_here");

        // Method 3: Factory method
        client = FastDataBrokerSDK.createClient(
            "appsettings.json",
            "acme-corp",
            "acme_your_api_key_here",
            "development"
        );

        // Connect
        if (client.connect()) {
            System.out.println("Connected successfully");
        }
    }
}
```

## Usage Examples

### Sending Messages with Tenant Isolation

```java
import java.util.*;

// Create message
FastDataBrokerSDK.Message message = new FastDataBrokerSDK.Message(
    "acme-corp",
    "user123",
    Arrays.asList("user456", "user789"),
    "Hello from ACME",
    "Message content".getBytes()
);

// Configure message
message.priority = FastDataBrokerSDK.Priority.HIGH;
message.ttlSeconds = 3600L;
message.tags.put("category", "notification");
message.tags.put("type", "welcome");
message.requireConfirm = true;

// Send message
FastDataBrokerSDK.DeliveryResult result = client.sendMessage(message);

System.out.println("Message ID: " + result.messageId);
System.out.println("Status: " + result.status);
System.out.println("Tenant: " + result.tenantId);
System.out.println("Delivered Channels: " + result.deliveredChannels);
```

### Generating Tenant-Specific API Keys

```java
// Generate new API key
String newApiKey = client.generateApiKey("client-001");
System.out.println("Generated API Key: " + newApiKey);

// Validate API key belongs to tenant
FastDataBrokerSDK.TenantConfig tenant = settings.getTenantByApiKey(newApiKey);
if (tenant != null) {
    System.out.println("Key belongs to tenant: " + tenant.tenantId);
}
```

### WebSocket Client Management (Tenant-Isolated)

```java
// Register WebSocket client
boolean registered = client.registerWebSocketClient("ws-client-001", "user123");
System.out.println("Registered: " + registered);

// Unregister WebSocket client
boolean unregistered = client.unregisterWebSocketClient("ws-client-001");
System.out.println("Unregistered: " + unregistered);
```

### Webhook Configuration

```java
// Create webhook config
FastDataBrokerSDK.WebhookConfig webhookConfig = 
    new FastDataBrokerSDK.WebhookConfig();

webhookConfig.url = "https://example.com/webhooks/fastdatabroker";
webhookConfig.headers.put("Authorization", "Bearer your-token");
webhookConfig.headers.put("X-Custom-Header", "value");
webhookConfig.retries = 3;
webhookConfig.timeoutMs = 30000;
webhookConfig.verifySsl = true;

// Register webhook
boolean registered = client.registerWebhook(
    FastDataBrokerSDK.NotificationChannel.WEBHOOK,
    webhookConfig
);

System.out.println("Webhook registered: " + registered);
```

### Accessing Tenant Configuration

```java
// Get tenant configuration for current client
FastDataBrokerSDK.TenantConfig tenantConfig = client.getTenantConfig();

if (tenantConfig != null) {
    System.out.println("Tenant ID: " + tenantConfig.tenantId);
    System.out.println("Tenant Name: " + tenantConfig.tenantName);
    System.out.println("Rate Limit: " + tenantConfig.rateLimitRps + " RPS");
    System.out.println("Max Connections: " + tenantConfig.maxConnections);
    System.out.println("Max Message Size: " + tenantConfig.maxMessageSize + " bytes");
    System.out.println("Retention Days: " + tenantConfig.retentionDays);
}
```

## Configuration Structure

### TenantConfig

```java
public static class TenantConfig {
    @JsonProperty("tenant_id")
    public String tenantId;

    @JsonProperty("tenant_name")
    public String tenantName;

    @JsonProperty("api_key_prefix")
    public String apiKeyPrefix;

    @JsonProperty("rate_limit_rps")
    public long rateLimitRps;

    @JsonProperty("max_connections")
    public long maxConnections;

    @JsonProperty("max_message_size")
    public long maxMessageSize;

    @JsonProperty("retention_days")
    public long retentionDays;

    public boolean enabled = true;

    @JsonProperty("metadata")
    public Map<String, Object> metadata;
}
```

### Validation Rules

- `tenantId` cannot be empty
- `apiKeyPrefix` must end with `_`
- `rateLimitRps` must be > 0
- `maxConnections` must be > 0

## Environment-Specific Configuration

### Development (appsettings.development.json)

```json
{
  "app": {
    "environment": "development"
  },
  "server": {
    "port": 6379,
    "enable_tls": false
  },
  "tenants": [
    {
      "tenant_id": "dev-tenant",
      "tenant_name": "Development",
      "api_key_prefix": "dev_",
      "rate_limit_rps": 100,
      "max_connections": 5,
      "max_message_size": 524288,
      "retention_days": 1,
      "enabled": true
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
    "port": 6379,
    "enable_tls": true,
    "cert_path": "/etc/certs/cert.pem",
    "key_path": "/etc/certs/key.pem"
  },
  "tenants": [
    {
      "tenant_id": "acme-corp",
      "tenant_name": "ACME Corporation",
      "api_key_prefix": "acme_",
      "rate_limit_rps": 5000,
      "max_connections": 500,
      "max_message_size": 10485760,
      "retention_days": 30,
      "enabled": true
    }
  ]
}
```

## Complete Example Application

```java
import com.fastdatabroker.sdk.FastDataBrokerSDK;
import java.io.IOException;
import java.util.*;

public class MultiTenantExample {
    public static void main(String[] args) {
        try {
            // Load configuration
            FastDataBrokerSDK.AppSettings settings = 
                FastDataBrokerSDK.AppSettings.loadFromFile(
                    "appsettings.json",
                    "development"
                );

            System.out.println("Configuration loaded: " + settings.tenants.size() + " tenants");

            // Create client for first tenant
            String tenantId = "acme-corp";
            String apiKey = "acme_test_key_xyz_12345";

            FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
                settings,
                tenantId,
                apiKey
            );

            // Connect
            if (client.connect()) {
                // Send message
                FastDataBrokerSDK.Message message = new FastDataBrokerSDK.Message(
                    tenantId,
                    "service-a",
                    Arrays.asList("user-1", "user-2"),
                    "Notification",
                    "Hello World".getBytes()
                );

                message.priority = FastDataBrokerSDK.Priority.HIGH;

                FastDataBrokerSDK.DeliveryResult result = client.sendMessage(message);
                System.out.println("Message sent: " + result.messageId);

                // Generate API key
                String newKey = client.generateApiKey("new-client");
                System.out.println("Generated key: " + newKey);

                // Get tenant config
                FastDataBrokerSDK.TenantConfig config = client.getTenantConfig();
                System.out.println("Rate limit: " + config.rateLimitRps + " RPS");

                // Disconnect
                client.disconnect();
            }
        } catch (IOException e) {
            System.err.println("Configuration error: " + e.getMessage());
            e.printStackTrace();
        } catch (IllegalArgumentException e) {
            System.err.println("Validation error: " + e.getMessage());
        } catch (IllegalStateException e) {
            System.err.println("State error: " + e.getMessage());
        } catch (Exception e) {
            System.err.println("Unexpected error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
```

## Error Handling

```java
try {
    FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
        settings,
        tenantId,
        apiKey
    );
    
    if (!client.connect()) {
        System.err.println("Failed to connect");
        return;
    }
    
    FastDataBrokerSDK.DeliveryResult result = client.sendMessage(message);
    
} catch (IllegalArgumentException e) {
    // Configuration or validation error
    System.err.println("Configuration error: " + e.getMessage());
} catch (IllegalStateException e) {
    // Connection or tenant isolation error
    System.err.println("Operational error: " + e.getMessage());
} catch (IOException e) {
    // File I/O error
    System.err.println("I/O error: " + e.getMessage());
} finally {
    if (client != null) {
        client.disconnect();
    }
}
```

## Tenant Isolation Guarantees

1. **Configuration Validation**: Tenant config is validated at load time
2. **API Key Validation**: API keys must match tenant prefix
3. **Message Isolation**: Messages are tied to tenant ID and validated
4. **Connection Context**: Each client operates in single tenant context
5. **Rate Limiting**: Per-tenant rate limits configured per settings

## Priority Levels

```java
FastDataBrokerSDK.Priority.DEFERRED    // byte value: 50
FastDataBrokerSDK.Priority.NORMAL      // byte value: 100
FastDataBrokerSDK.Priority.HIGH        // byte value: 150
FastDataBrokerSDK.Priority.URGENT      // byte value: 200
FastDataBrokerSDK.Priority.CRITICAL    // byte value: 255
```

## Notification Channels

```java
FastDataBrokerSDK.NotificationChannel.EMAIL
FastDataBrokerSDK.NotificationChannel.WEBSOCKET
FastDataBrokerSDK.NotificationChannel.PUSH
FastDataBrokerSDK.NotificationChannel.WEBHOOK
```

## Migration from Previous Version

### Before (v0.1.13)
```java
FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client("localhost", 6379);
client.connect();
```

### After (v0.1.16)
```java
FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
    "acme-corp",
    "acme_api_key_xxx",
    "localhost",
    6379
);
client.connect();
```

## Version History

### v0.1.16
- âœ… Multi-tenant support
- âœ… AppSettings configuration loader
- âœ… TenantConfig with validation
- âœ… Tenant-aware API key generation
- âœ… Environment-specific overrides

### v0.1.13
- Initial release

## Support

For issues or questions, refer to the main FastDataBroker documentation.

