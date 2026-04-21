using System;
using System.Collections.Generic;
using Xunit;
using FastDataBroker.SDK;

namespace FastDataBroker.SDK.Tests
{
    /// <summary>
    /// FastDataBroker C# SDK - Tenant-Specific QUIC Tests
    /// </summary>
    public class TenantQuicClientTest
    {
        [Fact]
        public void TestTenantConfigCreation()
        {
            var config = new TenantQuicClient.TenantConfig(
                "test-tenant-1",
                "super-secret-key",
                "client-001",
                "api_key_xxx"
            )
            {
                Role = TenantQuicClient.TenantRole.Admin,
                RateLimitRPS = 5000,
                MaxConnections = 200
            };

            Assert.Equal("test-tenant-1", config.TenantId);
            Assert.Equal("super-secret-key", config.PskSecret);
            Assert.Equal(TenantQuicClient.TenantRole.Admin, config.Role);
            Assert.Equal(5000, config.RateLimitRPS);
            Assert.Equal(200, config.MaxConnections);
        }

        [Fact]
        public void TestTenantQuicHandshake()
        {
            var config = new TenantQuicClient.TenantConfig(
                "acme-corp",
                "acme-psk-secret",
                "client-acme-01",
                "api_acme_xyz"
            );

            var client = new TenantQuicClient("localhost", 6000, config);

            Assert.Equal(TenantQuicClient.ConnectionState.Idle, client.ConnectionState);

            bool result = client.Connect();
            Assert.True(result);
            Assert.Equal(TenantQuicClient.ConnectionState.Established, client.ConnectionState);
            Assert.True(client.IsConnected());
            Assert.NotNull(client.SessionToken);
            Assert.NotNull(client.ConnectionId);

            var stats = client.GetStats();
            Assert.True(stats.HandshakeDurationMs > 0);

            client.Disconnect();
        }

        [Fact]
        public void TestTenantMessageIsolation()
        {
            var config1 = new TenantQuicClient.TenantConfig(
                "tenant-1",
                "secret-1",
                "client-1",
                "api_1"
            );

            var config2 = new TenantQuicClient.TenantConfig(
                "tenant-2",
                "secret-2",
                "client-2",
                "api_2"
            );

            var client1 = new TenantQuicClient("localhost", 6000, config1);
            var client2 = new TenantQuicClient("localhost", 6000, config2);

            Assert.True(client1.Connect());
            Assert.True(client2.Connect());

            var msg1 = new Dictionary<string, object> { { "data", "tenant1" } };
            var msg2 = new Dictionary<string, object> { { "data", "tenant2" } };

            var result1 = client1.SendMessage(msg1);
            var result2 = client2.SendMessage(msg2);

            Assert.Equal("tenant-1", result1.TenantId);
            Assert.Equal("tenant-2", result2.TenantId);
            Assert.NotEqual(result1.MessageId, result2.MessageId);

            Assert.NotEqual(client1.SessionToken, client2.SessionToken);
            Assert.NotEqual(client1.ConnectionId, client2.ConnectionId);

            client1.Disconnect();
            client2.Disconnect();
        }

        [Fact]
        public void TestConcurrentTenantConnections()
        {
            int numTenants = 5;
            var configs = new TenantQuicClient.TenantConfig[numTenants];
            var clients = new TenantQuicClient[numTenants];

            for (int i = 0; i < numTenants; i++)
            {
                configs[i] = new TenantQuicClient.TenantConfig(
                    $"tenant-{i}",
                    $"secret-{i}",
                    $"client-{i}",
                    $"api_{i}"
                );
                clients[i] = new TenantQuicClient("localhost", 6000, configs[i]);
            }

            // Connect all tenants
            foreach (var client in clients)
            {
                Assert.True(client.Connect());
            }

            // Send messages from each tenant
            int totalSent = 0;
            for (int i = 0; i < clients.Length; i++)
            {
                var msg = new Dictionary<string, object> { { "index", i } };

                var result = clients[i].SendMessage(msg);
                Assert.Equal("success", result.Status);
                Assert.Equal($"tenant-{i}", result.TenantId);
                totalSent++;
            }

            Assert.Equal(numTenants, totalSent);

            // Disconnect all clients
            foreach (var client in clients)
            {
                client.Disconnect();
            }
        }

        [Fact]
        public void TestPSKValidation()
        {
            var config = new TenantQuicClient.TenantConfig(
                "psk-test-tenant",
                "specific-psk-secret",
                "psk-client-01",
                "psk_api_key"
            );

            var client = new TenantQuicClient("localhost", 6000, config);

            bool result = client.Connect();
            Assert.True(result);

            var stats = client.GetStats();
            Assert.True(stats.IsConnected);

            client.Disconnect();
        }

