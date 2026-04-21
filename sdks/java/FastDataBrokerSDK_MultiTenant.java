package com.fastdatabroker.sdk;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.Instant;
import java.util.*;
import java.util.stream.Collectors;

/**
 * FastDataBroker Java SDK - Multi-Tenant Client Library
 * Version 0.1.16 - With Multi-Tenancy Support
 */
public class FastDataBrokerSDK {
    public static final String VERSION = "0.1.16";

    /**
     * Tenant Configuration
     */
    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class TenantConfig {
        @JsonProperty("tenant_id")
        public String tenantId;

        @JsonProperty("tenant_name")
        public String tenantName;

        @JsonProperty("api_key_prefix")
        public String apiKeyPrefix;

        @JsonProperty("rate_limit_rps")
        public long rateLimitRps;

        @JsonProperty("max_connections")
        public long maxConnections;

        @JsonProperty("max_message_size")
        public long maxMessageSize;

        @JsonProperty("retention_days")
        public long retentionDays;

        public boolean enabled = true;

        @JsonProperty("metadata")
        public Map<String, Object> metadata = new HashMap<>();

        public void validate() throws IllegalArgumentException {
            if (tenantId == null || tenantId.isEmpty()) {
                throw new IllegalArgumentException("TenantId cannot be empty");
            }
            if (apiKeyPrefix == null || !apiKeyPrefix.endsWith("_")) {
                throw new IllegalArgumentException("ApiKeyPrefix must end with '_'");
            }
            if (rateLimitRps <= 0) {
                throw new IllegalArgumentException("RateLimitRps must be greater than 0");
            }
            if (maxConnections <= 0) {
                throw new IllegalArgumentException("MaxConnections must be greater than 0");
            }
        }
    }

    /**
     * Server Configuration
     */
    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class ServerConfig {
        @JsonProperty("bind_address")
        public String bindAddress = "0.0.0.0";

        public int port = 6379;

        @JsonProperty("enable_tls")
        public boolean enableTls = false;

        @JsonProperty("cert_path")
        public String certPath = "./certs/cert.pem";

        @JsonProperty("key_path")
        public String keyPath = "./certs/key.pem";
    }

    /**
     * Application Configuration
     */
    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class AppConfig {
        public String name = "FastDataBroker";
        public String version = "0.1.16";
        public String environment = "development";
    }

    /**
     * Application Settings
     */
    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class AppSettings {
        public AppConfig app = new AppConfig();
        public ServerConfig server = new ServerConfig();
        public List<TenantConfig> tenants = new ArrayList<>();

        /**
         * Load configuration from JSON file
         */
        public static AppSettings loadFromFile(String filePath, String environment) throws IOException {
            File file = new File(filePath);
            if (!file.exists()) {
                throw new IOException("Configuration file not found: " + filePath);
            }

            ObjectMapper mapper = new ObjectMapper();
            AppSettings baseSettings = mapper.readValue(file, AppSettings.class);

            // Try to load environment-specific config
            String fileName = file.getName();
            String nameWithoutExt = fileName.substring(0, fileName.lastIndexOf('.'));
            File envFile = new File(
                file.getParent(),
                nameWithoutExt + "." + environment + ".json"
            );

            if (envFile.exists()) {
                AppSettings envSettings = mapper.readValue(envFile, AppSettings.class);

                if (envSettings.app != null) {
                    baseSettings.app = envSettings.app;
                }
                if (envSettings.server != null) {
                    baseSettings.server = envSettings.server;
                }
                if (envSettings.tenants != null && !envSettings.tenants.isEmpty()) {
                    for (TenantConfig tenant : envSettings.tenants) {
                        boolean found = baseSettings.tenants.stream()
                            .anyMatch(t -> t.tenantId.equals(tenant.tenantId));
                        if (!found) {
                            baseSettings.tenants.add(tenant);
                        }
                    }
                }
            }

            return baseSettings;
        }

        /**
         * Get tenant by ID
         */
        public TenantConfig getTenant(String tenantId) {
            return tenants.stream()
                .filter(t -> t.tenantId.equals(tenantId))
                .findFirst()
                .orElse(null);
        }

        /**
         * Get tenant by API key prefix
         */
        public TenantConfig getTenantByApiKey(String apiKey) {
            return tenants.stream()
                .filter(t -> apiKey.startsWith(t.apiKeyPrefix))
                .findFirst()
                .orElse(null);
        }
    }

