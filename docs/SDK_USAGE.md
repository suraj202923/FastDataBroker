# FastDataBroker SDK Usage Guide

Complete guide to using FastDataBroker SDKs in Python, Go, Java, and JavaScript.

## Overview

FastDataBroker provides four official SDKs:
- **Python**: `postoffice_sdk` (pip install postoffice-sdk)
- **Go**: `github.com/fastdatabroker/go-sdk`
- **Java**: `com.fastdatabroker:fastdatabroker-sdk`
- **JavaScript**: `npm install fastdatabroker-sdk`

## Python SDK

### Installation
```bash
pip install postoffice-sdk
```

### Basic Usage

```python
from postoffice_sdk import Producer, Consumer, ClusterClient

# Initialize client
client = ClusterClient(
    bootstrap_servers=['broker1:8080', 'broker2:8081', 'broker3:8082'],
    stream_name='orders'
)

# Send a message
producer = Producer(client)
message = {
    'order_id': '12345',
    'customer_id': 'cust_001',
    'amount': 100.00,
    'timestamp': '2024-01-15T10:30:00Z'
}
partition = producer.send(
    key='cust_001',  # Same customer → same partition
    value=message,
    headers={'source': 'web-app', 'version': '1.0'}
)
print(f"Message sent to partition {partition}")

# Consume messages
consumer = Consumer(client, group_id='order-processors')
for message in consumer.consume(timeout_ms=5000):
    print(f"Received: {message.value}")
    consumer.commit()  # Mark as processed

# Consume from specific partition
for offset, message in consumer.consume_partition(partition=0, start_offset=0):
    print(f"Offset: {offset}, Message: {message.value}")
```

### Async/Await Pattern

```python
import asyncio
from postoffice_sdk import AsyncProducer, AsyncConsumer

async def main():
    client = ClusterClient(bootstrap_servers=['broker1:8080'])
    
    # Send asynchronously
    producer = AsyncProducer(client)
    futures = []
    
    for i in range(1000):
        future = await producer.send_async(
            key=f'key-{i}',
            value={'id': i, 'data': 'test'}
        )
        futures.append(future)
    
    # Wait for all sends
    results = await asyncio.gather(*futures)
    print(f"Sent {len(results)} messages")
    
    # Consume asynchronously
    consumer = AsyncConsumer(client, group_id='async-group')
    async for message in consumer.consume_async():
        print(f"Received: {message.value}")
        await consumer.commit()

asyncio.run(main())
```

### Batch Operations

```python
from postoffice_sdk import BatchProducer

producer = BatchProducer(
    client,
    batch_size=100,          # Send every 100 messages
    batch_timeout_ms=5000    # Or every 5 seconds
)

for i in range(10000):
    producer.send(
        key=f'key-{i}',
        value={'id': i, 'data': f'message-{i}'}
    )

producer.flush()  # Send remaining messages
```

## Go SDK

### Installation
```bash
go get github.com/fastdatabroker/go-sdk
```

### Basic Usage

```go
package main

import (
    "fmt"
    "github.com/fastdatabroker/go-sdk"
)

func main() {
    // Initialize client
    config := &sdk.Config{
        BootstrapServers: []string{"broker1:8080", "broker2:8081"},
        StreamName:       "orders",
    }
    client := sdk.NewClient(config)
    defer client.Close()
    
    // Send message
    message := map[string]interface{}{
        "order_id": "12345",
        "amount": 100.0,
    }
    
    partition, offset, err := client.Send(
        "order_key",  // key
        message,      // value
    )
    if err != nil {
        fmt.Printf("Error: %v\n", err)
        return
    }
    fmt.Printf("Sent to partition %d at offset %d\n", partition, offset)
    
    // Consume messages
    consumer := sdk.NewConsumer(client, "order-processor-group")
    defer consumer.Close()
    
    for {
        message, err := consumer.NextMessage()
        if err != nil {
            break
        }
        fmt.Printf("Message: %v\n", message)
        consumer.Commit()
    }
}
```

### Goroutine Pattern

```go
package main

import (
    "fmt"
    "sync"
    "github.com/fastdatabroker/go-sdk"
)

func main() {
    client := sdk.NewClient(&sdk.Config{
        BootstrapServers: []string{"broker1:8080"},
    })
    defer client.Close()
    
    // Send from multiple goroutines
    var wg sync.WaitGroup
    for i := 0; i < 10; i++ {
        wg.Add(1)
        go func(id int) {
            defer wg.Done()
            producer := sdk.NewProducer(client)
            for j := 0; j < 1000; j++ {
                producer.Send(
                    fmt.Sprintf("key-%d", j),
                    map[string]interface{}{"id": j},
                )
            }
        }(i)
    }
    wg.Wait()
    fmt.Println("All messages sent")
}
```

## Java SDK

### Installation (Maven)
```xml
<dependency>
    <groupId>com.fastdatabroker</groupId>
    <artifactId>fastdatabroker-sdk</artifactId>
    <version>1.0.0</version>
</dependency>
```

### Basic Usage

```java
import com.fastdatabroker.sdk.*;
import java.util.*;

public class Example {
    public static void main(String[] args) {
        // Initialize client
        Properties config = new Properties();
        config.put("bootstrap.servers", "broker1:8080,broker2:8081");
        config.put("stream.name", "orders");
        
        Client client = new Client(config);
        
        // Send message
        Map<String, Object> message = new HashMap<>();
        message.put("order_id", "12345");
        message.put("amount", 100.0);
        
        Producer producer = new Producer(client);
        int partition = producer.send(
            "order_key",
            message
        );
        System.out.println("Sent to partition: " + partition);
        
        // Consume messages
        Consumer consumer = new Consumer(client, "order-group");
        consumer.consume((message) -> {
            System.out.println("Received: " + message);
            consumer.commit();
        });
        
        client.close();
    }
}
```

