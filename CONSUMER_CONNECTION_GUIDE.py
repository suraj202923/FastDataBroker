"""
FastDataBroker Consumer Connection Guide
========================================

This guide explains how consumers connect to FastDataBroker server,
including different connection types, protocols, and detailed workflows.
"""

import json
from typing import Callable, Dict, List, Optional
from datetime import datetime


# ============================================================================
# PART 1: UNDERSTANDING CONSUMER CONNECTIONS
# ============================================================================

print("""
╔═══════════════════════════════════════════════════════════════════════════╗
║         FastDataBroker Consumer Connection Architecture                   ║
╚═══════════════════════════════════════════════════════════════════════════╝

FastDataBroker supports 4 main connection types for consumers:

1. WEBSOCKET (Real-time, Long-lived)
   ├─ Protocol: WebSocket (ws://, wss://)
   ├─ Latency: < 10ms
   ├─ Best for: Live dashboards, real-time notifications
   ├─ Connection: Persistent bidirectional
   └─ Port: 6001 (default)

2. WEBHOOK (Server-to-Server, HTTP POST)
   ├─ Protocol: HTTP/HTTPS
   ├─ Latency: 10-100ms
   ├─ Best for: microservices, backend systems
   ├─ Connection: Event-driven HTTP callbacks
   └─ Port: Custom (80, 443, etc.)

3. EMAIL POLLING (Passive, IMAP)
   ├─ Protocol: IMAP
   ├─ Latency: 30 seconds - 5 minutes
   ├─ Best for: Email notifications, batch processing
   ├─ Connection: Periodic polling
   └─ Port: 993 (IMAPS)

4. DIRECT/GRPC (High-performance, Streaming)
   ├─ Protocol: gRPC, QUIC
   ├─ Latency: < 5ms
   ├─ Best for: High-throughput services
   ├─ Connection: Persistent streams
   └─ Port: 6000 (QUIC), 6002 (gRPC)
""")


# ============================================================================
# PART 2: WEBSOCKET CONSUMER CONNECTION
# ============================================================================

print("\n" + "="*75)
print("PART 1: WEBSOCKET CONSUMER CONNECTION")
print("="*75)

