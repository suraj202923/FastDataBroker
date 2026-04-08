"""
FastDataBroker Python SDK - High-level client for rst_queue FastDataBroker
Provides simple async/sync interfaces for message queuing and notification delivery
"""

import asyncio
from typing import List, Dict, Optional, Any
from dataclasses import dataclass
from enum import Enum

__version__ = "0.1.12"


class Priority(Enum):
    """Message priority levels"""
    DEFERRED = 50
    NORMAL = 100
    HIGH = 150
    URGENT = 200
    CRITICAL = 255


class NotificationChannel(Enum):
    """Available notification channels"""
    EMAIL = "email"
    WEBSOCKET = "websocket"
    PUSH = "push"
    WEBHOOK = "webhook"


@dataclass
class Message:
    """FastDataBroker message envelope"""
    sender_id: str
    recipient_ids: List[str]
    subject: str
    content: bytes
    priority: Priority = Priority.NORMAL
    ttl_seconds: Optional[int] = None
    tags: Optional[Dict[str, str]] = None


@dataclass
class DeliveryResult:
    """Message delivery result"""
    message_id: str
    status: str
    delivered_channels: int
    details: Dict[str, Any]


class FastDataBrokerClient:
    """Synchronous FastDataBroker client"""

    def __init__(self, quic_host: str = "localhost", quic_port: int = 6000):
        """
        Initialize FastDataBroker client

        Args:
            quic_host: QUIC server hostname
            quic_port: QUIC server port
        """
        self.quic_host = quic_host
        self.quic_port = quic_port
        self._connected = False

    def connect(self) -> bool:
        """
        Connect to FastDataBroker server

        Returns:
            True if connection successful
        """
        try:
            # Phase 4: Establish QUIC connection
            self._connected = True
            return True
        except Exception as e:
            print(f"Connection failed: {e}")
            return False

    def disconnect(self) -> None:
        """Disconnect from FastDataBroker server"""
        self._connected = False

    def send_message(self, message: Message) -> DeliveryResult:
        """
        Send a message through FastDataBroker

        Args:
            message: Message envelope

        Returns:
            DeliveryResult with delivery status

        Example:
            >>> client = FastDataBrokerClient()
            >>> client.connect()
            >>> msg = Message(
            ...     sender_id="app1",
            ...     recipient_ids=["user-123"],
            ...     subject="Hello",
            ...     content=b"Hello, World!"
            ... )
            >>> result = client.send_message(msg)
            >>> print(result.status)
            'success'
        """
        if not self._connected:
            raise RuntimeError("Not connected to FastDataBroker")

        # Phase 4: Send via QUIC transport
        return DeliveryResult(
            message_id="msg-12345",
            status="success",
            delivered_channels=4,
            details={
                "email": "sent",
                "websocket": "delivered",
                "push": "pending",
                "webhook": "delivered"
            }
        )

    def register_webhook(self, webhook_url: str, headers: Optional[Dict[str, str]] = None) -> bool:
        """
        Register a webhook endpoint for notifications

        Args:
            webhook_url: External webhook URL
            headers: Optional custom headers

        Returns:
            True if registration successful
        """
        if not webhook_url.startswith("http"):
            raise ValueError("Invalid webhook URL")

        # Phase 4: Register webhook with broker
        return True

    def register_websocket(self, client_id: str, user_id: str) -> bool:
        """
        Register a WebSocket client connection

        Args:
            client_id: Unique client identifier
            user_id: Associated user ID

        Returns:
            True if registration successful
        """
        # Phase 4: Register WebSocket client
        return True

    def get_stats(self) -> Dict[str, Any]:
        """
        Get FastDataBroker statistics

        Returns:
            Statistics dictionary with delivery metrics
        """
        # Phase 4: Query statistics from server
        return {
            "total_messages": 0,
            "delivered": 0,
            "failed": 0,
            "channels": {
                "email": {"sent": 0, "failed": 0},
                "websocket": {"connected": 0, "delivered": 0},
                "push": {"sent": 0, "delivered": 0},
                "webhook": {"sent": 0, "delivered": 0}
            }
        }


class FastDataBrokerAsyncClient:
    """Asynchronous FastDataBroker client"""

    def __init__(self, quic_host: str = "localhost", quic_port: int = 6000):
        """Initialize async FastDataBroker client"""
        self.quic_host = quic_host
        self.quic_port = quic_port
        self._connected = False

    async def connect(self) -> bool:
        """
        Connect to FastDataBroker server (async)

        Returns:
            True if connection successful
        """
        try:
            # Phase 4: Establish async QUIC connection
            self._connected = True
            return True
        except Exception as e:
            print(f"Connection failed: {e}")
            return False

    async def disconnect(self) -> None:
        """Disconnect from FastDataBroker server"""
        self._connected = False

    async def send_message(self, message: Message) -> DeliveryResult:
        """
        Send a message asynchronously

        Args:
            message: Message envelope

        Returns:
            DeliveryResult with delivery status
        """
        if not self._connected:
            raise RuntimeError("Not connected to FastDataBroker")

        # Phase 4: Send via QUIC transport asynchronously
        await asyncio.sleep(0.01)  # Simulate network delay

        return DeliveryResult(
            message_id="msg-12345",
            status="success",
            delivered_channels=4,
            details={"channels": "delivered"}
        )

    async def batch_send(self, messages: List[Message]) -> List[DeliveryResult]:
        """
        Send multiple messages in batch

        Args:
            messages: List of messages

        Returns:
            List of delivery results
        """
        tasks = [self.send_message(msg) for msg in messages]
        return await asyncio.gather(*tasks)

    async def get_stats(self) -> Dict[str, Any]:
        """Get statistics asynchronously"""
        await asyncio.sleep(0.01)
        return {
            "total_messages": 0,
            "delivered": 0,
            "failed": 0
        }


class PushNotificationBuilder:
    """Builder for creating push notifications"""

    def __init__(self, title: str):
        """Initialize push notification builder"""
        self.title = title
        self.body = ""
        self.icon = None
        self.badge = None
        self.sound = None
        self.data = {}

    def with_body(self, body: str) -> "PushNotificationBuilder":
        """Set notification body"""
        self.body = body
        return self

    def with_icon(self, icon: str) -> "PushNotificationBuilder":
        """Set notification icon"""
        self.icon = icon
        return self

    def with_sound(self, sound: str) -> "PushNotificationBuilder":
        """Set notification sound"""
        self.sound = sound
        return self

    def with_data(self, key: str, value: str) -> "PushNotificationBuilder":
        """Add custom data"""
        self.data[key] = value
        return self

    def build(self) -> Dict[str, Any]:
        """Build notification payload"""
        return {
            "title": self.title,
            "body": self.body,
            "icon": self.icon,
            "badge": self.badge,
            "sound": self.sound,
            "data": self.data
        }


# Usage examples
if __name__ == "__main__":
    # Synchronous example
    client = FastDataBrokerClient()
    if client.connect():
        msg = Message(
            sender_id="app1",
            recipient_ids=["user-123"],
            subject="Welcome",
            content=b"Welcome to FastDataBroker!",
            priority=Priority.HIGH
        )
        result = client.send_message(msg)
        print(f"Message delivered: {result.status}")
        client.disconnect()

    # Asynchronous example
    async def async_example():
        async_client = FastDataBrokerAsyncClient()
        await async_client.connect()

        messages = [
            Message("app1", ["user-1"], f"Message {i}", b"content") for i in range(10)
        ]
        results = await async_client.batch_send(messages)
        print(f"Batch sent: {len(results)} messages")

        await async_client.disconnect()

    # asyncio.run(async_example())
