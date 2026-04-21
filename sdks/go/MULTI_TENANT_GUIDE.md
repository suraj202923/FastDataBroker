# FastDataBroker Go SDK - Multi-Tenant Documentation
**Version 0.1.16**

## Overview
The FastDataBroker Go SDK now includes comprehensive multi-tenant support with configuration loading, tenant management, API key generation, and rate limiting.

## Key Features

- **Multi-Tenant Configuration**: Load from JSON files with environment-specific overrides
- **Tenant Validation**: Built-in validation for tenant IDs, API keys, and settings
- **API Key Management**: Tenant-aware key generation with prefix validation
- **Rate Limiting**: Per-tenant rate limit configuration and enforcement hooks
- **Configuration Overrides**: Environment-specific configuration files (development, staging, production)

## Quick Start

### 1. Import the Package

```go
package main

import (
	"fmt"
	"log"
	"github.com/your-org/fastdatabroker-go"
)
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

### 3. Create a Client

```go
// Method 1: Create with tenant ID and API key
client, err := fastdatabroker.NewClientWithTenant(
	"acme-corp",
	"acme_your_api_key_here",
	"localhost",
	6379,
)
if err != nil {
	log.Fatal(err)
}

// Method 2: Load from configuration file
settings, err := fastdatabroker.LoadFromFile("appsettings.json", "development")
if err != nil {
	log.Fatal(err)
}

client, err := fastdatabroker.NewClientFromSettings(settings, "acme-corp", "acme_your_api_key_here")
if err != nil {
	log.Fatal(err)
}

// Method 3: Factory function
client, err := fastdatabroker.CreateClient(
	"appsettings.json",
	"acme-corp",
	"acme_your_api_key_here",
	"development",
)
if err != nil {
	log.Fatal(err)
}

// Connect
if err := client.Connect(); err != nil {
	log.Fatal(err)
}
defer client.Disconnect()
```

## Usage Examples

### Sending Messages with Tenant Isolation

```go
message := &fastdatabroker.Message{
	TenantID:   "acme-corp",
	SenderID:   "user123",
	RecipientIDs: []string{"user456", "user789"},
	Subject:    "Hello from ACME",
	Content:    []byte("Message content"),
	Priority:   fastdatabroker.High,
	TTLSeconds: ptrInt64(3600),
	Tags: map[string]string{
		"category": "notification",
		"type":     "welcome",
	},
	RequireConfirm: false,
}

result, err := client.SendMessage(message)
if err != nil {
	log.Println("Error sending message:", err)
} else {
	fmt.Printf("Message ID: %s\n", result.MessageID)
	fmt.Printf("Status: %s\n", result.Status)
	fmt.Printf("Tenant: %s\n", result.TenantID)
}

func ptrInt64(i int64) *int64 {
	return &i
}
```

### Generating Tenant-Specific API Keys

```go
newAPIKey, err := client.GenerateAPIKey("client-001")
if err != nil {
	log.Fatal(err)
}

fmt.Printf("Generated API Key: %s\n", newAPIKey)

// Validate API key belongs to tenant
tenant := settings.GetTenantByAPIKey(newAPIKey)
if tenant != nil {
	fmt.Printf("Key belongs to tenant: %s\n", tenant.TenantID)
}
```

### WebSocket Clients (Tenant-Isolated)

```go
// Register WebSocket client
registered := client.RegisterWebSocketClient("ws-client-001", "user123")
fmt.Printf("WebSocket client registered: %v\n", registered)

// Unregister WebSocket client
unregistered := client.UnregisterWebSocketClient("ws-client-001")
fmt.Printf("WebSocket client unregistered: %v\n", unregistered)
```

### Webhook Configuration

```go
webhookConfig := &fastdatabroker.WebhookConfig{
	URL: "https://example.com/webhooks/fastdatabroker",
	Headers: map[string]string{
		"Authorization": "Bearer your-token",
		"X-Custom-Header": "value",
	},
	Retries:   3,
	TimeoutMs: 30000,
	VerifySSL: true,
}

