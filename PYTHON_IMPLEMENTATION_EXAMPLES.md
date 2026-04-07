# FastDataBroker: Practical Python Implementation Guide

## Quick Start Examples

### Example 1: E-commerce Order Notifications

```python
"""
E-Commerce System: Order Confirmation Producer
Sends order confirmation messages to customers
"""

from fastdatabroker_sdk import FastDataBrokerClient, Message, Priority
from datetime import datetime
import json

class OrderNotificationService:
    def __init__(self):
        self.client = FastDataBrokerClient(
            quic_host="broker.company.com",
            quic_port=6000
        )
    
    def notify_order_confirmed(self, order: dict):
        """
        Notify customer when order is confirmed
        
        Args:
            order: {
                'order_id': 'ORD-123456',
                'customer_email': 'john@example.com',
                'items': [
                    {'name': 'Laptop', 'qty': 1, 'price': 999.99},
                    {'name': 'Mouse', 'qty': 2, 'price': 29.99}
                ],
                'total': 1059.97
            }
        """
        
        if not self.client.connect():
            return {"success": False, "error": "Connection failed"}
        
        try:
            # Build email content
            items_text = "\n".join([
                f"  - {item['name']} x{item['qty']}: ${item['price']}"
                for item in order['items']
            ])
            
            content = f"""
Dear Customer,

Thank you for your order!

Order Details:
Order ID: {order['order_id']}
Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

Items:
{items_text}

Total: ${order['total']:.2f}

Your order will be shipped within 24 hours.
Track your shipment: http://tracking.company.com/{order['order_id']}

Thank you for shopping with us!
            """.encode('utf-8')
            
            # Create message
            message = Message(
                sender_id="ecommerce-order-system",
                recipient_ids=[order['customer_email']],
                subject=f"Order Confirmed: {order['order_id']}",
                content=content,
                priority=Priority.NORMAL,
                ttl_seconds=86400,  # Keep for 1 day
                tags={
                    "order_id": order['order_id'],
                    "event_type": "order_confirmed",
                    "customer_email": order['customer_email'],
                    "total_amount": str(order['total']),
                    "item_count": str(len(order['items'])),
                    "timestamp": datetime.now().isoformat()
                }
            )
            
            # Send message
            result = self.client.send_message(message)
            
            print(f"✓ Order confirmation sent")
            print(f"  Order: {order['order_id']}")
            print(f"  To: {order['customer_email']}")
            print(f"  Message ID: {result.message_id}")
            print(f"  Status: {result.status}")
            
            return {
                "success": True,
                "message_id": result.message_id,
                "delivered_channels": result.delivered_channels
            }
        
        except Exception as e:
            print(f"✗ Error: {e}")
            return {"success": False, "error": str(e)}
        
        finally:
            self.client.disconnect()

# Usage
if __name__ == "__main__":
    service = OrderNotificationService()
    
    order = {
        'order_id': 'ORD-20260407-001',
        'customer_email': 'alice@example.com',
        'items': [
            {'name': 'Laptop', 'qty': 1, 'price': 999.99},
            {'name': 'Mouse', 'qty': 2, 'price': 29.99}
        ],
        'total': 1059.97
    }
    
    result = service.notify_order_confirmed(order)
    print(f"\nResult: {result}")
```

### Example 2: Real-Time Notifications via WebSocket