class WebSocketConsumerConnection:
    """
    WebSocket Consumer Connection
    
    Real-time, persistent connection for receiving messages immediately
    """
    
    def __init__(self, broker_host: str, broker_port: int = 6001):
        """
        Initialize WebSocket consumer
        
        Args:
            broker_host: FastDataBroker server host
            broker_port: WebSocket port (default 6001)
        """
        self.broker_host = broker_host
        self.broker_port = broker_port
        self.websocket_url = f"ws://{broker_host}:{broker_port}"
        self.is_connected = False
        self.transaction_id = None
    
    def show_connection_flow(self):
        """Show detailed connection flow"""
        
        print("\n[WEBSOCKET] Connection Establishment Flow:")
        print("─" * 75)
        
        # Step 1: DNS Resolution
        print("\nStep 1: DNS RESOLUTION")
        print(f"  Query: {self.broker_host}")
        print(f"  Result: 192.168.1.100:6001")
        print(f"  Time: 2ms")
        print(f"  Status: ✓ Resolved")
        
        # Step 2: TCP Handshake
        print("\nStep 2: TCP HANDSHAKE")
        print(f"  Client sends SYN packet")
        print(f"  Server responds with SYN-ACK")
        print(f"  Client sends ACK")
        print(f"  Time: 3ms")
        print(f"  Status: ✓ TCP Connected")
        
        # Step 3: TLS Upgrade (for wss://)
        print("\nStep 3: TLS SETUP (Optional for wss://)")
        print(f"  Client initiates TLS handshake")
        print(f"  Exchange certificates")
        print(f"  Establish encryption keys")
        print(f"  Time: 5ms")
        print(f"  Status: ✓ Encrypted (if wss://)")
        
        # Step 4: WebSocket Upgrade
        print("\nStep 4: WEBSOCKET UPGRADE")
        print(f"  Client HTTP Request:")
        print(f"  ├─ GET /notify HTTP/1.1")
        print(f"  ├─ Host: {self.broker_host}:{self.broker_port}")
        print(f"  ├─ Upgrade: websocket")
        print(f"  ├─ Connection: Upgrade")
        print(f"  ├─ Sec-WebSocket-Key: x3JJHMbDL1EzLkh9...")
        print(f"  └─ Sec-WebSocket-Version: 13")
        print(f"  ")
        print(f"  Server Response:")
        print(f"  ├─ 101 Switching Protocols")
        print(f"  ├─ Upgrade: websocket")
        print(f"  ├─ Connection: Upgrade")
        print(f"  └─ Sec-WebSocket-Accept: HSmrc0sMlYUkAGmm...")
        print(f"  ")
        print(f"  Time: 2ms")
        print(f"  Status: ✓ Upgrade Complete")
        
        # Step 5: Authentication
        print("\nStep 5: AUTHENTICATION")
        print(f"  Send auth message:")
        print(f"  {{")
        print(f"    'type': 'auth',")
        print(f"    'token': 'consumer-token-abc123',")
        print(f"    'consumer_id': 'consumer-dashboard-1',")
        print(f"    'subscriptions': ['order_events', 'payment_events']")
        print(f"  }}")
        print(f"  Time: 1ms")
        print(f"  Status: ✓ Authenticated")
        
        # Step 6: Connection Ready
        print("\nStep 6: CONNECTION READY")
        print(f"  Server sends ready message:")
        print(f"  {{")
        print(f"    'type': 'ready',")
        print(f"    'consumer_id': 'consumer-dashboard-1',")
        print(f"    'transaction_id': 'txn-456789',")
        print(f"    'subscriptions_active': 2")
        print(f"  }}")
        print(f"  Time: 1ms")
        print(f"  Status: ✓ READY")
        
        print(f"\n{'Total Connection Time: 14ms':^75}")
        print("─" * 75)
        
        self.is_connected = True
        self.transaction_id = "txn-456789"
    
    def show_message_reception(self):
        """Show how messages are received"""
        
        if not self.is_connected:
            print("✗ Not connected. Call show_connection_flow() first.")
            return
        
        print("\n[WEBSOCKET] Message Reception Flow:")
        print("─" * 75)
        
        print("\nWhen a message for this consumer is queued:")
        
        print("\nStep 1: MESSAGE AVAILABLE IN BROKER")
        print(f"  Order ID: ORD-20260407-001")
        print(f"  Event: order_confirmed")
        print(f"  Recipients: ['consumer-dashboard-1']")
        print(f"  Time: T+0ms")
        
        print("\nStep 2: BROKER ROUTES TO WEBSOCKET CHANNEL")
        print(f"  Look up connected WebSocket clients")
        print(f"  Find: consumer-dashboard-1 → Connection open ✓")
        print(f"  Time: T+1ms")
        
        print("\nStep 3: SERIALIZE MESSAGE")
        print(f"  Convert to JSON format")
        print(f"  Add metadata (message_id, timestamp)")
        print(f"  Size: 1,234 bytes")
        print(f"  Time: T+2ms")
        
        print("\nStep 4: SEND WEBSOCKET FRAME")
        print(f"  WebSocket frame type: BINARY")
        print(f"  Payload:")
        print(f"  {{")
        print(f"    'message_id': 'msg-abc123',")
        print(f"    'event_type': 'order_confirmed',")
        print(f"    'order_id': 'ORD-20260407-001',")
        print(f"    'amount': 299.99,")
        print(f"    'customer_email': 'alice@example.com',")
        print(f"    'timestamp': '2026-04-07T14:30:45.085Z'")
        print(f"  }}")
        print(f"  Time: T+3ms")
        
        print("\nStep 5: CLIENT RECEIVES FRAME")
        print(f"  Browser WebSocket 'message' event triggered")
        print(f"  Data available to JavaScript immediately")
        print(f"  Time: T+5ms (network latency)")
        
        print("\nStep 6: CLIENT PROCESSES MESSAGE")
        print(f"  JavaScript handler called")
        print(f"  Parse JSON")
        print(f"  Update React state")
        print(f"  Re-render component")
        print(f"  Time: T+8ms")
        
        print("\nStep 7: VISUAL UPDATE")
        print(f"  Dashboard shows new order")
        print(f"  Animation triggers")
        print(f"  Time: T+12ms")
        
        print(f"\n{'Total: 12ms from message available to display':^75}")
        print("─" * 75)


