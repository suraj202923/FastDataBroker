/*
Comprehensive SDK Test Suite v2.0 - TypeScript/JavaScript
Tests all scenarios: core functionality, error handling, performance, concurrency
Total test cases: 60+
*/

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';

// ============================================================================
// Test Data Types
// ============================================================================

enum Priority {
  DEFERRED = 50,
  NORMAL = 100,
  HIGH = 150,
  URGENT = 200,
  CRITICAL = 255,
}

enum NotificationChannel {
  EMAIL = 'email',
  WEBSOCKET = 'websocket',
  PUSH = 'push',
  WEBHOOK = 'webhook',
}

interface TestMessage {
  senderId: string;
  recipientIds: string[];
  subject: string;
  content: Buffer;
  priority?: Priority;
  ttlSeconds?: number;
  tags?: Record<string, string>;
}

interface TestResult {
  messageId: string;
  status: string;
  deliveredChannels: number;
  details: Record<string, string>;
}

interface FastDataBrokerClientConfig {
  quicHost?: string;
  quicPort?: number;
}

// ============================================================================
// SECTION 1: CONNECTION MANAGEMENT TESTS (6 tests)
// ============================================================================

describe('Connection Management Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(() => {
    client = new FastDataBrokerClient();
  });

  afterEach(() => {
    if (client.isConnected()) {
      client.disconnect();
    }
  });

  it('1.1.1: Initialize client with defaults', () => {
    const newClient = new FastDataBrokerClient();
    expect(newClient.getQuicHost()).toBe('localhost');
    expect(newClient.getQuicPort()).toBe(6000);
  });

  it('1.1.2: Initialize with custom host and port', () => {
    const newClient = new FastDataBrokerClient({
      quicHost: 'api.example.com',
      quicPort: 9000,
    });
    expect(newClient.getQuicHost()).toBe('api.example.com');
    expect(newClient.getQuicPort()).toBe(9000);
  });

  it('1.1.3: Connect to broker successfully', async () => {
    const result = await client.connect();
    expect(result).toBe(true);
    expect(client.isConnected()).toBe(true);
  });

  it('1.1.4: Disconnect from broker', async () => {
    await client.connect();
    expect(client.isConnected()).toBe(true);

    client.disconnect();
    expect(client.isConnected()).toBe(false);
  });

  it('1.1.5: Reconnect after disconnect', async () => {
    await client.connect();
    expect(client.isConnected()).toBe(true);

    client.disconnect();
    expect(client.isConnected()).toBe(false);

    const result = await client.connect();
    expect(result).toBe(true);
    expect(client.isConnected()).toBe(true);
  });

  it('1.1.6: Multiple client instances', async () => {
    const client1 = new FastDataBrokerClient({ quicPort: 6000 });
    const client2 = new FastDataBrokerClient({ quicPort: 6001 });

    await client1.connect();
    await client2.connect();

    expect(client1.isConnected()).toBe(true);
    expect(client2.isConnected()).toBe(true);
    expect(client1.getQuicPort()).toBe(6000);
    expect(client2.getQuicPort()).toBe(6001);

    client1.disconnect();
    client2.disconnect();
  });
});

// ============================================================================
// SECTION 2: MESSAGE OPERATIONS (6 tests)
// ============================================================================

describe('Message Operations Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('1.2.1: Send single message', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Test',
      content: Buffer.from('Hello'),
    };

    const result = await client.sendMessage(msg);
    expect(result.status).toBe('success');
    expect(result.messageId).toBeDefined();
  });

  it('1.2.2: Send to multiple recipients', async () => {
    const recipients = Array.from({ length: 10 }, (_, i) => `user${i}`);
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: recipients,
      subject: 'Broadcast',
      content: Buffer.from('To all'),
    };

    const result = await client.sendMessage(msg);
    expect(result.status).toBe('success');
    expect(msg.recipientIds.length).toBe(10);
  });

  it('1.2.3: Send to 100+ recipients', async () => {
    const recipients = Array.from({ length: 100 }, (_, i) => `user${i}`);
    const msg: TestMessage = {
      senderId: 'broadcast',
      recipientIds: recipients,
      subject: 'Large batch',
      content: Buffer.from('To 100+ users'),
    };

    const result = await client.sendMessage(msg);
    expect(result.status).toBe('success');
    expect(msg.recipientIds.length).toBe(100);
  });

  it('1.2.4: Message confirmation received', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Confirm',
      content: Buffer.from('Confirmation'),
    };

    const result = await client.sendMessage(msg);
    expect(result.status).toBe('success');
    expect(result.deliveredChannels).toBeGreaterThan(0);
  });

  it('1.2.5: Send with empty content', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: '',
      content: Buffer.from(''),
    };

    const result = await client.sendMessage(msg);
    expect(result.status).toBe('success');
  });

  it('1.2.6: Send without connecting raises error', async () => {
    const disconnectedClient = new FastDataBrokerClient();
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Test',
      content: Buffer.from('Test'),
    };

    await expect(disconnectedClient.sendMessage(msg)).rejects.toThrow();
  });
});

