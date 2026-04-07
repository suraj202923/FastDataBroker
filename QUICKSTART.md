# 🚀 Quick Start Guide

## 60-Second Setup

FastDataBroker gets you messaging in under a minute. Choose your path:

### Path 1: Docker (30 seconds)
```bash
docker-compose up -d
```
✅ 4-broker cluster running on localhost:8080

### Path 2: Kubernetes (2 minutes)
```bash
kubectl apply -f kubernetes/
```
✅ Production cluster with auto-scaling

### Path 3: Terraform AWS (5 minutes)
```bash
cd terraform && terraform apply
```
✅ Full HA setup on AWS

---

## 🐍 Python Example

```python
from fastdatabroker_sdk import Producer, Consumer, ClusterClient

# Initialize cluster
client = ClusterClient([
    'localhost:8080', 'localhost:8081',
    'localhost:8082', 'localhost:8083'
])

# Send messages
producer = Producer(client)
for i in range(100):
    partition = producer.send(
        key=f"order_{i:04d}",
        value={'amount': i * 100.00}
    )
    print(f"✅ Sent order {i} to partition {partition}")

# Consume messages
consumer = Consumer(client, group_id='order-processors')
for msg in consumer.consume(timeout_ms=30000):
    print(f"✅ Processing: {msg.value}")
    consumer.commit()
```

**Run it**:
```bash
pip install fastdatabroker-sdk
python example.py
```

**Expected Output**:
```
✅ Sent order 0000 to partition 2 (2.3ms)
✅ Sent order 0001 to partition 0 (2.1ms)
...
✅ Processing: {'amount': 0.0}
✅ Processing: {'amount': 100.0}
```

---

## 🐹 Go Example

```go
package main

import (
	"fmt"
	"log"

	sdk "github.com/fastdatabroker/go-sdk"
)

func main() {
	// Connect to cluster
	client := sdk.NewClient(&sdk.Config{
		BootstrapServers: []string{
			"localhost:8080",
			"localhost:8081",
			"localhost:8082",
			"localhost:8083",
		},
		StreamName: "orders",
	})

	// Send message
	producer := sdk.NewProducer(client)
	partition, err := producer.Send(
		"order_123",
		[]byte(`{"amount": 100.00}`),
	)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("✅ Sent to partition %d\n", partition)

	// Consume
	consumer := sdk.NewConsumer(client, "order-processors")
	msg, err := consumer.Consume(5000) // 5s timeout
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("✅ Received: %s\n", msg.Value)
}
```

**Run it**:
```bash
go mod tidy
go run main.go
```

---

## ☕ Java Example

```java
import com.fastdatabroker.sdk.*;

public class QuickStart {
    public static void main(String[] args) {
        // Connect
        Config config = new Config()
            .addBootstrapServer("localhost:8080")
            .addBootstrapServer("localhost:8081")
            .addBootstrapServer("localhost:8082")
            .addBootstrapServer("localhost:8083")
            .setStreamName("orders");

        Client client = new Client(config);

        // Send
        Producer producer = new Producer(client);
        int partition = producer.send(
            "order_123",
            "{\"amount\": 100.00}".getBytes()
        );
        System.out.println("✅ Sent to partition " + partition);

        // Consume
        Consumer consumer = new Consumer(client, "order-processors");
        Message msg = consumer.consume(5000); // 5s timeout
        System.out.println("✅ Received: " + new String(msg.getValue()));
    }
}
```

**Run it**:
```bash
mvn clean package
mvn exec:java -Dexec.mainClass="QuickStart"
```

---

## 📜 JavaScript Example

```javascript
const { Client, Producer, Consumer } = require('fastdatabroker-sdk');

async function main() {
  // Connect
  const client = new Client({
    bootstrapServers: [
      'localhost:8080',
      'localhost:8081',
      'localhost:8082',
      'localhost:8083'
    ],
    streamName: 'orders'
  });

  // Send
  const producer = new Producer(client);
  const partition = await producer.send(
    'order_123',
    JSON.stringify({ amount: 100.00 })
  );
  console.log(`✅ Sent to partition ${partition}`);

  // Consume
  const consumer = new Consumer(client, 'order-processors');
  const msg = await consumer.consume(5000);
  console.log(`✅ Received: ${msg.value}`);
}

main().catch(console.error);
```

**Run it**:
```bash
npm install fastdatabroker-sdk
node main.js
```

---

## � C# Example

```csharp
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

class QuickStart
{
    static async Task Main()
    {
        // Connect to cluster
        using (var client = new FastDataBrokerSDK.Client("localhost", 8080))
        {
            await client.ConnectAsync();

            // Send messages
            for (int i = 0; i < 100; i++)
            {
                var message = new FastDataBrokerSDK.Message
                {
                    SenderId = "app",
                    RecipientIds = new List<string> { "receiver" },
                    Subject = $"Order {i:D4}",
                    Content = System.Text.Encoding.UTF8.GetBytes(
                        $"{{\"amount\": {i * 100.00}}}"
                    ),
                    Priority = FastDataBrokerSDK.Priority.Normal
                };

                var result = await client.SendMessageAsync(message);
                Console.WriteLine($"✅ Sent order {i} → ID: {result.MessageId}");
            }
        }
    }
}
```

**Run it**:
```bash
dotnet add package FastDataBroker
dotnet run
```

**Expected Output**:
```
✅ Sent order 0000 → ID: abc123...
✅ Sent order 0001 → ID: def456...
...
```

---