# ============================================================================
# PART 3: WEBHOOK CONSUMER CONNECTION
# ============================================================================

print("\n" + "="*75)
print("PART 2: WEBHOOK CONSUMER CONNECTION")
print("="*75)

class WebhookConsumerConnection:
    """
    Webhook Consumer Connection
    
    Server-to-server, HTTP POST based delivery for backend services
    """
    
    def __init__(self, webhook_endpoint: str, api_key: str = None):
        """
        Initialize Webhook consumer
        
        Args:
            webhook_endpoint: Full URL where FastDataBroker will POST messages
            api_key: Optional API key for authentication
        """
        self.webhook_endpoint = webhook_endpoint
        self.api_key = api_key
        self.is_registered = False
    
    def show_registration_flow(self):
        """Show webhook registration/connection flow"""
        
        print("\n[WEBHOOK] Registration & Setup Flow:")
        print("─" * 75)
        
        print("\nStep 1: WEBHOOK REGISTRATION (Setup phase)")
        print(f"  Your Server → FastDataBroker")
        print(f"  ")
        print(f"  POST /api/v1/webhooks/register HTTP/1.1")
        print(f"  Host: api.broker.company.com")
        print(f"  Content-Type: application/json")
        print(f"  ")
        print(f"  {{")
        print(f"    'webhook_url': '{self.webhook_endpoint}',")
        print(f"    'events': ['order_confirmed', 'payment_success'],")
        print(f"    'api_key': '{self.api_key if self.api_key else 'sk-xxx'}',")
        print(f"    'retry_policy': {{")
        print(f"      'max_attempts': 3,")
        print(f"      'timeout_ms': 5000,")
        print(f"      'backoff_ms': 1000")
        print(f"    }},")
        print(f"    'metadata': {{")
        print(f"      'service_name': 'payment-processor',")
        print(f"      'environment': 'production'")
        print(f"    }}")
        print(f"  }}")
        print(f"  Time: 2ms to send")
        
        print(f"\n  FastDataBroker Response:")
        print(f"  {{")
        print(f"    'status': 'success',")
        print(f"    'webhook_id': 'wh_abc123xyz',")
        print(f"    'registered_events': 2,")
        print(f"    'timestamp': '2026-04-07T14:30:00Z'")
        print(f"  }}")
        print(f"  Time: 2ms to receive")
        
        print(f"\n  Status: ✓ Webhook Registered")
        
        print("\nStep 2: WEBHOOK VERIFICATION (Test phase)")
        print(f"  FastDataBroker → Your Server")
        print(f"  ")
        print(f"  POST {self.webhook_endpoint} HTTP/1.1")
        print(f"  Host: your-server.com")
        print(f"  Content-Type: application/json")
        print(f"  X-Webhook-ID: wh_abc123xyz")
        print(f"  X-Signature: HMAC-SHA256-signature")
        print(f"  X-Delivery-Attempt: 1")
        print(f"  ")
        print(f"  {{")
        print(f"    'type': 'webhook.test',")
        print(f"    'event_id': 'evt_test_123'")
        print(f"  }}")
        print(f"  ")
        print(f"  Your Server Responds:")
        print(f"  HTTP 200 OK")
        print(f"  {{'status': 'received'}}")
        print(f"  ")
        print(f"  Time: 50ms (network latency)")
        print(f"  Status: ✓ Verified")
        
        print(f"\n{'Total Setup Time: ~60ms':^75}")
        print("─" * 75)
        
        self.is_registered = True
    
    def show_webhook_delivery(self):
        """Show how webhooks are delivered"""
        
        if not self.is_registered:
            print("✗ Not registered. Call show_registration_flow() first.")
            return
        
        print("\n[WEBHOOK] Delivery Flow:")
        print("─" * 75)
        
        print("\nWhen message is ready for delivery:")
        
        print("\nStep 1: MESSAGE ENQUEUED")
        print(f"  Message: ORD-20260407-001 placed")
        print(f"  Event: order_confirmed")
        print(f"  Target: webhook endpoint")
        print(f"  Time: T+0ms")
        
        print("\nStep 2: PREPARE PAYLOAD")
        print(f"  Serialize message")
        print(f"  Generate signature:")
        print(f"    timestamp = 2026-04-07T14:30:45.123Z")
        print(f"    secret = 'whsec_abc123xyz'")
        print(f"    signature = HMAC-SHA256(payload + timestamp, secret)")
        print(f"  Time: T+2ms")
        
        print("\nStep 3: RESOLVE ENDPOINT HOSTNAME")
        print(f"  DNS lookup: your-server.com → 203.0.113.42")
        print(f"  (cached, instant)")
        print(f"  Time: T+3ms")
        
        print("\nStep 4: ESTABLISH HTTP CONNECTION")
        print(f"  TCP connect to 203.0.113.42:443")
        print(f"  TLS handshake (5ms)")
        print(f"  Time: T+10ms")
        
        print("\nStep 5: SEND HTTP POST")
        print(f"  POST /webhooks/notifications HTTP/1.1")
        print(f"  Host: your-server.com")
        print(f"  Content-Type: application/json")
        print(f"  Content-Length: 1234")
        print(f"  X-Webhook-ID: wh_abc123xyz")
        print(f"  X-Signature: sha256=signature_here")
        print(f"  X-Timestamp: 2026-04-07T14:30:45.123Z")
        print(f"  X-Delivery-Attempt: 1/3")
        print(f"  ")
        print(f"  Payload:")
        print(f"  {{")
        print(f"    'webhook_id': 'wh_abc123xyz',")
        print(f"    'delivery_id': 'dlv_xyz789',")
        print(f"    'message_id': 'msg-abc123',")
        print(f"    'event': 'order_confirmed',")
        print(f"    'data': {{")
        print(f"      'order_id': 'ORD-20260407-001',")
        print(f"      'customer_email': 'alice@example.com',")
        print(f"      'amount': 299.99,")
        print(f"      'items': [...],")
        print(f"      'timestamp': '2026-04-07T14:30:45.085Z'")
        print(f"    }}")
        print(f"  }}")
        print(f"  Time: T+15ms")
        
        print("\nStep 6: YOUR SERVER PROCESSES")
        print(f"  Receive HTTP request")
        print(f"  Verify signature (HMAC validation)")
        print(f"  Parse JSON payload")
        print(f"  Extract order data")
        print(f"  Query database")
        print(f"  Update order status")
        print(f"  Time: T+25ms (processing)")
        
        print("\nStep 7: SEND RESPONSE")
        print(f"  HTTP 200 OK")
        print(f"  {{'status': 'processed', 'order_id': 'ORD-20260407-001'}}")
        print(f"  Time: T+30ms")
        
        print("\nStep 8: BROKER CONFIRMS DELIVERY")
        print(f"  Receives HTTP 200")
        print(f"  Marks delivery successful")
        print(f"  Logs delivery event")
        print(f"  Time: T+35ms")
        
        print(f"\n{'Total: 35ms from enqueue to confirmed delivery':^75}")
        
        print("\n[WEBHOOK] Retry Logic (if needed):")
        print(f"  ├─ Attempt 1: Delivery failed (timeout/500 error)")
        print(f"  ├─ Wait 1 second (backoff)")
        print(f"  ├─ Attempt 2: Retry")
        print(f"  │  ├─ If fails, wait 2 seconds")
        print(f"  ├─ Attempt 3: Final retry")
        print(f"  └─ If all fail: Move to dead letter queue")
        print("─" * 75)


