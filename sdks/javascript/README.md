# FastDataBroker JavaScript/TypeScript SDK

JavaScript/TypeScript SDK for FastDataBroker - A high-performance distributed message queue with built-in clustering and QUIC protocol support.

## Version
0.1.11

## Features

- 🚀 **Synchronous & Asynchronous APIs** - Promise-based async/await support
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

### JavaScript (ES6+)

```javascript
import { FastDataBrokerSDK, Priority } from '@fastdatabroker/sdk';

// Create client
const client = new FastDataBrokerSDK('localhost', 6000);

// Connect to FastDataBroker
await client.connect();

// Create a message
const message = {
  senderId: 'user-1',
  recipientIds: ['user-2', 'user-3'],
  subject: 'Hello from JavaScript',
  content: Buffer.from('This is a test message'),
  priority: Priority.HIGH,
};

// Send message
const result = await client.sendMessage(message);
console.log(`Message sent: ${result.messageId}`);

// Disconnect
await client.disconnect();
```

### TypeScript

```typescript
import { FastDataBrokerSDK, Priority, IMessage, ISendResult } from '@fastdatabroker/sdk';

const client = new FastDataBrokerSDK('localhost', 6000);
await client.connect();

const message: IMessage = {
  senderId: 'user-1',
  recipientIds: ['user-2', 'user-3'],
  subject: 'Hello from TypeScript',
  content: Buffer.from('This is a test message'),
  priority: Priority.HIGH,
};

const result: ISendResult = await client.sendMessage(message);
console.log(`Message sent: ${result.messageId}`);

await client.disconnect();
```

### Priority Levels

```javascript
import { Priority } from '@fastdatabroker/sdk';

message.priority = Priority.CRITICAL;    // 255
// Other options:
// - Priority.URGENT (200)
// - Priority.HIGH (150)
// - Priority.NORMAL (100)
// - Priority.DEFERRED (50)
```

### Message TTL (Time-To-Live)

```javascript
// Message expires after 3600 seconds (1 hour)
message.ttlSeconds = 3600;
```

### Message Tags

```javascript
message.tags = {
  region: 'us-east',
  category: 'notification',
  version: 'v1',
};
```

### Error Handling

```javascript
try {
  const result = await client.sendMessage(message);
  console.log('Message sent:', result);
} catch (error) {
  if (error.code === 'VALIDATION_ERROR') {
    console.error('Invalid message:', error.message);
  } else if (error.code === 'CONNECTION_ERROR') {
    console.error('Connection failed:', error.message);
  } else if (error.code === 'TIMEOUT_ERROR') {
    console.error('Request timeout');
  } else {
    console.error('Unexpected error:', error);
  }
}
```

### Batch Operations

```javascript
const messages = [
  { senderId: 'user-1', recipientIds: ['user-2'], subject: 'Msg1' },
  { senderId: 'user-1', recipientIds: ['user-3'], subject: 'Msg2' },
  { senderId: 'user-1', recipientIds: ['user-4'], subject: 'Msg3' },
];

const results = await client.sendBatch(messages);
results.forEach(result => console.log('Sent:', result.messageId));
```

### Streaming & Events

```javascript
// Subscribe to messages
const stream = client.subscribe('user-1');

stream.on('message', (message) => {
  console.log('Received:', message.subject);
});

stream.on('error', (error) => {
  console.error('Stream error:', error);
});

// Unsubscribe
stream.unsubscribe();
```

### WebSocket Integration

```javascript
const wsClient = new FastDataBrokerSDK.WebSocketClient('ws://localhost:6001');
await wsClient.connect();

// Send via WebSocket
const result = await wsClient.sendMessage(message);
console.log('Sent via WebSocket:', result);

// Listen for messages
wsClient.on('message', (message) => {
  console.log('Received:', message.subject);
});

await wsClient.disconnect();
```

### Clustering

```javascript
const clusterClient = new FastDataBrokerSDK.ClusterClient(
  ['node1:6000', 'node2:6000', 'node3:6000']
);
await clusterClient.connect();

// Automatic failover and load balancing
const result = await clusterClient.sendMessage(message);
console.log('Sent via cluster:', result);

await clusterClient.disconnect();
```

