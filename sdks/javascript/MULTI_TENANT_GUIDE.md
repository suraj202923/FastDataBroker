# FastDataBroker TypeScript/JavaScript SDK - Multi-Tenant Documentation
**Version 0.1.16**

## Overview
The FastDataBroker TypeScript/JavaScript SDK provides comprehensive multi-tenant support with type-safe configuration management, tenant isolation, API key management, and per-tenant rate limiting.

## Key Features

- **Type-Safe**: Full TypeScript support with interfaces and enums
- **Configuration Management**: JSON-based config with environment overrides
- **Tenant Validation**: Built-in validation for tenant IDs and API keys
- **API Key Generation**: Tenant-aware key generation with prefix validation
- **Async/Await**: Modern async/await pattern for all operations
- **Environment Support**: Development, staging, and production environments

## Installation

### npm

```bash
npm install @fastdatabroker/sdk-js
```

### yarn

```bash
yarn add @fastdatabroker/sdk-js
```

## Quick Start

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

### 2. Create and Connect Client

```typescript
import { Client, ConfigurationManager, AppSettings } from '@fastdatabroker/sdk-js';

async function main() {
  // Method 1: Create with tenant ID and API key
  const client1 = Client.createWithTenant(
    'acme-corp',
    'acme_your_api_key_here',
    'localhost',
    6379
  );

  // Method 2: Load from configuration file
  const settings = ConfigurationManager.loadFromFile('appsettings.json', 'development');
  const client2 = Client.createFromSettings(settings, 'acme-corp', 'acme_your_api_key_here');

  // Method 3: Factory function
  const client3 = await createClient(
    'appsettings.json',
    'acme-corp',
    'acme_your_api_key_here',
    'development'
  );

  // Connect
  await client3.connect();
  console.log('Connected to FastDataBroker');
}

main().catch(console.error);
```

## Usage Examples

### Sending Messages with Tenant Isolation

```typescript
import { Message, Priority, Client } from '@fastdatabroker/sdk-js';

const message = new Message(
  'acme-corp',                    // tenantId
  'user123',                      // senderId
  ['user456', 'user789'],         // recipientIds
  'Hello from ACME',              // subject
  Buffer.from('Message content')  // content
);

// Configure message
message.priority = Priority.High;
message.ttlSeconds = 3600;
message.tags.set('category', 'notification');
message.tags.set('type', 'welcome');
message.requireConfirm = true;

// Send message
const result = await client.sendMessage(message);
console.log(`Message ID: ${result.messageId}`);
console.log(`Status: ${result.status}`);
console.log(`Tenant: ${result.tenantId}`);
console.log(`Delivered Channels: ${result.deliveredChannels}`);
```

### Generating Tenant-Specific API Keys

```typescript
// Generate new API key
const newApiKey = client.generateApiKey('client-001');
console.log(`Generated API Key: ${newApiKey}`);

// Validate API key belongs to tenant
const settings = ConfigurationManager.loadFromFile('appsettings.json', 'development');
const tenant = ConfigurationManager.getTenantByApiKey(settings, newApiKey);
if (tenant) {
  console.log(`Key belongs to tenant: ${tenant.tenant_id}`);
}
```

### WebSocket Client Management

```typescript
// Register WebSocket client
const registered = client.registerWebSocketClient('ws-client-001', 'user123');
console.log(`WebSocket client registered: ${registered}`);

// Unregister WebSocket client
const unregistered = client.unregisterWebSocketClient('ws-client-001');
console.log(`WebSocket client unregistered: ${unregistered}`);
```

### Webhook Configuration

```typescript
import { NotificationChannel, WebhookConfig } from '@fastdatabroker/sdk-js';

const webhookConfig: WebhookConfig = {
  url: 'https://example.com/webhooks/fastdatabroker',
  headers: {
    'Authorization': 'Bearer your-token',
    'X-Custom-Header': 'value'
  },
  retries: 3,
  timeoutMs: 30000,
  verifySsl: true
};

const success = client.registerWebhook(NotificationChannel.Webhook, webhookConfig);
console.log(`Webhook registered: ${success}`);
```

### Getting Tenant Configuration

```typescript
const tenantConfig = client.getTenantConfig();
if (tenantConfig) {
  console.log(`Tenant: ${tenantConfig.tenant_id}`);
  console.log(`Rate Limit: ${tenantConfig.rate_limit_rps} RPS`);
  console.log(`Max Connections: ${tenantConfig.max_connections}`);
  console.log(`Max Message Size: ${tenantConfig.max_message_size} bytes`);
  console.log(`Retention: ${tenantConfig.retention_days} days`);
}
```

## Configuration Structure

### TenantConfig Interface

```typescript
interface TenantConfig {
  tenant_id: string;
  tenant_name: string;
  api_key_prefix: string;
  rate_limit_rps: number;
  max_connections: number;
  max_message_size: number;
  retention_days: number;
  enabled?: boolean;
  metadata?: Record<string, any>;
}
```

### Validation Rules

- `tenant_id` cannot be empty
- `api_key_prefix` must end with `_`
- `rate_limit_rps` must be > 0
- `max_connections` must be > 0

## Environment-Specific Configuration

### Development (appsettings.development.json)

```json
{
  "app": {
    "name": "FastDataBroker",
    "version": "0.1.16",
    "environment": "development"
  },
  "server": {
    "bind_address": "localhost",
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
      "retention_days": 1
    }
  ]
}
```

### Production (appsettings.production.json)

```json
{
  "app": {
    "name": "FastDataBroker",
    "version": "0.1.16",
    "environment": "production"
  },
  "server": {
    "bind_address": "0.0.0.0",
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
      "retention_days": 30
    }
  ]
}
```