        [Fact]
        public void TestHandshakeMetrics()
        {
            var config = new TenantQuicClient.TenantConfig(
                "metrics-tenant",
                "metrics-secret",
                "metrics-client",
                "metrics_api"
            );

            var client = new TenantQuicClient("localhost", 6000, config);
            client.Connect();

            var stats = client.GetStats();

            Assert.True(stats.IsConnected);
            Assert.True(stats.HandshakeDurationMs > 0);
            Assert.True(stats.UptimeSeconds >= 0);

            client.Disconnect();
        }

        [Fact]
        public void TestConnectionStateTransitions()
        {
            var config = new TenantQuicClient.TenantConfig(
                "state-test",
                "state-secret",
                "state-client",
                "state_api"
            );

            var client = new TenantQuicClient("localhost", 6000, config);

            Assert.Equal(TenantQuicClient.ConnectionState.Idle, client.ConnectionState);

            client.Connect();
            Assert.Equal(TenantQuicClient.ConnectionState.Established, client.ConnectionState);

            Assert.True(client.IsConnected());

            client.Disconnect();
            Assert.Equal(TenantQuicClient.ConnectionState.Closed, client.ConnectionState);
            Assert.False(client.IsConnected());
        }

        [Fact]
        public void TestRateLimitingConfig()
        {
            var config = new TenantQuicClient.TenantConfig(
                "rate-limit-tenant",
                "rate-secret",
                "rate-client",
                "rate_api"
            )
            {
                RateLimitRPS = 2000,
                MaxConnections = 50
            };

            var client = new TenantQuicClient("localhost", 6000, config);

            Assert.Equal(2000, config.RateLimitRPS);
            Assert.Equal(50, config.MaxConnections);

            client.Connect();

            var stats = client.GetStats();
            Assert.True(stats.IsConnected);

            client.Disconnect();
        }

        [Fact]
        public void TestCustomHeaders()
        {
            var config = new TenantQuicClient.TenantConfig(
                "custom-header-tenant",
                "custom-secret",
                "custom-client",
                "custom_api"
            );

            config.CustomHeaders.Add("X-Tenant-Region", "us-west");
            config.CustomHeaders.Add("X-Custom-Header", "custom-value");

            Assert.Equal("us-west", config.CustomHeaders["X-Tenant-Region"]);
            Assert.Equal("custom-value", config.CustomHeaders["X-Custom-Header"]);
        }

        [Fact]
        public void TestMessageHandlers()
        {
            var config = new TenantQuicClient.TenantConfig(
                "handler-test",
                "handler-secret",
                "handler-client",
                "handler_api"
            );

            var client = new TenantQuicClient("localhost", 6000, config);

            Action<Dictionary<string, object>> handler = (msg) => Console.WriteLine("Message: " + msg);

            client.OnMessage("test.topic", handler);
            client.OffMessage("test.topic");

            client.Connect();
            client.Disconnect();
        }

        [Fact]
        public void TestErrorHandling()
        {
            var config = new TenantQuicClient.TenantConfig(
                "error-test",
                "error-secret",
                "error-client",
                "error_api"
            );

            var client = new TenantQuicClient("localhost", 6000, config);

            var message = new Dictionary<string, object> { { "topic", "test" } };

            var exception = Assert.Throws<InvalidOperationException>(() =>
            {
                client.SendMessage(message);
            });

            Assert.Contains("not established", exception.Message);
        }

        [Fact]
        public void TestMultipleTenantIsolation()
        {
            var tenantData = new (string, string)[]
            {
                ("acme", "acme-secret"),
                ("globex", "globex-secret"),
                ("initech", "initech-secret")
            };

            var clients = new TenantQuicClient[tenantData.Length];

            for (int i = 0; i < tenantData.Length; i++)
            {
                var config = new TenantQuicClient.TenantConfig(
                    tenantData[i].Item1,
                    tenantData[i].Item2,
                    $"client-{tenantData[i].Item1}",
                    $"api_{tenantData[i].Item1}"
                );
                clients[i] = new TenantQuicClient("localhost", 6000, config);
            }

            // Connect all
            foreach (var client in clients)
            {
                Assert.True(client.Connect());
            }

            // Verify isolation
            var sessionTokens = new HashSet<string>();
            var connectionIds = new HashSet<string>();

            foreach (var client in clients)
            {
                sessionTokens.Add(client.SessionToken);
                connectionIds.Add(client.ConnectionId);
            }

            Assert.Equal(clients.Length, sessionTokens.Count);
            Assert.Equal(clients.Length, connectionIds.Count);

            // Disconnect all
            foreach (var client in clients)
            {
                client.Disconnect();
            }
        }
    }
}
