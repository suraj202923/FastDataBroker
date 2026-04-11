# FastDataBroker JavaScript/TypeScript SDK

JavaScript/TypeScript SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

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

- 🚀 **Promise-Based Async/Await** - Modern async/await and Promise support
- 📨 **Multi-Channel Delivery** - Email, WebSocket, Push Notifications, Webhooks
- 🎯 **Priority Levels** - 5 priority levels: Deferred, Normal, High, Urgent, Critical
- 🔄 **Message Confirmation** - Optional delivery confirmation
- 🏷️ **Message Tagging** - Tag messages for categorization
- ⏱️ **TTL Support** - Set time-to-live for messages
- 🔌 **WebSocket Support** - Real-time bidirectional communication
- 🪝 **Webhook Endpoints** - Integrate with external systems
- 🌐 **QUIC Protocol** - High-performance UDP-based protocol
- 🔐 **Clustering Support** - Multi-region failover and load balancing
- 📦 **TypeScript Support** - Full type definitions included
- 🎯 **Event Emitters** - Node.js EventEmitter pattern support
- ⚙️ **Node.js & Browser** - Works in both environments

## Installation

### NPM

```bash
npm install @fastdatabroker/sdk
```

### Yarn

```bash
yarn add @fastdatabroker/sdk
```

### PNPM

```bash
pnpm add @fastdatabroker/sdk
```

### From Source

```bash
cd sdks/javascript
npm install
npm run build
```

## Quick Start

### 1. Basic Client Setup (JavaScript)

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

// Create client
const client = new FastDataBrokerSDK('localhost', 6000);

// Connect to FastDataBroker
await client.connect();
console.log('Connected to FastDataBroker');

// Your code here

await client.disconnect();
```

### 2. Basic Client Setup (TypeScript)

```typescript
import { FastDataBrokerSDK, Priority, IMessage, ISendResult } from '@fastdatabroker/sdk';

const client = new FastDataBrokerSDK('localhost', 6000);

await client.connect();
console.log('Connected to FastDataBroker');

// Your code here

await client.disconnect();
```

### 3. Send a Simple Message

```javascript
const message = {
  senderId: 'app-001',
  recipientIds: ['user-123', 'user-456'],
  subject: 'Notification',
  content: Buffer.from('Hello from FastDataBroker!'),
  priority: Priority.HIGH,
};

const result = await client.sendMessage(message);
console.log(`✓ Message sent: ${result.messageId}`);
console.log(`  Status: ${result.status}`);
console.log(`  Delivered channels: ${result.deliveredChannels}`);
```

## Complete Examples

### Example 1: Priority-Based Messaging

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function priorityExample() {
  const client = new FastDataBrokerSDK('localhost', 6000);
  
  try {
    await client.connect();

    // Critical priority message
    const criticalMsg = {
      senderId: 'system',
      recipientIds: ['admin'],
      subject: 'CRITICAL: System Alert',
      content: Buffer.from('Immediate action required'),
      priority: Priority.CRITICAL,  // 255
    };

    const result1 = await client.sendMessage(criticalMsg);
    console.log(`✓ Critical message sent: ${result1.messageId}`);

    // Urgent priority message
    const urgentMsg = {
      senderId: 'system',
      recipientIds: ['manager'],
      subject: 'URGENT: Important Update',
      content: Buffer.from('Requires immediate attention'),
      priority: Priority.URGENT,  // 200
    };

    const result2 = await client.sendMessage(urgentMsg);
    console.log(`✓ Urgent message sent: ${result2.messageId}`);

    // Normal priority message
    const normalMsg = {
      senderId: 'system',
      recipientIds: ['user'],
      subject: 'Regular Update',
      content: Buffer.from('Routine notification'),
      priority: Priority.NORMAL,  // 100
    };

    const result3 = await client.sendMessage(normalMsg);
    console.log(`✓ Normal message sent: ${result3.messageId}`);

    // Deferred priority message
    const deferredMsg = {
      senderId: 'system',
      recipientIds: ['background-worker'],
      subject: 'Background Task',
      content: Buffer.from('Can be processed later'),
      priority: Priority.DEFERRED,  // 50
    };

    const result4 = await client.sendMessage(deferredMsg);
    console.log(`✓ Deferred message sent: ${result4.messageId}`);
  } finally {
    await client.disconnect();
  }
}

priorityExample();
```

