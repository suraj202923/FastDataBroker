/**
 * FastDataBroker JavaScript SDK - Tenant-Specific QUIC Tests
 */

import assert from 'assert';
import TenantQuicClient, { 
  ConnectionState, 
  TenantRole, 
  TenantConfig,
  Message 
} from '../src/tenant_quic_client';

describe('FastDataBroker Tenant-Specific QUIC Tests', () => {

  // ============== Tenant Configuration Tests ==============

  it('should create a tenant configuration', () => {
    const config: TenantConfig = {
      tenantId: 'test-tenant-1',
      pskSecret: 'super-secret-key',
      clientId: 'client-001',
      apiKey: 'api_key_xxx',
      role: TenantRole.ADMIN,
      rateLimitRPS: 5000,
      maxConnections: 200
    };

    assert.strictEqual(config.tenantId, 'test-tenant-1');
    assert.strictEqual(config.pskSecret, 'super-secret-key');
    assert.strictEqual(config.role, TenantRole.ADMIN);
    assert.strictEqual(config.rateLimitRPS, 5000);
    assert.strictEqual(config.maxConnections, 200);
  });

  // ============== QUIC Handshake Tests ==============

  it('should perform tenant-specific QUIC handshake', async () => {
    const config: TenantConfig = {
      tenantId: 'acme-corp',
      pskSecret: 'acme-psk-secret',
      clientId: 'client-acme-01',
      apiKey: 'api_acme_xyz'
    };

    const client = new TenantQuicClient('localhost', 6000, config);

    assert.strictEqual(client.getConnectionState(), ConnectionState.IDLE);

    // Perform handshake
    const result = await client.connect();

    assert.strictEqual(result, true);
    assert.strictEqual(client.getConnectionState(), ConnectionState.ESTABLISHED);
    assert(client.isConnected());
    assert(client.getSessionToken().length > 0);
    assert(client.getConnectionId().length > 0);

    const stats = client.getStats();
    assert(stats.handshakeDurationMs > 0);

    await client.disconnect();
  });

  // ============== Tenant Message Isolation Tests ==============

  it('should isolate messages between tenants', async () => {
    const config1: TenantConfig = {
      tenantId: 'tenant-1',
      pskSecret: 'secret-1',
      clientId: 'client-1',
      apiKey: 'api_1'
    };

    const config2: TenantConfig = {
      tenantId: 'tenant-2',
      pskSecret: 'secret-2',
      clientId: 'client-2',
      apiKey: 'api_2'
    };

    const client1 = new TenantQuicClient('localhost', 6000, config1);
    const client2 = new TenantQuicClient('localhost', 6000, config2);

    // Connect both tenants
    assert(await client1.connect());
    assert(await client2.connect());

    // Send messages
    const msg1: Message = {
      topic: 'test.topic',
      payload: { data: 'tenant1' }
    };

    const msg2: Message = {
      topic: 'test.topic',
      payload: { data: 'tenant2' }
    };

    const result1 = await client1.sendMessage(msg1);
    const result2 = await client2.sendMessage(msg2);

    // Verify messages have correct tenant context
    assert.strictEqual(result1.tenantId, 'tenant-1');
    assert.strictEqual(result2.tenantId, 'tenant-2');
    assert.notStrictEqual(result1.messageId, result2.messageId);

    // Verify session isolation
    assert.notStrictEqual(client1.getSessionToken(), client2.getSessionToken());
    assert.notStrictEqual(client1.getConnectionId(), client2.getConnectionId());

    await client1.disconnect();
    await client2.disconnect();
  });

  // ============== Concurrent Connections Tests ==============

  it('should handle concurrent connections from multiple tenants', async () => {
    const numTenants = 5;
    const configs: TenantConfig[] = [];

    for (let i = 0; i < numTenants; i++) {
      configs.push({
        tenantId: `tenant-${i}`,
        pskSecret: `secret-${i}`,
        clientId: `client-${i}`,
        apiKey: `api_${i}`
      });
    }

    const clients = configs.map(cfg => new TenantQuicClient('localhost', 6000, cfg));

    // Connect all tenants
    for (const client of clients) {
      assert(await client.connect());
    }

    // Send messages from each tenant
    let totalSent = 0;
    for (let i = 0; i < clients.length; i++) {
      const msg: Message = {
        topic: 'test.multi',
        payload: { index: i }
      };

      const result = await clients[i].sendMessage(msg);
      assert.strictEqual(result.status, 'success');
      assert.strictEqual(result.tenantId, `tenant-${i}`);
      totalSent++;
    }

    // Verify message counts
    assert.strictEqual(totalSent, numTenants);

    // Disconnect all clients
    for (const client of clients) {
      await client.disconnect();
    }
  });

  // ============== PSK Validation Tests ==============

  it('should validate PSK-based tenant authentication', async () => {
    const config: TenantConfig = {
      tenantId: 'psk-test-tenant',
      pskSecret: 'specific-psk-secret',
      clientId: 'psk-client-01',
      apiKey: 'psk_api_key'
    };

    const client = new TenantQuicClient('localhost', 6000, config);

    // Perform handshake
    const result = await client.connect();
    assert(result);

    const stats = client.getStats();
    assert(stats.isConnected);

    await client.disconnect();
  });

  // ============== Handshake Metrics Tests ==============

  it('should measure handshake performance metrics', async () => {
    const config: TenantConfig = {
      tenantId: 'metrics-tenant',
      pskSecret: 'metrics-secret',
      clientId: 'metrics-client',
      apiKey: 'metrics_api'
    };

    const client = new TenantQuicClient('localhost', 6000, config);
    await client.connect();

    const stats = client.getStats();

    assert(stats.isConnected);
    assert(stats.handshakeDurationMs > 0);
    assert(stats.uptimeSeconds >= 0);

    await client.disconnect();
  });

  // ============== Connection State Tests ==============

  it('should transition connection states correctly', async () => {
    const config: TenantConfig = {
      tenantId: 'state-test',
      pskSecret: 'state-secret',
      clientId: 'state-client',
      apiKey: 'state_api'
    };

    const client = new TenantQuicClient('localhost', 6000, config);

    // Initial state
    assert.strictEqual(client.getConnectionState(), ConnectionState.IDLE);

    // After connect
    await client.connect();
    assert.strictEqual(client.getConnectionState(), ConnectionState.ESTABLISHED);

    // Can send messages in established state
    assert(client.isConnected());

    // After disconnect
    await client.disconnect();
    assert.strictEqual(client.getConnectionState(), ConnectionState.CLOSED);
    assert(!client.isConnected());
  });

  // ============== Rate Limiting Config Tests ==============

  it('should support tenant-specific rate limiting configuration', async () => {
    const config: TenantConfig = {
      tenantId: 'rate-limit-tenant',
      pskSecret: 'rate-secret',
      clientId: 'rate-client',
      apiKey: 'rate_api',
      rateLimitRPS: 2000,
      maxConnections: 50
    };

    const client = new TenantQuicClient('localhost', 6000, config);

    await client.connect();

    const stats = client.getStats();
    assert(stats.isConnected);

    await client.disconnect();
  });

  // ============== Custom Headers Tests ==============

  it('should support tenant custom headers in configuration', async () => {
    const customHeaders: Record<string, string> = {
      'X-Tenant-Region': 'us-west',
      'X-Custom-Header': 'custom-value'
    };

    const config: TenantConfig = {
      tenantId: 'custom-header-tenant',
      pskSecret: 'custom-secret',
      clientId: 'custom-client',
      apiKey: 'custom_api',
      customHeaders
    };

    assert.strictEqual(config.customHeaders!['X-Tenant-Region'], 'us-west');
    assert.strictEqual(config.customHeaders!['X-Custom-Header'], 'custom-value');
  });

  // ============== Message Handler Tests ==============

  it('should register and unregister message handlers', async () => {
    const config: TenantConfig = {
      tenantId: 'handler-test',
      pskSecret: 'handler-secret',
      clientId: 'handler-client',
      apiKey: 'handler_api'
    };

    const client = new TenantQuicClient('localhost', 6000, config);
    
    const handler = (message: Message) => {
      console.log('Message received:', message);
    };

    client.onMessage('test.topic', handler);
    client.offMessage('test.topic');

    await client.connect();
    await client.disconnect();
  });

  // ============== Error Handling Tests ==============

  it('should handle errors gracefully', async () => {
    const config: TenantConfig = {
      tenantId: 'error-test',
      pskSecret: 'error-secret',
      clientId: 'error-client',
      apiKey: 'error_api'
    };

    const client = new TenantQuicClient('localhost', 6000, config);

    // Send message without connecting should fail
    try {
      await client.sendMessage({
        topic: 'test',
        payload: {}
      });
      assert.fail('Should have thrown error');
    } catch (error: any) {
      assert(error.message.includes('not established'));
    }
  });

  // ============== Multiple Tenant Isolation Tests ==============

  it('should maintain complete isolation between multiple tenants', async () => {
    const tenants = [
      { id: 'acme', psk: 'acme-secret' },
      { id: 'globex', psk: 'globex-secret' },
      { id: 'initech', psk: 'initech-secret' }
    ];

    const clients = tenants.map(t => 
      new TenantQuicClient('localhost', 6000, {
        tenantId: t.id,
        pskSecret: t.psk,
        clientId: `client-${t.id}`,
        apiKey: `api_${t.id}`
      })
    );

    // Connect all
    for (const client of clients) {
      await client.connect();
    }

    // Verify complete isolation
    const sessionTokens = clients.map(c => c.getSessionToken());
    const connectionIds = clients.map(c => c.getConnectionId());

    // All tokens should be unique
    const uniqueTokens = new Set(sessionTokens);
    assert.strictEqual(uniqueTokens.size, sessionTokens.length);

    // All connection IDs should be unique
    const uniqueIds = new Set(connectionIds);
    assert.strictEqual(uniqueIds.size, connectionIds.length);

    // Disconnect all
    for (const client of clients) {
      await client.disconnect();
    }
  });
});
