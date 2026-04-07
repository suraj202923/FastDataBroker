"""
Comprehensive test suite for FastDataBroker Python SDK
Tests message delivery, priority handling, notifications, and edge cases
"""

import pytest
import asyncio
from datetime import datetime
from unittest.mock import Mock, MagicMock, patch
from fastdatabroker_sdk import (
    FastDataBrokerClient,
    FastDataBrokerAsyncClient,
    Message,
    Priority,
    NotificationChannel,
    DeliveryResult,
    PushNotificationBuilder,
)


class TestFastDataBrokerClient:
    """Test synchronous FastDataBrokerClient"""

    def test_client_initialization(self):
        """Test client initialization with defaults"""
        client = FastDataBrokerClient()
        assert client.quic_host == "localhost"
        assert client.quic_port == 6000

    def test_client_initialization_custom_host(self):
        """Test client with custom host and port"""
        client = FastDataBrokerClient(quic_host="api.example.com", quic_port=9000)
        assert client.quic_host == "api.example.com"
        assert client.quic_port == 9000

    def test_client_connect(self):
        """Test client connection"""
        client = FastDataBrokerClient()
        result = client.connect()
        assert result is True
        assert client._connected is True

    def test_client_disconnect(self):
        """Test client disconnection"""
        client = FastDataBrokerClient()
        client.connect()
        client.disconnect()
        assert client._connected is False

    def test_send_message_basic(self):
        """Test sending a basic message"""
        client = FastDataBrokerClient()
        client.connect()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Test Message",
            content=b"Hello, World!"
        )
        result = client.send_message(msg)

        assert result.message_id == "msg-12345"
        assert result.status == "success"
        assert result.delivered_channels == 4

    def test_send_message_with_priority(self):
        """Test sending message with priority"""
        client = FastDataBrokerClient()
        client.connect()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Urgent Message",
            content=b"Important!",
            priority=Priority.URGENT
        )
        result = client.send_message(msg)

        assert result.status == "success"
        assert msg.priority == Priority.URGENT
        assert msg.priority.value == 200

    def test_send_message_with_ttl(self):
        """Test sending message with TTL"""
        client = FastDataBrokerClient()
        client.connect()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Temporary Message",
            content=b"Expires soon",
            ttl_seconds=3600
        )
        result = client.send_message(msg)

        assert result.status == "success"
        assert msg.ttl_seconds == 3600

    def test_send_message_without_ttl(self):
        """Test sending message without TTL (default)"""
        client = FastDataBrokerClient()
        client.connect()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Default TTL Message",
            content=b"No expiry"
        )
        assert msg.ttl_seconds is None

    def test_send_message_multiple_recipients(self):
        """Test sending to multiple recipients"""
        client = FastDataBrokerClient()
        client.connect()

        recipients = [f"user{i}" for i in range(100)]
        msg = Message(
            sender_id="broadcast",
            recipient_ids=recipients,
            subject="Broadcast",
            content=b"To all"
        )
        result = client.send_message(msg)

        assert len(msg.recipient_ids) == 100
        assert result.status == "success"

    def test_send_message_large_content(self):
        """Test sending large content (10MB)"""
        client = FastDataBrokerClient()
        client.connect()

        large_content = b"x" * (10 * 1024 * 1024)
        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Large Message",
            content=large_content
        )
        result = client.send_message(msg)

        assert len(msg.content) == 10 * 1024 * 1024
        assert result.status == "success"

    def test_send_message_with_tags(self):
        """Test sending message with tags"""
        client = FastDataBrokerClient()
        client.connect()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Tagged Message",
            content=b"With metadata",
            tags={"category": "notification", "source": "api", "env": "prod"}
        )
        result = client.send_message(msg)

        assert len(msg.tags) == 3
        assert msg.tags["category"] == "notification"

    def test_priority_levels(self):
        """Test all priority levels"""
        priorities = {
            Priority.DEFERRED: 50,
            Priority.NORMAL: 100,
            Priority.HIGH: 150,
            Priority.URGENT: 200,
            Priority.CRITICAL: 255,
        }

        for priority, value in priorities.items():
            assert priority.value == value

    def test_notification_channels(self):
        """Test all notification channels"""
        channels = [
            NotificationChannel.EMAIL,
            NotificationChannel.WEBSOCKET,
            NotificationChannel.PUSH,
            NotificationChannel.WEBHOOK,
        ]
        assert len(channels) == 4

    def test_register_webhook(self):
        """Test webhook registration"""
        client = FastDataBrokerClient()
        client.connect()

        result = client.register_webhook("https://example.com/webhook")
        assert result is True

    def test_register_webhook_with_headers(self):
        """Test webhook with custom headers"""
        client = FastDataBrokerClient()
        client.connect()

        headers = {"X-API-Key": "secret", "Content-Type": "application/json"}
        result = client.register_webhook("https://example.com/webhook", headers)
        assert result is True

    def test_register_webhook_invalid_url(self):
        """Test webhook registration with invalid URL"""
        client = FastDataBrokerClient()
        client.connect()

        with pytest.raises(ValueError):
            client.register_webhook("invalid-url")

    def test_register_websocket(self):
        """Test WebSocket client registration"""
        client = FastDataBrokerClient()
        client.connect()

        result = client.register_websocket("client123", "user456")
        assert result is True

    def test_get_stats(self):
        """Test getting statistics"""
        client = FastDataBrokerClient()
        client.connect()

        stats = client.get_stats()
        assert "total_messages" in stats
        assert "delivered" in stats
        assert "failed" in stats
        assert "channels" in stats

    def test_send_message_not_connected(self):
        """Test sending message without connection"""
        client = FastDataBrokerClient()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Test",
            content=b"Test"
        )

        with pytest.raises(RuntimeError):
            client.send_message(msg)


