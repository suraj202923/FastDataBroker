using Microsoft.VisualStudio.TestTools.UnitTesting;
using FastDataBroker;
using System;
using System.Collections.Generic;
using System.Linq;

namespace FastDataBroker.Tests
{
    [TestClass]
    public class MultiTenantConfigurationTests
    {
        [TestMethod]
        public void TestTenantConfigCreation()
        {
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                TenantName = "ACME Corporation",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100,
                MaxMessageSize = 1048576,
                RetentionDays = 30,
                Enabled = true
            };

            Assert.AreEqual("acme-corp", tenant.TenantId);
            Assert.AreEqual("acme_", tenant.ApiKeyPrefix);
            Assert.AreEqual(1000u, tenant.RateLimitRps);
            Assert.AreEqual(100u, tenant.MaxConnections);
            Assert.IsTrue(tenant.Enabled);
        }

        [TestMethod]
        public void TestTenantValidation_Success()
        {
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            // Should not throw
            tenant.Validate();
        }

        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestTenantValidation_EmptyId()
        {
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            tenant.Validate();
        }

        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestTenantValidation_BadPrefix()
        {
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme",  // Missing underscore
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            tenant.Validate();
        }

        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestTenantValidation_ZeroRateLimit()
        {
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 0,
                MaxConnections = 100
            };

            tenant.Validate();
        }

        [TestMethod]
        public void TestAppSettingsCreation()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            Assert.IsNotNull(settings.Tenants);
            Assert.AreEqual(0, settings.Tenants.Count);
        }

        [TestMethod]
        public void TestAppSettingsGetTenant()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            settings.Tenants.Add(tenant);

            var retrieved = settings.GetTenant("acme-corp");
            Assert.IsNotNull(retrieved);
            Assert.AreEqual("acme-corp", retrieved.TenantId);
        }

        [TestMethod]
        public void TestAppSettingsGetTenantByApiKey()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            settings.Tenants.Add(tenant);

            var retrieved = settings.GetTenantByApiKey("acme_550e8400e29b41d4a716446655440000");
            Assert.IsNotNull(retrieved);
            Assert.AreEqual("acme-corp", retrieved.TenantId);
        }

        [TestMethod]
        public void TestMultipleTenantIsolation()
        {
            var settings = new FastDataBrokerSDK.AppSettings();

            var tenant1 = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            var tenant2 = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "startup-xyz",
                ApiKeyPrefix = "xyz_",
                RateLimitRps = 100,
                MaxConnections = 10
            };

            settings.Tenants.Add(tenant1);
            settings.Tenants.Add(tenant2);

            var t1 = settings.GetTenant("acme-corp");
            var t2 = settings.GetTenant("startup-xyz");

            Assert.AreEqual(1000u, t1.RateLimitRps);
            Assert.AreEqual(100u, t2.RateLimitRps);
            Assert.AreEqual(100u, t1.MaxConnections);
            Assert.AreEqual(10u, t2.MaxConnections);
        }
    }

    [TestClass]
    public class MultiTenantClientTests
    {
        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestClientCreation_MissingTenantId()
        {
            var client = new FastDataBrokerSDK.Client(
                tenantId: "",
                apiKey: "acme_key",
                host: "localhost",
                port: 6379
            );
        }

        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestClientCreation_MissingApiKey()
        {
            var client = new FastDataBrokerSDK.Client(
                tenantId: "acme-corp",
                apiKey: "",
                host: "localhost",
                port: 6379
            );
        }

        [TestMethod]
        public void TestClientCreation_WithSettings()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            settings.Tenants.Add(tenant);

            var client = new FastDataBrokerSDK.Client(
                settings: settings,
                tenantId: "acme-corp",
                apiKey: "acme_valid_key"
            );

            Assert.IsNotNull(client);
        }

        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestClientCreation_InvalidApiKeyPrefix()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            settings.Tenants.Add(tenant);

            // Wrong prefix
            var client = new FastDataBrokerSDK.Client(
                settings: settings,
                tenantId: "acme-corp",
                apiKey: "xyz_invalid_key"
            );
        }

        [TestMethod]
        [ExpectedException(typeof(ArgumentException))]
        public void TestClientCreation_TenantNotFound()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            
            var client = new FastDataBrokerSDK.Client(
                settings: settings,
                tenantId: "nonexistent",
                apiKey: "any_key"
            );
        }

        [TestMethod]
        public async System.Threading.Tasks.Task TestMessageIsolation()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            settings.Tenants.Add(tenant);

            var client = new FastDataBrokerSDK.Client(
                settings: settings,
                tenantId: "acme-corp",
                apiKey: "acme_valid_key"
            );

            await client.ConnectAsync();

            // Valid message with matching tenant
            var validMsg = new FastDataBrokerSDK.Message(
                "acme-corp",
                "user1",
                new List<string> { "user2" },
                "Subject",
                System.Text.Encoding.UTF8.GetBytes("content")
            );

            var result = await client.SendMessageAsync(validMsg);
            Assert.IsNotNull(result);
            Assert.AreEqual("acme-corp", result.TenantId);

            client.Disconnect();
        }

        [TestMethod]
        public void TestApiKeyGeneration()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100
            };

            settings.Tenants.Add(tenant);

            var client = new FastDataBrokerSDK.Client(
                settings: settings,
                tenantId: "acme-corp",
                apiKey: "acme_existing_key"
            );

            var newKey = client.GenerateApiKey("client-1");
            Assert.IsTrue(newKey.StartsWith("acme_"));
            Assert.IsTrue(newKey.Length > 6); // At least prefix + something
        }

        [TestMethod]
        public void TestGetTenantConfig()
        {
            var settings = new FastDataBrokerSDK.AppSettings();
            var tenant = new FastDataBrokerSDK.TenantConfig
            {
                TenantId = "acme-corp",
                ApiKeyPrefix = "acme_",
                RateLimitRps = 1000,
                MaxConnections = 100,
                RetentionDays = 30
            };

            settings.Tenants.Add(tenant);

            var client = new FastDataBrokerSDK.Client(
                settings: settings,
                tenantId: "acme-corp",
                apiKey: "acme_key"
            );

            var config = client.GetTenantConfig();
            Assert.IsNotNull(config);
            Assert.AreEqual("acme-corp", config.TenantId);
            Assert.AreEqual(1000u, config.RateLimitRps);
            Assert.AreEqual(30u, config.RetentionDays);
        }
    }
}
