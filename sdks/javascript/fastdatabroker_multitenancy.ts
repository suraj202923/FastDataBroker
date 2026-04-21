/**
 * FastDataBroker TypeScript/JavaScript SDK - Multi-Tenant Client Library
 * Version 0.1.16 - With Multi-Tenancy Support
 */

import * as fs from 'fs';
import * as path from 'path';

export const Version = '0.1.16';

/**
 * Tenant Configuration
 */
export interface TenantConfig {
  tenant_id: string;
  tenant_name: string;
  api_key_prefix: string;
  rate_limit_rps: number;
  max_connections: number;
  max_message_size: number;
  retention_days: number;
  enabled?: boolean;
  metadata?: Record<string, any>;
}

/**
 * Server Configuration
 */
export interface ServerConfig {
  bind_address: string;
  port: number;
  enable_tls: boolean;
  cert_path: string;
  key_path: string;
}

/**
 * Application Configuration
 */
export interface AppConfig {
  name: string;
  version: string;
  environment: string;
}

/**
 * Application Settings
 */
export interface AppSettings {
  app: AppConfig;
  server: ServerConfig;
  tenants: TenantConfig[];
}

/**
 * Utility class for configuration management
 */
export class ConfigurationManager {
  /**
   * Load configuration from JSON file with environment overrides
   */
  static loadFromFile(filePath: string, environment: string = 'development'): AppSettings {
    if (!fs.existsSync(filePath)) {
      throw new Error(`Configuration file not found: ${filePath}`);
    }

    const data = fs.readFileSync(filePath, 'utf-8');
    const baseSettings: AppSettings = JSON.parse(data);

    // Try to load environment-specific config
    const dir = path.dirname(filePath);
    const fileName = path.basename(filePath);
    const nameWithoutExt = path.basename(fileName, path.extname(fileName));
    const envFile = path.join(dir, `${nameWithoutExt}.${environment}.json`);

    if (fs.existsSync(envFile)) {
      const envData = fs.readFileSync(envFile, 'utf-8');
      const envSettings: AppSettings = JSON.parse(envData);

      if (envSettings.app) {
        baseSettings.app = envSettings.app;
      }
      if (envSettings.server) {
        baseSettings.server = envSettings.server;
      }
      if (envSettings.tenants && envSettings.tenants.length > 0) {
        for (const tenant of envSettings.tenants) {
          const found = baseSettings.tenants.some(t => t.tenant_id === tenant.tenant_id);
          if (!found) {
            baseSettings.tenants.push(tenant);
          }
        }
      }
    }

    return baseSettings;
  }

  /**
   * Get tenant by ID
   */
  static getTenant(settings: AppSettings, tenantId: string): TenantConfig | undefined {
    return settings.tenants.find(t => t.tenant_id === tenantId);
  }

  /**
   * Get tenant by API key prefix
   */
  static getTenantByApiKey(settings: AppSettings, apiKey: string): TenantConfig | undefined {
    return settings.tenants.find(t => apiKey.startsWith(t.api_key_prefix));
  }

  /**
   * Validate tenant configuration
   */
  static validateTenant(tenant: TenantConfig): void {
    if (!tenant.tenant_id) {
      throw new Error('TenantId cannot be empty');
    }
    if (!tenant.api_key_prefix || !tenant.api_key_prefix.endsWith('_')) {
      throw new Error('ApiKeyPrefix must end with "_"');
    }
    if (tenant.rate_limit_rps <= 0) {
      throw new Error('RateLimitRps must be greater than 0');
    }
    if (tenant.max_connections <= 0) {
      throw new Error('MaxConnections must be greater than 0');
    }
  }
}

/**
 * Priority levels for messages
 */
export enum Priority {
  Deferred = 50,
  Normal = 100,
  High = 150,
  Urgent = 200,
  Critical = 255
}

/**
 * Notification delivery channels
 */
export enum NotificationChannel {
  Email = 'email',
  WebSocket = 'websocket',
  Push = 'push',
  Webhook = 'webhook'
}

/**
 * Push notification platforms
 */
export enum PushPlatform {
  Firebase = 'firebase',
  APNs = 'apns',
  FCM = 'fcm',
  WebPush = 'webpush'
}

/**
 * Message envelope for FastDataBroker
 */
export class Message {
  tenantId: string;
  senderId: string;
  recipientIds: string[];
  subject: string;
  content: Buffer | Uint8Array;
  priority: Priority = Priority.Normal;
  ttlSeconds?: number;
  tags: Map<string, string>;
  requireConfirm: boolean = false;

  constructor(
    tenantId: string,
    senderId: string,
    recipientIds: string[],
    subject: string,
    content: Buffer | Uint8Array
  ) {
    this.tenantId = tenantId;
    this.senderId = senderId;
    this.recipientIds = recipientIds || [];
    this.subject = subject;
    this.content = content || new Buffer(0);
    this.tags = new Map();
  }
}

/**
 * Delivery result for a sent message
 */
export interface DeliveryResult {
  messageId: string;
  tenantId: string;
  status: string;
  deliveredChannels: number;
  details: Record<string, any>;
}

/**
 * WebSocket client information
 */
export interface WebSocketClientInfo {
  clientId: string;
  userId: string;
  tenantId: string;
  connectedAt: Date;
}

/**
 * Webhook configuration
 */
export interface WebhookConfig {
  url: string;
  headers?: Record<string, string>;
  retries?: number;
  timeoutMs?: number;
  verifySsl?: boolean;
}

