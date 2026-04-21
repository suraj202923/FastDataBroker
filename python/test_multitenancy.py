"""
Comprehensive tests for FastDataBroker Python SDK Multi-Tenant Support
"""

import unittest
import json
import tempfile
import os
from fastdatabroker_sdk import (
    TenantConfig,
    AppSettings,
    Client,
    Message,
    Priority,
    NotificationChannel
)


class TenantConfigTests(unittest.TestCase):
    """Tests for TenantConfig class"""

    def test_tenant_creation(self):
        """Test creating a valid tenant configuration"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME Corporation",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100,
            max_message_size=1048576,
            retention_days=30,
            enabled=True
        )

        self.assertEqual(tenant.tenant_id, "acme-corp")
        self.assertEqual(tenant.api_key_prefix, "acme_")
        self.assertEqual(tenant.rate_limit_rps, 1000)
        self.assertEqual(tenant.max_connections, 100)
        self.assertTrue(tenant.enabled)

    def test_tenant_validation_success(self):
        """Test tenant validation with valid config"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        # Should not raise
        tenant.validate()

    def test_tenant_validation_empty_id(self):
        """Test tenant validation with empty tenant ID"""
        tenant = TenantConfig(
            tenant_id="",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        with self.assertRaises(ValueError):
            tenant.validate()

    def test_tenant_validation_bad_prefix(self):
        """Test tenant validation with bad API key prefix"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme",  # Missing underscore
            rate_limit_rps=1000,
            max_connections=100
        )

        with self.assertRaises(ValueError):
            tenant.validate()

    def test_tenant_validation_zero_rate_limit(self):
        """Test tenant validation with zero rate limit"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=0,
            max_connections=100
        )

        with self.assertRaises(ValueError):
            tenant.validate()

    def test_tenant_validation_zero_connections(self):
        """Test tenant validation with zero max connections"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=0
        )

        with self.assertRaises(ValueError):
            tenant.validate()


class AppSettingsTests(unittest.TestCase):
    """Tests for AppSettings class"""

    def setUp(self):
        """Set up test fixtures"""
        self.settings = AppSettings()

    def test_app_settings_creation(self):
        """Test creating AppSettings"""
        self.assertEqual(len(self.settings.tenants), 0)

    def test_add_tenant(self):
        """Test adding a tenant to AppSettings"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        self.settings.add_tenant(tenant)
        self.assertEqual(len(self.settings.tenants), 1)

    def test_get_tenant_by_id(self):
        """Test retrieving tenant by ID"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        self.settings.add_tenant(tenant)
        retrieved = self.settings.get_tenant("acme-corp")

        self.assertIsNotNone(retrieved)
        self.assertEqual(retrieved.tenant_id, "acme-corp")

    def test_get_nonexistent_tenant(self):
        """Test retrieving nonexistent tenant returns None"""
        retrieved = self.settings.get_tenant("nonexistent")
        self.assertIsNone(retrieved)

    def test_get_tenant_by_api_key(self):
        """Test retrieving tenant by API key prefix"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        self.settings.add_tenant(tenant)
        retrieved = self.settings.get_tenant_by_api_key("acme_550e8400e29b41d4a716446655440000")

        self.assertIsNotNone(retrieved)
        self.assertEqual(retrieved.tenant_id, "acme-corp")

    def test_multiple_tenants_isolation(self):
        """Test isolation of multiple tenants"""
        tenant1 = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        tenant2 = TenantConfig(
            tenant_id="startup-xyz",
            tenant_name="Startup",
            api_key_prefix="xyz_",
            rate_limit_rps=100,
            max_connections=10
        )

        self.settings.add_tenant(tenant1)
        self.settings.add_tenant(tenant2)

        t1 = self.settings.get_tenant("acme-corp")
        t2 = self.settings.get_tenant("startup-xyz")

        self.assertEqual(t1.rate_limit_rps, 1000)
        self.assertEqual(t2.rate_limit_rps, 100)
        self.assertEqual(t1.max_connections, 100)
        self.assertEqual(t2.max_connections, 10)


