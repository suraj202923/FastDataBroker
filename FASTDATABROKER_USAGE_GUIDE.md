# FastDataBroker: Complete Guide for Python Producers and Consumers

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Installation & Setup](#installation--setup)
4. [Producer: Sending Messages](#producer-sending-messages)
5. [Receiver: Consuming Messages](#receiver-consuming-messages)
6. [Notification Channels](#notification-channels)
7. [Advanced Usage](#advanced-usage)
8. [Complete Example](#complete-example)

---

## Overview

**FastDataBroker** is a high-performance asynchronous message queue system built with Rust and distributed SDKs. It enables reliable message delivery with multiple notification channels (Email, WebSocket, Push, Webhook).

### Key Features
- **Async Processing**: Non-blocking message handling
- **Multi-Channel Delivery**: Email, WebSocket, Push, Webhook notifications
- **Priority-Based Routing**: 5 priority levels (Deferred, Normal, High, Urgent, Critical)
- **Reliability**: Persistent queue with durability guarantees
- **Concurrency**: Handles 1000s of concurrent messages
- **TTL Support**: Message expiration configuration
- **QUIC Transport**: Modern, fast UDP-based protocol

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      FastDataBroker                         │
│                   High-Performance Queue                    │
└──────────┬──────────────────────────┬──────────────────────┘
           │                          │
    ┌──────▼─────────┐        ┌───────▼────────┐
    │   Producers    │        │   Consumers    │
    │  (Send msgs)   │        │  (Receive msgs)│
    └────────────────┘        └────────────────┘
           │                          │
           └──────────┬───────────────┘
                      │
        ┌─────────────┴─────────────┐
        │   Notification Channels    │
        ├─────────────┬──────────────┤
        │   Email     │  WebSocket   │
        │   Push      │  Webhook     │
        └─────────────┴──────────────┘
```

---

## Installation & Setup

### 1. **Install Python SDK**

```bash
# Using pip
pip install fastdatabroker-sdk

# Or from source
cd python
python setup.py install
```

### 2. **Import FastDataBroker**

```python
from fastdatabroker_sdk import (
    FastDataBrokerClient,
    FastDataBrokerAsyncClient,
    Message,
    Priority,
    NotificationChannel,
    DeliveryResult,
)
```

### 3. **Configure Connection**

```python
# Synchronous client (blocking)
client = FastDataBrokerClient(
    quic_host="localhost",
    quic_port=6000
)

# Asynchronous client (non-blocking)
async_client = FastDataBrokerAsyncClient(
    quic_host="localhost",
    quic_port=6000
)
```

---

## Producer: Sending Messages

### What is a Producer?

A **Producer** is an application that sends messages to FastDataBroker. It creates messages with content, recipients, and delivery settings, then publishes them to the queue.

### Step 1: Connect to FastDataBroker

```python
from fastdatabroker_sdk import FastDataBrokerClient, Message, Priority

# Create client instance
client = FastDataBrokerClient(quic_host="localhost", quic_port=6000)

# Connect to broker
if client.connect():
    print("✓ Connected to FastDataBroker")
else:
    print("✗ Connection failed")
```

### Step 2: Create a Message

```python
# Basic message
message = Message(
    sender_id="app-1",
    recipient_ids=["user-123", "user-456"],
    subject="Order Confirmation",
    content=b"Your order #12345 has been confirmed",
)

# Message with optional fields
message_advanced = Message(
    sender_id="payment-service",
    recipient_ids=["user-789"],
    subject="Payment Received",
    content=b"Amount: $99.99 has been received",
    priority=Priority.HIGH,           # High priority
    ttl_seconds=86400,                # Expires in 24 hours
    tags={
        "category": "payment",
        "order_id": "12345",
        "currency": "USD"
    }
)
```

### Step 3: Send Message

```python
# Send synchronously (blocking)
try:
    result = client.send_message(message)
    
    print(f"Message ID: {result.message_id}")
    print(f"Status: {result.status}")
    print(f"Delivered via channels: {result.delivered_channels}")
    
except Exception as e:
    print(f"Error sending message: {e}")
```

### Complete Producer Example

```python
from fastdatabroker_sdk import FastDataBrokerClient, Message, Priority

def send_order_confirmation(order_id, user_email):
    """Producer: Send order confirmation message"""
    
    # Initialize client
    client = FastDataBrokerClient(
        quic_host="localhost",
        quic_port=6000
    )
    
    # Connect
    if not client.connect():
        print("Failed to connect to FastDataBroker")
        return False
    
    # Create message
    message = Message(
        sender_id="order-service",
        recipient_ids=[user_email],
        subject=f"Order #{order_id} Confirmed",
        content=f"Your order {order_id} has been placed successfully".encode(),
        priority=Priority.NORMAL,
        ttl_seconds=7200,  # 2 hours
        tags={
            "order_id": order_id,
            "type": "confirmation",
            "timestamp": str(datetime.now())
        }
    )
    
    # Send message
    try:
        result = client.send_message(message)
        print(f"✓ Order confirmation sent")
        print(f"  Message ID: {result.message_id}")
        print(f"  Status: {result.status}")
        return True
    except Exception as e:
        print(f"✗ Failed to send: {e}")
        return False
    finally:
        client.disconnect()

# Usage
if __name__ == "__main__":
    send_order_confirmation("ORD-123456", "customer@example.com")
```

---

## Receiver: Consuming Messages

### What is a Receiver?

A **Receiver** (Consumer) is an application that listens for messages from FastDataBroker and processes them. It registers with the broker via specific notification channels (WebSocket, Webhook, or Email).

### Approach 1: WebSocket Receiver (Real-time)

**Best for**: Real-time notifications, web applications, mobile apps

```python
from fastdatabroker_sdk import FastDataBrokerClient
import json
import websocket

class WebSocketReceiver:
    """Receive messages via WebSocket connection"""
    
    def __init__(self, client_id, user_id):
        self.client = FastDataBrokerClient()
        self.client_id = client_id
        self.user_id = user_id
        self.ws = None
    
    def connect(self):
        """Connect and register WebSocket"""
        if not self.client.connect():
            print("Failed to connect to FastDataBroker")
            return False
        
        # Register WebSocket client
        success = self.client.register_websocket(
            client_id=self.client_id,
            user_id=self.user_id
        )
        
        if success:
            print(f"✓ WebSocket registered for user {self.user_id}")
            return True
        return False
    
    def on_message(self, message):
        """Handle incoming message"""
        print(f"\n📨 New Message Received:")
        print(f"   From: {message.get('sender_id')}")
        print(f"   Subject: {message.get('subject')}")
        print(f"   Content: {message.get('content')}")
        print(f"   Priority: {message.get('priority')}")
        print(f"   Tags: {message.get('tags')}")
        
        # Process message
        self.process_message(message)
    
    def process_message(self, message):
        """Business logic to handle message"""
        # Example: Update database, trigger workflows, etc.
        print(f"   ✓ Processing message ID: {message.get('message_id')}")
    
    def listen(self):
        """Listen for incoming messages"""
        print("Listening for messages on WebSocket...")
        # WebSocket listener would be implemented here
        # This is a simplified example

# Usage
receiver = WebSocketReceiver(
    client_id="app-client-1",
    user_id="user-789"
)

if receiver.connect():
    receiver.listen()
```

### Approach 2: Webhook Receiver (HTTP Callback)

**Best for**: Server-to-server communication, webhook integrations

```python
from fastdatabroker_sdk import FastDataBrokerClient
from flask import Flask, request, jsonify

app = Flask(__name__)

class WebhookReceiver:
    """Receive messages via HTTP Webhook callbacks"""
    
    def __init__(self, webhook_url):
        self.client = FastDataBrokerClient()
        self.webhook_url = webhook_url
    
    def register_webhook(self):
        """Register webhook endpoint with FastDataBroker"""
        if not self.client.connect():
            return False
        
        success = self.client.register_webhook(
            webhook_url=self.webhook_url,
            headers={
                "Authorization": "Bearer SECRET_TOKEN",
                "Content-Type": "application/json"
            }
        )
        
        print(f"✓ Webhook registered: {self.webhook_url}")
        return success

# Initialize receiver
webhook_receiver = WebhookReceiver(
    webhook_url="https://myapp.com/webhook/fastdatabroker"
)

# Flask endpoint to receive webhook calls
@app.route('/webhook/fastdatabroker', methods=['POST'])
def receive_webhook():
    """Handle incoming webhook from FastDataBroker"""
    
    try:
        # Parse request
        data = request.get_json()
        
        message = data.get('message', {})
        
        print(f"\n📨 Webhook Message Received:")
        print(f"   Message ID: {message.get('message_id')}")
        print(f"   From: {message.get('sender_id')}")
        print(f"   Subject: {message.get('subject')}")
        print(f"   Content: {message.get('content')}")
        
        # Process the message
        process_webhook_message(message)
        
        # Return acknowledgment
        return jsonify({
            "status": "received",
            "message_id": message.get('message_id')
        }), 200
    
    except Exception as e:
        print(f"Error processing webhook: {e}")
        return jsonify({"error": str(e)}), 400

def process_webhook_message(message):
    """Business logic to handle webhook message"""
    print(f"   ✓ Processing message...")
    # Update database, trigger workflows, send notifications, etc.

if __name__ == "__main__":
    # Register webhook
    webhook_receiver.register_webhook()
    
    # Start Flask server
    app.run(host='0.0.0.0', port=5000)
```

### Approach 3: Email Receiver

**Best for**: Notifications, alerts, non-real-time messages

```python
import imaplib
import email
from email.parser import Parser

class EmailReceiver:
    """Receive messages via Email"""
    
    def __init__(self, email_address, password, imap_server):
        self.email = email_address
        self.password = password
        self.imap_server = imap_server
        self.imap = None
    
    def connect(self):
        """Connect to email server"""
        try:
            self.imap = imaplib.IMAP4_SSL(self.imap_server)
            self.imap.login(self.email, self.password)
            print(f"✓ Connected to {self.imap_server}")
            return True
        except Exception as e:
            print(f"✗ Email connection failed: {e}")
            return False
    
    def fetch_messages(self):
        """Fetch FastDataBroker email notifications"""
        try:
            # Select inbox
            self.imap.select('INBOX')
            
            # Search for FastDataBroker emails
            status, messages = self.imap.search(
                None,
                'FROM', 'noreply@fastdatabroker.local'
            )
            
            if status != 'OK':
                print("No messages found")
                return []
            
            email_ids = messages[0].split()
            print(f"Found {len(email_ids)} FastDataBroker messages")
            
            results = []
            for email_id in email_ids:
                status, msg_data = self.imap.fetch(email_id, '(RFC822)')
                
                if status == 'OK':
                    msg = Parser().parsestr(msg_data[0][1].decode())
                    results.append({
                        'subject': msg['Subject'],
                        'from': msg['From'],
                        'body': msg.get_payload(),
                        'timestamp': msg['Date']
                    })
                    
                    print(f"\n📧 Email Message Received:")
                    print(f"   From: {msg['From']}")
                    print(f"   Subject: {msg['Subject']}")
                    print(f"   Body: {msg.get_payload()[:100]}...")
            
            return results
        
        except Exception as e:
            print(f"Error fetching messages: {e}")
            return []
    
    def listen(self, interval=60):
        """Listen for new email messages"""
        import time
        
        print(f"Listening for emails every {interval} seconds...")
        
        while True:
            messages = self.fetch_messages()
            
            for msg in messages:
                self.process_email_message(msg)
            
            time.sleep(interval)
    
    def process_email_message(self, message):
        """Business logic for email messages"""
        print(f"   ✓ Processing email notification...")
        # Update database, trigger actions, etc.

# Usage
email_receiver = EmailReceiver(
    email_address="notifications@company.com",
    password="email_password",
    imap_server="imap.gmail.com"
)

if email_receiver.connect():
    email_receiver.listen(interval=30)
```

### Approach 4: Push Notification Receiver

**Best for**: Mobile apps, phone notifications

```python
from fastdatabroker_sdk import FastDataBrokerClient

class PushNotificationReceiver:
    """Receive messages via Push Notifications"""
    
    def __init__(self, device_token, platform):
        self.client = FastDataBrokerClient()
        self.device_token = device_token
        self.platform = platform  # firebase, apns, fcm, webpush
    
    def register_push_device(self):
        """Register device for push notifications"""
        if not self.client.connect():
            return False
        
        # Register push device
        print(f"✓ Push device registered")
        print(f"  Device Token: {self.device_token}")
        print(f"  Platform: {self.platform.upper()}")
        
        return True
    
    def on_push_received(self, notification):
        """Handle incoming push notification"""
        print(f"\n🔔 Push Notification Received:")
        print(f"   Title: {notification.get('title')}")
        print(f"   Body: {notification.get('body')}")
        print(f"   Data: {notification.get('data')}")
        
        # Process notification
        self.process_notification(notification)
    
    def process_notification(self, notification):
        """Handle notification action"""
        print(f"   ✓ Processing notification...")
        # Update UI, trigger background tasks, etc.

# Usage (Mobile App)
push_receiver = PushNotificationReceiver(
    device_token="device_token_abc123xyz",
    platform="firebase"
)

push_receiver.register_push_device()
```

---

## Notification Channels

### Channel Comparison

| Channel | Type | Real-time | Use Case | Implementation |
|---------|------|-----------|----------|-----------------|
| **WebSocket** | TCP | ✅ Yes | Real-time web apps | Long-lived connection |
| **Webhook** | HTTP | ✅ Yes | Server integrations | HTTP POST callback |
| **Email** | SMTP | ❌ No | Notifications, alerts | Polling IMAP |
| **Push** | Mobile | ✅ Yes | Mobile apps | Platform SDKs |

### Priority Levels

```python
from fastdatabroker_sdk import Priority

# Priority levels (lower value = lower priority)
Priority.DEFERRED = 50      # Scheduled/low-priority tasks
Priority.NORMAL = 100       # Regular messages
Priority.HIGH = 150         # Important messages
Priority.URGENT = 200       # Time-sensitive messages
Priority.CRITICAL = 255     # System alerts, emergencies
```

### Example: Using All Features

```python
message = Message(
    sender_id="system",
    recipient_ids=["admin@company.com"],
    subject="Critical Database Alert",
    content=b"Database CPU usage at 95%",
    priority=Priority.CRITICAL,
    ttl_seconds=3600,
    tags={
        "severity": "critical",
        "service": "database",
        "alert_id": "ALERT-12345"
    }
)
```

---

## Advanced Usage

### 1. Batch Message Sending

```python
async def send_batch_messages():
    """Send multiple messages concurrently"""
    
    client = FastDataBrokerAsyncClient()
    await client.connect()
    
    messages = [
        Message(
            sender_id="batch-service",
            recipient_ids=[f"user-{i}"],
            subject=f"Batch message {i}",
            content=f"Message content {i}".encode(),
        )
        for i in range(100)
    ]
    
    # Send all concurrently
    results = await client.batch_send(messages)
    
    print(f"Sent {len(results)} messages")
    
    await client.disconnect()

# Run async code
import asyncio
asyncio.run(send_batch_messages())
```

### 2. Message with Confirmation Required

```python
message = Message(
    sender_id="critical-service",
    recipient_ids=["supervisor@company.com"],
    subject="Action Required",
    content=b"Please review and approve this request",
    priority=Priority.URGENT,
    require_confirmation=True  # Require acknowledgment
)
```

### 3. Message with Large Content

```python
# Send large message (up to 100MB)
large_content = open('document.pdf', 'rb').read()

message = Message(
    sender_id="document-service",
    recipient_ids=["user@company.com"],
    subject="PDF Document",
    content=large_content,
    ttl_seconds=86400 * 7  # Keep for 7 days
)

client.send_message(message)
```

### 4. Message with Multiple Recipients

```python
# Send to many users
recipients = [
    f"user{i}@company.com" 
    for i in range(1, 101)
]

message = Message(
    sender_id="broadcast",
    recipient_ids=recipients,
    subject="Company-wide Announcement",
    content=b"Please read this important notice",
    priority=Priority.HIGH
)

result = client.send_message(message)
print(f"Sent to {len(recipients)} recipients")
```

### 5. Message with Custom Tags

```python
message = Message(
    sender_id="order-service",
    recipient_ids=["customer@example.com"],
    subject="Order Shipped",
    content=b"Your order has been shipped",
    tags={
        "order_id": "ORD-123456",
        "tracking_id": "TRK-789",
        "carrier": "FedEx",
        "estimated_delivery": "2026-04-10",
        "customer_id": "CUST-999",
        "order_total": "$99.99",
        "environment": "production"
    }
)
```

---

## Complete Example

### Full Producer-Consumer Application

```python
"""
FastDataBroker Complete Example
- Producer: Sends order confirmation messages
- Consumer: Receives and processes notifications via webhook
"""

from fastdatabroker_sdk import (
    FastDataBrokerClient,
    Message,
    Priority,
)
from flask import Flask, request, jsonify
from datetime import datetime
import threading

# ============ PRODUCER ============

class OrderProducer:
    """Send order notifications"""
    
    def __init__(self):
        self.client = FastDataBrokerClient(
            quic_host="localhost",
            quic_port=6000
        )
    
    def send_order_confirmation(self, order_id, customer_email, items, total):
        """Send order confirmation"""
        
        if not self.client.connect():
            print("Failed to connect to broker")
            return False
        
        content = f"""
Order Confirmation:
Order ID: {order_id}
Customer: {customer_email}
Items: {items}
Total: ${total}
Status: Confirmed
Date: {datetime.now()}
        """.encode()
        
        message = Message(
            sender_id="order-service",
            recipient_ids=[customer_email],
            subject=f"Order {order_id} Confirmed",
            content=content,
            priority=Priority.NORMAL,
            ttl_seconds=7200,
            tags={
                "order_id": order_id,
                "type": "confirmation",
                "amount": str(total)
            }
        )
        
        try:
            result = self.client.send_message(message)
            print(f"✓ Order confirmation sent: {result.message_id}")
            return True
        except Exception as e:
            print(f"✗ Error: {e}")
            return False
        finally:
            self.client.disconnect()

# ============ CONSUMER ============

app = Flask(__name__)

class OrderConsumer:
    """Receive and process notifications"""
    
    def __init__(self):
        self.orders = {}
    
    def register_webhook(self):
        """Register webhook with FastDataBroker"""
        client = FastDataBrokerClient()
        
        if client.connect():
            client.register_webhook(
                webhook_url="https://localhost:5000/webhook/orders",
                headers={"Authorization": "Bearer token123"}
            )
            print("✓ Webhook registered")
    
    def process_notification(self, data):
        """Process incoming notification"""
        
        message = data.get('message', {})
        order_id = message.get('tags', {}).get('order_id')
        
        # Store order
        self.orders[order_id] = {
            'status': 'received',
            'timestamp': datetime.now(),
            'sender': message.get('sender_id'),
            'content': message.get('content')
        }
        
        print(f"✓ Order {order_id} received and stored")
        
        # Trigger downstream processes
        self.handle_confirmed_order(order_id)
    
    def handle_confirmed_order(self, order_id):
        """Process confirmed order"""
        print(f"  → Updating inventory for order {order_id}")
        print(f"  → Generating packing slip")
        print(f"  → Scheduling shipment")

# Initialize consumer
consumer = OrderConsumer()

@app.route('/webhook/orders', methods=['POST'])
def webhook_handler():
    """Webhook endpoint for FastDataBroker"""
    
    try:
        data = request.get_json()
        consumer.process_notification(data)
        
        return jsonify({
            "status": "processed",
            "timestamp": datetime.now().isoformat()
        }), 200
    
    except Exception as e:
        print(f"✗ Error: {e}")
        return jsonify({"error": str(e)}), 400

@app.route('/orders', methods=['GET'])
def get_orders():
    """Get all processed orders"""
    return jsonify(consumer.orders), 200

# ============ MAIN ============

if __name__ == "__main__":
    # Start consumer in background
    def run_server():
        consumer.register_webhook()
        app.run(host='0.0.0.0', port=5000, debug=False)
    
    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()
    
    # Simulate producer sending orders
    producer = OrderProducer()
    
    print("\n=== Sending Test Orders ===\n")
    
    orders = [
        ("ORD-001", "alice@example.com", ["Laptop", "Mouse"], 1299.99),
        ("ORD-002", "bob@example.com", ["Phone", "Case"], 899.99),
        ("ORD-003", "charlie@example.com", ["Tablet", "Stylus"], 599.99),
    ]
    
    for order_id, email, items, total in orders:
        producer.send_order_confirmation(order_id, email, items, total)
        print()
    
    # Keep server running
    import time
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\n\nShutdown...")
```

### Running the Example

```bash
# Terminal 1: Start the consumer (webhook server)
python example.py

# Terminal 2: Send messages (producer)
from example import OrderProducer

producer = OrderProducer()
producer.send_order_confirmation(
    "ORD-123",
    "customer@example.com",
    ["Item1", "Item2"],
    99.99
)
```

### Expected Output

```
=== Consumer (Webhook Server) ===
✓ Webhook registered
✓ Order ORD-001 received and stored
  → Updating inventory for order ORD-001
  → Generating packing slip
  → Scheduling shipment

=== Producer ===
✓ Order confirmation sent: msg-abc123def456
✓ Order confirmation sent: msg-xyz789abc123
✓ Order confirmation sent: msg-def456xyz789
```

---

## Summary

| Role | Function | Channel | Implementation |
|------|----------|---------|-----------------|
| **Producer** | Send messages | All | `client.send_message()` |
| **Consumer** | Receive messages | WebSocket | `register_websocket()` + listener |
| **Consumer** | Receive messages | Webhook | `register_webhook()` + HTTP endpoint |
| **Consumer** | Receive messages | Email | IMAP polling |
| **Consumer** | Receive messages | Push | Device registration + SDK |

---

## Best Practices

✅ **Do:**
- Set appropriate TTL for time-sensitive messages
- Use priority levels correctly
- Register webhooks over HTTPS
- Handle connection failures with retry logic
- Batch messages when possible for better performance
- Tag messages with relevant metadata
- Implement idempotency for message processing

❌ **Don't:**
- Send highly sensitive data in plain text
- Ignore connection errors
- Use Priority.CRITICAL for non-critical messages
- Store long-lived connections without keepalive
- Process the same message twice
- Send to invalid email addresses

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection refused | Check broker is running on correct host:port |
| Messages not received | Verify receiver is registered on correct channel |
| Webhook not triggered | Ensure webhook URL is HTTPS and accessible |
| Email not received | Check IMAP server credentials and folders |
| Messages expire | Increase TTL for messages that need longer lifetime |

---

## Next Steps

1. ✅ Install FastDataBroker SDK
2. ✅ Start the broker server
3. ✅ Implement your producer
4. ✅ Implement your consumer
5. ✅ Test message flow
6. ✅ Monitor delivery via dashboard
7. ✅ Configure monitoring and alerts