# ============================================================================
# PART 4: DIRECT/GRPC CONSUMER CONNECTION
# ============================================================================

print("\n" + "="*75)
print("PART 3: DIRECT (gRPC) CONSUMER CONNECTION")
print("="*75)

class DirectConsumerConnection:
    """
    Direct/gRPC Consumer Connection
    
    High-performance, persistent streaming connection
    """
    
    def __init__(self, broker_host: str, broker_port: int = 6002):
        """
        Initialize Direct consumer
        
        Args:
            broker_host: FastDataBroker server host
            broker_port: gRPC port (default 6002)
        """
        self.broker_host = broker_host
        self.broker_port = broker_port
        self.grpc_url = f"grpc://{broker_host}:{broker_port}"
        self.is_connected = False
        self.stream_id = None
    
    def show_connection_flow(self):
        """Show gRPC connection flow"""
        
        print("\n[gRPC] Connection Establishment Flow:")
        print("─" * 75)
        
        print("\nStep 1: CREATE CHANNEL")
        print(f"  gRPC client creates channel to {self.grpc_url}")
        print(f"  Target: {self.broker_host}:{self.broker_port}")
        print(f"  Protocol: HTTP/2")
        print(f"  Time: 1ms")
        
        print("\nStep 2: RESOLVE & CONNECT")
        print(f"  DNS resolution: {self.broker_host} → 192.168.1.100")
        print(f"  TCP handshake to port {self.broker_port}")
        print(f"  Time: 5ms")
        
        print("\nStep 3: TLS HANDSHAKE")
        print(f"  Establish TLS 1.3 encryption")
        print(f"  Exchange certificates")
        print(f"  Verify server identity")
        print(f"  Time: 8ms")
        
        print("\nStep 4: HTTP/2 CONNECTION PREFACE")
        print(f"  Send HTTP/2 connection preface")
        print(f"  Exchange SETTINGS frames")
        print(f"  Initialize stream flow control")
        print(f"  Time: 2ms")
        
        print("\nStep 5: GRPC SUBSCRIBE RPC")
        print(f"  Method: fastdatabroker.Broker/Subscribe")
        print(f"  Request:")
        print(f"  {{")
        print(f"    'consumer_id': 'grpc-processor-1',")
        print(f"    'subscriptions': ['order_events', 'payment_events'],")
        print(f"    'start_from': 'latest',")
        print(f"    'batch_size': 100")
        print(f"  }}")
        print(f"  Time: 2ms")
        
        print("\nStep 6: STREAMING ESTABLISHED")
        print(f"  Server responds with stream")
        print(f"  Bi-directional streaming begins")
        print(f"  Client can send flow control messages")
        print(f"  Server sends messages in real-time")
        print(f"  Time: 1ms")
        
        print(f"\n{'Total Connection Time: 19ms':^75}")
        print("─" * 75)
        
        self.is_connected = True
        self.stream_id = "stream-grpc-123"
    
    def show_stream_delivery(self):
        """Show how messages flow through gRPC stream"""
        
        if not self.is_connected:
            print("✗ Not connected. Call show_connection_flow() first.")
            return
        
        print("\n[gRPC] Stream Message Delivery:")
        print("─" * 75)
        
        print("\nOnce connected, messages flow continuously:")
        
        print("\nT+0ms: Message Available")
        print(f"  Order ID: ORD-20260407-001")
        print(f"  Queue position: Front of queue")
        
        print("\nT+1ms: Serialize to gRPC Format")
        print(f"  Protocol Buffer encoding")
        print(f"  Very compact binary format")
        print(f"  Size: 234 bytes (vs 1,234 JSON)")
        
        print("\nT+2ms: Send via HTTP/2 Frame")
        print(f"  DATA frame on stream: stream-grpc-123")
        print(f"  Payload: gRPC message")
        print(f"  Flow control: Automatic")
        
        print("\nT+5ms: Client Receives Frame")
        print(f"  gRPC client library processes")
        print(f"  Decode Protocol Buffer")
        print(f"  Call onNext() callback")
        
        print("\nT+8ms: Application Processes")
        print(f"  Your handler function executes")
        print(f"  Direct access to message fields")
        print(f"  No parsing needed")
        
        print(f"\n{'Total: 8ms from available to processed':^75}")
        
        print("\n[gRPC] Advantages:")
        print(f"  ✓ Low latency (5-8ms)")
        print(f"  ✓ Compact binary format (80% smaller)")
        print(f"  ✓ Bi-directional streaming")
        print(f"  ✓ Multiplexing (multiple streams per connection)")
        print(f"  ✓ Server push capability")
        print("─" * 75)