### Example 2: Batch Message Sending with TTL

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function batchExample() {
  const client = new FastDataBrokerSDK('localhost', 6000);
  
  try {
    await client.connect();

    const messages = [];
    
    // Create 5 messages with different TTLs
    for (let i = 0; i < 5; i++) {
      messages.push({
        senderId: 'batch-sender',
        recipientIds: [`recipient-${i}`],
        subject: `Batch Message ${i + 1}`,
        content: Buffer.from(`Content for message ${i + 1}`),
        priority: Priority.HIGH,
        ttlSeconds: 3600 * (i + 1),  // 1, 2, 3, 4, 5 hours
        requireConfirmation: true,
      });
    }

    // Send all messages
    for (const msg of messages) {
      try {
        const result = await client.sendMessage(msg);
        console.log(`✓ Message sent - ID: ${result.messageId}, TTL: ${msg.ttlSeconds}s`);
      } catch (error) {
        console.error(`✗ Failed: ${error.message}`);
      }
    }
  } finally {
    await client.disconnect();
  }
}

batchExample();
```

### Example 3: Tagged Messages for Organization

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function taggedExample() {
  const client = new FastDataBrokerSDK('localhost', 6000);
  
  try {
    await client.connect();

    const message = {
      senderId: 'order-service',
      recipientIds: ['customer-789'],
      subject: 'Order Confirmation',
      content: Buffer.from('Your order has been confirmed'),
      priority: Priority.HIGH,
      tags: {
        'order-id': 'ORD-2024-001234',
        'region': 'us-west-2',
        'category': 'order-notification',
        'version': 'v2',
        'timestamp': new Date().toISOString(),
      },
    };

    const result = await client.sendMessage(message);
    console.log(`✓ Tagged message sent: ${result.messageId}`);
    console.log('  Tags:');
    Object.entries(message.tags).forEach(([key, value]) => {
      console.log(`    - ${key}: ${value}`);
    });
  } finally {
    await client.disconnect();
  }
}

taggedExample();
```

### Example 4: WebSocket Integration

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function webSocketExample() {
  const wsClient = new FastDataBrokerSDK.WebSocketClient('ws://localhost:6001');

  try {
    await wsClient.connect();
    console.log('✓ WebSocket client connected');

    // Send message via WebSocket
    const message = {
      senderId: 'app',
      recipientIds: ['user-1', 'user-2', 'user-3'],
      subject: 'Real-time Update',
      content: Buffer.from('WebSocket real-time notification'),
      priority: Priority.URGENT,
    };

    const result = await wsClient.sendMessage(message);
    console.log(`✓ Message sent via WebSocket: ${result.messageId}`);

    // Listen for messages using event emitter
    wsClient.on('message', (message) => {
      console.log(`✓ Received: ${message.subject}`);
    });

    wsClient.on('error', (error) => {
      console.error(`✗ WebSocket error: ${error.message}`);
    });

    // Subscribe to specific user messages
    const stream = wsClient.subscribe('user-1');
    stream.on('message', (message) => {
      console.log(`New message for user-1: ${message.subject}`);
    });

  } finally {
    await wsClient.disconnect();
  }
}