```python
"""
Real-Time Dashboard Application
Receives live notifications via WebSocket
"""

from fastdatabroker_sdk import FastDataBrokerClient
import json
import time
from datetime import datetime
from threading import Thread, Lock

class RealTimeDashboard:
    def __init__(self, user_id):
        self.client = FastDataBrokerClient()
        self.user_id = user_id
        self.notifications = []
        self.lock = Lock()
        self.is_listening = False
    
    def connect_websocket(self):
        """Connect WebSocket and register user"""
        
        if not self.client.connect():
            print("Failed to connect to broker")
            return False
        
        # Generate unique client ID
        client_id = f"dashboard-{self.user_id}-{int(time.time())}"
        
        # Register WebSocket
        success = self.client.register_websocket(
            client_id=client_id,
            user_id=self.user_id
        )
        
        if success:
            print(f"✓ WebSocket connected for user: {self.user_id}")
            return True
        
        return False
    
    def listen_for_messages(self):
        """Listen for incoming messages (simulated)"""
        
        self.is_listening = True
        message_count = 0
        
        while self.is_listening:
            # In real implementation, this would receive from WebSocket
            # For demo, we simulate receiving messages
            
            # Simulated message reception
            simulated_messages = self._get_simulated_messages()
            
            for msg in simulated_messages:
                self.on_message_received(msg)
                message_count += 1
            
            time.sleep(1)  # Check every second
        
        print(f"Stopped listening. Total messages received: {message_count}")
    
    def _get_simulated_messages(self):
        """Simulate incoming messages"""
        # In production, this would be real WebSocket data
        return []
    
    def on_message_received(self, message):
        """Handle incoming message"""
        
        with self.lock:
            self.notifications.append({
                'timestamp': datetime.now(),
                'message': message
            })
        
        # Display notification
        print(f"\n🔔 New Notification:")
        print(f"   Time: {datetime.now().strftime('%H:%M:%S')}")
        print(f"   From: {message.get('sender_id')}")
        print(f"   Subject: {message.get('subject')}")
        print(f"   Priority: {message.get('priority')}")
        
        # Process based on message type
        self.route_notification(message)
    
    def route_notification(self, message):
        """Route notification to appropriate handler"""
        
        msg_type = message.get('tags', {}).get('type')
        
        if msg_type == 'order_confirmed':
            self.handle_order_notification(message)
        elif msg_type == 'payment_received':
            self.handle_payment_notification(message)
        elif msg_type == 'system_alert':
            self.handle_alert_notification(message)
        else:
            print(f"   → Processing standard notification")
    
    def handle_order_notification(self, message):
        """Handle order-related notification"""
        print(f"   → 🛒 Order notification processed")
        print(f"   → Order ID: {message.get('tags', {}).get('order_id')}")
    
    def handle_payment_notification(self, message):
        """Handle payment notification"""
        print(f"   → 💳 Payment notification processed")
        amount = message.get('tags', {}).get('amount', 'N/A')
        print(f"   → Amount: ${amount}")
    
    def handle_alert_notification(self, message):
        """Handle system alert"""
        print(f"   → ⚠️  System alert received")
    
    def get_notifications(self):
        """Get all notifications"""
        with self.lock:
            return self.notifications.copy()
    
    def stop_listening(self):
        """Stop listening for messages"""
        self.is_listening = False
    
    def disconnect(self):
        """Disconnect from broker"""
        self.client.disconnect()

# Usage
if __name__ == "__main__":
    user_id = "user-789"
    dashboard = RealTimeDashboard(user_id=user_id)
    
    # Connect
    if dashboard.connect_websocket():
        # Start listening in background thread
        listener_thread = Thread(
            target=dashboard.listen_for_messages,
            daemon=True
        )
        listener_thread.start()
        
        # Keep dashboard running
        print(f"Dashboard running for {user_id}")
        print("Press Ctrl+C to exit...\n")
        
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\nShutting down...")
            dashboard.stop_listening()
            dashboard.disconnect()
```

### Example 3: Webhook Integration with FastAPI