class ClientTests(unittest.TestCase):
    """Tests for Client class"""

    def test_client_creation_with_tenant(self):
        """Test creating client with tenant ID and API key"""
        client = Client(
            tenant_id="acme-corp",
            api_key="acme_key",
            host="localhost",
            port=6379
        )

        self.assertEqual(client.tenant_id, "acme-corp")
        self.assertFalse(client.is_connected())

    def test_client_creation_empty_tenant_id(self):
        """Test client creation fails with empty tenant ID"""
        with self.assertRaises(ValueError):
            Client(
                tenant_id="",
                api_key="key",
                host="localhost",
                port=6379
            )

    def test_client_creation_empty_api_key(self):
        """Test client creation fails with empty API key"""
        with self.assertRaises(ValueError):
            Client(
                tenant_id="acme-corp",
                api_key="",
                host="localhost",
                port=6379
            )

    def test_client_creation_from_settings_success(self):
        """Test creating client from AppSettings"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        settings = AppSettings()
        settings.add_tenant(tenant)

        client = Client.from_settings(
            settings=settings,
            tenant_id="acme-corp",
            api_key="acme_valid_key"
        )

        self.assertEqual(client.tenant_id, "acme-corp")

    def test_client_creation_from_settings_api_key_mismatch(self):
        """Test client creation fails with API key prefix mismatch"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        settings = AppSettings()
        settings.add_tenant(tenant)

        with self.assertRaises(ValueError):
            Client.from_settings(
                settings=settings,
                tenant_id="acme-corp",
                api_key="xyz_invalid_key"
            )

    def test_client_creation_from_settings_tenant_not_found(self):
        """Test client creation fails when tenant not found"""
        settings = AppSettings()

        with self.assertRaises(ValueError):
            Client.from_settings(
                settings=settings,
                tenant_id="nonexistent",
                api_key="any_key"
            )

    def test_api_key_generation(self):
        """Test API key generation"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100
        )

        settings = AppSettings()
        settings.add_tenant(tenant)

        client = Client.from_settings(
            settings=settings,
            tenant_id="acme-corp",
            api_key="acme_existing_key"
        )

        new_key = client.generate_api_key("client-1")

        self.assertTrue(new_key.startswith("acme_"))
        self.assertGreater(len(new_key), 6)

    def test_get_tenant_config(self):
        """Test getting tenant config from client"""
        tenant = TenantConfig(
            tenant_id="acme-corp",
            tenant_name="ACME",
            api_key_prefix="acme_",
            rate_limit_rps=1000,
            max_connections=100,
            retention_days=30
        )

        settings = AppSettings()
        settings.add_tenant(tenant)

        client = Client.from_settings(
            settings=settings,
            tenant_id="acme-corp",
            api_key="acme_key"
        )

        config = client.get_tenant_config()

        self.assertIsNotNone(config)
        self.assertEqual(config.tenant_id, "acme-corp")
        self.assertEqual(config.rate_limit_rps, 1000)
        self.assertEqual(config.retention_days, 30)


class MessageTests(unittest.TestCase):
    """Tests for Message class"""

    def test_message_creation(self):
        """Test creating a message"""
        message = Message(
            tenant_id="acme-corp",
            sender_id="user1",
            recipient_ids=["user2", "user3"],
            subject="Test Subject",
            content=b"Test content"
        )

        self.assertEqual(message.tenant_id, "acme-corp")
        self.assertEqual(message.sender_id, "user1")
        self.assertEqual(len(message.recipient_ids), 2)
        self.assertEqual(message.subject, "Test Subject")

    def test_message_with_priority(self):
        """Test setting message priority"""
        message = Message(
            tenant_id="acme-corp",
            sender_id="user1",
            recipient_ids=[],
            subject="Subject",
            content=b""
        )

        message.priority = Priority.HIGH

        self.assertEqual(message.priority, Priority.HIGH)

    def test_message_with_tags(self):
        """Test adding tags to message"""
        message = Message(
            tenant_id="acme-corp",
            sender_id="user1",
            recipient_ids=[],
            subject="Subject",
            content=b""
        )

        message.tags["category"] = "notification"
        message.tags["type"] = "welcome"

        self.assertEqual(message.tags["category"], "notification")
        self.assertEqual(message.tags["type"], "welcome")


class PriorityTests(unittest.TestCase):
    """Tests for Priority enum"""

    def test_priority_values(self):
        """Test priority enum values"""
        self.assertEqual(Priority.DEFERRED, 50)
        self.assertEqual(Priority.NORMAL, 100)
        self.assertEqual(Priority.HIGH, 150)
        self.assertEqual(Priority.URGENT, 200)
        self.assertEqual(Priority.CRITICAL, 255)


if __name__ == '__main__':
    unittest.main()