## Complete Example Application

```typescript
import {
  Client,
  ConfigurationManager,
  Message,
  Priority,
  NotificationChannel,
  createClient,
  AppSettings,
  TenantConfig,
  WebhookConfig,
  DeliveryResult
} from '@fastdatabroker/sdk-js';

async function multiTenantExample() {
  try {
    // Load configuration
    const settings = ConfigurationManager.loadFromFile('appsettings.json', 'development');
    console.log(`Loaded ${settings.tenants.length} tenants`);

    // Create client for ACME tenant
    const client = Client.createFromSettings(
      settings,
      'acme-corp',
      'acme_test_key_xyz_12345'
    );

    // Connect
    if (await client.connect()) {
      console.log('Connected successfully');

      // Send message
      const message = new Message(
        'acme-corp',
        'service-a',
        ['user-1', 'user-2'],
        'Notification',
        Buffer.from('Hello World')
      );
      message.priority = Priority.High;

      const result: DeliveryResult = await client.sendMessage(message);
      console.log(`Message sent: ${result.messageId}`);

      // Generate API key
      const newKey = client.generateApiKey('new-client');
      console.log(`Generated key: ${newKey}`);

      // Get tenant config
      const tenantConfig = client.getTenantConfig();
      if (tenantConfig) {
        console.log(`Rate limit: ${tenantConfig.rate_limit_rps} RPS`);
      }

      // Register webhook
      const webhookConfig: WebhookConfig = {
        url: 'https://example.com/webhooks',
        headers: { 'Authorization': 'Bearer token' },
        retries: 3
      };

      const webhookRegistered = client.registerWebhook(
        NotificationChannel.Webhook,
        webhookConfig
      );
      console.log(`Webhook registered: ${webhookRegistered}`);

      // Register WebSocket client
      const wsRegistered = client.registerWebSocketClient('ws-001', 'user-001');
      console.log(`WebSocket client registered: ${wsRegistered}`);

      // Disconnect
      client.disconnect();
      console.log('Disconnected');
    }
  } catch (error) {
    console.error('Error:', error);
  }
}

multiTenantExample();
```

## Error Handling

```typescript
import { Client } from '@fastdatabroker/sdk-js';

async function errorHandlingExample() {
  try {
    const client = Client.createWithTenant(
      'acme-corp',
      'acme_api_key',
      'localhost',
      6379
    );

    await client.connect();
    
    // ... operations ...
    
  } catch (error) {
    if (error instanceof Error) {
      if (error.message.includes('not found')) {
        console.error('Tenant configuration error:', error.message);
      } else if (error.message.includes('already')) {
        console.error('Connection error:', error.message);
      } else {
        console.error('Unexpected error:', error.message);
      }
    }
  }
}
```

## Priority Enum

```typescript
enum Priority {
  Deferred = 50,
  Normal = 100,
  High = 150,
  Urgent = 200,
  Critical = 255
}
```

## Notification Channels

```typescript
enum NotificationChannel {
  Email = 'email',
  WebSocket = 'websocket',
  Push = 'push',
  Webhook = 'webhook'
}
```

## Push Platforms

```typescript
enum PushPlatform {
  Firebase = 'firebase',
  APNs = 'apns',
  FCM = 'fcm',
  WebPush = 'webpush'
}
```

## Tenant Isolation Guarantees

1. **Type Safety**: TypeScript interfaces enforce type checking at compile time
2. **Configuration Validation**: Tenant configs validated at load time
3. **API Key Validation**: API keys must match tenant prefix
4. **Message Isolation**: Messages cannot cross tenant boundaries
5. **Connection Context**: Each client operates in single tenant context

## Configuration Manager API

```typescript
class ConfigurationManager {
  static loadFromFile(
    filePath: string,
    environment?: string
  ): AppSettings;

  static getTenant(
    settings: AppSettings,
    tenantId: string
  ): TenantConfig | undefined;

  static getTenantByApiKey(
    settings: AppSettings,
    apiKey: string
  ): TenantConfig | undefined;

  static validateTenant(tenant: TenantConfig): void;
}
```

## Client API

```typescript
class Client {
  static createWithTenant(
    tenantId: string,
    apiKey: string,
    host?: string,
    port?: number
  ): Client;

  static createFromSettings(
    settings: AppSettings,
    tenantId: string,
    apiKey: string
  ): Client;

  async connect(): Promise<boolean>;
  async sendMessage(message: Message): Promise<DeliveryResult>;
  registerWebSocketClient(clientId: string, userId: string): boolean;
  unregisterWebSocketClient(clientId: string): boolean;
  registerWebhook(channel: NotificationChannel, config: WebhookConfig): boolean;
  generateApiKey(clientId: string): string;
  getTenantConfig(): TenantConfig | undefined;
  disconnect(): void;
  isConnected(): boolean;
  getTenantId(): string;
}
```

## Migration from Previous Version

### Before (v0.1.13)
```typescript
const client = new Client('localhost', 6379);
await client.connect();
```

### After (v0.1.16)
```typescript
const client = Client.createWithTenant(
  'acme-corp',
  'acme_api_key_xxx',
  'localhost',
  6379
);
await client.connect();
```

## Version History

### v0.1.16
- âœ… Multi-tenant support
- âœ… Type-safe configuration
- âœ… Environment-specific overrides
- âœ… Tenant validation
- âœ… API key generation
- âœ… Full async/await support

### v0.1.13
- Initial release

## Support

For issues or questions, refer to the main FastDataBroker documentation.

