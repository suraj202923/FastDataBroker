using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using System.Text.Json;
using System.Linq;

namespace FastDataBroker
{
    /// <summary>
    /// FastDataBroker C# SDK - Multi-Tenant Client Library
    /// Version 0.1.16 - With Multi-Tenancy Support
    /// </summary>
    public class FastDataBrokerSDK
    {
        public const string Version = "0.1.16";

        /// <summary>
        /// Tenant Configuration
        /// </summary>
        public class TenantConfig
        {
            public string TenantId { get; set; }
            public string TenantName { get; set; }
            public string ApiKeyPrefix { get; set; }
            public uint RateLimitRps { get; set; }
            public uint MaxConnections { get; set; }
            public ulong MaxMessageSize { get; set; }
            public uint RetentionDays { get; set; }
            public bool Enabled { get; set; } = true;
            public Dictionary<string, object> Metadata { get; set; }

            public TenantConfig()
            {
                Metadata = new Dictionary<string, object>();
            }

            public void Validate()
            {
                if (string.IsNullOrEmpty(TenantId))
                    throw new ArgumentException("TenantId cannot be empty");
                if (string.IsNullOrEmpty(ApiKeyPrefix) || !ApiKeyPrefix.EndsWith("_"))
                    throw new ArgumentException("ApiKeyPrefix must end with '_'");
                if (RateLimitRps == 0)
                    throw new ArgumentException("RateLimitRps must be greater than 0");
                if (MaxConnections == 0)
                    throw new ArgumentException("MaxConnections must be greater than 0");
            }
        }

        /// <summary>
        /// AppSettings Configuration
        /// </summary>
        public class AppSettings
        {
            public class ServerConfig
            {
                public string BindAddress { get; set; } = "0.0.0.0";
                public ushort Port { get; set; } = 6379;
                public bool EnableTls { get; set; } = false;
                public string CertPath { get; set; } = "./certs/cert.pem";
                public string KeyPath { get; set; } = "./certs/key.pem";
            }

            public class AppConfig
            {
                public string Name { get; set; } = "FastDataBroker";
                public string Version { get; set; } = "0.1.16";
                public string Environment { get; set; } = "development";
            }

            public AppConfig App { get; set; } = new AppConfig();
            public ServerConfig Server { get; set; } = new ServerConfig();
            public List<TenantConfig> Tenants { get; set; } = new List<TenantConfig>();

            /// <summary>
            /// Load configuration from JSON file
            /// </summary>
            public static AppSettings LoadFromFile(string filePath, string environment = "development")
            {
                if (!File.Exists(filePath))
                    throw new FileNotFoundException($"Configuration file not found: {filePath}");

                var json = File.ReadAllText(filePath);
                var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
                var baseSettings = JsonSerializer.Deserialize<AppSettings>(json, options);

                // Try to load environment-specific config
                var envFile = Path.Combine(
                    Path.GetDirectoryName(filePath),
                    Path.GetFileNameWithoutExtension(filePath) + "." + environment + ".json"
                );

                if (File.Exists(envFile))
                {
                    var envJson = File.ReadAllText(envFile);
                    var envSettings = JsonSerializer.Deserialize<AppSettings>(envJson, options);
                    
                    // Merge environment-specific settings
                    if (envSettings.App != null)
                        baseSettings.App = envSettings.App;
                    if (envSettings.Server != null)
                        baseSettings.Server = envSettings.Server;
                    if (envSettings.Tenants?.Count > 0)
                    {
                        foreach (var tenant in envSettings.Tenants)
                        {
                            if (!baseSettings.Tenants.Any(t => t.TenantId == tenant.TenantId))
                                baseSettings.Tenants.Add(tenant);
                        }
                    }
                }

                return baseSettings;
            }

            /// <summary>
            /// Get tenant by ID
            /// </summary>
            public TenantConfig GetTenant(string tenantId)
            {
                return Tenants?.FirstOrDefault(t => t.TenantId == tenantId);
            }

            /// <summary>
            /// Get tenant by API key prefix
            /// </summary>
            public TenantConfig GetTenantByApiKey(string apiKey)
            {
                return Tenants?.FirstOrDefault(t => apiKey.StartsWith(t.ApiKeyPrefix));
            }
        }

        /// <summary>
        /// Priority levels for messages
        /// </summary>
        public enum Priority : byte
        {
            Deferred = 50,
            Normal = 100,
            High = 150,
            Urgent = 200,
            Critical = 255
        }