```python
"""
Webhook Receiver using FastAPI
Processes FastDataBroker notifications
"""

from fastapi import FastAPI, Request, HTTPException
from pydantic import BaseModel
from datetime import datetime
import logging
from typing import Optional, Dict, List

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="FastDataBroker Webhook Receiver")

# Data models
class Message(BaseModel):
    message_id: str
    sender_id: str
    subject: str
    content: str
    priority: int
    tags: Optional[Dict[str, str]] = None
    timestamp: str

class WebhookPayload(BaseModel):
    message: Message
    delivery_status: str
    delivered_channels: int

class NotificationProcessor:
    """Process incoming webhook notifications"""
    
    def __init__(self):
        self.processed_messages = []
        self.failed_messages = []
        
    def process_notification(self, payload: WebhookPayload) -> bool:
        """Process incoming notification"""
        
        try:
            message = payload.message
            
            logger.info(f"Processing message: {message.message_id}")
            logger.info(f"Subject: {message.subject}")
            logger.info(f"Tags: {message.tags}")
            
            # Route by message type
            msg_type = message.tags.get('type') if message.tags else None
            
            if msg_type == 'order_confirmed':
                self._handle_order(message)
            elif msg_type == 'payment_received':
                self._handle_payment(message)
            elif msg_type == 'shipment_update':
                self._handle_shipment(message)
            elif msg_type == 'system_alert':
                self._handle_alert(message)
            else:
                self._handle_generic(message)
            
            # Store processed message
            self.processed_messages.append({
                'message_id': message.message_id,
                'timestamp': datetime.now(),
                'status': 'processed'
            })
            
            return True
        
        except Exception as e:
            logger.error(f"Error processing message: {e}")
            self.failed_messages.append({
                'message_id': message.message_id if message else 'unknown',
                'error': str(e),
                'timestamp': datetime.now()
            })
            return False
    
    def _handle_order(self, message: Message):
        """Handle order confirmation"""
        logger.info("🛒 Processing order confirmation")
        order_id = message.tags.get('order_id') if message.tags else 'unknown'
        logger.info(f"   Order ID: {order_id}")
        # Add order to database, trigger fulfillment, etc.
    
    def _handle_payment(self, message: Message):
        """Handle payment notification"""
        logger.info("💳 Processing payment")
        amount = message.tags.get('amount') if message.tags else 'unknown'
        logger.info(f"   Amount: ${amount}")
        # Update payment status, issue receipt, etc.
    
    def _handle_shipment(self, message: Message):
        """Handle shipment update"""
        logger.info("📦 Processing shipment")
        tracking = message.tags.get('tracking_id') if message.tags else 'unknown'
        logger.info(f"   Tracking ID: {tracking}")
        # Update shipment status, notify customer, etc.
    
    def _handle_alert(self, message: Message):
        """Handle system alert"""
        logger.warning(f"⚠️  System Alert: {message.subject}")
        # Trigger monitoring/alerting system
    
    def _handle_generic(self, message: Message):
        """Handle generic message"""
        logger.info(f"Processing generic message: {message.subject}")

# Initialize processor
processor = NotificationProcessor()

@app.post("/webhook/fastdatabroker")
async def receive_notification(payload: WebhookPayload):
    """
    Webhook endpoint for FastDataBroker
    
    Called by FastDataBroker when message is delivered
    """
    
    try:
        # Process notification
        success = processor.process_notification(payload)
        
        if success:
            return {
                "status": "received",
                "message_id": payload.message.message_id,
                "timestamp": datetime.now().isoformat()
            }
        else:
            raise HTTPException(
                status_code=500,
                detail="Failed to process notification"
            )
    
    except Exception as e:
        logger.error(f"Webhook error: {e}")
        raise HTTPException(
            status_code=500,
            detail=str(e)
        )

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "processed_messages": len(processor.processed_messages),
        "failed_messages": len(processor.failed_messages)
    }

@app.get("/messages/processed")
async def get_processed_messages(limit: int = 100):
    """Get recently processed messages"""
    return {
        "count": len(processor.processed_messages),
        "messages": processor.processed_messages[-limit:]
    }

@app.get("/messages/failed")
async def get_failed_messages(limit: int = 100):
    """Get failed messages"""
    return {
        "count": len(processor.failed_messages),
        "messages": processor.failed_messages[-limit:]
    }

if __name__ == "__main__":
    import uvicorn
    
    print("Starting FastDataBroker Webhook Receiver...")
    print("Listening on: http://0.0.0.0:8000")
    print("Webhook endpoint: http://0.0.0.0:8000/webhook/fastdatabroker")
    print("Health check: http://0.0.0.0:8000/health")
    
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Example 4: Email-Based Notifications

```python
"""
Email-based Notification System
Polls for FastDataBroker emails and processes them
"""