success := client.RegisterWebhook(fastdatabroker.Webhook, webhookConfig)
fmt.Printf("Webhook registered: %v\n", success)
```

### Getting Tenant Configuration

```go
tenantConfig := client.GetTenantConfig()
if tenantConfig != nil {
	fmt.Printf("Tenant: %s\n", tenantConfig.TenantID)
	fmt.Printf("Rate Limit: %d RPS\n", tenantConfig.RateLimitRps)
	fmt.Printf("Max Connections: %d\n", tenantConfig.MaxConnections)
}
```

## Configuration Structure

### TenantConfig Fields

```go
type TenantConfig struct {
	TenantID       string                 `json:"tenant_id"`
	TenantName     string                 `json:"tenant_name"`
	APIKeyPrefix   string                 `json:"api_key_prefix"`
	RateLimitRps   uint32                 `json:"rate_limit_rps"`
	MaxConnections uint32                 `json:"max_connections"`
	MaxMessageSize uint64                 `json:"max_message_size"`
	RetentionDays  uint32                 `json:"retention_days"`
	Enabled        bool                   `json:"enabled"`
	Metadata       map[string]interface{} `json:"metadata,omitempty"`
}
```

### Validation Rules

- TenantID cannot be empty
- APIKeyPrefix must end with `_`
- RateLimitRps must be > 0
- MaxConnections must be > 0

## Environment-Specific Configuration

### Development Environment

**appsettings.development.json**
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
      "api_key_prefix": "dev_",
      "rate_limit_rps": 100,
      "max_connections": 5
    }
  ]
}
```

### Production Environment

**appsettings.production.json**
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
      "api_key_prefix": "acme_",
      "rate_limit_rps": 5000,
      "max_connections": 500
    }
  ]
}
```

## Complete Example

```go
package main

import (
	"fmt"
	"log"
	"github.com/your-org/fastdatabroker-go"
)

func main() {
	// Load settings from file
	settings, err := fastdatabroker.LoadFromFile("appsettings.json", "development")
	if err != nil {
		log.Fatal(err)
	}

	// Create client
	client, err := fastdatabroker.NewClientFromSettings(
		settings,
		"acme-corp",
		"acme_test_key_xyz",
	)
	if err != nil {
		log.Fatal(err)
	}

	// Connect
	if err := client.Connect(); err != nil {
		log.Fatal(err)
	}
	defer client.Disconnect()

	// Send message
	message := &fastdatabroker.Message{
		TenantID:       "acme-corp",
		SenderID:       "service-a",
		RecipientIDs:   []string{"user-1", "user-2"},
		Subject:        "Notification",
		Content:        []byte("Hello World"),
		Priority:       fastdatabroker.Normal,
		RequireConfirm: true,
	}

	result, err := client.SendMessage(message)
	if err != nil {
		log.Println("Error:", err)
	} else {
		fmt.Printf("Sent Message ID: %s\n", result.MessageID)
	}

	// Generate API key for new client
	apiKey, err := client.GenerateAPIKey("new-client")
	if err != nil {
		log.Println("Error generating key:", err)
	} else {
		fmt.Printf("Generated API Key: %s\n", apiKey)
	}
}
```

## Error Handling

```go
import "errors"

// Verify error type
if errors.Is(err, fastdatabroker.ErrInvalidTenant) {
	log.Println("Invalid tenant configuration")
} else if errors.Is(err, fastdatabroker.ErrNotConnected) {
	log.Println("Client not connected")
} else {
	log.Printf("Unexpected error: %v\n", err)
}
```

## Tenant Isolation Guarantees

1. **API Key Validation**: API keys must match the tenant's prefix
2. **Message Isolation**: Messages are tied to tenant ID
3. **Connection Context**: Each client operates within a single tenant context
4. **Configuration Isolation**: Each tenant has independent settings
5. **Rate Limiting**: Per-tenant rate limits are tracked

## Migration Guide

### From Previous Version

```go
// Old (v0.1.13)
client := fastdatabroker.NewClient("localhost", 6379)

// New (v0.1.16)
client, err := fastdatabroker.NewClientWithTenant(
	"acme-corp",
	"acme_api_key",
	"localhost",
	6379,
)
```

## Performance Considerations

- Configuration is loaded once at startup
- API key generation is fast (< 1ms)
- Tenant lookups are O(n) - consider caching frequently accessed tenants
- WebSocket client registration uses in-memory map for O(1) lookups

## Version History

### v0.1.16
- âœ… Multi-tenant support
- âœ… AppSettings configuration loader
- âœ… Environment-specific overrides
- âœ… Tenant validation
- âœ… Tenant-aware API key generation

### v0.1.13
- Initial release

## Support

For issues or questions, refer to the main FastDataBroker documentation.