webSocketExample();
```

### Example 5: Promise vs Async/Await

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

const client = new FastDataBrokerSDK('localhost', 6000);
const message = {
  senderId: 'app',
  recipientIds: ['user-1'],
  subject: 'Test',
  content: Buffer.from('Test content'),
  priority: Priority.HIGH,
};

// Promise-based approach
client.connect()
  .then(() => client.sendMessage(message))
  .then(result => console.log(`Sent: ${result.messageId}`))
  .catch(error => console.error(`Error: ${error.message}`))
  .finally(() => client.disconnect());

// Async/Await approach (recommended)
async function asyncAwaitExample() {
  try {
    await client.connect();
    const result = await client.sendMessage(message);
    console.log(`Sent: ${result.messageId}`);
  } catch (error) {
    console.error(`Error: ${error.message}`);
  } finally {
    await client.disconnect();
  }
}

asyncAwaitExample();
```

### Example 6: Streaming & Events

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function streamingExample() {
  const client = new FastDataBrokerSDK('localhost', 6000);
  
  try {
    await client.connect();

    // Subscribe to messages
    const stream = client.subscribe('user-1');

    stream.on('message', (message) => {
      console.log(`Received: ${message.subject}`);
      console.log(`  From: ${message.senderId}`);
      console.log(`  Content: ${Buffer.from(message.content).toString()}`);
    });

    stream.on('error', (error) => {
      console.error(`Stream error: ${error.message}`);
    });

    // Send a test message
    const message = {
      senderId: 'test-app',
      recipientIds: ['user-1'],
      subject: 'Test Stream Message',
      content: Buffer.from('This is a stream test'),
      priority: Priority.NORMAL,
    };

    const result = await client.sendMessage(message);
    console.log(`Message sent: ${result.messageId}`);

    // Unsubscribe after some time
    setTimeout(() => {
      stream.unsubscribe();
      console.log('Unsubscribed from messages');
    }, 5000);
  } finally {
    await client.disconnect();
  }
}

streamingExample();
```

### Example 7: Error Handling

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function errorHandlingExample() {
  const client = new FastDataBrokerSDK('localhost', 6000);

  try {
    await client.connect();

    const message = {
      senderId: 'app',
      recipientIds: ['user-1'],
      subject: 'Test',
      content: Buffer.from('Test'),
      priority: Priority.HIGH,
    };

    try {
      const result = await client.sendMessage(message);
      console.log(`Message sent: ${result.messageId}`);
    } catch (error) {
      if (error.code === 'VALIDATION_ERROR') {
        console.error(`Validation error: ${error.message}`);
      } else if (error.code === 'CONNECTION_ERROR') {
        console.error(`Connection error: ${error.message}`);
      } else if (error.code === 'TIMEOUT_ERROR') {
        console.error('Request timeout');
      } else {
        console.error(`Unexpected error: ${error.message}`);
      }
    }
  } finally {
    await client.disconnect();
  }
}

errorHandlingExample();
```

### Example 8: Complete End-to-End Application

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