# ============================================================================
# PART 5: EMAIL POLLING CONSUMER CONNECTION
# ============================================================================

print("\n" + "="*75)
print("PART 4: EMAIL POLLING CONSUMER CONNECTION")
print("="*75)

class EmailPollingConnector:
    """
    Email Polling Consumer Connection
    
    Periodic polling via IMAP for email-based notifications
    """
    
    def __init__(self, imap_host: str, email: str, polling_interval: int = 30):
        """
        Initialize Email polling consumer
        
        Args:
            imap_host: IMAP server host
            email: Email address to poll
            polling_interval: Seconds between polls
        """
        self.imap_host = imap_host
        self.email = email
        self.polling_interval = polling_interval
        self.is_configured = False
    
    def show_setup(self):
        """Show email polling setup"""
        
        print("\n[EMAIL] Setup & Configuration:")
        print("─" * 75)
        
        print(f"\nEmail Configuration:")
        print(f"  Email: {self.email}")
        print(f"  IMAP Server: {self.imap_host}")
        print(f"  Port: 993 (IMAPS)")
        print(f"  Polling Interval: {self.polling_interval} seconds")
        print(f"  Folder: INBOX")
        print(f"  Filter: From='noreply@fastdatabroker.local'")
        
        print(f"\nStep 1: CONFIGURE EMAIL INBOX")
        print(f"  Create label: 'FastDataBroker'")
        print(f"  Set up rule: Filter messages from noreply@fastdatabroker.local")
        print(f"  Status: ✓ Ready")
        
        print(f"\nStep 2: FIRST CONNECTION")
        print(f"  Connect to {self.imap_host}:993")
        print(f"  Authenticate with email/password")
        print(f"  Select INBOX folder")
        print(f"  Status: ✓ Connected")
        
        self.is_configured = True
    
    def show_polling_cycle(self):
        """Show polling cycle"""
        
        if not self.is_configured:
            print("✗ Not configured. Call show_setup() first.")
            return
        
        print("\n[EMAIL] Polling Cycle:")
        print("─" * 75)
        
        print(f"\nEvery {self.polling_interval} seconds:")
        
        print(f"\nT+0s: POLLING TIMER FIRES")
        print(f"  Scheduled poll time reached")
        
        print(f"T+1s: CONNECT TO IMAP")
        print(f"  TCP connect to {self.imap_host}:993")
        print(f"  TLS handshake")
        print(f"  Time: 5ms")
        
        print(f"T+2s: AUTHENTICATE")
        print(f"  LOGIN {self.email} [password]")
        print(f"  Time: 10ms")
        
        print(f"T+3s: SELECT FOLDER")
        print(f"  SELECT INBOX")
        print(f"  Response: [UNSEEN 1] FLAGS (\\Seen \\Flagged)")
        print(f"  Time: 5ms")
        
        print(f"T+4s: FETCH NEW MESSAGES")
        print(f"  FETCH 1:* (ALL)")
        print(f"  Results: 1 new unread message from noreply@fastdatabroker.local")
        print(f"  Time: 15ms")
        
        print(f"T+5s: READ MESSAGE")
        print(f"  FETCH 1 (BODY[TEXT])")
        print(f"  Message body contains:")
        print(f"  {{")
        print(f"    'message_id': 'msg-abc123',")
        print(f"    'event_type': 'order_confirmed',")
        print(f"    'order_id': 'ORD-20260407-001',")
        print(f"    'customer_email': 'alice@example.com',")
        print(f"    'amount': 299.99")
        print(f"  }}")
        print(f"  Time: 20ms")
        
        print(f"T+6s: MARK AS READ")
        print(f"  STORE 1 +FLAGS (\\Seen)")
        print(f"  Time: 5ms")
        
        print(f"T+7s: DISCONNECT")
        print(f"  LOGOUT")
        print(f"  Close IMAP connection")
        print(f"  Time: 2ms")
        
        print(f"T+8s: PROCESS MESSAGE")
        print(f"  Application receives message data")
        print(f"  Triggers event handlers")
        print(f"  Updates database")
        print(f"  Time: 50-200ms (depends on app)")
        
        print(f"\n{'Total Poll Cycle: ~8-200ms (mostly app processing)':^75}")
        print(f"{'Next poll: {self.polling_interval}s later':^75}")
        print("─" * 75)


