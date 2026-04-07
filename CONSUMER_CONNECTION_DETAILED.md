# Consumer Connection to FastDataBroker - Detailed Guide

## Overview

When a consumer connects to FastDataBroker, it establishes a channel through which it will receive messages. There are 4 different ways to connect, each optimized for different scenarios.

---

## 1. WebSocket Connection (Real-Time)

### How It Works

```
CONSUMER CLIENT                          FASTDATABROKER SERVER
    │                                              │
    ├─ 1. DNS Resolution                          │
    │   (Resolve broker.company.com)               │
    │                                              │
    ├─ 2. TCP Connect ──────────────────────────→ │
    │   (Port 6001)                                │
    │   └─ Three-way handshake                     │
    │                                              │
    ├─ 3. TLS Handshake ───────────────────────→ │
    │   (Certificate exchange, encryption setup)   │
    │                                              │
    ├─ 4. HTTP Upgrade ────────────────────────→ │
    │   GET /notify HTTP/1.1                       │
    │   Upgrade: websocket                         │
    │   Sec-WebSocket-Key: xyz123...               │
    │                                              │
    │   ← HTTP 101 Switching Protocols ───────────┤
    │     Upgrade: websocket                       │
    │     Sec-WebSocket-Accept: abc456...          │
    │                                              │
    ├─ 5. Authentication ──────────────────────→ │
    │   {                                          │
    │     "type": "auth",                          │
    │     "token": "consumer-token-xyz",           │
    │     "consumer_id": "dashboard-1"             │
    │   }                                          │
    │                                              │
    │   ← Ready Message ─────────────────────────┤
    │     {                                        │
    │       "type": "ready",                       │
    │       "consumer_id": "dashboard-1",          │
    │       "status": "connected"                  │
    │     }                                        │
    │                                              │
    └─ CONNECTED ─ Ready to receive messages ─────┘
       (Persistent bidirectional connection)
```

### Connection Timing Details

```
T+0ms:   DNS Query starts
T+2ms:   DNS Response received → 192.168.1.100
T+3ms:   TCP SYN packet sent → Broker
T+5ms:   TCP SYN-ACK received ← Broker
T+6ms:   TCP ACK sent → Broker
T+8ms:   TLS ClientHello → Broker
T+10ms:  TLS ServerHello ← Broker
T+12ms:  TLS Finished → Broker
T+13ms:  TLS Finished ← Broker
T+14ms:  HTTP Upgrade Request → Broker
T+15ms:  HTTP 101 Response ← Broker
T+16ms:  Auth Message → Broker
T+17ms:  Ready Response ← Broker
T+18ms:  ✓ CONNECTED & READY

Total: ~18ms to establish connection
```

### Message Reception Flow

Once connected:

```
BROKER SIDE                              CLIENT SIDE
    │                                        │
Order placed                                │
    │ Check subscribers                     │
    ├─ Consumer "dashboard-1" → Connected ✓ │
    │                                        │
    ├─ Prepare WebSocket frame               │
    │  - Serialize message to JSON           │
    │  - Create frame header                 │
    │  - Add message metadata                │
    │                                        │
    ├─ Send via HTTP/2 or TCP ────────────→ │
    │  (WebSocket BINARY frame)              │
    │  Size: 1,234 bytes                     │
    │                                        │
    │                                    Receive frame
    │                                    Deserialize JSON
    │                                    Trigger onmessage event
    │                                    JavaScript handler runs
    │                                    Update React state
    │                                    DOM updates
    │                                    ✓ Dashboard refreshes
    │                                        │
    └────────────────────────────────────────┤
    Message delivered in < 10ms
```

---

## 2. Webhook Connection (HTTP POST)

### Registration & Delivery Flow

```
YOUR APPLICATION                 FASTDATABROKER SERVER
    │                                      │
1. REGISTRATION PHASE (One-time setup):
    │                                      │
    ├─ POST /api/webhooks/register ──────→│
    │  {                                   │
    │    "webhook_url": "https://your.../notify",
    │    "events": ["order_events"],       │
    │    "api_key": "sk_test_abc123"       │
    │  }                                   │
    │                                      │ Validate
    │                                      │ Store config
    │                                      │ Test webhook
    │  ← Response: webhook_id registered ─┤
    │                                      │
2. DELIVERY PHASE (Per message):
    │                                      │
    │                          Order placed
    │                          Find webhook consumers
    │                          Prepare payload
    │                          │
    │  ← POST /webhook/notify ┤
    │    Headers:              │
    │    X-Webhook-ID: ...     │
    │    X-Signature: ...      │
    │    X-Timestamp: ...      │
    │                          │
    │    Body: Message JSON    │
    │                          │
JSON: 1,234 bytes             │
HTTP POST sent                 │
Network latency: 50ms          │
    │                          │
    Process request         ┤
    Verify signature        │
    Update database         │
    │                          │
    HTTP 200 OK ─────────────→│
    │                       ✓ Delivery confirmed
    │
    All done!
```

