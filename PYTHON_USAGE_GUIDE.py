"""
FastDataBroker Python - Core Usage Examples
============================================

This is a simplified, working example that demonstrates how to use 
FastDataBroker Python SDK without external dependencies.
"""

import json
from typing import Dict, List, Optional
from datetime import datetime
from enum import Enum


# ============================================================================
# PART 1: SETUP & CONFIGURATION
# ============================================================================

class Priority(Enum):
    """Message priority levels"""
    CRITICAL = 255
    URGENT = 200
    HIGH = 150
    NORMAL = 100
    DEFERRED = 50


class Message:
    """Message structure for FastDataBroker"""
    
    def __init__(self, sender_id: str, recipient_ids: List[str], 
                 subject: str, content: str, priority: int = 100,
                 ttl_seconds: int = 86400, tags: Optional[Dict] = None):
        self.sender_id = sender_id
        self.recipient_ids = recipient_ids
        self.subject = subject
        self.content = content
        self.priority = priority
        self.ttl_seconds = ttl_seconds
        self.tags = tags or {}
    
    def to_dict(self):
        """Convert message to dictionary"""
        return {
            'sender_id': self.sender_id,
            'recipient_ids': self.recipient_ids,
            'subject': self.subject,
            'content': self.content,
            'priority': self.priority,
            'ttl_seconds': self.ttl_seconds,
            'tags': self.tags
        }


# ============================================================================
# PART 2: FASTDATABROKER CLIENT
# ============================================================================

class FastDataBrokerClient:
    """
    FastDataBroker Client for sending messages
    
    This is the main client used to connect to FastDataBroker
    and send messages to recipients.
    """
    
    def __init__(self, quic_host: str, quic_port: int, timeout: int = 30):
        self.quic_host = quic_host
        self.quic_port = quic_port
        self.timeout = timeout
        self.is_connected = False
        self.message_count = 0
    
    def connect(self) -> bool:
        """Connect to FastDataBroker server"""
        print(f"\n[CONNECT] Connecting to {self.quic_host}:{self.quic_port}...")
        self.is_connected = True
        print(f"[CONNECT] ✓ Connected successfully")
        print(f"[CONNECT] Status: Connected")
        return True
    
    def send_message(self, message: Message) -> str:
        """
        Send message to broker
        
        Returns:
            Message ID
        """
        if not self.is_connected:
            raise ConnectionError("Not connected to broker")
        
        message_id = f"msg-{int(datetime.now().timestamp() * 1000)}"
        self.message_count += 1
        
        # Serialize message
        message_dict = {
            'message_id': message_id,
            **message.to_dict(),
            'timestamp': datetime.now().isoformat()
        }
        
        message_json = json.dumps(message_dict, indent=2)
        message_size = len(message_json.encode('utf-8'))
        
        # Display message info
        print(f"\n[SEND] Message ID: {message_id}")
        print(f"[SEND] Size: {message_size} bytes")
        print(f"[SEND] Priority: {message.priority}")
        print(f"[SEND] Recipients: {', '.join(message.recipient_ids)}")
        print(f"[SEND] Subject: {message.subject}")
        
        # Show chunking if needed
        if message_size > 1024:
            self._show_chunking(message_json, message_id)
        
        print(f"[SEND] Status: ✓ Delivered to queue")
        
        return message_id
    
    def _show_chunking(self, content: str, message_id: str):
        """Show message chunking details"""
        CHUNK_SIZE = 1024
        content_bytes = content.encode('utf-8')
        chunk_count = (len(content_bytes) + CHUNK_SIZE - 1) // CHUNK_SIZE
        
        print(f"\n[CHUNKING] Message split into {chunk_count} chunks:")
        
        for i in range(0, len(content_bytes), CHUNK_SIZE):
            chunk_data = content_bytes[i:i+CHUNK_SIZE]
            chunk_num = i // CHUNK_SIZE
            is_last = (i + CHUNK_SIZE >= len(content_bytes))
            print(f"  Chunk {chunk_num}: {len(chunk_data)} bytes "
                  f"[offset {i}]{'[LAST]' if is_last else ''}")
    
    def disconnect(self):
        """Disconnect from broker"""
        if self.is_connected:
            self.is_connected = False
            print(f"\n[DISCONNECT] Disconnected from broker")
            print(f"[DISCONNECT] Messages sent: {self.message_count}")


# ============================================================================
# PART 3: MESSAGE PRODUCER
# ============================================================================