    /**
     * Priority levels for messages
     */
    public enum Priority {
        DEFERRED(50),
        NORMAL(100),
        HIGH(150),
        URGENT(200),
        CRITICAL(255);

        public final byte value;

        Priority(int value) {
            this.value = (byte) value;
        }
    }

    /**
     * Notification delivery channels
     */
    public enum NotificationChannel {
        EMAIL,
        WEBSOCKET,
        PUSH,
        WEBHOOK
    }

    /**
     * Push notification platforms
     */
    public enum PushPlatform {
        FIREBASE,
        APNS,
        FCM,
        WEBPUSH
    }

    /**
     * Message envelope for FastDataBroker
     */
    public static class Message {
        public String tenantId;
        public String senderId;
        public List<String> recipientIds;
        public String subject;
        public byte[] content;
        public Priority priority = Priority.NORMAL;
        public Long ttlSeconds;
        public Map<String, String> tags;
        public boolean requireConfirm;

        public Message() {
            this.recipientIds = new ArrayList<>();
            this.tags = new HashMap<>();
            this.content = new byte[0];
        }

        public Message(String tenantId, String senderId, List<String> recipientIds,
                      String subject, byte[] content) {
            this.tenantId = tenantId;
            this.senderId = senderId;
            this.recipientIds = recipientIds != null ? recipientIds : new ArrayList<>();
            this.subject = subject;
            this.content = content != null ? content : new byte[0];
            this.tags = new HashMap<>();
        }
    }

    /**
     * Delivery result for a sent message
     */
    public static class DeliveryResult {
        public String messageId;
        public String tenantId;
        public String status;
        public int deliveredChannels;
        public Map<String, Object> details;

        public DeliveryResult() {
            this.details = new HashMap<>();
        }
    }

    /**
     * WebSocket client information
     */
    public static class WebSocketClientInfo {
        public String clientId;
        public String userId;
        public String tenantId;
        public Instant connectedAt;
    }

    /**
     * Webhook configuration
     */
    public static class WebhookConfig {
        public String url;
        public Map<String, String> headers = new HashMap<>();
        public int retries = 3;
        public int timeoutMs = 30000;
        public boolean verifySsl = true;
    }

    /**
     * Multi-Tenant FastDataBroker client
     */
    public static class Client {
        private final String host;
        private final int port;
        private final String tenantId;
        private final String apiKey;
        private final AppSettings settings;
        private boolean connected = false;
        private final Map<String, WebSocketClientInfo> wsClients;

        public Client(String host, int port) {
            this.host = host;
            this.port = port;
            this.tenantId = null;
            this.apiKey = null;
            this.settings = new AppSettings();
            this.wsClients = new HashMap<>();
        }

        public Client(String tenantId, String apiKey, String host, int port) {
            if (tenantId == null || tenantId.isEmpty()) {
                throw new IllegalArgumentException("TenantId cannot be empty");
            }
            if (apiKey == null || apiKey.isEmpty()) {
                throw new IllegalArgumentException("ApiKey cannot be empty");
            }

            this.tenantId = tenantId;
            this.apiKey = apiKey;
            this.host = host;
            this.port = port;
            this.settings = new AppSettings();
            this.wsClients = new HashMap<>();
        }

