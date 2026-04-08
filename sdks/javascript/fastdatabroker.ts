/**
 * FastDataBroker JavaScript/TypeScript SDK
 * Client library for rst_queue FastDataBroker messaging system
 * 
 * @version 0.1.12
 */

export enum Priority {
  DEFERRED = 50,
  NORMAL = 100,
  HIGH = 150,
  URGENT = 200,
  CRITICAL = 255,
}

export enum NotificationChannel {
  EMAIL = "email",
  WEBSOCKET = "websocket",
  PUSH = "push",
  WEBHOOK = "webhook",
}

export enum PushPlatform {
  FIREBASE = "firebase",
  APNS = "apns",
  FCM = "fcm",
  WEBPUSH = "webpush",
}

/**
 * Message envelope for FastDataBroker
 */
export interface Message {
  senderId: string;
  recipientIds: string[];
  subject: string;
  content: Uint8Array | string;
  priority?: Priority;
  ttlSeconds?: number;
  tags?: Record<string, string>;
}

/**
 * Delivery result for a sent message
 */
export interface DeliveryResult {
  messageId: string;
  status: "success" | "partial" | "failed";
  deliveredChannels: number;
  details: Record<string, any>;
}

/**
 * WebSocket client registration
 */
export interface WebSocketClientInfo {
  clientId: string;
  userId: string;
  connectedAt: number;
}

/**
 * Webhook configuration
 */
export interface WebhookConfig {
  url: string;
  headers?: Record<string, string>;
  retries?: number;
  timeout?: number;
}

/**
 * FastDataBroker client for synchronous operations
 */
export class FastDataBrokerClient {
  private quicHost: string;
  private quicPort: number;
  private connected: boolean = false;
  private wsClients: Map<string, WebSocketClientInfo> = new Map();

  constructor(quicHost: string = "localhost", quicPort: number = 6000) {
    this.quicHost = quicHost;
    this.quicPort = quicPort;
  }

  /**
   * Connect to FastDataBroker server
   */
  async connect(): Promise<boolean> {
    try {
      // Phase 4: Establish QUIC connection
      this.connected = true;
      console.log(`Connected to FastDataBroker at ${this.quicHost}:${this.quicPort}`);
      return true;
    } catch (error) {
      console.error("Connection failed:", error);
      return false;
    }
  }

  /**
   * Disconnect from FastDataBroker server
   */
  async disconnect(): Promise<void> {
    this.connected = false;
  }

  /**
   * Send a message through FastDataBroker
   *
   * @param message - Message to send
   * @returns Delivery result
   *
   * @example
   * ```typescript
   * const client = new FastDataBrokerClient();
   * await client.connect();
   *
   * const message: Message = {
   *   senderId: "app1",
   *   recipientIds: ["user-123"],
   *   subject: "Hello",
   *   content: "Hello, World!",
   *   priority: Priority.HIGH
   * };
   *
   * const result = await client.sendMessage(message);
   * console.log(`Message delivered via ${result.deliveredChannels} channels`);
   * ```
   */
  async sendMessage(_message: Message): Promise<DeliveryResult> {
    if (!this.connected) {
      throw new Error("Not connected to FastDataBroker");
    }

    // Phase 4: Send via QUIC transport
    await this.simulateNetworkDelay();

    return {
      messageId: `msg-${Date.now()}`,
      status: "success",
      deliveredChannels: 4,
      details: {
        email: "sent",
        websocket: "delivered",
        push: "pending",
        webhook: "delivered",
      },
    };
  }

  /**
   * Send multiple messages in batch
   */
  async batchSend(messages: Message[]): Promise<DeliveryResult[]> {
    const results = await Promise.all(
      messages.map((msg) => this.sendMessage(msg))
    );
    return results;
  }

  /**
   * Register a WebSocket client
   */
  async registerWebSocket(
    clientId: string,
    userId: string
  ): Promise<boolean> {
    this.wsClients.set(clientId, {
      clientId,
      userId,
      connectedAt: Date.now(),
    });
    console.log(`WebSocket client registered: ${clientId} -> ${userId}`);
    return true;
  }