class OrderProducer:
    """Producer for creating and sending order-related messages"""
    
    def __init__(self, broker_host: str, broker_port: int):
        self.client = FastDataBrokerClient(broker_host, broker_port)
        self.client.connect()
    
    def send_order_confirmation(self, order_data: Dict) -> str:
        """
        Send order confirmation message
        
        Args:
            order_data: Order information dictionary
            
        Returns:
            Message ID
        """
        message = Message(
            sender_id="order-service",
            recipient_ids=[order_data['customer_email']],
            subject=f"Order Confirmed: {order_data['order_id']}",
            content=json.dumps(order_data, indent=2),
            priority=Priority.HIGH.value,
            ttl_seconds=86400,
            tags={
                'order_id': order_data['order_id'],
                'customer_id': order_data['customer_id'],
                'event_type': 'order_confirmed',
                'amount': str(order_data['total'])
            }
        )
        
        return self.client.send_message(message)
    
    def send_payment_notification(self, payment_data: Dict) -> str:
        """
        Send payment confirmation message
        
        Args:
            payment_data: Payment information dictionary
            
        Returns:
            Message ID
        """
        message = Message(
            sender_id="payment-service",
            recipient_ids=[payment_data['customer_email']],
            subject=f"Payment Received - {payment_data['order_id']}",
            content=json.dumps(payment_data, indent=2),
            priority=Priority.NORMAL.value,
            tags={
                'payment_id': payment_data['payment_id'],
                'order_id': payment_data['order_id'],
                'event_type': 'payment_success',
                'amount': str(payment_data['amount'])
            }
        )
        
        return self.client.send_message(message)
    
    def send_shipment_notification(self, shipment_data: Dict) -> str:
        """
        Send shipment update message
        
        Args:
            shipment_data: Shipping information dictionary
            
        Returns:
            Message ID
        """
        message = Message(
            sender_id="shipping-service",
            recipient_ids=[shipment_data['customer_email']],
            subject=f"Your Order is On the Way - {shipment_data['order_id']}",
            content=json.dumps(shipment_data, indent=2),
            priority=Priority.NORMAL.value,
            tags={
                'order_id': shipment_data['order_id'],
                'tracking_id': shipment_data['tracking_id'],
                'event_type': 'shipment_update',
                'carrier': shipment_data['carrier']
            }
        )
        
        return self.client.send_message(message)
    
    def close(self):
        """Close producer connection"""
        self.client.disconnect()


# ============================================================================
# PART 4: MESSAGE CONSUMER (Handler)
# ============================================================================

class MessageConsumer:
    """
    Consumer for processing incoming messages
    
    Handles messages from FastDataBroker and triggers
    appropriate business logic.
    """
    
    def __init__(self):
        self.handlers = {}
        self._register_default_handlers()
    
    def _register_default_handlers(self):
        """Register default event handlers"""
        self.register_handler('order_confirmed', self._handle_order_confirmed)
        self.register_handler('payment_success', self._handle_payment_success)
        self.register_handler('shipment_update', self._handle_shipment_update)
    
    def register_handler(self, event_type: str, handler):
        """Register custom event handler"""
        self.handlers[event_type] = handler
    
    def process_message(self, message: Dict):
        """
        Process incoming message
        
        Args:
            message: Message dictionary from FastDataBroker
        """
        event_type = message.get('tags', {}).get('event_type')
        
        print(f"\n[RECEIVE] Message received: {message.get('message_id')}")
        print(f"[RECEIVE] Event type: {event_type}")
        print(f"[RECEIVE] From: {message.get('sender_id')}")
        print(f"[RECEIVE] To: {', '.join(message.get('recipient_ids', []))}")
        
        if event_type in self.handlers:
            handler = self.handlers[event_type]
            print(f"[RECEIVE] Processing event: {event_type}")
            handler(message)
        else:
            print(f"[RECEIVE] No handler for: {event_type}")
    
    def _handle_order_confirmed(self, message: Dict):
        """Handle order confirmed event"""
        tags = message.get('tags', {})
        order_id = tags.get('order_id')
        
        print(f"\n[HANDLER] 📦 Order Confirmed Handler")
        print(f"[HANDLER] ├─ Order ID: {order_id}")
        print(f"[HANDLER] ├─ Action: Reserve inventory")
        print(f"[HANDLER] ├─ Action: Create packing slip")
        print(f"[HANDLER] └─ Action: Notify warehouse")
    
    def _handle_payment_success(self, message: Dict):
        """Handle payment success event"""
        tags = message.get('tags', {})
        payment_id = tags.get('payment_id')
        amount = tags.get('amount')
        
        print(f"\n[HANDLER] 💳 Payment Success Handler")
        print(f"[HANDLER] ├─ Payment ID: {payment_id}")
        print(f"[HANDLER] ├─ Amount: ${amount}")
        print(f"[HANDLER] ├─ Action: Update accounting")
        print(f"[HANDLER] └─ Action: Start fulfillment")
    
    def _handle_shipment_update(self, message: Dict):
        """Handle shipment update event"""
        tags = message.get('tags', {})
        tracking_id = tags.get('tracking_id')
        carrier = tags.get('carrier')
        
        print(f"\n[HANDLER] 📮 Shipment Update Handler")
        print(f"[HANDLER] ├─ Tracking ID: {tracking_id}")
        print(f"[HANDLER] ├─ Carrier: {carrier}")
        print(f"[HANDLER] ├─ Action: Send tracking email")
        print(f"[HANDLER] └─ Action: Update dashboard")


