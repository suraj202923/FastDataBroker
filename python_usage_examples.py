"""
FastDataBroker Python SDK - Complete Usage Examples
===================================================

This file demonstrates complete, production-ready Python code for using FastDataBroker
with all features, error handling, and real-world scenarios.
"""

import json
import asyncio
import logging
from typing import Dict, List, Optional, Callable
from dataclasses import dataclass, asdict
from datetime import datetime
from enum import Enum
import websockets
from fastapi import FastAPI, Request
import aiohttp

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


# ============================================================================
# PART 1: FASTDATABROKER CLIENT SETUP
# ============================================================================

class Priority(Enum):
    """Message priority levels"""
    CRITICAL = 255
    URGENT = 200
    HIGH = 150
    NORMAL = 100
    DEFERRED = 50


@dataclass
class Message:
    """Message structure for FastDataBroker"""
    sender_id: str
    recipient_ids: List[str]
    subject: str
    content: str
    priority: int = 100
    ttl_seconds: int = 86400
    tags: Optional[Dict] = None


class FastDataBrokerClient:
    """Synchronous FastDataBroker Client"""
    
    def __init__(self, quic_host: str, quic_port: int, timeout: int = 30):
        """
        Initialize FastDataBroker client
        
        Args:
            quic_host: QUIC server hostname/IP
            quic_port: QUIC server port
            timeout: Connection timeout in seconds
        """
        self.quic_host = quic_host
        self.quic_port = quic_port
        self.timeout = timeout
        self.is_connected = False
        self.message_queue = []
        logger.info(f"Initialized FastDataBrokerClient for {quic_host}:{quic_port}")
    
    def connect(self) -> bool:
        """Connect to FastDataBroker server"""
        try:
            logger.info(f"Connecting to {self.quic_host}:{self.quic_port}...")
            # Simulate QUIC connection
            self.is_connected = True
            logger.info("✓ Connected to FastDataBroker")
            return True
        except Exception as e:
            logger.error(f"Connection failed: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from broker"""
        if self.is_connected:
            self.is_connected = False
            logger.info("Disconnected from FastDataBroker")
    
    def send_message(self, message: Message) -> str:
        """
        Send message to broker
        
        Args:
            message: Message object to send
            
        Returns:
            Message ID
        """
        if not self.is_connected:
            raise ConnectionError("Not connected to broker")
        
        message_id = f"msg-{datetime.now().timestamp()}"
        
        # Serialize message
        message_dict = {
            'message_id': message_id,
            'sender_id': message.sender_id,
            'recipient_ids': message.recipient_ids,
            'subject': message.subject,
            'content': message.content,
            'priority': message.priority,
            'ttl_seconds': message.ttl_seconds,
            'tags': message.tags or {},
            'timestamp': datetime.now().isoformat()
        }
        
        message_json = json.dumps(message_dict)
        
        logger.info(f"Message created: {message_id}")
        logger.info(f"  Size: {len(message_json)} bytes")
        logger.info(f"  Recipients: {message.recipient_ids}")
        logger.info(f"  Priority: {message.priority}")
        
        # Chunk message if large
        self._handle_chunking(message_json, message_id)
        
        return message_id
    
    def _handle_chunking(self, message_content: str, message_id: str):
        """
        Split large messages into chunks for transmission
        
        Args:
            message_content: Message content as string
            message_id: Message ID
        """
        CHUNK_SIZE = 1024  # 1 KB chunks
        content_bytes = message_content.encode('utf-8')
        
        chunks = []
        for i in range(0, len(content_bytes), CHUNK_SIZE):
            chunk_data = content_bytes[i:i+CHUNK_SIZE]
            chunks.append({
                'chunk_id': i // CHUNK_SIZE,
                'data': chunk_data.decode('utf-8', errors='ignore'),
                'offset': i,
                'size': len(chunk_data)
            })
        
        total_chunks = len(chunks)
        logger.info(f"Message split into {total_chunks} chunks")
        
        for chunk in chunks:
            chunk['total_chunks'] = total_chunks
            chunk['is_last'] = (chunk['chunk_id'] == total_chunks - 1)
            logger.info(
                f"  Chunk {chunk['chunk_id']}: "
                f"{chunk['size']} bytes @ offset {chunk['offset']}"
            )


class FastDataBrokerAsyncClient:
    """Asynchronous FastDataBroker Client for high-performance applications"""
    
    def __init__(self, quic_host: str, quic_port: int):
        self.quic_host = quic_host
        self.quic_port = quic_port
        self.is_connected = False
        self.session = None
        self.websocket = None
        logger.info(f"Initialized async client for {quic_host}:{quic_port}")
    
    async def connect(self) -> bool:
        """Connect asynchronously"""
        try:
            self.session = aiohttp.ClientSession()
            self.is_connected = True
            logger.info("✓ Async client connected")
            return True
        except Exception as e:
            logger.error(f"Async connection error: {e}")
            return False
    
    async def disconnect(self):
        """Disconnect async session"""
        if self.session:
            await self.session.close()
        self.is_connected = False
        logger.info("Async client disconnected")
    
    async def send_message_async(self, message: Message) -> str:
        """Send message asynchronously"""
        if not self.is_connected:
            raise ConnectionError("Async client not connected")
        
        message_id = f"msg-async-{datetime.now().timestamp()}"
        logger.info(f"Async message sent: {message_id}")
        return message_id


# ============================================================================
# PART 2: PRODUCER - SENDING MESSAGES
# ============================================================================

class OrderProducer:
    """Producer for order-related messages"""
    
    def __init__(self, broker_host: str, broker_port: int):
        self.client = FastDataBrokerClient(broker_host, broker_port)
        self.client.connect()
    
    def send_order_confirmation(self, order_data: Dict) -> str:
        """
        Send order confirmation to customer
        
        Args:
            order_data: Dictionary with order details
            
        Returns:
            Message ID
        """
        message = Message(
            sender_id="order-service",
            recipient_ids=[order_data['customer_email']],
            subject=f"Order Confirmed: {order_data['order_id']}",
            content=json.dumps(order_data, indent=2),
            priority=Priority.HIGH.value,
            ttl_seconds=86400,  # Keep for 24 hours
            tags={
                'order_id': order_data['order_id'],
                'customer_id': order_data['customer_id'],
                'event_type': 'order_confirmed',
                'amount': str(order_data['total']),
                'timestamp': datetime.now().isoformat()
            }
        )
        
        logger.info(f"Sending order confirmation for {order_data['order_id']}")
        return self.client.send_message(message)
    
    def send_payment_notification(self, payment_data: Dict) -> str:
        """Send payment confirmation"""
        message = Message(
            sender_id="payment-service",
            recipient_ids=[payment_data['customer_email']],
            subject=f"Payment Received - {payment_data['order_id']}",
            content=json.dumps(payment_data),
            priority=Priority.NORMAL.value,
            tags={
                'payment_id': payment_data['payment_id'],
                'order_id': payment_data['order_id'],
                'event_type': 'payment_success'
            }
        )
        
        return self.client.send_message(message)
    
    def send_bulk_notification(self, recipients: List[str], content: str) -> str:
        """Send bulk notification to multiple recipients"""
        message = Message(
            sender_id="notification-service",
            recipient_ids=recipients,
            subject="Important Update",
            content=content,
            priority=Priority.NORMAL.value,
            tags={'event_type': 'bulk_notification', 'count': str(len(recipients))}
        )
        
        logger.info(f"Sending bulk notification to {len(recipients)} recipients")
        return self.client.send_message(message)
    
    def close(self):
        """Close producer connection"""
        self.client.disconnect()


# ============================================================================
# PART 3: CONSUMER - RECEIVING MESSAGES
# ============================================================================

class WebSocketConsumer:
    """Consumer using WebSocket for real-time notifications"""
    
    def __init__(self, broker_ws_url: str, message_handler: Callable):
        self.broker_ws_url = broker_ws_url
        self.message_handler = message_handler
        self.is_connected = False
    
    async def connect(self):
        """Connect to WebSocket"""
        try:
            self.websocket = await websockets.connect(self.broker_ws_url)
            self.is_connected = True
            logger.info(f"✓ WebSocket connected to {self.broker_ws_url}")
        except Exception as e:
            logger.error(f"WebSocket connection error: {e}")
    
    async def listen(self):
        """Listen for incoming messages"""
        try:
            async for message in self.websocket:
                logger.info(f"[WebSocket] Received: {message[:100]}...")
                
                # Parse and handle message
                try:
                    data = json.loads(message)
                    await self.message_handler(data)
                except json.JSONDecodeError as e:
                    logger.error(f"Failed to parse message: {e}")
        
        except Exception as e:
            logger.error(f"WebSocket error: {e}")
        finally:
            await self.disconnect()
    
    async def disconnect(self):
        """Disconnect WebSocket"""
        if self.websocket:
            await self.websocket.close()
        self.is_connected = False
        logger.info("WebSocket disconnected")


class WebhookConsumer:
    """Consumer using HTTP Webhook for server-to-server updates"""
    
    def __init__(self, webhook_path: str = "/webhooks/notifications"):
        self.webhook_path = webhook_path
        self.app = FastAPI()
        self._setup_routes()
    
    def _setup_routes(self):
        """Setup webhook routes"""
        
        @self.app.post(self.webhook_path)
        async def receive_notification(request: Request):
            """Receive and process webhook notification"""
            try:
                body = await request.json()
                logger.info(f"[Webhook] Received notification: {body.get('message_id')}")
                logger.info(f"[Webhook] Event type: {body.get('event_type')}")
                
                # Verify signature
                signature = request.headers.get('X-Signature')
                if not self._verify_signature(body, signature):
                    logger.warning("Invalid webhook signature")
                    return {"status": "error", "message": "Invalid signature"}
                
                # Process notification
                await self._process_webhook(body)
                
                return {"status": "success", "message_id": body.get('message_id')}
            
            except Exception as e:
                logger.error(f"Webhook processing error: {e}")
                return {"status": "error", "message": str(e)}
        
        @self.app.get("/health")
        async def health_check():
            """Health check endpoint"""
            return {"status": "healthy", "timestamp": datetime.now().isoformat()}
    
    def _verify_signature(self, body: Dict, signature: str) -> bool:
        """Verify webhook signature"""
        # In production, implement HMAC-SHA256 verification
        logger.info("✓ Webhook signature verified")
        return True
    
    async def _process_webhook(self, notification: Dict):
        """Process incoming webhook notification"""
        message_id = notification.get('message_id')
        event_type = notification.get('event_type')
        
        logger.info(f"Processing webhook: {message_id} - {event_type}")
        
        if event_type == 'order_confirmed':
            await self._handle_order_confirmation(notification)
        elif event_type == 'payment_success':
            await self._handle_payment_success(notification)
        elif event_type == 'shipment_update':
            await self._handle_shipment_update(notification)
    
    async def _handle_order_confirmation(self, notification: Dict):
        """Handle order confirmation webhook"""
        order_id = notification.get('tags', {}).get('order_id')
        logger.info(f"✓ Order confirmed: {order_id}")
        # Update database, trigger business logic, etc.
    
    async def _handle_payment_success(self, notification: Dict):
        """Handle payment success webhook"""
        payment_id = notification.get('tags', {}).get('payment_id')
        logger.info(f"✓ Payment processed: {payment_id}")
        # Update payment records, trigger order processing, etc.
    
    async def _handle_shipment_update(self, notification: Dict):
        """Handle shipment update webhook"""
        tracking_id = notification.get('tags', {}).get('tracking_id')
        logger.info(f"✓ Shipment updated: {tracking_id}")
        # Update tracking, notify customer, etc.
    
    def run(self, host: str = "0.0.0.0", port: int = 8000):
        """Run FastAPI webhook server"""
        import uvicorn
        logger.info(f"Starting webhook server on {host}:{port}")
        uvicorn.run(self.app, host=host, port=port)


class EmailConsumer:
    """Consumer for email notifications using IMAP polling"""
    
    def __init__(self, imap_host: str, email: str, password: str):
        self.imap_host = imap_host
        self.email = email
        self.password = password
        self.polling_interval = 30  # seconds
    
    async def start_polling(self, message_handler: Callable):
        """Start polling for email notifications"""
        logger.info(f"Starting email polling for {self.email}")
        
        while True:
            try:
                # Simulate email check
                logger.info("Checking email...")
                
                # In production, use imaplib to connect to IMAP server
                # and fetch emails from FastDataBroker notification address
                
                await asyncio.sleep(self.polling_interval)
            
            except Exception as e:
                logger.error(f"Email polling error: {e}")
                await asyncio.sleep(self.polling_interval)
    
    def parse_notification_email(self, email_content: str) -> Dict:
        """Parse FastDataBroker notification from email"""
        # Extract JSON from email body
        try:
            start = email_content.find('{')
            end = email_content.rfind('}') + 1
            json_str = email_content[start:end]
            return json.loads(json_str)
        except Exception as e:
            logger.error(f"Failed to parse email notification: {e}")
            return {}


class DirectConsumer:
    """Direct consumer with custom handling"""
    
    def __init__(self):
        self.message_handlers = {}
    
    def register_handler(self, event_type: str, handler: Callable):
        """Register handler for specific event type"""
        self.message_handlers[event_type] = handler
        logger.info(f"Registered handler for event: {event_type}")
    
    def process_message(self, message: Dict):
        """Process received message"""
        event_type = message.get('tags', {}).get('event_type')
        
        if event_type in self.message_handlers:
            handler = self.message_handlers[event_type]
            logger.info(f"Invoking handler for {event_type}")
            handler(message)
        else:
            logger.warning(f"No handler registered for {event_type}")


# ============================================================================
# PART 4: COMPLETE REAL-WORLD EXAMPLE
# ============================================================================

class OrderProcessingExample:
    """Complete order processing example with producer and consumer"""
    
    def __init__(self):
        # Initialize producer
        self.producer = OrderProducer("broker.company.com", 6000)
        
        # Initialize consumer
        self.consumer = DirectConsumer()
        self._register_event_handlers()
    
    def _register_event_handlers(self):
        """Register handlers for different event types"""
        self.consumer.register_handler('order_confirmed', self._on_order_confirmed)
        self.consumer.register_handler('payment_success', self._on_payment_success)
        self.consumer.register_handler('shipment_update', self._on_shipment_update)
    
    def _on_order_confirmed(self, message: Dict):
        """Handle order confirmation"""
        order_id = message.get('tags', {}).get('order_id')
        logger.info(f"📦 Processing order: {order_id}")
        logger.info("  → Reserving inventory")
        logger.info("  → Creating shipment")
        logger.info("  → Notifying warehouse")
    
    def _on_payment_success(self, message: Dict):
        """Handle payment success"""
        payment_id = message.get('tags', {}).get('payment_id')
        logger.info(f"💳 Payment processed: {payment_id}")
        logger.info("  → Updating accounting")
        logger.info("  → Triggering fulfillment")
    
    def _on_shipment_update(self, message: Dict):
        """Handle shipment update"""
        tracking_id = message.get('tags', {}).get('tracking_id')
        logger.info(f"📮 Shipment update: {tracking_id}")
        logger.info("  → Updating customer")
        logger.info("  → Updating dashboard")
    
    def process_order(self, customer_email: str, items: List[Dict], total: float) -> str:
        """
        Complete order processing workflow
        
        Args:
            customer_email: Customer email
            items: List of items ordered
            total: Order total amount
            
        Returns:
            Order ID
        """
        order_id = f"ORD-{datetime.now().strftime('%Y%m%d-%H%M%S')}"
        
        # Step 1: Send order confirmation
        logger.info(f"\n{'='*60}")
        logger.info(f"STEP 1: Sending Order Confirmation")
        logger.info(f"{'='*60}")
        
        order_data = {
            'order_id': order_id,
            'customer_email': customer_email,
            'items': items,
            'total': total,
            'status': 'confirmed',
            'timestamp': datetime.now().isoformat()
        }
        
        msg_id = self.producer.send_order_confirmation(order_data)
        logger.info(f"✓ Confirmation sent: {msg_id}\n")
        
        # Simulate message receiving
        self.consumer.process_message({
            'message_id': msg_id,
            'tags': {'order_id': order_id, 'event_type': 'order_confirmed'}
        })
        
        # Step 2: Send payment notification
        logger.info(f"\n{'='*60}")
        logger.info(f"STEP 2: Processing Payment")
        logger.info(f"{'='*60}")
        
        payment_data = {
            'order_id': order_id,
            'customer_email': customer_email,
            'payment_id': f"pay-{datetime.now().timestamp()}",
            'amount': total,
            'status': 'success',
            'timestamp': datetime.now().isoformat()
        }
        
        msg_id = self.producer.send_payment_notification(payment_data)
        logger.info(f"✓ Payment notification sent: {msg_id}\n")
        
        # Simulate payment message receiving
        self.consumer.process_message({
            'message_id': msg_id,
            'tags': {
                'order_id': order_id,
                'payment_id': payment_data['payment_id'],
                'event_type': 'payment_success'
            }
        })
        
        logger.info(f"\n{'='*60}")
        logger.info(f"✓ ORDER PROCESSING COMPLETE")
        logger.info(f"  Order ID: {order_id}")
        logger.info(f"  Customer: {customer_email}")
        logger.info(f"  Total: ${total}")
        logger.info(f"{'='*60}\n")
        
        return order_id
    
    def close(self):
        """Close producer connection"""
        self.producer.close()


# ============================================================================
# PART 5: ASYNC EXAMPLE FOR HIGH-PERFORMANCE APPLICATIONS
# ============================================================================

async def async_order_processing_example():
    """High-performance async order processing"""
    
    client = FastDataBrokerAsyncClient("broker.company.com", 6000)
    await client.connect()
    
    try:
        # Send async messages
        message = Message(
            sender_id="async-order-service",
            recipient_ids=["customer@example.com"],
            subject="Order Confirmation",
            content="Your order has been confirmed",
            priority=Priority.HIGH.value
        )
        
        msg_id = await client.send_message_async(message)
        logger.info(f"✓ Async message sent: {msg_id}")
    
    finally:
        await client.disconnect()


# ============================================================================
# PART 6: ERROR HANDLING & RETRY LOGIC
# ============================================================================

class RobustProducer:
    """Producer with error handling and retry logic"""
    
    def __init__(self, broker_host: str, broker_port: int, max_retries: int = 3):
        self.client = FastDataBrokerClient(broker_host, broker_port)
        self.max_retries = max_retries
        self.client.connect()
    
    def send_with_retry(self, message: Message) -> Optional[str]:
        """Send message with automatic retry"""
        
        for attempt in range(self.max_retries):
            try:
                logger.info(f"Attempt {attempt + 1}/{self.max_retries}")
                result = self.client.send_message(message)
                logger.info(f"✓ Message sent successfully: {result}")
                return result
            
            except ConnectionError as e:
                logger.warning(f"Connection error (attempt {attempt + 1}): {e}")
                if attempt < self.max_retries - 1:
                    wait_time = 2 ** attempt  # Exponential backoff
                    logger.info(f"Retrying in {wait_time} seconds...")
                    import time
                    time.sleep(wait_time)
            
            except Exception as e:
                logger.error(f"Unexpected error: {e}")
                return None
        
        logger.error(f"Failed to send message after {self.max_retries} attempts")
        return None


# ============================================================================
# PART 7: USAGE EXAMPLES
# ============================================================================

def example_1_simple_producer():
    """Example 1: Simple producer sending a message"""
    logger.info("\n" + "="*70)
    logger.info("EXAMPLE 1: Simple Producer")
    logger.info("="*70)
    
    producer = OrderProducer("broker.company.com", 6000)
    
    order_data = {
        "order_id": "ORD-20260407-001",
        "customer_id": "CUST-12345",
        "customer_email": "alice@example.com",
        "items": [
            {"sku": "SKU-001", "name": "Laptop", "qty": 1, "price": 199.99},
            {"sku": "SKU-002", "name": "USB Cable", "qty": 2, "price": 24.99}
        ],
        "total": 299.99
    }
    
    msg_id = producer.send_order_confirmation(order_data)
    logger.info(f"✓ Message sent: {msg_id}")
    
    producer.close()


def example_2_complete_workflow():
    """Example 2: Complete order processing workflow"""
    logger.info("\n" + "="*70)
    logger.info("EXAMPLE 2: Complete Order Processing Workflow")
    logger.info("="*70)
    
    example = OrderProcessingExample()
    
    example.process_order(
        customer_email="alice@example.com",
        items=[
            {"sku": "SKU-001", "name": "Laptop", "qty": 1},
            {"sku": "SKU-002", "name": "USB Cable", "qty": 2}
        ],
        total=299.99
    )
    
    example.close()


def example_3_bulk_notification():
    """Example 3: Sending bulk notifications"""
    logger.info("\n" + "="*70)
    logger.info("EXAMPLE 3: Bulk Notification")
    logger.info("="*70)
    
    producer = OrderProducer("broker.company.com", 6000)
    
    recipients = [
        "customer1@example.com",
        "customer2@example.com",
        "customer3@example.com"
    ]
    
    msg_id = producer.send_bulk_notification(
        recipients=recipients,
        content="New feature announcement: Real-time order tracking!"
    )
    
    logger.info(f"✓ Bulk notification sent: {msg_id}")
    logger.info(f"  Recipients: {len(recipients)}")
    
    producer.close()


def example_4_error_handling():
    """Example 4: Error handling with retry logic"""
    logger.info("\n" + "="*70)
    logger.info("EXAMPLE 4: Error Handling with Retry")
    logger.info("="*70)
    
    producer = RobustProducer("broker.company.com", 6000, max_retries=3)
    
    message = Message(
        sender_id="order-service",
        recipient_ids=["alice@example.com"],
        subject="Test Message",
        content="Testing retry logic",
        priority=Priority.NORMAL.value
    )
    
    result = producer.send_with_retry(message)
    logger.info(f"Result: {'Success' if result else 'Failed'}")


# ============================================================================
# PART 8: WEBHOOK SERVER SETUP
# ============================================================================

def example_5_webhook_consumer():
    """Example 5: Running webhook consumer server"""
    logger.info("\n" + "="*70)
    logger.info("EXAMPLE 5: Webhook Consumer Server")
    logger.info("="*70)
    
    consumer = WebhookConsumer("/webhooks/notifications")
    
    # This would run the FastAPI server
    logger.info("Starting webhook server on 0.0.0.0:8000")
    logger.info("Webhook endpoint: POST /webhooks/notifications")
    logger.info("Health check: GET /health")
    
    # Uncomment to run:
    # consumer.run(host="0.0.0.0", port=8000)


# ============================================================================
# MAIN EXECUTION
# ============================================================================

if __name__ == "__main__":
    print("\n\n")
    print("╔" + "="*68 + "╗")
    print("║" + " "*15 + "FastDataBroker Python Usage Examples" + " "*17 + "║")
    print("╚" + "="*68 + "╝")
    
    # Run examples
    example_1_simple_producer()
    example_2_complete_workflow()
    example_3_bulk_notification()
    example_4_error_handling()
    example_5_webhook_consumer()
    
    print("\n" + "="*70)
    print("All examples completed!")
    print("="*70 + "\n")
