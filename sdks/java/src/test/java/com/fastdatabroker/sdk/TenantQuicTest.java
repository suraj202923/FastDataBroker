package com.fastdatabroker.sdk;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import java.util.*;

import static org.junit.jupiter.api.Assertions.*;

/**
 * FastDataBroker Java SDK - Tenant-Specific QUIC Tests
 */
@DisplayName("Tenant-Specific QUIC Tests")
public class TenantQuicTest {

    @Test
    @DisplayName("should create a tenant configuration")
    public void testTenantConfigCreation() {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "test-tenant-1",
                "super-secret-key",
                "client-001",
                "api_key_xxx"
        );
        config.role = TenantQuicClient.TenantRole.ADMIN;
        config.rateLimitRPS = 5000;
        config.maxConnections = 200;

        assertEquals("test-tenant-1", config.tenantId);
        assertEquals("super-secret-key", config.pskSecret);
        assertEquals(TenantQuicClient.TenantRole.ADMIN, config.role);
        assertEquals(5000, config.rateLimitRPS);
        assertEquals(200, config.maxConnections);
    }

    @Test
    @DisplayName("should perform tenant-specific QUIC handshake")
    public void testTenantQuicHandshake() {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "acme-corp",
                "acme-psk-secret",
                "client-acme-01",
                "api_acme_xyz"
        );

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);

        assertEquals(TenantQuicClient.ConnectionState.IDLE, client.getConnectionState());
        assertFalse(client.isAuthenticated());

        boolean result = client.connect();
        assertTrue(result);
        assertEquals(TenantQuicClient.ConnectionState.ESTABLISHED, client.getConnectionState());
        assertTrue(client.isConnected());
        assertNotNull(client.getSessionToken());
        assertNotNull(client.getConnectionId());

        TenantQuicClient.ConnectionStats stats = client.getStats();
        assertTrue(stats.handshakeDurationMs > 0);

        client.disconnect();
    }

    @Test
    @DisplayName("should isolate messages between tenants")
    public void testTenantMessageIsolation() throws Exception {
        TenantQuicClient.TenantConfig config1 = new TenantQuicClient.TenantConfig(
                "tenant-1",
                "secret-1",
                "client-1",
                "api_1"
        );

        TenantQuicClient.TenantConfig config2 = new TenantQuicClient.TenantConfig(
                "tenant-2",
                "secret-2",
                "client-2",
                "api_2"
        );

        TenantQuicClient client1 = new TenantQuicClient("localhost", 6000, config1);
        TenantQuicClient client2 = new TenantQuicClient("localhost", 6000, config2);

        assertTrue(client1.connect());
        assertTrue(client2.connect());

        Map<String, Object> msg1 = new HashMap<>();
        msg1.put("topic", "test.topic");
        msg1.put("data", "tenant1");

        Map<String, Object> msg2 = new HashMap<>();
        msg2.put("topic", "test.topic");
        msg2.put("data", "tenant2");

        TenantQuicClient.DeliveryResult result1 = client1.sendMessage(msg1);
        TenantQuicClient.DeliveryResult result2 = client2.sendMessage(msg2);

        assertEquals("tenant-1", result1.tenantId);
        assertEquals("tenant-2", result2.tenantId);
        assertNotEquals(result1.messageId, result2.messageId);

        assertNotEquals(client1.getSessionToken(), client2.getSessionToken());
        assertNotEquals(client1.getConnectionId(), client2.getConnectionId());

        client1.disconnect();
        client2.disconnect();
    }

    @Test
    @DisplayName("should handle concurrent connections from multiple tenants")
    public void testConcurrentTenantConnections() throws Exception {
        int numTenants = 5;
        TenantQuicClient.TenantConfig[] configs = new TenantQuicClient.TenantConfig[numTenants];
        TenantQuicClient[] clients = new TenantQuicClient[numTenants];

        for (int i = 0; i < numTenants; i++) {
            configs[i] = new TenantQuicClient.TenantConfig(
                    "tenant-" + i,
                    "secret-" + i,
                    "client-" + i,
                    "api_" + i
            );
            clients[i] = new TenantQuicClient("localhost", 6000, configs[i]);
        }

        // Connect all tenants
        for (TenantQuicClient client : clients) {
            assertTrue(client.connect());
        }

        // Send messages from each tenant
        int totalSent = 0;
        for (int i = 0; i < clients.length; i++) {
            Map<String, Object> msg = new HashMap<>();
            msg.put("topic", "test.multi");
            msg.put("index", i);

            TenantQuicClient.DeliveryResult result = clients[i].sendMessage(msg);
            assertEquals("success", result.status);
            assertEquals("tenant-" + i, result.tenantId);
            totalSent++;
        }

        assertEquals(numTenants, totalSent);

        // Disconnect all clients
        for (TenantQuicClient client : clients) {
            client.disconnect();
        }
    }

    @Test
    @DisplayName("should validate PSK-based tenant authentication")
    public void testPSKValidation() throws Exception {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "psk-test-tenant",
                "specific-psk-secret",
                "psk-client-01",
                "psk_api_key"
        );

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);

        boolean result = client.connect();
        assertTrue(result);

        TenantQuicClient.ConnectionStats stats = client.getStats();
        assertTrue(stats.isConnected);

        client.disconnect();
    }

    @Test
    @DisplayName("should measure handshake performance metrics")
    public void testHandshakeMetrics() throws Exception {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "metrics-tenant",
                "metrics-secret",
                "metrics-client",
                "metrics_api"
        );

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);
        client.connect();

        TenantQuicClient.ConnectionStats stats = client.getStats();

        assertTrue(stats.isConnected);
        assertTrue(stats.handshakeDurationMs > 0);
        assertTrue(stats.uptimeSeconds >= 0);

        client.disconnect();
    }

    @Test
    @DisplayName("should transition connection states correctly")
    public void testConnectionStateTransitions() throws Exception {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "state-test",
                "state-secret",
                "state-client",
                "state_api"
        );

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);

        assertEquals(TenantQuicClient.ConnectionState.IDLE, client.getConnectionState());

        client.connect();
        assertEquals(TenantQuicClient.ConnectionState.ESTABLISHED, client.getConnectionState());

        assertTrue(client.isConnected());

        client.disconnect();
        assertEquals(TenantQuicClient.ConnectionState.CLOSED, client.getConnectionState());
        assertFalse(client.isConnected());
    }

    @Test
    @DisplayName("should support tenant-specific rate limiting configuration")
    public void testRateLimitingConfig() throws Exception {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "rate-limit-tenant",
                "rate-secret",
                "rate-client",
                "rate_api"
        );
        config.rateLimitRPS = 2000;
        config.maxConnections = 50;

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);

        assertEquals(2000, config.rateLimitRPS);
        assertEquals(50, config.maxConnections);

        client.connect();

        TenantQuicClient.ConnectionStats stats = client.getStats();
        assertTrue(stats.isConnected);

        client.disconnect();
    }

    @Test
    @DisplayName("should support tenant custom headers in configuration")
    public void testCustomHeaders() {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "custom-header-tenant",
                "custom-secret",
                "custom-client",
                "custom_api"
        );

        config.customHeaders.put("X-Tenant-Region", "us-west");
        config.customHeaders.put("X-Custom-Header", "custom-value");

        assertEquals("us-west", config.customHeaders.get("X-Tenant-Region"));
        assertEquals("custom-value", config.customHeaders.get("X-Custom-Header"));
    }

    @Test
    @DisplayName("should register and unregister message handlers")
    public void testMessageHandlers() {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "handler-test",
                "handler-secret",
                "handler-client",
                "handler_api"
        );

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);

        Object handler = (Object) message -> System.out.println("Message received: " + message);

        client.onMessage("test.topic", handler);
        client.offMessage("test.topic");

        client.connect();
        client.disconnect();
    }

    @Test
    @DisplayName("should handle errors gracefully")
    public void testErrorHandling() {
        TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                "error-test",
                "error-secret",
                "error-client",
                "error_api"
        );

        TenantQuicClient client = new TenantQuicClient("localhost", 6000, config);

        Map<String, Object> message = new HashMap<>();
        message.put("topic", "test");

        Exception thrown = assertThrows(Exception.class, () -> {
            client.sendMessage(message);
        });

        assertTrue(thrown.getMessage().contains("not established"));
    }

    @Test
    @DisplayName("should maintain complete isolation between multiple tenants")
    public void testMultipleTenantIsolation() throws Exception {
        String[][] tenantData = {
                {"acme", "acme-secret"},
                {"globex", "globex-secret"},
                {"initech", "initech-secret"}
        };

        TenantQuicClient[] clients = new TenantQuicClient[tenantData.length];

        for (int i = 0; i < tenantData.length; i++) {
            TenantQuicClient.TenantConfig config = new TenantQuicClient.TenantConfig(
                    tenantData[i][0],
                    tenantData[i][1],
                    "client-" + tenantData[i][0],
                    "api_" + tenantData[i][0]
            );
            clients[i] = new TenantQuicClient("localhost", 6000, config);
        }

        // Connect all
        for (TenantQuicClient client : clients) {
            assertTrue(client.connect());
        }

        // Verify isolation
        Set<String> sessionTokens = new HashSet<>();
        Set<String> connectionIds = new HashSet<>();

        for (TenantQuicClient client : clients) {
            sessionTokens.add(client.getSessionToken());
            connectionIds.add(client.getConnectionId());
        }

        assertEquals(clients.length, sessionTokens.size());
        assertEquals(clients.length, connectionIds.size());

        // Disconnect all
        for (TenantQuicClient client : clients) {
            client.disconnect();
        }
    }
}