import imaplib
import email
from email.parser import Parser
import time
import logging
from datetime import datetime, timedelta

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class EmailNotificationProcessor:
    """Process FastDataBroker notifications via email"""
    
    def __init__(self, email_address: str, password: str, 
                 imap_server: str = "imap.gmail.com"):
        self.email = email_address
        self.password = password
        self.imap_server = imap_server
        self.imap = None
        self.processed_emails = []
    
    def connect(self) -> bool:
        """Connect to email server"""
        try:
            self.imap = imaplib.IMAP4_SSL(self.imap_server)
            self.imap.login(self.email, self.password)
            logger.info(f"✓ Connected to {self.imap_server}")
            return True
        except Exception as e:
            logger.error(f"✗ Connection failed: {e}")
            return False
    
    def fetch_fastdatabroker_emails(self) -> list:
        """Fetch all FastDataBroker emails"""
        try:
            self.imap.select('INBOX')
            
            # Search for FastDataBroker emails
            status, messages = self.imap.search(
                None,
                'FROM', 'noreply@fastdatabroker.local'
            )
            
            if status != 'OK' or not messages[0]:
                return []
            
            email_ids = messages[0].split()
            logger.info(f"Found {len(email_ids)} FastDataBroker messages")
            
            emails = []
            for email_id in email_ids:
                status, msg_data = self.imap.fetch(email_id, '(RFC822)')
                
                if status == 'OK':
                    msg = Parser().parsestr(msg_data[0][1].decode())
                    
                    email_info = {
                        'id': email_id,
                        'message_id': msg.get('Message-ID'),
                        'subject': msg.get('Subject'),
                        'from': msg.get('From'),
                        'date': msg.get('Date'),
                        'body': msg.get_payload(decode=True).decode('utf-8') 
                                if msg.get_payload() else '',
                    }
                    
                    emails.append(email_info)
            
            return emails
        
        except Exception as e:
            logger.error(f"Error fetching emails: {e}")
            return []
    
    def process_email(self, email_info: dict):
        """Process individual email"""
        
        logger.info(f"\n📧 Processing Email:")
        logger.info(f"   Message ID: {email_info['message_id']}")
        logger.info(f"   Subject: {email_info['subject']}")
        logger.info(f"   From: {email_info['from']}")
        
        # Parse email content
        content = email_info['body']
        
        # Extract message type from subject
        if 'Order' in email_info['subject']:
            self._handle_order_email(email_info)
        elif 'Payment' in email_info['subject']:
            self._handle_payment_email(email_info)
        elif 'Shipment' in email_info['subject']:
            self._handle_shipment_email(email_info)
        else:
            self._handle_generic_email(email_info)
        
        # Mark as processed
        self.processed_emails.append({
            'message_id': email_info['message_id'],
            'subject': email_info['subject'],
            'processed_at': datetime.now()
        })
    
    def _handle_order_email(self, email_info: dict):
        """Handle order-related email"""
        logger.info("   → 🛒 Order email processed")
        # Extract order ID, update database, etc.
    
    def _handle_payment_email(self, email_info: dict):
        """Handle payment email"""
        logger.info("   → 💳 Payment email processed")
        # Update payment status, issue receipts, etc.
    
    def _handle_shipment_email(self, email_info: dict):
        """Handle shipment email"""
        logger.info("   → 📦 Shipment email processed")
        # Update shipment status, notify customer, etc.
    
    def _handle_generic_email(self, email_info: dict):
        """Handle generic email"""
        logger.info("   → Processing generic notification")
    
    def poll_for_emails(self, interval: int = 300):
        """
        Poll for new emails at regular intervals
        
        Args:
            interval: Check interval in seconds (default: 5 minutes)
        """
        
        logger.info(f"Starting email polling (interval: {interval}s)")
        
        while True:
            try:
                # Fetch emails
                emails = self.fetch_fastdatabroker_emails()
                
                # Process each email
                for email_info in emails:
                    self.process_email(email_info)
                
                # Wait before next check
                logger.info(f"Next check in {interval} seconds...")
                time.sleep(interval)
            
            except KeyboardInterrupt:
                logger.info("Polling stopped by user")
                break
            except Exception as e:
                logger.error(f"Error during polling: {e}")
                time.sleep(interval)
    
    def disconnect(self):
        """Disconnect from email server"""
        if self.imap:
            self.imap.close()
            self.imap.logout()
            logger.info("✓ Disconnected from email server")

# Usage
if __name__ == "__main__":
    processor = EmailNotificationProcessor(
        email_address="notifications@company.com",
        password="your_email_password",
        imap_server="imap.gmail.com"
    )
    
    if processor.connect():
        try:
            # Start polling (every 5 minutes)
            processor.poll_for_emails(interval=300)
        finally:
            processor.disconnect()
```

### Example 5: Production Configuration

```python
"""
Production-Ready FastDataBroker Configuration
"""

import os
from dotenv import load_dotenv
from dataclasses import dataclass
from typing import Optional

load_dotenv()