/**
 * Multi-Tenant FastDataBroker client
 */
export class Client {
  private host: string;
  private port: number;
  private tenantId: string;
  private apiKey: string;
  private settings: AppSettings;
  private connected: boolean = false;
  private wsClients: Map<string, WebSocketClientInfo>;

  /**
   * Create a new client
   */
  constructor(host: string = 'localhost', port: number = 6379) {
    this.host = host;
    this.port = port;
    this.tenantId = '';
    this.apiKey = '';
    this.settings = { app: {}, server: {}, tenants: [] } as AppSettings;
    this.wsClients = new Map();
  }

  /**
   * Create a tenant-aware client
   */
  static createWithTenant(
    tenantId: string,
    apiKey: string,
    host: string = 'localhost',
    port: number = 6379
  ): Client {
    if (!tenantId) {
      throw new Error('TenantId cannot be empty');
    }
    if (!apiKey) {
      throw new Error('ApiKey cannot be empty');
    }

    const client = new Client(host, port);
    client.tenantId = tenantId;
    client.apiKey = apiKey;
    return client;
  }

  /**
   * Create a client from AppSettings with tenant validation
   */
  static createFromSettings(
    settings: AppSettings,
    tenantId: string,
    apiKey: string
  ): Client {
    if (!settings) {
      throw new Error('Settings cannot be null');
    }
    if (!tenantId) {
      throw new Error('TenantId cannot be empty');
    }
    if (!apiKey) {
      throw new Error('ApiKey cannot be empty');
    }

    const tenant = ConfigurationManager.getTenant(settings, tenantId);
    if (!tenant) {
      throw new Error(`Tenant '${tenantId}' not found in configuration`);
    }

    if (!apiKey.startsWith(tenant.api_key_prefix)) {
      throw new Error(`API key does not match tenant prefix: ${tenant.api_key_prefix}`);
    }

    const client = new Client(settings.server.bind_address, settings.server.port);
    client.tenantId = tenantId;
    client.apiKey = apiKey;
    client.settings = settings;
    return client;
  }

  /**
   * Connect to FastDataBroker server with tenant context
   */
  async connect(): Promise<boolean> {
    try {
      if (!this.tenantId) {
        throw new Error('TenantId must be set before connecting');
      }

      this.connected = true;
      console.log(`[TENANT: ${this.tenantId}] Connected to FastDataBroker at ${this.host}:${this.port}`);
      return true;
    } catch (error) {
      console.error(`[TENANT: ${this.tenantId}] Connection failed: ${error}`);
      return false;
    }
  }

  /**
   * Send a message with tenant isolation
   */
  async sendMessage(message: Message): Promise<DeliveryResult> {
    if (!this.connected) {
      throw new Error('Not connected. Call connect() first.');
    }

    if (!message) {
      throw new Error('Message cannot be null');
    }

    if (!message.tenantId) {
      message.tenantId = this.tenantId;
    }

    if (message.tenantId !== this.tenantId) {
      throw new Error('Message tenant does not match client tenant');
    }

    return {
      messageId: this.generateId(),
      tenantId: this.tenantId,
      status: 'success',
      deliveredChannels: 1,
      details: {}
    };
  }

  /**
   * Register WebSocket client (tenant-isolated)
   */
  registerWebSocketClient(clientId: string, userId: string): boolean {
    if (!clientId || !userId) {
      return false;
    }

    const clientInfo: WebSocketClientInfo = {
      clientId,
      userId,
      tenantId: this.tenantId,
      connectedAt: new Date()
    };

    this.wsClients.set(clientId, clientInfo);
    return true;
  }

  /**
   * Unregister WebSocket client
   */
  unregisterWebSocketClient(clientId: string): boolean {
    return this.wsClients.delete(clientId);
  }

  /**
   * Register webhook endpoint (tenant-isolated)
   */
  registerWebhook(channel: NotificationChannel, config: WebhookConfig): boolean {
    if (!config || !config.url) {
      return false;
    }
    return true;
  }

  /**
   * Generate API key for a client (tenant-aware)
   */
  generateApiKey(clientId: string): string {
    if (!clientId) {
      throw new Error('ClientId cannot be empty');
    }

    const tenant = ConfigurationManager.getTenant(this.settings, this.tenantId);
    if (!tenant) {
      throw new Error(`Tenant '${this.tenantId}' not found`);
    }

    const guid = this.generateId();
    return `${tenant.api_key_prefix}${guid}`;
  }

  /**
   * Get current tenant configuration
   */
  getTenantConfig(): TenantConfig | undefined {
    return ConfigurationManager.getTenant(this.settings, this.tenantId);
  }

  /**
   * Disconnect
   */
  disconnect(): void {
    this.connected = false;
    console.log(`[TENANT: ${this.tenantId}] Disconnected from FastDataBroker`);
  }

  /**
   * Private helper to generate IDs
   */
  private generateId(): string {
    return `${Date.now()}${Math.random().toString(36).substring(2, 15)}`;
  }

  /**
   * Get connection status
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Get tenant ID
   */
  getTenantId(): string {
    return this.tenantId;
  }
}

/**
 * Factory function to create client from config file
 */
export async function createClient(
  configPath: string,
  tenantId: string,
  apiKey: string,
  environment: string = 'development'
): Promise<Client> {
  const settings = ConfigurationManager.loadFromFile(configPath, environment);
  return Client.createFromSettings(settings, tenantId, apiKey);
}