### Stream Processing

```java
import com.fastdatabroker.sdk.*;
import java.util.*;

public class StreamProcessor {
    public static void main(String[] args) {
        Client client = new Client(createConfig());
        
        // Process stream of events
        StreamProcessor processor = new StreamProcessor(client, "events");
        processor.subscribe((message) -> {
            if (shouldProcess(message)) {
                processMessage(message);
            }
        });
        
        processor.start();
    }
    
    private static Properties createConfig() {
        Properties config = new Properties();
        config.put("bootstrap.servers", "broker1:8080");
        config.put("stream.name", "events");
        return config;
    }
}
```

## JavaScript SDK

### Installation
```bash
npm install fastdatabroker-sdk
```

### Basic Usage

```javascript
const { Client, Producer, Consumer } = require('fastdatabroker-sdk');

async function main() {
    // Initialize client
    const client = new Client({
        bootstrapServers: ['broker1:8080', 'broker2:8081'],
        streamName: 'orders'
    });
    
    // Send message
    const producer = new Producer(client);
    const partition = await producer.send(
        'order_key',
        {
            order_id: '12345',
            amount: 100.0,
            timestamp: Date.now()
        },
        { headers: { source: 'web' } }
    );
    console.log(`Message sent to partition ${partition}`);
    
    // Consume messages
    const consumer = new Consumer(client, 'order-processor');
    consumer.on('message', async (message) => {
        console.log('Received:', message.value);
        await consumer.commit();
    });
    
    consumer.on('error', (error) => {
        console.error('Error:', error);
    });
}

main().catch(console.error);
```

### Promise/Async Pattern

```javascript
const { Client, Producer } = require('fastdatabroker-sdk');

async function sendBatch() {
    const client = new Client({
        bootstrapServers: ['broker1:8080']
    });
    
    const producer = new Producer(client);
    const messages = [];
    
    // Queue messages
    for (let i = 0; i < 10000; i++) {
        messages.push(
            producer.send(
                `key-${i}`,
                { id: i, data: `message-${i}` }
            )
        );
    }
    
    // Wait for all
    const results = await Promise.all(messages);
    console.log(`Sent ${results.length} messages`);
}

sendBatch().catch(console.error);
```

## Common Patterns

### Request-Reply Pattern

```python
from postoffice_sdk import Producer, Consumer
import uuid

def send_request(client, request_data):
    correlation_id = str(uuid.uuid4())
    
    producer = Producer(client)
    producer.send(
        key=correlation_id,
        value=request_data,
        headers={'correlation_id': correlation_id, 'reply_to': 'reply_topic'}
    )
    
    # Create consumer for replies
    consumer = Consumer(client, group_id=f"reply-{correlation_id}")
    for message in consumer.consume(timeout_ms=10000):
        if message.headers.get('correlation_id') == correlation_id:
            return message.value
    
    return None
```

### Fanout Pattern

```go
func fanout(client *sdk.Client, event map[string]interface{}) {
    topics := []string{"notifications", "analytics", "archival"}
    
    for _, topic := range topics {
        producer := sdk.NewProducer(client)
        producer.Stream = topic
        producer.Send("event", event)
    }
}
```

### Aggregation Pattern

```java
public class Aggregator {
    private Map<String, List<Message>> groups = new HashMap<>();
    
    public void aggregate(Message message) {
        String key = (String) message.getKey();
        groups.computeIfAbsent(key, k -> new ArrayList<>())
              .add(message);
        
        if (groups.get(key).size() >= 100) {
            flushGroup(key);
        }
    }
    
    private void flushGroup(String key) {
        // Send aggregated message
        List<Message> messages = groups.remove(key);
        sendAggregated(key, messages);
    }
}
```

## Error Handling

### Python
```python
try:
    producer.send(key='key', value=data)
except ConnectionError:
    print("Cannot connect to broker")
except TimeoutError:
    print("Request timed out")
except Exception as e:
    print(f"Error: {e}")
```

### Go
```go
message, err := consumer.NextMessage()
if err != nil {
    if err == sdk.ErrTimeout {
        // Timeout
    } else if err == sdk.ErrConnectionClosed {
        // Connection closed
    } else {
        log.Fatalf("Error: %v", err)
    }
}
```

### Java
```java
try {
    producer.send(key, value);
} catch (ConnectionException e) {
    // Handle connection error
} catch (TimeoutException e) {
    // Handle timeout
} catch (Exception e) {
    // Handle other errors
}
```

### JavaScript
```javascript
try {
    await producer.send(key, value);
} catch (error) {
    if (error.code === 'ECONNREFUSED') {
        console.error('Cannot connect');
    } else if (error.code === 'ETIMEDOUT') {
        console.error('Request timeout');
    } else {
        console.error('Error:', error);
    }
}
```

## Configuration Reference

### Common Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| bootstrap_servers | list | required | List of broker addresses |
| stream_name | string | required | Default stream name |
| batch_size | int | 100 | Messages per batch |
| batch_timeout_ms | int | 5000 | Max time to wait for batch |
| request_timeout_ms | int | 5000 | RPC request timeout |
| compression | string | none | snappy, gzip, or none |
| ssl_enabled | bool | false | Enable TLS |
| ssl_ca_path | string | - | Path to CA certificate |
| api_key | string | - | API key for auth |

---

**Last Updated**: Phase 7 - Complete SDK documentation