# ============================================================================
# PART 6: CONNECTION COMPARISON
# ============================================================================

print("\n" + "="*75)
print("PART 5: CONSUMER CONNECTION METHODS COMPARISON")
print("="*75)

comparison_data = [
    ["Feature", "WebSocket", "Webhook", "gRPC", "Email"],
    ["─" * 20, "─" * 15, "─" * 15, "─" * 10, "─" * 15],
    ["Latency", "< 10ms", "10-100ms", "< 5ms", "30s-5m"],
    ["Connection Type", "Persistent", "HTTP Callback", "Streaming", "Polling"],
    ["Protocol", "WebSocket", "HTTP/HTTPS", "gRPC/HTTP2", "IMAP"],
    ["Bidirectional", "Yes", "One-way", "Yes", "No"],
    ["Authentication", "Token", "HMAC Signature", "mTLS", "Email/Pass"],
    ["Throughput", "Medium", "Medium", "High", "Low"],
    ["Resource Usage", "Low", "Very Low", "Very Low", "Medium"],
    ["Scalability", "Good", "Excellent", "Excellent", "Fair"],
    ["Best For", "Real-time UI", "Microservices", "High-perf", "Async tasks"],
    ["Retry Logic", "Client", "Broker", "Broker", "App"],
    ["Cloud Ready", "Yes (wss://)", "Yes", "Yes", "Yes"],
]

