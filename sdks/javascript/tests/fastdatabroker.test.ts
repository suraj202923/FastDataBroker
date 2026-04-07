import {
  FastDataBrokerClient,
  Priority,
  NotificationChannel,
  Message,
  DeliveryResult,
} from '../src/index';

describe('FastDataBroker SDK', () => {
  let client: FastDataBrokerClient;

  beforeEach(() => {
    client = new FastDataBrokerClient('localhost', 6000);
  });

  test('should create client instance', () => {
    expect(client).toBeDefined();
    expect(client).toBeInstanceOf(FastDataBrokerClient);
  });

  test('should define Priority enum', () => {
    expect(Priority.NORMAL).toEqual(100);
    expect(Priority.HIGH).toEqual(150);
    expect(Priority.URGENT).toEqual(200);
    expect(Priority.CRITICAL).toEqual(255);
  });

  test('should define NotificationChannel enum', () => {
    expect(NotificationChannel.EMAIL).toEqual('email');
    expect(NotificationChannel.WEBSOCKET).toEqual('websocket');
    expect(NotificationChannel.PUSH).toEqual('push');
    expect(NotificationChannel.WEBHOOK).toEqual('webhook');
  });

  test('should connect to broker', async () => {
    const connected = await client.connect();
    expect(connected).toBe(true);
  });

  test('should send message after connect', async () => {
    await client.connect();

    const message: Message = {
      senderId: 'test-app',
      recipientIds: ['user-123'],
      subject: 'Test Message',
      content: 'Test content',
      priority: Priority.HIGH,
    };

    const result = await client.sendMessage(message);
    expect(result.status).toBe('success');
    expect(result.deliveredChannels).toBeGreaterThan(0);
  });

  test('should throw error when sending without connection', async () => {
    const message: Message = {
      senderId: 'test-app',
      recipientIds: ['user-123'],
      subject: 'Test Message',
      content: 'Test content',
    };

    await expect(client.sendMessage(message)).rejects.toThrow(
      'Not connected to FastDataBroker'
    );
  });

  test('should register and unregister WebSocket client', async () => {
    await client.connect();
    const registered = await client.registerWebSocket('ws-client-1', 'user-1');
    expect(registered).toBe(true);

    const unregistered = await client.unregisterWebSocket('ws-client-1');
    expect(unregistered).toBe(true);
  });

  test('should register webhook', async () => {
    await client.connect();
    const registered = await client.registerWebhook('user-123', {
      url: 'https://example.com/webhook',
    });
    expect(registered).toBe(true);
  });

  test('should reject invalid webhook URL', async () => {
    await client.connect();
    await expect(
      client.registerWebhook('user-123', {
        url: 'not-a-url',
      })
    ).rejects.toThrow('Invalid webhook URL');
  });

  test('should batch send messages', async () => {
    await client.connect();

    const messages: Message[] = [
      {
        senderId: 'app1',
        recipientIds: ['user-1'],
        subject: 'Message 1',
        content: 'Content 1',
      },
      {
        senderId: 'app1',
        recipientIds: ['user-2'],
        subject: 'Message 2',
        content: 'Content 2',
      },
    ];

    const results = await client.batchSend(messages);
    expect(results).toHaveLength(2);
    expect(results.every((r) => r.status === 'success')).toBe(true);
  });

  afterEach(async () => {
    await client.disconnect();
  });
});