### Promise vs Async/Await

```javascript
// Promise-based
client.sendMessage(message)
  .then(result => console.log('Sent:', result))
  .catch(error => console.error('Error:', error));

// Async/Await (recommended)
async function sendMessages() {
  try {
    const result = await client.sendMessage(message);
    console.log('Sent:', result);
  } catch (error) {
    console.error('Error:', error);
  }
}
```

## Configuration

### Client Options

```javascript
const client = new FastDataBrokerSDK('localhost', 6000, {
  timeout: 30000,          // Timeout in milliseconds
  retries: 3,              // Number of retries
  compression: true,       // Enable compression
  encryption: true,        // Enable encryption
  connectionPoolSize: 10,  // Connection pool size
  debug: true,             // Enable debug logging
});
```

### Event Listeners

```javascript
client.on('connected', () => console.log('Connected'));
client.on('disconnected', () => console.log('Disconnected'));
client.on('error', (error) => console.error('Error:', error));
client.on('message:sent', (result) => console.log('Message sent:', result));
client.on('message:failed', (error) => console.error('Send failed:', error));
```

## API Reference

### Message Interface

```typescript
interface IMessage {
  messageId?: string;           // Unique identifier
  senderId: string;             // Sender ID
  recipientIds: string[];       // List of recipients
  subject: string;              // Message subject
  content: Buffer | string;     // Message body
  priority?: Priority;          // Priority level
  ttlSeconds?: number;          // Time to live
  tags?: Record<string, string>;// Custom tags
  requireConfirmation?: boolean;// Request confirmation
  timestamp?: number;           // Creation timestamp
}
```

### Client Methods

#### connect
Connect to FastDataBroker.
```javascript
await client.connect();
```

#### disconnect
Disconnect from FastDataBroker.
```javascript
await client.disconnect();
```

#### sendMessage
Send a single message.
```javascript
const result = await client.sendMessage(message);
// Returns: { messageId, status, timestamp }
```

#### sendBatch
Send multiple messages.
```javascript
const results = await client.sendBatch(messages);
// Returns: Array of send results
```

#### subscribe
Subscribe to user messages.
```javascript
const stream = client.subscribe(userId);
stream.on('message', (msg) => {});
```

#### getMessageStatus
Get message delivery status.
```javascript
const status = await client.getMessageStatus(messageId);
```

#### waitForConfirmation
Wait for message confirmation.
```javascript
const confirmation = await client.waitForConfirmation(messageId, 5000);
```

## Examples

- [Basic Example](examples/basic.ts)
- [Async/Await](examples/async-await.ts)
- [WebSocket Example](examples/websocket.ts)
- [Batch Operations](examples/batch.ts)
- [Clustering](examples/clustering.ts)
- [Error Handling](examples/error-handling.ts)
- [React Integration](examples/react-integration.tsx)
- [Node.js Stream Example](examples/stream.js)

## Error Codes

- **1000**: Connection error
- **1001**: Validation error
- **1002**: Timeout error
- **1003**: Message not found
- **1004**: Authentication failed
- **1005**: Rate limit exceeded

## Testing

```bash
npm test                    # Run all tests
npm run test:watch         # Run tests in watch mode
npm run test:coverage      # Generate coverage report
npm run benchmark          # Run performance benchmarks
```

## Benchmarks

Performance on Node.js v18+:

| Operation | Throughput | Latency |
|-----------|-----------|---------|
| Send Message | 75k msgs/sec | <2ms |
| Batch Send (100) | 180k msgs/sec | <150ms |
| WebSocket | 35k msgs/sec | <5ms |

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Node.js Support

- Node.js 16.x
- Node.js 18.x
- Node.js 20.x

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

### Version 0.1.11
- Initial SDK release
- TypeScript support with full type definitions
- Synchronous and asynchronous message sending
- Multi-channel delivery support
- WebSocket integration
- Event emitter pattern
- Clustering and failover support
- Batch operations
- Promise/async-await support