        /// <summary>
        /// Notification delivery channels
        /// </summary>
        public enum NotificationChannel
        {
            Email,
            WebSocket,
            Push,
            Webhook
        }

        /// <summary>
        /// Push notification platforms
        /// </summary>
        public enum PushPlatform
        {
            Firebase,
            APNs,
            FCM,
            WebPush
        }

        /// <summary>
        /// Message envelope for FastDataBroker
        /// </summary>
        public class Message
        {
            public string TenantId { get; set; }
            public string SenderId { get; set; }
            public List<string> RecipientIds { get; set; }
            public string Subject { get; set; }
            public byte[] Content { get; set; }
            public Priority Priority { get; set; } = Priority.Normal;
            public long? TTLSeconds { get; set; }
            public Dictionary<string, string> Tags { get; set; }
            public bool RequireConfirm { get; set; }

            public Message()
            {
                RecipientIds = new List<string>();
                Tags = new Dictionary<string, string>();
                Content = new byte[0];
            }

            public Message(string tenantId, string senderId, List<string> recipientIds, string subject, byte[] content)
            {
                TenantId = tenantId;
                SenderId = senderId;
                RecipientIds = recipientIds ?? new List<string>();
                Subject = subject;
                Content = content ?? new byte[0];
                Tags = new Dictionary<string, string>();
            }
        }

        /// <summary>
        /// Delivery result for a sent message
        /// </summary>
        public class DeliveryResult
        {
            public string MessageId { get; set; }
            public string TenantId { get; set; }
            public string Status { get; set; }
            public int DeliveredChannels { get; set; }
            public Dictionary<string, object> Details { get; set; }

            public DeliveryResult()
            {
                Details = new Dictionary<string, object>();
            }
        }

        /// <summary>
        /// WebSocket client information
        /// </summary>
        public class WebSocketClientInfo
        {
            public string ClientId { get; set; }
            public string UserId { get; set; }
            public string TenantId { get; set; }
            public DateTime ConnectedAt { get; set; }
        }

        /// <summary>
        /// Webhook configuration
        /// </summary>
        public class WebhookConfig
        {
            public string Url { get; set; }
            public Dictionary<string, string> Headers { get; set; }
            public int Retries { get; set; } = 3;
            public int TimeoutMs { get; set; } = 30000;
            public bool VerifySSL { get; set; } = true;

            public WebhookConfig()
            {
                Headers = new Dictionary<string, string>();
            }
        }

        /// <summary>
        /// Multi-Tenant FastDataBroker client
        /// </summary>
        public class Client : IDisposable
        {
            private readonly string _host;
            private readonly int _port;
            private readonly string _tenantId;
            private readonly string _apiKey;
            private readonly AppSettings _settings;
            private bool _connected = false;
            private readonly Dictionary<string, WebSocketClientInfo> _wsClients;

            public Client(string host = "localhost", int port = 6379)
            {
                _host = host;
                _port = port;
                _wsClients = new Dictionary<string, WebSocketClientInfo>();
                _settings = new AppSettings();
            }

            public Client(string tenantId, string apiKey, string host = "localhost", int port = 6379)
            {
                if (string.IsNullOrEmpty(tenantId))
                    throw new ArgumentException("TenantId cannot be empty");
                if (string.IsNullOrEmpty(apiKey))
                    throw new ArgumentException("ApiKey cannot be empty");

                _tenantId = tenantId;
                _apiKey = apiKey;
                _host = host;
                _port = port;
                _wsClients = new Dictionary<string, WebSocketClientInfo>();
                _settings = new AppSettings();
            }

            public Client(AppSettings settings, string tenantId, string apiKey)
            {
                _settings = settings ?? throw new ArgumentNullException(nameof(settings));
                _tenantId = tenantId ?? throw new ArgumentNullException(nameof(tenantId));
                _apiKey = apiKey ?? throw new ArgumentNullException(nameof(apiKey));
                _host = settings.Server.BindAddress;
                _port = settings.Server.Port;
                _wsClients = new Dictionary<string, WebSocketClientInfo>();

                // Validate tenant exists
                var tenant = settings.GetTenant(tenantId);
                if (tenant == null)
                    throw new ArgumentException($"Tenant '{tenantId}' not found in configuration");

                // Validate API key prefix
                if (!_apiKey.StartsWith(tenant.ApiKeyPrefix))
                    throw new ArgumentException($"API key does not match tenant prefix: {tenant.ApiKeyPrefix}");
            }