print("\n")
for row in comparison_data:
    print(f"  {row[0]:<20} {row[1]:<15} {row[2]:<15} {row[3]:<10} {row[4]:<15}")


# ============================================================================
# PART 7: COMPLETE CONNECTION EXAMPLE
# ============================================================================

print("\n" + "="*75)
print("PART 6: COMPLETE EXAMPLE - CHOOSING CONNECTION TYPE")
print("="*75)

def example_choose_connection():
    """Show how to choose connection type"""
    
    scenarios = [
        {
            "name": "Real-time Dashboard",
            "choice": "WebSocket",
            "reason": "Instant updates needed, user viewing screen"
        },
        {
            "name": "Backend Service",
            "choice": "Webhook",
            "reason": "Server-to-server, stateless, scalable"
        },
        {
            "name": "Order Processing",
            "choice": "gRPC",
            "reason": "High throughput, low latency, efficiency"
        },
        {
            "name": "Email Notifications",
            "choice": "Email Polling",
            "reason": "Passive consumption, no infrastructure needed"
        },
        {
            "name": "Mixed (All channels)",
            "choice": "Multiple Channels",
            "reason": "Different messages to different consumers"
        }
    ]
    
    print("\n[SCENARIOS] Which connection type to use?")
    print("─" * 75)
    
    for scenario in scenarios:
        print(f"\nScenario: {scenario['name']}")
        print(f"  ✓ Use: {scenario['choice']}")
        print(f"  Reason: {scenario['reason']}")


