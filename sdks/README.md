# FastDataBroker SDKs - QUIC with Pre-Shared Key Authentication

## Overview

Welcome to the FastDataBroker SDK suite! This directory contains production-ready client libraries for building high-performance, real-time messaging applications with **QUIC 1.0** protocol and **TLS 1.3 PSK** authentication.

### 🚀 Quick Links

- **Quick Start**: See [SDK_QUICK_REFERENCE.md](../SDK_QUICK_REFERENCE.md) for 5-minute setup
- **Full Documentation**: [SDK_DOCUMENTATION_QUIC_PSK.md](../SDK_DOCUMENTATION_QUIC_PSK.md)
- **Testing Guide**: [SDK_TESTING_GUIDE.md](../SDK_TESTING_GUIDE.md)
- **Delivery Summary**: [SDK_DELIVERY_SUMMARY.md](../SDK_DELIVERY_SUMMARY.md)

---

## What's Included

### Supported Languages

| Language | Status | Location | Features |
|----------|--------|----------|----------|
| JavaScript/TypeScript | ✅ Stable | `javascript/` | Async/await, ESM, CommonJS |
| Python | ✅ Stable | `python/` | Threading, Logging, Dataclasses |
| Go | ✅ Stable | `go/` | Goroutines, Channels, Concurrency |
| Java | ✅ Stable | `java/` | OOP, GSON, Executor Service |
| C# | ✅ Stable | `csharp/` | Async/await, IDisposable, LINQ |

### Features Available in All SDKs

- ✅ **QUIC 1.0** (RFC 9000) Protocol Support
- ✅ **PSK Authentication** (Pre-Shared Key)
- ✅ **Message Priorities** (4 levels)
- ✅ **TTL Support** (Time-To-Live)
- ✅ **Topic Subscriptions** (Handler-based)
- ✅ **Connection Statistics** (Real-time metrics)
- ✅ **Error Handling** (Comprehensive)
- ✅ **Thread-Safe** (All operations safe)

---

## Installation

### JavaScript/TypeScript
```bash
npm install fastdatabroker-sdk-quic
# or from source
cd sdks/javascript
npm install
```

### Python
```bash
pip install fastdatabroker-sdk-quic
# or from source
cd sdks/python
pip install -e .
```

### Go
```bash
go get github.com/fastdatabroker/sdk-go
# or from source
cd sdks/go
go mod install
```

### Java
```gradle
// In build.gradle
dependencies {
    implementation 'com.fastdatabroker:sdk-quic:1.0.0'
}
```

### C#
```bash
dotnet add package FastDataBroker.SDK.Quic
# or from source
cd sdks/csharp
dotnet build
```

---

## Quick Example

### Get Your PSK Secret (First Step!)

```bash
# Request PSK secret from server
curl -X POST http://your-server:9000/api/quic/psks \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "your-tenant", "client_id": "your-client"}'

# Result: {"psk_secret": "psk_xxxxx", "expires_at": "..."}

# Export for use
export QUIC_PSK_SECRET="psk_xxxxx"
```

### Send Your First Message (JavaScript)

```typescript
import { createQuicClient, Priority } from 'fastdatabroker-sdk-quic';

const client = createQuicClient({
  host: 'localhost',
  port: 6000,
  tenantId: 'your-tenant',
  clientId: 'your-client',
  pskSecret: process.env.QUIC_PSK_SECRET!,
});

await client.connect();

const result = await client.sendMessage({
  topic: 'orders.created',
  payload: { orderId: '123', amount: 99.99 },
  priority: Priority.HIGH,
});

console.log(`✓ Message sent: ${result.messageId}`);
await client.disconnect();
```

**See [SDK_QUICK_REFERENCE.md](../SDK_QUICK_REFERENCE.md) for examples in all 5 languages!**

---

## Project Structure

```
sdks/
├── javascript/
│   ├── fastdatabroker_quic_psk.ts     # Main SDK
│   ├── package.json                    # Dependencies
│   ├── tsconfig.json                   # TypeScript config
│   └── tests/                          # Test suite
├── python/
│   ├── fastdatabroker_quic_psk.py     # Main SDK
│   ├── setup.py                        # Package config
│   ├── requirements.txt                # Dependencies
│   └── tests/                          # Test suite
├── go/
│   ├── fastdatabroker_quic_psk.go     # Main SDK
│   ├── go.mod                          # Module definition
│   ├── go.sum                          # Dependencies
│   └── *_test.go                       # Tests
├── java/
│   ├── FastDataBrokerQuicClient.java  # Main SDK
│   ├── build.gradle                    # Build config
│   ├── src/test/                       # Test suite
│   └── pom.xml                         # Maven config (optional)
└── csharp/
    ├── FastDataBrokerQuicClient.cs    # Main SDK
    ├── FastDataBrokerQuicClient.csproj # Project file
    ├── Tests/                          # Test suite
    └── packages.config                 # Dependencies
```

---

## Key Concepts

### Authentication (PSK)

All SDKs use **Pre-Shared Key (PSK)** authentication via TLS 1.3:
- Secure credential exchange
- SHA-256 hashing of secrets
- Identity-based client identification
- No certificate management required

### Message Priority

Messages can have 4 priority levels:
```
Priority.CRITICAL  (20)  - System alerts
Priority.HIGH      (10)  - Business-critical events
Priority.NORMAL    (5)   - Regular operations (default)
Priority.LOW       (1)   - Background tasks
```

### Connection Management

- Establish connection with `connect()`
- Check status with `isConnected()`
- Get statistics with `getStats()`
- Close with `disconnect()`

### Topic-Based Messaging

