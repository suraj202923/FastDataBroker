package com.fastdatabroker.sdk;

import org.junit.Before;
import org.junit.Test;
import static org.junit.Assert.*;

import java.util.*;

public class MultiTenantConfigurationTest {

    private FastDataBrokerSDK.AppSettings settings;
    private FastDataBrokerSDK.TenantConfig tenant;

    @Before
    public void setUp() {
        settings = new FastDataBrokerSDK.AppSettings();
        tenant = new FastDataBrokerSDK.TenantConfig();
    }

    // ============== Tenant Creation Tests ==============

    @Test
    public void testTenantConfigCreation() {
        tenant.tenantId = "acme-corp";
        tenant.tenantName = "ACME Corporation";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;
        tenant.maxMessageSize = 1048576;
        tenant.retentionDays = 30;
        tenant.enabled = true;

        assertEquals("acme-corp", tenant.tenantId);
        assertEquals("acme_", tenant.apiKeyPrefix);
        assertEquals(1000L, tenant.rateLimitRps);
        assertEquals(100L, tenant.maxConnections);
        assertTrue(tenant.enabled);
    }

    // ============== Tenant Validation Tests ==============

    @Test
    public void testTenantValidation_Success() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        // Should not throw
        tenant.validate();
    }

    @Test(expected = IllegalArgumentException.class)
    public void testTenantValidation_EmptyId() {
        tenant.tenantId = "";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        tenant.validate();
    }

    @Test(expected = IllegalArgumentException.class)
    public void testTenantValidation_BadPrefix() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme";  // Missing underscore
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        tenant.validate();
    }

    @Test(expected = IllegalArgumentException.class)
    public void testTenantValidation_ZeroRateLimit() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 0;
        tenant.maxConnections = 100;

        tenant.validate();
    }

    @Test(expected = IllegalArgumentException.class)
    public void testTenantValidation_ZeroConnections() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 0;

        tenant.validate();
    }

    // ============== AppSettings Tests ==============

    @Test
    public void testAppSettingsCreation() {
        assertNotNull(settings.tenants);
        assertEquals(0, settings.tenants.size());
    }

    @Test
    public void testAppSettingsAddTenant() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);

        assertEquals(1, settings.tenants.size());
    }

    @Test
    public void testAppSettingsGetTenant() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);

        FastDataBrokerSDK.TenantConfig retrieved = settings.getTenant("acme-corp");
        assertNotNull(retrieved);
        assertEquals("acme-corp", retrieved.tenantId);
    }

    @Test
    public void testAppSettingsGetTenant_NotFound() {
        assertNull(settings.getTenant("nonexistent"));
    }

    @Test
    public void testAppSettingsGetTenantByApiKey() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);

        FastDataBrokerSDK.TenantConfig retrieved = 
            settings.getTenantByApiKey("acme_550e8400e29b41d4a716446655440000");
        assertNotNull(retrieved);
        assertEquals("acme-corp", retrieved.tenantId);
    }

    // ============== Multi-Tenant Isolation ==============

    @Test
    public void testMultipleTenantIsolation() {
        FastDataBrokerSDK.TenantConfig tenant1 = new FastDataBrokerSDK.TenantConfig();
        tenant1.tenantId = "acme-corp";
        tenant1.apiKeyPrefix = "acme_";
        tenant1.rateLimitRps = 1000;
        tenant1.maxConnections = 100;

        FastDataBrokerSDK.TenantConfig tenant2 = new FastDataBrokerSDK.TenantConfig();
        tenant2.tenantId = "startup-xyz";
        tenant2.apiKeyPrefix = "xyz_";
        tenant2.rateLimitRps = 100;
        tenant2.maxConnections = 10;

        settings.tenants.add(tenant1);
        settings.tenants.add(tenant2);

        FastDataBrokerSDK.TenantConfig t1 = settings.getTenant("acme-corp");
        FastDataBrokerSDK.TenantConfig t2 = settings.getTenant("startup-xyz");

        assertEquals(1000L, t1.rateLimitRps);
        assertEquals(100L, t2.rateLimitRps);
        assertEquals(100L, t1.maxConnections);
        assertEquals(10L, t2.maxConnections);
    }

    // ============== Client Creation Tests ==============

    @Test(expected = IllegalArgumentException.class)
    public void testClientCreation_EmptyTenantId() {
        new FastDataBrokerSDK.Client("", "key", "localhost", 6379);
    }

    @Test(expected = IllegalArgumentException.class)
    public void testClientCreation_EmptyAPIKey() {
        new FastDataBrokerSDK.Client("acme-corp", "", "localhost", 6379);
    }

    @Test
    public void testClientCreation_WithTenant() {
        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            "acme-corp",
            "acme_key",
            "localhost",
            6379
        );

        assertNotNull(client);
    }

    @Test
    public void testClientCreation_FromSettings_Success() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);
        settings.server.bindAddress = "localhost";
        settings.server.port = 6379;

        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            settings,
            "acme-corp",
            "acme_valid_key"
        );

        assertNotNull(client);
    }

    @Test(expected = IllegalArgumentException.class)
    public void testClientCreation_FromSettings_APIKeyMismatch() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);

        new FastDataBrokerSDK.Client(settings, "acme-corp", "xyz_invalid_key");
    }

    @Test(expected = IllegalArgumentException.class)
    public void testClientCreation_FromSettings_TenantNotFound() {
        new FastDataBrokerSDK.Client(settings, "nonexistent", "any_key");
    }

    // ============== Message Isolation Tests ==============

    @Test
    public void testMessageCreation() {
        FastDataBrokerSDK.Message message = new FastDataBrokerSDK.Message(
            "acme-corp",
            "user1",
            Arrays.asList("user2", "user3"),
            "Subject",
            "Content".getBytes()
        );

        assertEquals("acme-corp", message.tenantId);
        assertEquals("user1", message.senderId);
        assertEquals(2, message.recipientIds.size());
        assertEquals("Subject", message.subject);
    }

    @Test
    public void testMessageWithPriority() {
        FastDataBrokerSDK.Message message = new FastDataBrokerSDK.Message(
            "acme-corp",
            "user1",
            new ArrayList<>(),
            "Subject",
            new byte[0]
        );

        message.priority = FastDataBrokerSDK.Priority.HIGH;
        assertEquals(FastDataBrokerSDK.Priority.HIGH, message.priority);
    }

    @Test
    public void testMessageTags() {
        FastDataBrokerSDK.Message message = new FastDataBrokerSDK.Message(
            "acme-corp",
            "user1",
            new ArrayList<>(),
            "Subject",
            new byte[0]
        );

        message.tags.put("category", "notification");
        message.tags.put("type", "welcome");

        assertEquals("notification", message.tags.get("category"));
        assertEquals("welcome", message.tags.get("type"));
    }

    // ============== API Key Generation ==============

    @Test
    public void testAPIKeyGeneration() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);
        settings.server.bindAddress = "localhost";
        settings.server.port = 6379;

        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            settings,
            "acme-corp",
            "acme_existing_key"
        );

        String newKey = client.generateApiKey("client-1");

        assertNotNull(newKey);
        assertTrue(newKey.startsWith("acme_"));
        assertTrue(newKey.length() > 6);
    }

    // ============== WebSocket Client Management ==============

    @Test
    public void testWebSocketClientRegistration() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);
        settings.server.bindAddress = "localhost";
        settings.server.port = 6379;

        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            settings,
            "acme-corp",
            "acme_key"
        );

        boolean registered = client.registerWebSocketClient("ws-001", "user-001");
        assertTrue(registered);
    }

    @Test
    public void testWebSocketClientUnregistration() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;

        settings.tenants.add(tenant);
        settings.server.bindAddress = "localhost";
        settings.server.port = 6379;

        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            settings,
            "acme-corp",
            "acme_key"
        );

        client.registerWebSocketClient("ws-001", "user-001");
        boolean unregistered = client.unregisterWebSocketClient("ws-001");

        assertTrue(unregistered);
    }

    // ============== Tenant Config Access ==============

    @Test
    public void testGetTenantConfig() {
        tenant.tenantId = "acme-corp";
        tenant.apiKeyPrefix = "acme_";
        tenant.rateLimitRps = 1000;
        tenant.maxConnections = 100;
        tenant.retentionDays = 30;

        settings.tenants.add(tenant);
        settings.server.bindAddress = "localhost";
        settings.server.port = 6379;

        FastDataBrokerSDK.Client client = new FastDataBrokerSDK.Client(
            settings,
            "acme-corp",
            "acme_key"
        );

        FastDataBrokerSDK.TenantConfig config = client.getTenantConfig();

        assertNotNull(config);
        assertEquals("acme-corp", config.tenantId);
        assertEquals(1000L, config.rateLimitRps);
        assertEquals(30L, config.retentionDays);
    }

    // ============== Priority Enum Tests ==============

    @Test
    public void testPriorityEnumValues() {
        assertEquals(50, FastDataBrokerSDK.Priority.DEFERRED.ordinal() + 1);
        assertEquals(100, FastDataBrokerSDK.Priority.NORMAL.ordinal() + 1);
        assertEquals(150, FastDataBrokerSDK.Priority.HIGH.ordinal() + 1);
        assertEquals(200, FastDataBrokerSDK.Priority.URGENT.ordinal() + 1);
        assertEquals(255, FastDataBrokerSDK.Priority.CRITICAL.ordinal() + 1);
    }

    // ============== Webhook Configuration ==============

    @Test
    public void testWebhookConfigCreation() {
        FastDataBrokerSDK.WebhookConfig config = new FastDataBrokerSDK.WebhookConfig();
        config.url = "https://example.com/webhook";
        config.headers.put("Authorization", "Bearer token");
        config.retries = 3;
        config.timeoutMs = 30000;

        assertEquals("https://example.com/webhook", config.url);
        assertEquals("Bearer token", config.headers.get("Authorization"));
        assertEquals(3, config.retries);
        assertEquals(30000, config.timeoutMs);
    }
}