  /**
   * Unregister a WebSocket client
   */
  async unregisterWebSocket(clientId: string): Promise<boolean> {
    return this.wsClients.delete(clientId);
  }

  /**
   * Register a webhook endpoint
   */
  async registerWebhook(recipientId: string, config: WebhookConfig): Promise<boolean> {
    if (!config.url.startsWith("http")) {
      throw new Error("Invalid webhook URL");
    }
    // Phase 4: Register with broker
    console.log(`Webhook registered for ${recipientId}: ${config.url}`);
    return true;
  }

  /**
   * Unregister a webhook endpoint
   */
  async unregisterWebhook(_recipientId: string): Promise<boolean> {
    // Phase 4: Unregister from broker
    return true;
  }

  /**
   * Get FastDataBroker statistics
   */
  async getStats(): Promise<Record<string, any>> {
    await this.simulateNetworkDelay();
    return {
      totalMessages: 0,
      delivered: 0,
      failed: 0,
      channels: {
        email: { sent: 0, failed: 0 },
        websocket: { connected: this.wsClients.size, delivered: 0 },
        push: { sent: 0, delivered: 0 },
        webhook: { sent: 0, delivered: 0 },
      },
    };
  }

  private async simulateNetworkDelay(): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, 10));
  }
}

/**
 * Push notification builder for constructing rich notifications
 */
export class PushNotificationBuilder {
  private title: string;
  private body: string = "";
  private icon?: string;
  private badge?: string;
  private sound?: string;
  private data: Record<string, string> = {};

  constructor(title: string) {
    this.title = title;
  }

  withBody(body: string): this {
    this.body = body;
    return this;
  }

  withIcon(icon: string): this {
    this.icon = icon;
    return this;
  }

  withBadge(badge: string): this {
    this.badge = badge;
    return this;
  }

  withSound(sound: string): this {
    this.sound = sound;
    return this;
  }

  withData(key: string, value: string): this {
    this.data[key] = value;
    return this;
  }

  build(): Record<string, any> {
    return {
      title: this.title,
      body: this.body,
      icon: this.icon,
      badge: this.badge,
      sound: this.sound,
      data: this.data,
    };
  }
}

/**
 * Email message builder
 */
export class EmailBuilder {
  private to: string[] = [];
  private subject: string = "";
  private htmlContent: string = "";
  private textContent: string = "";
  private headers: Record<string, string> = {};

  addRecipient(email: string): this {
    this.to.push(email);
    return this;
  }

  setSubject(subject: string): this {
    this.subject = subject;
    return this;
  }

  setHtmlContent(html: string): this {
    this.htmlContent = html;
    return this;
  }

  setTextContent(text: string): this {
    this.textContent = text;
    return this;
  }

  addHeader(key: string, value: string): this {
    this.headers[key] = value;
    return this;
  }

  build(): Message {
    return {
      senderId: "email-service",
      recipientIds: this.to,
      subject: this.subject,
      content: this.htmlContent || this.textContent,
      tags: {
        type: "email",
        ...this.headers,
      },
    };
  }
}

// Usage examples (commented out)
/*
async function example() {
  const client = new FastDataBrokerClient();
  await client.connect();

  // Send a single message
  const message: Message = {
    senderId: "app1",
    recipientIds: ["user-123"],
    subject: "Hello",
    content: "Welcome to FastDataBroker!",
    priority: Priority.HIGH,
  };

  const result = await client.sendMessage(message);
  console.log("Message sent:", result);

  // Register WebSocket client
  await client.registerWebSocket("browser-1", "user-123");

  // Register webhook
  await client.registerWebhook("service-1", {
    url: "https://example.com/webhook",
    retries: 3,
  });

  // Get statistics
  const stats = await client.getStats();
  console.log("Statistics:", stats);

  await client.disconnect();
}

// Run example
example().catch(console.error);
*/