### Webhook Verification

```
When webhook arrives at your server:

Headers:
  X-Webhook-ID: wh_abc123xyz
  X-Signature: sha256=73e09b18d3c2e...
  X-Timestamp: 2026-04-07T14:30:45.123Z

Your verification code:
  1. Get raw request body
  2. Get received signature from header
  3. Get timestamp from header
  4. Compute: HMAC-SHA256(body + "." + timestamp, secret)
  5. Compare computed vs received signature
  
  ✓ Match = Legitimate FastDataBroker webhook
  ✗ No match = Spoofed/tampered - REJECT
```

### Retry Logic

```
Attempt 1: Send webhook
  └─ Server error (500) or timeout
     └─ Wait 1 second

Attempt 2: Retry
  └─ Connection refused
     └─ Wait 2 seconds

Attempt 3: Final retry
  └─ Timeout
     └─ Max retries exceeded

Result:
  ├─ Success (200): ✓ Delivered
  ├─ All attempts failed: ↪ Dead Letter Queue
  └─ Webhook admin notified
```

---

## 3. gRPC Connection (High-Performance)

### Connection Establishment

```
CLIENT                             FASTDATABROKER SERVER
  │                                       │
  ├─ Create gRPC Channel                  │
  │  target = "grpc://.../6002"           │
  │                                       │
  ├─ DNS Resolution                       │
  │  (Resolve broker hostname)            │
  │                                       │
  ├─ TCP Connect ───────────────────────→│
  │  (Port 6002)                          │
  │                                       │
  ├─ TLS Handshake ──────────────────────│
  │  (mTLS with mutual cert validation)   │
  │                                       │
  ├─ HTTP/2 Connection Preface           │
  │  Establish HTTP/2                    │
  │  Exchange SETTINGS frames             │
  │  Initialize flow control              │
  │                                       │
  ├─ Call Subscribe RPC ─────────────────│
  │  Method: Broker/Subscribe             │
  │  Request body:                        │
  │  {                                    │
  │    consumer_id: "processor-1",        │
  │    topics: ["order_events"],          │
  │    batch_size: 100                    │
  │  }                                    │
  │                                       │
  │  ← Stream Response begins ────────────┤
  │    (Messages flow continuously)       │
  │                                       │
  └─ STREAMING ─────────────────────────→│
    Message 1: order_confirmed
    Message 2: payment_success
    Message 3: shipment_update
    ... (continuous stream)
```

### gRPC Message Format

```
Comparison:

JSON (HTTP/REST):
{
  "message_id": "msg-abc123",
  "event_type": "order_confirmed",
  "order_id": "ORD-20260407-001",
  "customer_email": "alice@example.com",
  "amount": 299.99,
  "items": [...],
  "timestamp": "2026-04-07T14:30:45.085Z"
}

Size: 1,234 bytes
Parsing: Need JSON parser
Network efficiency: Good

Protobuf (gRPC):
[Binary format - not human readable]

Size: 234 bytes (81% smaller!)
Parsing: Direct deserialization (very fast)
Network efficiency: Excellent
Multiplexing: Multiple streams per connection
Header compression: Automatic
```

### gRPC Benefits

```
✓ Ultra-low latency (5-8ms vs 50-100ms for webhooks)
✓ 80% smaller payload size
✓ Multiple streams on one connection (multiplexing)
✓ Server push capability
✓ Built-in flow control
✓ Header compression
✓ Strong typing via protobuf
✓ Better for high-throughput scenarios
```

---

## 4. Email Polling Connection

### Polling Cycle