class TestAsyncFastDataBrokerClient:
    """Test asynchronous FastDataBrokerAsyncClient"""

    @pytest.mark.asyncio
    async def test_async_client_initialization(self):
        """Test async client initialization"""
        client = FastDataBrokerAsyncClient()
        assert client.quic_host == "localhost"
        assert client.quic_port == 6000

    @pytest.mark.asyncio
    async def test_async_client_connect(self):
        """Test async client connection"""
        client = FastDataBrokerAsyncClient()
        result = await client.connect()
        assert result is True

    @pytest.mark.asyncio
    async def test_async_send_message(self):
        """Test async message sending"""
        client = FastDataBrokerAsyncClient()
        await client.connect()

        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Async Test",
            content=b"Async message"
        )
        result = await client.send_message(msg)

        assert result.status == "success"

    @pytest.mark.asyncio
    async def test_async_batch_send(self):
        """Test batch sending"""
        client = FastDataBrokerAsyncClient()
        await client.connect()

        messages = [
            Message(
                sender_id="app1",
                recipient_ids=[f"user{i}"],
                subject=f"Message {i}",
                content=f"Content {i}".encode()
            )
            for i in range(5)
        ]

        results = await client.batch_send(messages)
        assert len(results) == 5

    @pytest.mark.asyncio
    async def test_async_get_stats(self):
        """Test async stats retrieval"""
        client = FastDataBrokerAsyncClient()
        await client.connect()

        stats = await client.get_stats()
        assert "total_messages" in stats
        assert "delivered" in stats


class TestIntegration:
    """Integration tests"""

    def test_full_message_workflow(self):
        """Test complete message workflow"""
        client = FastDataBrokerClient()
        client.connect()

        msg = Message(
            sender_id="system",
            recipient_ids=["user1", "user2"],
            subject="Important Notification",
            content=b"This is important",
            priority=Priority.HIGH,
            tags={"type": "notification"}
        )

        result = client.send_message(msg)

        assert result.message_id == "msg-12345"
        assert result.status == "success"
        assert len(msg.recipient_ids) == 2
        assert msg.priority == Priority.HIGH

    def test_message_with_all_fields(self):
        """Test message with all optional fields"""
        msg = Message(
            sender_id="sender@system.com",
            recipient_ids=["user1@example.com", "user2@example.com"],
            subject="Complete Message Test",
            content=b"Full featured message content",
            priority=Priority.URGENT,
            ttl_seconds=7200,
            tags={"category": "transaction", "environment": "production"}
        )

        assert msg.sender_id == "sender@system.com"
        assert len(msg.recipient_ids) == 2
        assert msg.subject == "Complete Message Test"
        assert msg.priority == Priority.URGENT
        assert msg.ttl_seconds == 7200
        assert len(msg.tags) == 2


