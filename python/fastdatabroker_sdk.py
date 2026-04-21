"""
FastDataBroker Python SDK - High-level client for rst_queue FastDataBroker
Provides simple async/sync interfaces for message queuing and notification delivery
"""

import asyncio
from typing import List, Dict, Optional, Any
from dataclasses import dataclass, field
from enum import Enum, IntEnum
import uuid

__version__ = "0.1.13"


class Priority(IntEnum):
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
class TenantConfig:
    """Tenant-specific configuration for multi-tenant deployments."""
    tenant_id: str
    tenant_name: str
    api_key_prefix: str
    rate_limit_rps: int
    max_connections: int
    max_message_size: int = 1048576
    retention_days: int = 30
    enabled: bool = True
    metadata: Dict[str, Any] = field(default_factory=dict)

    def validate(self) -> None:
        if not self.tenant_id:
            raise ValueError("tenant_id cannot be empty")
        if not self.api_key_prefix or not self.api_key_prefix.endswith("_"):
            raise ValueError("api_key_prefix must end with '_' ")
        if self.rate_limit_rps <= 0:
            raise ValueError("rate_limit_rps must be greater than 0")
        if self.max_connections <= 0:
            raise ValueError("max_connections must be greater than 0")


class AppSettings:
    """Container for tenant settings and lookup helpers."""

    def __init__(self) -> None:
        self.tenants: List[TenantConfig] = []

    def add_tenant(self, tenant: TenantConfig) -> None:
        self.tenants.append(tenant)

    def get_tenant(self, tenant_id: str) -> Optional[TenantConfig]:
        for tenant in self.tenants:
            if tenant.tenant_id == tenant_id:
                return tenant
        return None

    def get_tenant_by_api_key(self, api_key: str) -> Optional[TenantConfig]:
        for tenant in self.tenants:
            if api_key.startswith(tenant.api_key_prefix):
                return tenant
        return None


class Client:
    """Simple multi-tenant client facade for compatibility tests."""

    def __init__(self, tenant_id: str, api_key: str, host: str = "localhost", port: int = 6379):
        if not tenant_id:
            raise ValueError("tenant_id cannot be empty")
        if not api_key:
            raise ValueError("api_key cannot be empty")
        self.tenant_id = tenant_id
        self.api_key = api_key
        self.host = host
        self.port = port
        self._connected = False
        self._settings: Optional[AppSettings] = None

    @classmethod
    def from_settings(cls, settings: AppSettings, tenant_id: str, api_key: str) -> "Client":
        tenant = settings.get_tenant(tenant_id)
        if tenant is None:
            raise ValueError(f"tenant '{tenant_id}' not found")
        if not api_key.startswith(tenant.api_key_prefix):
            raise ValueError("api_key does not match tenant prefix")
        client = cls(tenant_id=tenant_id, api_key=api_key)
        client._settings = settings
        return client

    def is_connected(self) -> bool:
        return self._connected

    def connect(self) -> bool:
        self._connected = True
        return True

    def disconnect(self) -> None:
        self._connected = False

    def get_tenant_config(self) -> Optional[TenantConfig]:
        if self._settings is None:
            return None
        return self._settings.get_tenant(self.tenant_id)

    def generate_api_key(self, client_id: str) -> str:
        tenant = self.get_tenant_config()
        if tenant is None:
            raise ValueError(f"tenant '{self.tenant_id}' not found")
        return f"{tenant.api_key_prefix}{client_id}_{uuid.uuid4().hex[:12]}"


@dataclass
class Message:
    """FastDataBroker message envelope"""
    sender_id: str
    recipient_ids: List[str]
    subject: str
    content: bytes
    priority: Priority = Priority.NORMAL
    ttl_seconds: Optional[int] = None
    tags: Dict[str, str] = field(default_factory=dict)
    tenant_id: Optional[str] = None


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