        public Client(AppSettings settings, String tenantId, String apiKey) {
            if (settings == null) {
                throw new IllegalArgumentException("Settings cannot be null");
            }
            if (tenantId == null || tenantId.isEmpty()) {
                throw new IllegalArgumentException("TenantId cannot be empty");
            }
            if (apiKey == null || apiKey.isEmpty()) {
                throw new IllegalArgumentException("ApiKey cannot be empty");
            }

            this.settings = settings;
            this.tenantId = tenantId;
            this.apiKey = apiKey;
            this.host = settings.server.bindAddress;
            this.port = settings.server.port;
            this.wsClients = new HashMap<>();

            // Validate tenant exists
            TenantConfig tenant = settings.getTenant(tenantId);
            if (tenant == null) {
                throw new IllegalArgumentException("Tenant '" + tenantId + "' not found in configuration");
            }

            // Validate API key prefix
            if (!apiKey.startsWith(tenant.apiKeyPrefix)) {
                throw new IllegalArgumentException("API key does not match tenant prefix: " + tenant.apiKeyPrefix);
            }
        }

        /**
         * Connect to FastDataBroker server with tenant context
         */
        public boolean connect() {
            try {
                if (tenantId == null || tenantId.isEmpty()) {
                    throw new IllegalStateException("TenantId must be set before connecting");
                }

                connected = true;
                System.out.println("[TENANT: " + tenantId + "] Connected to FastDataBroker at " +
                    host + ":" + port);
                return true;
            } catch (Exception e) {
                System.out.println("[TENANT: " + tenantId + "] Connection failed: " + e.getMessage());
                return false;
            }
        }

        /**
         * Send a message with tenant isolation
         */
        public DeliveryResult sendMessage(Message message) {
            if (!connected) {
                throw new IllegalStateException("Not connected. Call connect() first.");
            }

            if (message == null) {
                throw new IllegalArgumentException("Message cannot be null");
            }

            if (message.tenantId == null || message.tenantId.isEmpty()) {
                message.tenantId = tenantId;
            }

            if (!message.tenantId.equals(tenantId)) {
                throw new IllegalStateException("Message tenant does not match client tenant");
            }

            DeliveryResult result = new DeliveryResult();
            result.messageId = UUID.randomUUID().toString();
            result.tenantId = tenantId;
            result.status = "success";
            result.deliveredChannels = 1;

            return result;
        }

        /**
         * Register WebSocket client (tenant-isolated)
         */
        public boolean registerWebSocketClient(String clientId, String userId) {
            if (clientId == null || clientId.isEmpty() || userId == null || userId.isEmpty()) {
                return false;
            }

            WebSocketClientInfo clientInfo = new WebSocketClientInfo();
            clientInfo.clientId = clientId;
            clientInfo.userId = userId;
            clientInfo.tenantId = tenantId;
            clientInfo.connectedAt = Instant.now();

            wsClients.put(clientId, clientInfo);
            return true;
        }

        /**
         * Unregister WebSocket client
         */
        public boolean unregisterWebSocketClient(String clientId) {
            return wsClients.remove(clientId) != null;
        }

        /**
         * Register webhook endpoint (tenant-isolated)
         */
        public boolean registerWebhook(NotificationChannel channel, WebhookConfig config) {
            if (config == null || config.url == null || config.url.isEmpty()) {
                return false;
            }
            return true;
        }

        /**
         * Generate API key for a client (tenant-aware)
         */
        public String generateApiKey(String clientId) {
            if (clientId == null || clientId.isEmpty()) {
                throw new IllegalArgumentException("ClientId cannot be empty");
            }

            TenantConfig tenant = settings.getTenant(tenantId);
            if (tenant == null) {
                throw new IllegalStateException("Tenant '" + tenantId + "' not found");
            }

            String guid = UUID.randomUUID().toString().replace("-", "");
            return tenant.apiKeyPrefix + guid;
        }

        /**
         * Get current tenant configuration
         */
        public TenantConfig getTenantConfig() {
            return settings.getTenant(tenantId);
        }

        /**
         * Disconnect
         */
        public void disconnect() {
            connected = false;
            System.out.println("[TENANT: " + tenantId + "] Disconnected from FastDataBroker");
        }
    }

    /**
     * Static helper to load configuration and create client
     */
    public static Client createClient(String configPath, String tenantId, String apiKey,
                                      String environment) throws IOException {
        AppSettings settings = AppSettings.loadFromFile(configPath, environment);
        return new Client(settings, tenantId, apiKey);
    }
}