async function completeExample() {
  console.log('=== FastDataBroker JavaScript SDK Complete Example ===\n');

  const client = new FastDataBrokerSDK('localhost', 6000);

  try {
    // 1. Connection
    console.log('1. Connecting to FastDataBroker...');
    await client.connect();
    console.log('✓ Connected successfully\n');

    // 2. Send critical message
    console.log('2. Sending critical priority message...');
    const start = Date.now();
    const criticalMsg = {
      senderId: 'app',
      recipientIds: ['user-123'],
      subject: 'Critical Alert',
      content: Buffer.from('This is critical'),
      priority: Priority.CRITICAL,
      ttlSeconds: 3600,
      tags: {
        'severity': 'critical',
        'timestamp': new Date().toISOString(),
      },
    };

    const result1 = await client.sendMessage(criticalMsg);
    const duration = Date.now() - start;
    console.log(`✓ Message sent: ${result1.messageId} (took ${duration}ms)`);
    console.log(`  Status: ${result1.status}`);
    console.log();

    // 3. Send batch messages
    console.log('3. Sending batch messages...');
    for (let i = 0; i < 3; i++) {
      const msg = {
        senderId: 'batch-app',
        recipientIds: [`user-${i}`],
        subject: `Batch message ${i + 1}`,
        content: Buffer.from(`Content ${i + 1}`),
        priority: Priority.NORMAL,
        ttlSeconds: 7200,
      };
      const res = await client.sendMessage(msg);
      console.log(`  ✓ Message ${i + 1}: ${res.messageId}`);
    }
    console.log();

    // 4. Send async messages
    console.log('4. Sending async messages...');
    const asyncPromises = [];
    for (let i = 0; i < 2; i++) {
      const msg = {
        senderId: 'async-app',
        recipientIds: [`user-async-${i}`],
        subject: `Async message ${i + 1}`,
        content: Buffer.from(`Async content ${i + 1}`),
        priority: Priority.HIGH,
      };
      asyncPromises.push(
        client.sendMessage(msg)
          .then(res => console.log(`  ✓ Async message sent: ${res.messageId}`))
          .catch(err => console.error(`  ✗ Error: ${err.message}`))
      );
    }
    await Promise.all(asyncPromises);
    console.log();

    // 5. Statistics
    console.log('5. Final Statistics:');
    console.log('  ✓ All messages sent successfully');
    console.log('  ✓ Client connected');

    // 6. Cleanup
    console.log('\n6. Cleaning up...');
    await client.disconnect();
    console.log('✓ Disconnected\n');

    console.log('=== Example completed successfully ===');
  } catch (error) {
    console.error(`✗ Error occurred: ${error.message}`);
    console.error(error.stack);
  }
}

completeExample();
```

## API Reference

### Priority Enum

```typescript
export enum Priority {
  DEFERRED = 50,       // Low priority, can be delayed
  NORMAL = 100,        // Standard priority (default)
  HIGH = 150,          // Higher priority
  URGENT = 200,        // Very high priority
  CRITICAL = 255,      // Critical, process immediately
}
```

### NotificationChannel Enum

```typescript
export enum NotificationChannel {
  EMAIL = "email",           // Email delivery
  WEBSOCKET = "websocket",   // WebSocket push
  PUSH = "push",             // Push notifications
  WEBHOOK = "webhook",       // Webhook callback
}
```

### Message Interface

```typescript
export interface IMessage {
  senderId: string;           // Sender identifier
  recipientIds: string[];     // Recipient identifiers
  subject: string;            // Message subject
  content: Buffer | string;   // Message body (will be converted to Buffer)
  priority?: Priority;        // Message priority (default: NORMAL)
  ttlSeconds?: number;        // Time to live in seconds
  tags?: Record<string, string>; // Custom metadata tags
  requireConfirmation?: boolean; // Request delivery confirmation
  timestamp?: number;         // Creation timestamp
}
```

### DeliveryResult Interface

```typescript
export interface ISendResult {
  messageId: string;          // Unique message identifier
  status: 'success' | 'partial' | 'failed'; // Delivery status
  deliveredChannels: number;  // Number of channels delivered
  details: Record<string, any>; // Additional details
  timestamp?: number;         // Delivery timestamp
}
```

### Client Methods

```typescript
export class FastDataBrokerSDK {
  // Constructor
  constructor(host?: string, port?: number, options?: IClientOptions);

  // Connection methods
  connect(): Promise<boolean>;
  disconnect(): Promise<void>;
  isConnected(): boolean;

  // Message sending
  sendMessage(message: IMessage): Promise<ISendResult>;
  sendBatch(messages: IMessage[]): Promise<ISendResult[]>;

  // Messaging
  subscribe(userId: string): EventEmitter;
  on(event: string, listener: (...args: any[]) => void): this;