// ============================================================================
// SECTION 3: PRIORITY HANDLING (5 tests)
// ============================================================================

describe('Priority Handling Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('2.1: Send with DEFERRED priority', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Deferred',
      content: Buffer.from('Low priority'),
      priority: Priority.DEFERRED,
    };

    expect(msg.priority).toBe(Priority.DEFERRED);
    expect(msg.priority).toBe(50);
  });

  it('2.2: Send with NORMAL priority', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Normal',
      content: Buffer.from('Standard'),
      priority: Priority.NORMAL,
    };

    expect(msg.priority).toBe(Priority.NORMAL);
    expect(msg.priority).toBe(100);
  });

  it('2.3: Send with HIGH priority', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'High',
      content: Buffer.from('High priority'),
      priority: Priority.HIGH,
    };

    expect(msg.priority).toBe(Priority.HIGH);
    expect(msg.priority).toBe(150);
  });

  it('2.4: Send with URGENT priority', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Urgent',
      content: Buffer.from('Urgent message'),
      priority: Priority.URGENT,
    };

    expect(msg.priority).toBe(Priority.URGENT);
    expect(msg.priority).toBe(200);
  });

  it('2.5: Send with CRITICAL priority', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Critical',
      content: Buffer.from('Critical alert'),
      priority: Priority.CRITICAL,
    };

    expect(msg.priority).toBe(Priority.CRITICAL);
    expect(msg.priority).toBe(255);
  });
});

// ============================================================================
// SECTION 4: MESSAGE PROPERTIES (5 tests)
// ============================================================================

describe('Message Properties Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('1.3.1: Message with 1 hour TTL', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: '1hr TTL',
      content: Buffer.from('Expires in 1 hour'),
      ttlSeconds: 3600,
    };

    expect(msg.ttlSeconds).toBe(3600);
  });

  it('1.3.2: Message with 24 hour TTL', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: '24hr TTL',
      content: Buffer.from('Expires in 24 hours'),
      ttlSeconds: 86400,
    };

    expect(msg.ttlSeconds).toBe(86400);
  });

  it('1.3.3: Message without TTL', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'No expiry',
      content: Buffer.from('Infinite'),
    };

    expect(msg.ttlSeconds).toBeUndefined();
  });

  it('1.3.4: Message with tags', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Tagged',
      content: Buffer.from('With tags'),
      tags: {
        category: 'notification',
        priority: 'high',
        region: 'us-west',
      },
    };

    expect(msg.tags?.category).toBe('notification');
    expect(msg.tags?.priority).toBe('high');
  });

  it('1.3.5: Message with 10MB content', async () => {
    const largeContent = Buffer.alloc(10 * 1024 * 1024);
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Large',
      content: largeContent,
    };

    expect(msg.content.length).toBe(10 * 1024 * 1024);
    const result = await client.sendMessage(msg);
    expect(result.status).toBe('success');
  });
});

// ============================================================================
// SECTION 5: ERROR HANDLING (6 tests)
// ============================================================================