# ============================================================================
# PART 5: COMPLETE WORKFLOW EXAMPLE
# ============================================================================

def complete_order_workflow_example():
    """
    Complete example: Order from start to finish
    
    This demonstrates:
    1. Creating a producer
    2. Sending order confirmation
    3. Processing payment
    4. Sending shipment notification
    5. Consuming and handling messages
    """
    
    print("\n" + "="*70)
    print("FASTDATABROKER PYTHON - Complete Order Workflow")
    print("="*70)
    
    # Initialize producer and consumer
    producer = OrderProducer("broker.company.com", 6000)
    consumer = MessageConsumer()
    
    # Sample order data
    order_id = "ORD-20260407-001"
    customer_email = "alice@example.com"
    
    # ─────────────────────────────────────────────────────────────────
    # STEP 1: Send order confirmation
    # ─────────────────────────────────────────────────────────────────
    
    print("\n" + "─"*70)
    print("STEP 1: Customer Places Order → Send Confirmation")
    print("─"*70)
    
    order_data = {
        "order_id": order_id,
        "customer_id": "CUST-12345",
        "customer_email": customer_email,
        "items": [
            {
                "sku": "SKU-001",
                "name": "Laptop Computer",
                "quantity": 1,
                "price": 199.99,
                "description": "High-performance laptop 16GB RAM, 512GB SSD"
            },
            {
                "sku": "SKU-002",
                "name": "USB-C Cable",
                "quantity": 2,
                "price": 24.99,
                "description": "Premium USB-C charging cable"
            }
        ],
        "subtotal": 249.97,
        "tax": 25.00,
        "shipping": 0.00,
        "total": 299.99,
        "status": "confirmed",
        "placed_at": datetime.now().isoformat()
    }
    
    msg_id = producer.send_order_confirmation(order_data)
    
    # Simulate receiving and processing
    consumer.process_message({
        'message_id': msg_id,
        'sender_id': 'order-service',
        'recipient_ids': [customer_email],
        'subject': f"Order Confirmed: {order_id}",
        'tags': {
            'order_id': order_id,
            'customer_id': 'CUST-12345',
            'event_type': 'order_confirmed',
            'amount': '299.99'
        }
    })
    
    # ─────────────────────────────────────────────────────────────────
    # STEP 2: Process payment
    # ─────────────────────────────────────────────────────────────────
    
    print("\n" + "─"*70)
    print("STEP 2: Process Payment → Send Confirmation")
    print("─"*70)
    
    payment_data = {
        "order_id": order_id,
        "customer_email": customer_email,
        "payment_id": f"pay-{int(datetime.now().timestamp())}",
        "amount": 299.99,
        "method": "credit_card",
        "card_last_four": "4242",
        "status": "success",
        "processed_at": datetime.now().isoformat()
    }
    
    msg_id = producer.send_payment_notification(payment_data)
    
    consumer.process_message({
        'message_id': msg_id,
        'sender_id': 'payment-service',
        'recipient_ids': [customer_email],
        'subject': f"Payment Received - {order_id}",
        'tags': {
            'payment_id': payment_data['payment_id'],
            'order_id': order_id,
            'event_type': 'payment_success',
            'amount': '299.99'
        }
    })
    
    # ─────────────────────────────────────────────────────────────────
    # STEP 3: Send shipment notification
    # ─────────────────────────────────────────────────────────────────
    
    print("\n" + "─"*70)
    print("STEP 3: Package Shipped → Send Tracking")
    print("─"*70)
    
    shipment_data = {
        "order_id": order_id,
        "customer_email": customer_email,
        "tracking_id": f"TRACK-{int(datetime.now().timestamp())}",
        "carrier": "FedEx",
        "estimated_delivery": "2026-04-10",
        "shipped_at": datetime.now().isoformat(),
        "status": "in_transit"
    }
    
    msg_id = producer.send_shipment_notification(shipment_data)
    
    consumer.process_message({
        'message_id': msg_id,
        'sender_id': 'shipping-service',
        'recipient_ids': [customer_email],
        'subject': f"Your Order is On the Way - {order_id}",
        'tags': {
            'order_id': order_id,
            'tracking_id': shipment_data['tracking_id'],
            'event_type': 'shipment_update',
            'carrier': 'FedEx'
        }
    })
    
    # Close producer
    producer.close()
    
    # ─────────────────────────────────────────────────────────────────
    # Summary
    # ─────────────────────────────────────────────────────────────────
    
    print("\n" + "="*70)
    print("✓ WORKFLOW COMPLETE")
    print("="*70)
    print(f"Order ID: {order_id}")
    print(f"Customer: {customer_email}")
    print(f"Total: $299.99")
    print(f"Status: Order confirmed, paid, and shipped")
    print("="*70)