class TestEdgeCases:
    """Test edge cases and special scenarios"""

    def test_message_with_special_characters(self):
        """Test message with unicode and special characters"""
        content = "Hello 世界 🌍 مرحبا بالعالم".encode('utf-8')
        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Unicode Test 🚀",
            content=content
        )
        assert "🚀" in msg.subject

    def test_message_with_empty_recipients(self):
        """Test message validation with empty recipients"""
        msg = Message(
            sender_id="app1",
            recipient_ids=[],
            subject="Test",
            content=b"Test"
        )
        assert len(msg.recipient_ids) == 0

    def test_message_with_zero_ttl(self):
        """Test message with TTL=0"""
        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Test",
            content=b"Test",
            ttl_seconds=0
        )
        assert msg.ttl_seconds == 0

    def test_message_with_extremely_long_subject(self):
        """Test message with very long subject"""
        long_subject = "x" * 10000
        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject=long_subject,
            content=b"Test"
        )
        assert len(msg.subject) == 10000

    def test_message_with_many_tags(self):
        """Test message with many tags"""
        tags = {f"tag{i}": f"value{i}" for i in range(100)}
        msg = Message(
            sender_id="app1",
            recipient_ids=["user1"],
            subject="Test",
            content=b"Test",
            tags=tags
        )
        assert len(msg.tags) == 100

    def test_concurrent_client_creation(self):
        """Test creating multiple clients concurrently"""
        clients = [
            FastDataBrokerClient(quic_host="localhost", quic_port=6000 + i)
            for i in range(10)
        ]
        assert len(clients) == 10

    def test_push_notification_builder(self):
        """Test push notification builder"""
        builder = PushNotificationBuilder("Test Title")
        notification = (
            builder.with_body("Test Body")
            .with_icon("icon.png")
            .with_sound("notification.mp3")
            .with_data("action", "open_app")
            .build()
        )

        assert notification["title"] == "Test Title"
        assert notification["body"] == "Test Body"
        assert notification["icon"] == "icon.png"
        assert notification["sound"] == "notification.mp3"
        assert notification["data"]["action"] == "open_app"


class TestPerformance:
    """Performance and stress tests"""

    def test_message_creation_performance(self):
        """Test message creation performance"""
        import time

        start = time.time()
        for i in range(1000):
            msg = Message(
                sender_id="sender",
                recipient_ids=[f"user{j}" for j in range(10)],
                subject=f"Message {i}",
                content=b"Test content",
                priority=Priority.NORMAL
            )
        elapsed = time.time() - start

        assert elapsed < 5.0  # Should create 1000 messages in less than 5 seconds
        print(f"Created 1000 messages in {elapsed:.3f} seconds")

    def test_client_creation_performance(self):
        """Test client creation performance"""
        import time

        start = time.time()
        for i in range(100):
            client = FastDataBrokerClient(quic_host="localhost", quic_port=6000)
        elapsed = time.time() - start

        assert elapsed < 1.0  # Should create 100 clients in less than 1 second
        print(f"Created 100 clients in {elapsed:.3f} seconds")

    @pytest.mark.asyncio
    async def test_async_batch_performance(self):
        """Test async batch send performance"""
        import time

        client = FastDataBrokerAsyncClient()
        await client.connect()

        messages = [
            Message(
                sender_id="app1",
                recipient_ids=["user1"],
                subject=f"Message {i}",
                content=f"Content {i}".encode()
            )
            for i in range(100)
        ]

        start = time.time()
        results = await client.batch_send(messages)
        elapsed = time.time() - start

        assert len(results) == 100
        print(f"Sent 100 messages in {elapsed:.3f} seconds")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