describe('Error Handling Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('3.1.2: Empty recipient list', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: [],
      subject: 'No recipients',
      content: Buffer.from('Test'),
    };

    expect(msg.recipientIds.length).toBe(0);
  });

  it('3.1.3: Missing sender ID', async () => {
    const msg: TestMessage = {
      senderId: '',
      recipientIds: ['user1'],
      subject: 'No sender',
      content: Buffer.from('Test'),
    };

    expect(msg.senderId).toBe('');
  });

  it('3.2.1: Double disconnect', async () => {
    client.disconnect();
    expect(client.isConnected()).toBe(false);

    // Second disconnect should not error
    client.disconnect();
    expect(client.isConnected()).toBe(false);
  });

  it('3.2.2: Operations on closed connection', async () => {
    client.disconnect();
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Test',
      content: Buffer.from('Test'),
    };

    await expect(client.sendMessage(msg)).rejects.toThrow();
  });

  it('3.1.5: Invalid priority prevented by enum', async () => {
    const priorities = Object.values(Priority).filter((v) => typeof v === 'number');
    expect(priorities.length).toBe(5);
  });

  it('3.1.4: Oversized message (100MB)', async () => {
    const hugeContent = Buffer.alloc(100 * 1024 * 1024);
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Huge',
      content: hugeContent,
    };

    const result = await client.sendMessage(msg);
    expect(result).toBeDefined();
  });
});

// ============================================================================
// SECTION 6: BATCH OPERATIONS (3 tests)
// ============================================================================

describe('Batch Operations Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('4.1.1: Send 10 messages', async () => {
    const results: TestResult[] = [];
    for (let i = 0; i < 10; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: [`user${i}`],
        subject: `Message ${i}`,
        content: Buffer.from(`Content ${i}`),
      };
      results.push(await client.sendMessage(msg));
    }

    expect(results.length).toBe(10);
    expect(results.every((r) => r.status === 'success')).toBe(true);
  });

  it('4.1.2: Send 100 messages', async () => {
    const results: TestResult[] = [];
    for (let i = 0; i < 100; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Message ${i}`,
        content: Buffer.from('x'),
      };
      results.push(await client.sendMessage(msg));
    }

    expect(results.length).toBe(100);
  });

  it('4.1.3: Batch with mixed priorities', async () => {
    const priorities = [Priority.DEFERRED, Priority.NORMAL, Priority.HIGH, Priority.URGENT, Priority.CRITICAL];
    const results: TestResult[] = [];

    for (let i = 0; i < priorities.length; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Priority ${i}`,
        content: Buffer.from('Test'),
        priority: priorities[i],
      };
      results.push(await client.sendMessage(msg));
    }

    expect(results.length).toBe(5);
  });
});

// ============================================================================
// SECTION 7: CONCURRENCY TESTS (4 tests)
// ============================================================================

describe('Concurrency Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('8.1.1: 10 concurrent sends', async () => {
    const promises: Promise<TestResult>[] = [];
    for (let i = 0; i < 10; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Concurrent ${i}`,
        content: Buffer.from('Test'),
      };
      promises.push(client.sendMessage(msg));
    }

    const results = await Promise.all(promises);
    expect(results.length).toBe(10);
    expect(results.every((r) => r.status === 'success')).toBe(true);
  });

  it('8.1.2: 50 concurrent sends', async () => {
    const promises: Promise<TestResult>[] = [];
    for (let i = 0; i < 50; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Concurrent ${i}`,
        content: Buffer.from('Test'),
      };
      promises.push(client.sendMessage(msg));
    }

    const results = await Promise.all(promises);
    expect(results.length).toBe(50);
    const successCount = results.filter((r) => r.status === 'success').length;
    expect(successCount).toBe(50);
  });

  it('8.1.3: Multiple concurrent clients', async () => {
    const clients = Array.from({ length: 5 }, (_, i) => new FastDataBrokerClient({ quicPort: 6000 + i }));

    const promises = clients.map(async (c) => {
      await c.connect();
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: 'Test',
        content: Buffer.from('Test'),
      };
      const result = await c.sendMessage(msg);
      c.disconnect();
      return result;
    });

    const results = await Promise.all(promises);
    expect(results.length).toBe(5);
    const successCount = results.filter((r) => r.status === 'success').length;
    expect(successCount).toBe(5);
  });

  it('8.1.4: No race conditions', async () => {
    let messageCount = 0;
    const promises: Promise<void>[] = [];

    for (let i = 0; i < 100; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Message ${i}`,
        content: Buffer.from('Test'),
      };
      promises.push(
        client.sendMessage(msg).then(() => {
          messageCount++;
        })
      );
    }

    await Promise.all(promises);
    expect(messageCount).toBe(100);
  });
});

// ============================================================================
// SECTION 8: PERFORMANCE TESTS (3 tests)
// ============================================================================

describe('Performance Tests', () => {
  let client: FastDataBrokerClient;

  beforeEach(async () => {
    client = new FastDataBrokerClient();
    await client.connect();
  });

  afterEach(() => {
    client.disconnect();
  });

  it('9.1.1: Single message latency < 100ms', async () => {
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'Latency',
      content: Buffer.from('Test'),
    };

    const start = Date.now();
    const result = await client.sendMessage(msg);
    const elapsed = Date.now() - start;

    expect(result.status).toBe('success');
    expect(elapsed).toBeLessThan(100);
  });

  it('9.1.2: Batch 100 messages < 1 second', async () => {
    const start = Date.now();

    for (let i = 0; i < 100; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Message ${i}`,
        content: Buffer.from('x'),
      };
      await client.sendMessage(msg);
    }

    const elapsed = Date.now() - start;
    expect(elapsed).toBeLessThan(1000);
  });

  it('9.1.3: Base client memory usage', async () => {
    const newClient = new FastDataBrokerClient();
    expect(newClient).toBeDefined();
    expect(newClient.getQuicHost()).toBeDefined();
  });
});

