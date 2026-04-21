import {
  Client,
  ConfigurationManager,
  Message,
  Priority,
  NotificationChannel,
  TenantConfig,
  AppSettings
} from '../fastdatabroker_multitenancy';
import * as assert from 'assert';
import * as fs from 'fs';

describe('FastDataBroker Multi-Tenant Configuration Tests', () => {

  // ============== Tenant Creation Tests ==============

  it('should create a tenant with valid config', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME Corporation',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30,
      enabled: true,
      metadata: {}
    };

    assert.strictEqual(tenant.tenant_id, 'acme-corp');
    assert.strictEqual(tenant.api_key_prefix, 'acme_');
    assert.strictEqual(tenant.rate_limit_rps, 1000);
    assert.strictEqual(tenant.max_connections, 100);
    assert.strictEqual(tenant.enabled, true);
  });

  // ============== Tenant Validation Tests ==============

  it('should validate tenant with valid config', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30,
      enabled: true
    };

    // Should not throw
    ConfigurationManager.validateTenant(tenant);
  });

  it('should reject empty tenant ID', () => {
    const tenant: TenantConfig = {
      tenant_id: '',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    assert.throws(() => ConfigurationManager.validateTenant(tenant));
  });

  it('should reject API key prefix without underscore', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme',  // Missing underscore
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    assert.throws(() => ConfigurationManager.validateTenant(tenant));
  });

  it('should reject zero rate limit', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 0,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    assert.throws(() => ConfigurationManager.validateTenant(tenant));
  });

  it('should reject zero max connections', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 0,
      max_message_size: 1048576,
      retention_days: 30
    };

    assert.throws(() => ConfigurationManager.validateTenant(tenant));
  });

  // ============== AppSettings Tests ==============

  it('should create empty AppSettings', () => {
    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: []
    };

    assert.strictEqual(settings.tenants.length, 0);
  });

  it('should get tenant by ID', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant]
    };

    const retrieved = ConfigurationManager.getTenant(settings, 'acme-corp');
    assert.strictEqual(retrieved?.tenant_id, 'acme-corp');
  });

  it('should return undefined for nonexistent tenant', () => {
    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: []
    };

    const retrieved = ConfigurationManager.getTenant(settings, 'nonexistent');
    assert.strictEqual(retrieved, undefined);
  });

  it('should get tenant by API key prefix', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant]
    };

    const retrieved = ConfigurationManager.getTenantByApiKey(settings, 'acme_550e8400e29b41d4a716446655440000');
    assert.strictEqual(retrieved?.tenant_id, 'acme-corp');
  });

  // ============== Multi-Tenant Isolation ==============

  it('should isolate multiple tenants with different rate limits', () => {
    const tenant1: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30,
      enabled: true
    };

    const tenant2: TenantConfig = {
      tenant_id: 'startup-xyz',
      tenant_name: 'Startup',
      api_key_prefix: 'xyz_',
      rate_limit_rps: 100,
      max_connections: 10,
      max_message_size: 524288,
      retention_days: 7,
      enabled: true
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant1, tenant2]
    };

    const t1 = ConfigurationManager.getTenant(settings, 'acme-corp');
    const t2 = ConfigurationManager.getTenant(settings, 'startup-xyz');

    assert.strictEqual(t1?.rate_limit_rps, 1000);
    assert.strictEqual(t2?.rate_limit_rps, 100);
    assert.strictEqual(t1?.max_connections, 100);
    assert.strictEqual(t2?.max_connections, 10);
  });

  // ============== Client Creation Tests ==============

  it('should create client with tenant ID and API key', () => {
    const client = Client.createWithTenant('acme-corp', 'acme_key', 'localhost', 6379);

    assert.strictEqual(client.getTenantId(), 'acme-corp');
    assert.strictEqual(client.isConnected(), false);
  });

  it('should throw error for empty tenant ID', () => {
    assert.throws(() => {
      Client.createWithTenant('', 'key', 'localhost', 6379);
    });
  });

  it('should throw error for empty API key', () => {
    assert.throws(() => {
      Client.createWithTenant('acme-corp', '', 'localhost', 6379);
    });
  });

  it('should create client from settings', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant]
    };

    const client = Client.createFromSettings(settings, 'acme-corp', 'acme_valid_key');

    assert.strictEqual(client.getTenantId(), 'acme-corp');
  });

  it('should reject API key with wrong prefix', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant]
    };

    assert.throws(() => {
      Client.createFromSettings(settings, 'acme-corp', 'xyz_invalid_key');
    });
  });

  it('should reject nonexistent tenant', () => {
    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: []
    };

    assert.throws(() => {
      Client.createFromSettings(settings, 'nonexistent', 'any_key');
    });
  });

  // ============== Message Creation Tests ==============

  it('should create message with tenant context', () => {
    const message = new Message(
      'acme-corp',
      'user1',
      ['user2', 'user3'],
      'Subject',
      Buffer.from('Content')
    );

    assert.strictEqual(message.tenantId, 'acme-corp');
    assert.strictEqual(message.senderId, 'user1');
    assert.strictEqual(message.recipientIds.length, 2);
    assert.strictEqual(message.subject, 'Subject');
  });

  it('should set message priority', () => {
    const message = new Message('acme-corp', 'user1', [], 'Subject', Buffer.from(''));
    message.priority = Priority.High;

    assert.strictEqual(message.priority, Priority.High);
  });

  it('should add message tags', () => {
    const message = new Message('acme-corp', 'user1', [], 'Subject', Buffer.from(''));
    message.tags.set('category', 'notification');
    message.tags.set('type', 'welcome');

    assert.strictEqual(message.tags.get('category'), 'notification');
    assert.strictEqual(message.tags.get('type'), 'welcome');
  });

  // ============== API Key Generation ==============

  it('should generate API key with correct prefix', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant]
    };

    const client = Client.createFromSettings(settings, 'acme-corp', 'acme_existing_key');
    const newKey = client.generateApiKey('client-1');

    assert.strictEqual(newKey.startsWith('acme_'), true);
    assert.strictEqual(newKey.length > 6, true);
  });

  // ============== WebSocket Client Management ==============

  it('should register WebSocket client', () => {
    const client = Client.createWithTenant('acme-corp', 'acme_key', 'localhost', 6379);
    const registered = client.registerWebSocketClient('ws-001', 'user-001');

    assert.strictEqual(registered, true);
  });

  it('should unregister WebSocket client', () => {
    const client = Client.createWithTenant('acme-corp', 'acme_key', 'localhost', 6379);
    client.registerWebSocketClient('ws-001', 'user-001');
    const unregistered = client.unregisterWebSocketClient('ws-001');

    assert.strictEqual(unregistered, true);
  });

  // ============== Tenant Config Access ==============

  it('should get tenant config from client', () => {
    const tenant: TenantConfig = {
      tenant_id: 'acme-corp',
      tenant_name: 'ACME',
      api_key_prefix: 'acme_',
      rate_limit_rps: 1000,
      max_connections: 100,
      max_message_size: 1048576,
      retention_days: 30,
      enabled: true
    };

    const settings: AppSettings = {
      app: { name: 'Test', version: '0.1.16', environment: 'test' },
      server: { bind_address: 'localhost', port: 6379, enable_tls: false, cert_path: '', key_path: '' },
      tenants: [tenant]
    };

    const client = Client.createFromSettings(settings, 'acme-corp', 'acme_key');
    const config = client.getTenantConfig();

    assert.strictEqual(config?.tenant_id, 'acme-corp');
    assert.strictEqual(config?.rate_limit_rps, 1000);
    assert.strictEqual(config?.retention_days, 30);
  });

  // ============== Priority Enum Tests ==============

  it('should have correct priority values', () => {
    assert.strictEqual(Priority.Deferred, 50);
    assert.strictEqual(Priority.Normal, 100);
    assert.strictEqual(Priority.High, 150);
    assert.strictEqual(Priority.Urgent, 200);
    assert.strictEqual(Priority.Critical, 255);
  });

  // ============== Webhook Configuration ==============

  it('should create webhook config', () => {
    const webhookConfig = {
      url: 'https://example.com/webhook',
      headers: { 'Authorization': 'Bearer token' },
      retries: 3,
      timeoutMs: 30000,
      verifySsl: true
    };

    assert.strictEqual(webhookConfig.url, 'https://example.com/webhook');
    assert.strictEqual(webhookConfig.headers['Authorization'], 'Bearer token');
    assert.strictEqual(webhookConfig.retries, 3);
    assert.strictEqual(webhookConfig.timeoutMs, 30000);
  });
});