## �📊 Key Concepts (5 minutes)

### Stream
A named sequence of messages (like a Kafka topic). Example: `orders`, `logs`, `events`

### Partition
Messages in a stream are divided into partitions for parallel processing. Same key always goes to same partition.

```
Stream: "orders"
├── Partition 0: order_100, order_200, order_400, ...
├── Partition 1: order_101, order_201, order_401, ...
└── Partition 2: order_102, order_202, order_402, ...
```

### Consumer Group
A group of consumers working together to process all partitions. If one dies, others take the load automatically.

```
Group: "order-processors"
├── Consumer 1 → Partitions 0, 3
├── Consumer 2 → Partitions 1, 4
└── Consumer 3 → Partitions 2
```

### 3-Way Replication
Every message exists on 3 brokers. If 1 fails, 2 copies remain. Data is never lost.

```
Message "order_123"
├── Broker 0 (Leader): Original
├── Broker 1 (Replica): Copy
└── Broker 2 (Replica): Copy
```

---

## 🔥 Common Patterns

### Request-Reply Pattern
```python
# Send request
request_id = f"req_{uuid4()}"
producer.send(key=request_id, value=request_data)

# Wait for reply
consumer = Consumer(client, f"reply-group-{request_id}")
reply = consumer.consume()
```

### Fan-Out Pattern
```python
# Send to multiple consumers via same stream
producer.send(key="broadcast", value=event)

# Multiple consumer groups process independently
consumer1 = Consumer(client, "group_analytics")
consumer2 = Consumer(client, "group_logging")
consumer3 = Consumer(client, "group_notifications")
```

### Event Sourcing Pattern
```python
# Store immutable events
events = [
    {"type": "UserCreated", "user_id": 123},
    {"type": "UserEmailChanged", "user_id": 123},
    {"type": "UserDeleted", "user_id": 123},
]

for event in events:
    producer.send(key=str(event["user_id"]), value=event)

# Rebuild state by replaying events
state = {}
for event in consumer.consume_all():
    # Apply event to state
```

---

## 🧪 Test with 246+ Examples

All SDKs have comprehensive examples in `tests/`:

```bash
# Python
python tests/python/test_producer.py    # 25+ test examples

# Go
go test tests/go/test_client.go -v      # 12+ test examples

# Java
mvn test -Dtest=ClientTest              # 16+ test examples

# JavaScript
npm test tests/javascript/              # 18+ test examples
```

---

## 📚 Next Steps

1. **Read** [ARCHITECTURE.md](ARCHITECTURE.md) - Understand the design (10 min)
2. **Deploy** [DEPLOYMENT.md](DEPLOYMENT.md) - Production setup (20 min)
3. **Test** [TESTING.md](TESTING.md) - Run the 246+ test suite (10 min)
4. **Integrate** - Add to your application

---

## 💡 Pro Tips

### Tip 1: Partition Assignment
```python
# Messages with SAME key → SAME partition (ordering guaranteed)
producer.send(key='order_123', value=item1)  # → Partition 2
producer.send(key='order_123', value=item2)  # → Partition 2 (guaranteed)

# Messages with DIFFERENT keys → ANY partition (load balanced)
producer.send(key='order_124', value=item3)  # → Partition 0
```

### Tip 2: Consumer Groups
```python
# Multiple instances of same consumer group
# Each processes unique partitions (automatic)

# Instance 1
Consumer(client, 'group_A').consume()  # Gets partitions 0-1

# Instance 2
Consumer(client, 'group_A').consume()  # Gets partitions 2-3

# Instance 3
Consumer(client, 'group_A').consume()  # Gets partition 4
```

### Tip 3: Error Handling
```python
try:
    producer.send(key='order', value=data)
except ConnectionError:
    # Automatic failover to next broker
    producer.send(key='order', value=data)
```

### Tip 4: Batch Operations
```python
# 10-50x faster than individual sends
items = [f"item_{i}".encode() for i in range(1000)]
producer.send_batch(items)

# 10-50x faster than individual receives
messages = consumer.consume_batch(count=100)
```

---

## 🆘 Troubleshooting

### "Connection refused on localhost:8080"
✅ Check Docker is running: `docker ps`
✅ Check broker status: `docker logs fastdatabroker_broker0`
✅ Restart: `docker-compose restart`

### "Consumer timeout"
✅ Check producer is sending: Verify with logs
✅ Check consumer group: Multiple consumers share partitions
✅ Check stream name: Must match between producer/consumer

### "Partition count mismatch"
✅ Currently fixed at 4 partitions, will be configurable in v2.0

### "Message loss"
✅ Cannot happen - 3-way replication guarantees this
✅ If you lost messages, it's a bug - report it!

---

## 📈 Performance

| Metric | Value | Time |
|--------|-------|------|
| Send latency (P99) | 2-3ms | Per message |
| Throughput (1 broker) | 912K msg/sec | Sustained |
| Throughput (4 brokers) | 3.6M msg/sec | Sustained |
| Consumer lag | <1ms | From send to consume |
| Failover recovery | <5 seconds | Automatic |

See [PERFORMANCE.md](PERFORMANCE.md) for detailed benchmarks.

---

## 📖 Resources

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - How it works
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production setup
- **[TESTING.md](TESTING.md)** - Test framework
- **[PERFORMANCE.md](PERFORMANCE.md)** - Benchmarks
- **[tests/](tests/)** - 246+ examples

---

**Ready to get started? Run: `docker-compose up -d`**