```
Every 30 seconds:

YOUR APP                            EMAIL SERVER (IMAP)
  │                                       │
T+0s: Timer fires                         │
      │                                   │
T+1s: TCP Connect ──────────────────────→│
      │ Port 993 (IMAPS)                  │
      │                                   │
T+5s: TLS Handshake ───────────────────→│
      │                                   │
T+6s: LOGIN ───────────────────────────→│
      │ LOGIN email@example.com password   │
      │                                   │
      │ ← OK LOGIN completed ─────────────┤
      │                                   │
T+7s: SELECT INBOX ────────────────────→│
      │                                   │
      │ ← OK [UNSEEN 2] ──────────────────┤
      │   (2 unread messages)             │
      │                                   │
T+8s: FETCH UNREAD ────────────────────→│
      │ FETCH 1:* (BODY[TEXT])            │
      │                                   │
      │ ← Message list ───────────────────┤
      │   [1] From: noreply@...           │
      │   [2] From: noreply@... (NEW)     │
      │                                   │
T+9s: READ NEW MESSAGE ────────────────→│
      │ FETCH 2 BODY[TEXT]                │
      │                                   │
      │ ← Email content ──────────────────┤
      │{                                  │
      │  "message_id": "msg-xyz",         │
      │  "event_type": "order_confirmed"  │
      │}                                  │
      │                                   │
T+10s: MARK AS READ ───────────────────→│
       │ STORE 2 +FLAGS (\Seen)           │
       │                                  │
       │ ← OK [FLAGGED] ───────────────────┤
       │                                  │
T+11s: LOGOUT ─────────────────────────→│
       │ LOGOUT                           │
       │                                  │
       │ ← BYE Server logging out ────────┤
       │                                  │
Process message locally           
Update database
Wait 30 seconds
Repeat
```

### Email Message Format

```
From: noreply@fastdatabroker.local
To: your-email@example.com
Subject: Order Confirmation: ORD-20260407-001
Date: Mon, 07 Apr 2026 14:30:45 +0000

Dear Customer,

Your order ORD-20260407-001 has been confirmed!

---BEGIN FASTDATABROKER NOTIFICATION---
{
  "message_id": "msg-abc123",
  "event_type": "order_confirmed",
  "order_id": "ORD-20260407-001",
  "customer_email": "alice@example.com",
  "amount": 299.99,
  "items": [
    {"sku": "SKU-001", "qty": 1, "price": 199.99},
    {"sku": "SKU-002", "qty": 2, "price": 24.99}
  ],
  "timestamp": "2026-04-07T14:30:45.085Z"
}
---END FASTDATABROKER NOTIFICATION---

Thank you for your purchase!
```

---

## Connection Comparison Table

| Feature | WebSocket | Webhook | gRPC | Email |
|---------|-----------|---------|------|-------|
| **Latency** | < 10ms | 10-100ms | < 5ms | 30s-5m |
| **Setup Time** | ~18ms | ~60ms | ~20ms | Minutes |
| **Connection Type** | Persistent | Stateless | Streaming | Polling |
| **Throughput** | Medium | Medium | High | Low |
| **Message Size** | 1,234 bytes | 1,234 bytes | 234 bytes | Email |
| **Infrastructure** | Broker + Client | Broker only | Broker + Client | Email provider |
| **Scalability** | Good | Excellent | Excellent | Fair |
| **Retry Logic** | Manual | Automatic | Automatic | Manual |
| **Best For** | Real-time UI | Microservices | High-perf | Batch/async |
| **Persistence** | Per session | Stateless | Per session | Permanent |

---

## Which Connection to Use?

### Scenario 1: Real-Time Dashboard
```
✓ BEST: WebSocket (wss://)

Why:
- User is actively watching screen
- Need instant updates (< 100ms)
- Orders appear immediately in browser
- Low cost (single persistent connection)

Latency: < 10ms
Example: Order management dashboard
```

### Scenario 2: Microservice Architecture
```
✓ BEST: Webhook

Why:
- Server-to-server integration
- Stateless (no persistent connections)
- Highly scalable (load balanced)
- Automatic retry handling
- Cloud-native friendly

Latency: 10-100ms
Example: Order processing service
```

### Scenario 3: High-Throughput Processing
```
✓ BEST: gRPC

Why:
- Process thousands of messages/second
- Ultra-low latency required
- Binary format efficiency matters
- Multiplexed streaming

Latency: < 5ms
Example: Real-time analytics engine
```

### Scenario 4: Async Email Notifications
```
✓ BEST: Email Polling

Why:
- No infrastructure on consumer side
- Asynchronous processing OK
- Works with any email provider
- Email as fallback channel

Latency: 30s-5m
Example: Customer notification service
```

### Scenario 5: All of the Above
```
✓ USE ALL CHANNELS

Single message can be delivered via:
1. WebSocket → Dashboard (instant visual update)
2. Webhook → Backend service (processing)
3. gRPC → Analytics (real-time tracking)
4. Email → Customer (confirmation email)

FastDataBroker routes same message to all registered consumers
```

---

## Summary

**Consumer connects via:**

| Channel | How | When | Latency |
|---------|-----|------|---------|
| **WebSocket** | Persistent upgrade-HTTP | Real-time UI | < 10ms |
| **Webhook** | HTTP POST callbacks | Backend services | 10-100ms |
| **gRPC** | Streaming RPC | High-performance | < 5ms |
| **Email** | IMAP polling | Async/batch | 30s-5m |

Choose the right connection type for your use case, and FastDataBroker automatically handles delivery via your preferred channel!