  // Event methods
  on(event: 'connected' | 'disconnected' | 'error'): void;
}
```

### Client Configuration

```typescript
export interface IClientOptions {
  timeout?: number;           // Timeout in milliseconds (default: 30000)
  retries?: number;           // Number of retries (default: 3)
  compression?: boolean;      // Enable compression (default: true)
  encryption?: boolean;       // Enable encryption (default: false)
  connectionPoolSize?: number; // Connection pool size (default: 10)
  debug?: boolean;            // Enable debug logging (default: false)
}
```

## Error Handling

### Try-Catch Pattern

```javascript
try {
  const result = await client.sendMessage(message);
  console.log('Success:', result.messageId);
} catch (error) {
  console.error('Error:', error.message);
  console.error('Code:', error.code);
}
```

### Connection Validation

```javascript
if (!client.isConnected()) {
  await client.connect();
}
```

### Event Error Handling

```javascript
client.on('error', (error) => {
  console.error('Client error:', error.message);
});

stream.on('error', (error) => {
  console.error('Stream error:', error.message);
});
```

## Advanced Features

### Configuration Options

```javascript
const client = new FastDataBrokerSDK('localhost', 6000, {
  timeout: 30000,          // 30 seconds
  retries: 3,              // 3 retries on failure
  compression: true,       // Enable compression
  encryption: true,        // Enable encryption
  connectionPoolSize: 10,  // Size of connection pool
  debug: true,             // Enable debug logging
});
```

### Event Listeners

```javascript
client.on('connected', () => console.log('Connected'));
client.on('disconnected', () => console.log('Disconnected'));
client.on('error', (error) => console.error('Error:', error));
client.on('message:sent', (result) => console.log('Sent:', result));
client.on('message:failed', (error) => console.error('Failed:', error));
```

### Batch Processing with Concurrency

```javascript
const messages = [/* ... */];
const batchSize = 10;

for (let i = 0; i < messages.length; i += batchSize) {
  const batch = messages.slice(i, i + batchSize);
  const results = await Promise.all(
    batch.map(msg => client.sendMessage(msg))
  );
  results.forEach(r => console.log(`Sent: ${r.messageId}`));
}
```

### Retry Logic

```javascript
async function sendWithRetry(message, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await client.sendMessage(message);
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
    }
  }
}
```

## Testing

### Run Tests (Node.js)

```bash
cd sdks/javascript
npm test
npm test -- --coverage  # With coverage
```

### Unit Test Example (Jest)

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

describe('FastDataBrokerSDK', () => {
  let client;

  beforeEach(() => {
    client = new FastDataBrokerSDK('localhost', 6000);
  });

  afterEach(async () => {
    if (client.isConnected()) {
      await client.disconnect();
    }
  });

  test('should send message successfully', async () => {
    await client.connect();

    const message = {
      senderId: 'test',
      recipientIds: ['test-recipient'],
      subject: 'Test',
      content: Buffer.from('Test content'),
      priority: Priority.HIGH,
    };

    const result = await client.sendMessage(message);
    expect(result.messageId).toBeDefined();
    expect(result.status).toBe('success');
  });

  test('should connect and disconnect', async () => {
    await client.connect();
    expect(client.isConnected()).toBe(true);

    await client.disconnect();
    expect(client.isConnected()).toBe(false);
  });
});
```

## Requirements

- Node.js 14.0 or higher
- Browser with ES2017 support

## Building and Publishing

### Build

```bash
npm run build
npm run build:prod  # Production build
```

### Run Tests

```bash
npm test
npm test -- --coverage
```

## Publishing to NPM

```bash
npm version patch|minor|major
npm publish
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

### Version 0.1.14 (Latest)
- Enhanced documentation with 8 complete examples
- WebSocket streaming integration
- Event emitter support
- Comprehensive error handling
- TypeScript full support with interfaces
- Advanced configuration options
- Batch processing patterns
- Retry logic patterns

### Version 0.1.12
- Initial JavaScript/TypeScript SDK release
- Promise-based async APIs with async/await
- Priority-based message routing
- TTL and tagging support
- QUIC protocol integration