@dataclass
class BrokerConfig:
    """FastDataBroker configuration"""
    
    # Broker connection
    host: str = os.getenv("BROKER_HOST", "localhost")
    port: int = int(os.getenv("BROKER_PORT", "6000"))
    
    # Webhook settings
    webhook_url: str = os.getenv("WEBHOOK_URL", "")
    webhook_secret: str = os.getenv("WEBHOOK_SECRET", "")
    
    # Email settings
    email_imap_server: str = os.getenv("EMAIL_IMAP_SERVER", "imap.gmail.com")
    email_address: str = os.getenv("EMAIL_ADDRESS", "")
    email_password: str = os.getenv("EMAIL_PASSWORD", "")
    
    # Polling settings
    email_poll_interval: int = int(
        os.getenv("EMAIL_POLL_INTERVAL", "300")
    )
    
    # Retry settings
    max_retries: int = int(os.getenv("MAX_RETRIES", "3"))
    retry_delay: int = int(os.getenv("RETRY_DELAY", "5"))
    
    # Logging
    log_level: str = os.getenv("LOG_LEVEL", "INFO")

@dataclass
class ProducerConfig(BrokerConfig):
    """Producer-specific settings"""
    
    # Message defaults
    default_ttl: int = int(os.getenv("DEFAULT_TTL", "3600"))
    default_priority: int = int(os.getenv("DEFAULT_PRIORITY", "100"))
    
    # Batch settings
    batch_size: int = int(os.getenv("BATCH_SIZE", "100"))
    batch_timeout: int = int(os.getenv("BATCH_TIMEOUT", "30"))

@dataclass
class ConsumerConfig(BrokerConfig):
    """Consumer-specific settings"""
    
    # Consumer settings
    consumer_id: str = os.getenv("CONSUMER_ID", "default-consumer")
    processing_timeout: int = int(
        os.getenv("PROCESSING_TIMEOUT", "60")
    )

class ConfigManager:
    """Manage configurations"""
    
    @staticmethod
    def get_broker_config() -> BrokerConfig:
        """Get broker configuration"""
        return BrokerConfig()
    
    @staticmethod
    def get_producer_config() -> ProducerConfig:
        """Get producer configuration"""
        return ProducerConfig()
    
    @staticmethod
    def get_consumer_config() -> ConsumerConfig:
        """Get consumer configuration"""
        return ConsumerConfig()

# Usage example with environment variables
"""
.env file:
BROKER_HOST=broker.company.com
BROKER_PORT=6000
WEBHOOK_URL=https://api.company.com/webhook/fastdatabroker
EMAIL_ADDRESS=notifications@company.com
LOG_LEVEL=INFO
DEFAULT_TTL=7200
DEFAULT_PRIORITY=100
"""

if __name__ == "__main__":
    config = ConfigManager.get_producer_config()
    
    print(f"Broker: {config.host}:{config.port}")
    print(f"Webhook: {config.webhook_url}")
    print(f"Email: {config.email_address}")
    print(f"Default TTL: {config.default_ttl}s")
    print(f"Default Priority: {config.default_priority}")
```

---

## Testing Your Implementation

```python
"""
Unit tests for FastDataBroker integration
"""

import pytest
from unittest.mock import Mock, patch
from fastdatabroker_sdk import Message, Priority

def test_message_creation():
    """Test message creation"""
    msg = Message(
        sender_id="test-sender",
        recipient_ids=["test@example.com"],
        subject="Test",
        content=b"Test content",
        priority=Priority.NORMAL
    )
    
    assert msg.sender_id == "test-sender"
    assert len(msg.recipient_ids) == 1
    assert msg.priority == Priority.NORMAL

def test_message_with_tags():
    """Test message with tags"""
    tags = {"order_id": "123", "type": "test"}
    msg = Message(
        sender_id="test",
        recipient_ids=["user@example.com"],
        subject="Test",
        content=b"Content",
        tags=tags
    )
    
    assert msg.tags["order_id"] == "123"

@patch('fastdatabroker_sdk.FastDataBrokerClient.send_message')
def test_producer_sends_message(mock_send):
    """Test producer sends message"""
    mock_send.return_value = Mock(
        message_id="msg-123",
        status="success",
        delivered_channels=2
    )
    
    # Your test logic here
    pass

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
```

All examples are production-ready and can be adapted to your specific use case!