Subscribe to topics and receive messages:
```typescript
client.onMessage('orders.updated', (message) => {
  // Handle message
});

// Later: unsubscribe
client.offMessage('orders.updated');
```

---

## Performance Characteristics

### Benchmarks

| Metric | Target | Notes |
|--------|--------|-------|
| Connection Latency | < 100ms | QUIC handshake |
| Message Latency (p50) | < 10ms | Typical message |
| Message Latency (p99) | < 50ms | Worst case |
| Throughput | > 10,000 msg/s | Sustained rate |
| Memory | < 10MB/conn | Per connection |

### Optimization Tips

1. **Reuse connections** - One client per application
2. **Batch messages** - Send multiple in single batch
3. **Use correct priority** - Only HIGH/CRITICAL when needed
4. **Monitor stats** - Track latency and throughput
5. **Handle errors gracefully** - Implement retry logic

---

## API Consistency

All SDKs implement the same interface:

```
// Connection
connect()           - Establish connection
disconnect()        - Close connection
isConnected()       - Check if connected
getStats()          - Get statistics

// Messaging
sendMessage(msg)    - Send message (returns DeliveryResult)
onMessage(topic)    - Register handler
offMessage(topic)   - Unregister handler

// Configuration
host, port          - Server address
tenantId, clientId  - Client identification
pskSecret           - Pre-Shared Key
```

---

## Testing

### Run Tests

**JavaScript**
```bash
cd sdks/javascript
npm test
```

**Python**
```bash
cd sdks/python
pytest tests/
```

**Go**
```bash
cd sdks/go
go test -v
```

**Java**
```bash
cd sdks/java
gradle test
```

**C#**
```bash
cd sdks/csharp
dotnet test
```

### Coverage Target

All SDKs maintain **> 80% code coverage**. See [SDK_TESTING_GUIDE.md](../SDK_TESTING_GUIDE.md) for comprehensive testing strategies.

---

## Real-World Examples

### Example 1: Event Publishing Service

```typescript
// Publish order events
async function publishOrderEvent(orderId: string, status: string) {
  const client = createQuicClient(config);
  await client.connect();

  try {
    const result = await client.sendMessage({
      topic: `orders.${status}`,
      payload: { orderId, status, timestamp: new Date().toISOString() },
      priority: Priority.HIGH,
      headers: { 'source': 'order-service' }
    });
    
    console.log(`Event published: ${result.messageId}`);
  } finally {
    await client.disconnect();
  }
}
```

### Example 2: Event Processing Service

```python
# Listen to and process events
def process_events():
  client = create_quic_client(config)
  client.connect()

  def handle_order_created(msg):
    print(f"Processing new order: {msg['payload']['orderId']}")
    # Process order...

  def handle_payment_processed(msg):
    print(f"Payment confirmed: {msg['payload']['amount']}")
    # Update database...

  client.on_message('orders.created', handle_order_created)
  client.on_message('payments.processed', handle_payment_processed)

  # Keep listening
  try:
    while True:
      time.sleep(1)
  finally:
    client.disconnect()
```

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| `QUIC_PSK_SECRET not set` | `export QUIC_PSK_SECRET="your-secret"` |
| Connection refused | Verify server is running (`docker-compose up`) |
| Authentication failed | Check PSK secret is correct |
| Timeout errors | Increase timeout in configuration |
| High latency | Check network connection and server load |

### Debug Tips

1. **Enable logging** - Set log level to DEBUG
2. **Check server health** - `curl http://localhost:9000/health`
3. **Inspect stats** - Call `getStats()` to see metrics
4. **Use network tools** - `tcpdump`, `wireshark` for packet analysis
5. **Check logs** - Review server and client logs for errors

---

## Contributing

We welcome contributions! Please:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit changes** (`git commit -m 'Add amazing feature'`)
4. **Push to branch** (`git push origin feature/amazing-feature`)
5. **Open Pull Request**

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

---

## Documentation

- **[Quick Reference](../SDK_QUICK_REFERENCE.md)** - Get started in 5 minutes
- **[Full Documentation](../SDK_DOCUMENTATION_QUIC_PSK.md)** - Comprehensive API reference
- **[Testing Guide](../SDK_TESTING_GUIDE.md)** - Testing strategies and examples
- **[Delivery Summary](../SDK_DELIVERY_SUMMARY.md)** - Project overview and metrics

---

## Support

- **Issues**: [GitHub Issues](https://github.com/fastdatabroker/sdks/issues)
- **Discussions**: [GitHub Discussions](https://github.com/fastdatabroker/sdks/discussions)
- **Email**: support@fastdatabroker.com
- **Slack**: [Join our community](https://fastdatabroker.slack.com)

---

## Version Information

| SDK | Version | Release Date | Status |
|-----|---------|--------------|--------|
| JavaScript/TypeScript | 1.0.0 | Jan 2024 | ✅ Stable |
| Python | 1.0.0 | Jan 2024 | ✅ Stable |
| Go | 1.0.0 | Jan 2024 | ✅ Stable |
| Java | 1.0.0 | Jan 2024 | ✅ Stable |
| C# | 1.0.0 | Jan 2024 | ✅ Stable |

---

## License

All FastDataBroker SDKs are licensed under **Apache License 2.0**.

See [LICENSE](../LICENSE) for details.

---

## Next Steps

1. **Install** an SDK for your language
2. **Get PSK secret** from your FastDataBroker server
3. **Read** [SDK_QUICK_REFERENCE.md](../SDK_QUICK_REFERENCE.md)
4. **Run** example code
5. **Test** with your application
6. **Deploy** to production!

---

**Made with ❤️ by the FastDataBroker Team**

*Fast. Secure. Reliable. Real-time Messaging.*

**Ready to build something amazing? Let's go! 🚀**