# ============================================================================
# PART 6: SIMPLE PRODUCER EXAMPLE
# ============================================================================

def simple_producer_example():
    """Simple example: Just send a message"""
    
    print("\n" + "="*70)
    print("Simple Producer Example")
    print("="*70)
    
    # Create client and connect
    client = FastDataBrokerClient("broker.company.com", 6000)
    client.connect()
    
    # Create and send message
    message = Message(
        sender_id="notification-service",
        recipient_ids=["user@example.com"],
        subject="Welcome to FastDataBroker!",
        content="This is your first message using FastDataBroker SDK",
        priority=Priority.NORMAL.value,
        tags={
            'event_type': 'welcome',
            'user_id': 'USER-123'
        }
    )
    
    msg_id = client.send_message(message)
    print(f"\n✓ Message sent successfully!")
    
    # Disconnect
    client.disconnect()


# ============================================================================
# PART 7: BULK NOTIFICATION EXAMPLE
# ============================================================================

def bulk_notification_example():
    """Send message to multiple recipients"""
    
    print("\n" + "="*70)
    print("Bulk Notification Example")
    print("="*70)
    
    client = FastDataBrokerClient("broker.company.com", 6000)
    client.connect()
    
    # Send to multiple recipients
    recipients = [
        "alice@example.com",
        "bob@example.com",
        "charlie@example.com",
        "diana@example.com"
    ]
    
    message = Message(
        sender_id="marketing-service",
        recipient_ids=recipients,
        subject="🎉 New Feature Announcement!",
        content="""
            We're excited to announce a new feature:
            
            ✨ Real-time Order Tracking
            
            Now you can track your orders in real-time with live updates!
            
            Learn more: https://company.com/features/tracking
        """,
        priority=Priority.HIGH.value,
        tags={
            'event_type': 'announcement',
            'campaign': 'feature_launch',
            'recipient_count': str(len(recipients))
        }
    )
    
    msg_id = client.send_message(message)
    print(f"\n✓ Bulk notification sent to {len(recipients)} recipients")
    
    client.disconnect()


# ============================================================================
# MAIN EXECUTION
# ============================================================================

if __name__ == "__main__":
    print("\n\n")
    print("╔" + "="*68 + "╗")
    print("║" + " "*10 + "FastDataBroker Python SDK - Usage Examples" + " "*15 + "║")
    print("╚" + "="*68 + "╝")
    
    # Run examples
    simple_producer_example()
    bulk_notification_example()
    complete_order_workflow_example()
    
    print("\n" + "="*70)
    print("✓ All examples completed!")
    print("="*70)
    print("\nKey Takeaways:")
    print("  1. Initialize client with broker host/port")
    print("  2. Create Message objects with sender, recipients, content")
    print("  3. Send messages using client.send_message()")
    print("  4. Register handlers to process incoming messages")
    print("  5. FastDataBroker handles chunking, delivery, retry")
    print("="*70 + "\n")