example_choose_connection()


# ============================================================================
# PART 8: CONNECTION STATE DIAGRAM
# ============================================================================

print("\n" + "="*75)
print("PART 7: CONNECTION LIFECYCLE STATE DIAGRAM")
print("="*75)

print("""
[CONNECTION LIFECYCLE]

┌────────────────┐
│   DISCONNECTED │ (Initial state)
└────────┬───────┘
         │ connect()
         ▼
┌────────────────────┐
│  CONNECTING        │ (DNS, TCP, TLS handshake)
└────────┬───────────┘
         │ Handshake complete
         ▼
┌────────────────────┐
│  AUTHENTICATED     │ (Send credentials/token)
└────────┬───────────┘
         │ Auth success
         ▼
┌────────────────────┐
│  CONNECTED/READY   │ (Can send/receive)
└────────┬───────────┘
         │
    ┌────┴────┐
    │          │
    │ (Normal operation, listening for messages)
    │
    ├─ Receive message → Process
    ├─ Send keepalive/ping
    └─ Keep connection alive
    │
    │ Error/Timeout occurs
    ▼
┌────────────────────┐
│  RECONNECTING      │ (Exponential backoff)
└────────┬───────────┘
         │ Reconnect successful
         ▼ (back to AUTHENTICATED)
         │
         │ Max retries exceeded
         ▼
┌────────────────────┐
│  FAILED            │ (Manual reconnect needed)
└────────────────────┘
         │ User calls reconnect()
         └──→ (back to CONNECTING)
""")


# ============================================================================
# PART 9: SUMMARY
# ============================================================================

print("\n" + "="*75)
print("SUMMARY: How Consumer Connects to Server")
print("="*75)

print("""
1. WEBSOCKET CONNECTION (Real-time)
   ├─ Browser/App initiates WebSocket upgrade
   ├─ Establishes persistent TCP connection
   ├─ Upgrades to WebSocket protocol
   ├─ Authenticates with token
   └─ Receives messages in real-time (< 10ms)

2. WEBHOOK CONNECTION (Server-to-Server)
   ├─ Register webhook endpoint URL with broker
   ├─ Broker verifies endpoint is reachable
   ├─ For each message, FastDataBroker sends HTTP POST
   ├─ Your server processes and responds
   └─ Automatic retry if fails (max 3 attempts)

3. gRPC CONNECTION (High-Performance)
   ├─ Client creates gRPC channel
   ├─ Establishes HTTP/2 connection with TLS
   ├─ Opens bi-directional streaming RPC
   ├─ Server pushes messages as they arrive
   └─ Receives in compact protobuf format (< 5ms)

4. EMAIL POLLING (Passive)
   ├─ Configure email inbox to receive notifications
   ├─ Application polls IMAP server every 30-60s
   ├─ Downloads new messages from FastDataBroker
   ├─ Parses JSON from email body
   └─ Processes asynchronously (30s-5m delay)

BEST PRACTICES:
✓ Use multiple connection types for different use cases
✓ Implement exponential backoff for reconnection
✓ Keep-alive/heartbeat to detect broken connections
✓ Verify signatures/authenticity of messages
✓ Log connection state changes for debugging
✓ Set appropriate timeouts for each connection type
""")

print("\n" + "="*75 + "\n")