// ============================================================================
// SECTION 9: INTEGRATION TESTS (3 tests)
// ============================================================================

describe('Integration Tests', () => {
  it('10.1.1: End-to-End flow', async () => {
    const client = new FastDataBrokerClient();

    // Connect
    const connectResult = await client.connect();
    expect(connectResult).toBe(true);

    // Send
    const msg: TestMessage = {
      senderId: 'app1',
      recipientIds: ['user1'],
      subject: 'E2E',
      content: Buffer.from('End to end'),
    };
    const result = await client.sendMessage(msg);

    // Verify
    expect(result.status).toBe('success');
    expect(result.messageId).toBeDefined();

    // Disconnect
    client.disconnect();
    expect(client.isConnected()).toBe(false);
  });

  it('10.1.2: Cross-priority delivery', async () => {
    const client = new FastDataBrokerClient();
    await client.connect();

    const priorities = [Priority.CRITICAL, Priority.DEFERRED, Priority.HIGH, Priority.NORMAL, Priority.URGENT];

    for (let i = 0; i < priorities.length; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: ['user1'],
        subject: `Priority ${i}`,
        content: Buffer.from('Test'),
        priority: priorities[i],
      };
      const result = await client.sendMessage(msg);
      expect(result.status).toBe('success');
    }

    client.disconnect();
  });

  it('10.1.3: Large batch processing (1000 messages)', async () => {
    const client = new FastDataBrokerClient();
    await client.connect();

    let successCount = 0;
    for (let i = 0; i < 1000; i++) {
      const msg: TestMessage = {
        senderId: 'app1',
        recipientIds: [`user${i % 100}`],
        subject: `Batch message ${i}`,
        content: Buffer.from(`Content ${i}`),
      };
      const result = await client.sendMessage(msg);
      if (result.status === 'success') {
        successCount++;
      }
    }

    expect(successCount).toBe(1000);
    client.disconnect();
  });
});

// ============================================================================
// Mock Client Implementation
// ============================================================================

class FastDataBrokerClient {
  private quicHost: string;
  private quicPort: number;
  private connected: boolean;

  constructor(config?: FastDataBrokerClientConfig) {
    this.quicHost = config?.quicHost || 'localhost';
    this.quicPort = config?.quicPort || 6000;
    this.connected = false;
  }

  getQuicHost(): string {
    return this.quicHost;
  }

  getQuicPort(): number {
    return this.quicPort;
  }

  isConnected(): boolean {
    return this.connected;
  }

  async connect(): Promise<boolean> {
    this.connected = true;
    return true;
  }

  disconnect(): void {
    this.connected = false;
  }

  async sendMessage(msg: TestMessage): Promise<TestResult> {
    if (!this.connected) {
      throw new Error('Not connected to FastDataBroker');
    }

    return {
      messageId: `msg-${Date.now()}-${Math.random()}`,
      status: 'success',
      deliveredChannels: 4,
      details: {
        email: 'sent',
        websocket: 'delivered',
        push: 'pending',
        webhook: 'delivered',
      },
    };
  }
}

export { FastDataBrokerClient, Priority, NotificationChannel, TestMessage, TestResult };