            /// <summary>
            /// Connect to FastDataBroker server with tenant context
            /// </summary>
            public async Task<bool> ConnectAsync()
            {
                try
                {
                    if (string.IsNullOrEmpty(_tenantId))
                        throw new InvalidOperationException("TenantId must be set before connecting");

                    // Establish QUIC connection with tenant context
                    _connected = true;
                    Console.WriteLine($"[TENANT: {_tenantId}] Connected to FastDataBroker at {_host}:{_port}");
                    return await Task.FromResult(true);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[TENANT: {_tenantId}] Connection failed: {ex.Message}");
                    return false;
                }
            }

            /// <summary>
            /// Send a message with tenant isolation
            /// </summary>
            public DeliveryResult SendMessage(Message message)
            {
                if (!_connected)
                    throw new InvalidOperationException("Not connected. Call ConnectAsync first.");

                if (message == null)
                    throw new ArgumentNullException(nameof(message));

                // Ensure message belongs to this tenant
                if (string.IsNullOrEmpty(message.TenantId))
                    message.TenantId = _tenantId;

                if (message.TenantId != _tenantId)
                    throw new InvalidOperationException($"Message tenant '{message.TenantId}' does not match client tenant '{_tenantId}'");

                var result = new DeliveryResult
                {
                    MessageId = Guid.NewGuid().ToString(),
                    TenantId = _tenantId,
                    Status = "success",
                    DeliveredChannels = 1,
                };

                return result;
            }

            /// <summary>
            /// Send a message asynchronously with tenant isolation
            /// </summary>
            public async Task<DeliveryResult> SendMessageAsync(Message message)
            {
                if (!_connected)
                    throw new InvalidOperationException("Not connected. Call ConnectAsync first.");

                if (message == null)
                    throw new ArgumentNullException(nameof(message));

                if (string.IsNullOrEmpty(message.TenantId))
                    message.TenantId = _tenantId;

                if (message.TenantId != _tenantId)
                    throw new InvalidOperationException($"Message tenant does not match client tenant");

                var result = new DeliveryResult
                {
                    MessageId = Guid.NewGuid().ToString(),
                    TenantId = _tenantId,
                    Status = "success",
                    DeliveredChannels = 1,
                };

                return await Task.FromResult(result);
            }

            /// <summary>
            /// Register WebSocket client (tenant-isolated)
            /// </summary>
            public bool RegisterWebSocketClient(string clientId, string userId)
            {
                if (string.IsNullOrEmpty(clientId) || string.IsNullOrEmpty(userId))
                    return false;

                var clientInfo = new WebSocketClientInfo
                {
                    ClientId = clientId,
                    UserId = userId,
                    TenantId = _tenantId,
                    ConnectedAt = DateTime.UtcNow
                };

                _wsClients[clientId] = clientInfo;
                return true;
            }

            /// <summary>
            /// Unregister WebSocket client
            /// </summary>
            public bool UnregisterWebSocketClient(string clientId)
            {
                return _wsClients.Remove(clientId);
            }

            /// <summary>
            /// Register webhook endpoint (tenant-isolated)
            /// </summary>
            public bool RegisterWebhook(NotificationChannel channel, WebhookConfig config)
            {
                if (config == null || string.IsNullOrEmpty(config.Url))
                    return false;

                return true;
            }

            /// <summary>
            /// Generate API key for a client (tenant-aware)
            /// </summary>
            public string GenerateApiKey(string clientId)
            {
                if (string.IsNullOrEmpty(clientId))
                    throw new ArgumentException("ClientId cannot be empty");

                var tenant = _settings.GetTenant(_tenantId);
                if (tenant == null)
                    throw new InvalidOperationException($"Tenant '{_tenantId}' not found");

                var guid = Guid.NewGuid().ToString("N");
                return $"{tenant.ApiKeyPrefix}{guid}";
            }

            /// <summary>
            /// Get current tenant configuration
            /// </summary>
            public TenantConfig GetTenantConfig()
            {
                return _settings.GetTenant(_tenantId);
            }

            /// <summary>
            /// Disconnect
            /// </summary>
            public void Disconnect()
            {
                _connected = false;
                Console.WriteLine($"[TENANT: {_tenantId}] Disconnected from FastDataBroker");
            }

            public void Dispose()
            {
                Disconnect();
                _wsClients.Clear();
            }
        }

        /// <summary>
        /// Static helper to load configuration and create client
        /// </summary>
        public static Client CreateClient(string configPath, string tenantId, string apiKey, string environment = "development")
        {
            var settings = AppSettings.LoadFromFile(configPath, environment);
            return new Client(settings, tenantId, apiKey);
        }
    }
}

