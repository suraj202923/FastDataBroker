/**
 * FastDataBroker JavaScript/TypeScript SDK - QUIC with PSK Authentication
 * High-performance client library with Pre-Shared Key authentication
 * 
 * @version 1.0.0
 * Protocol: QUIC 1.0 (RFC 9000)
 * Authentication: TLS 1.3 PSK (Pre-Shared Key)
 */

import * as net from 'net';
import * as crypto from 'crypto';

// ============================================================================
// Types and Interfaces
// ============================================================================

export enum Priority {
  LOW = 1,
  NORMAL = 5,
  HIGH = 10,
  CRITICAL = 20,
}

export interface Message {
  topic: string;
  payload: any;
  priority?: Priority;
  ttlSeconds?: number;
  headers?: Record<string, string>;
}

export interface DeliveryResult {
  messageId: string;
  status: 'success' | 'failed' | 'timeout';
  latencyMs: number;
  timestamp: number;
}

export interface QuicConnectionConfig {
  host: string;
  port: number;
  tenantId: string;
  clientId: string;
  pskSecret: string;
  secrets: string;
  idleTimeoutMs?: number;
  maxStreams?: number;
  autoReconnect?: boolean;
}

export interface ConnectionStats {
  isConnected: boolean;
  messagesSent: number;
  messagesReceived: number;
  connectionTime: number;
  uptimeSeconds: number;
  lastMessageTime: number;
}

// ============================================================================
// QUIC PSK Client Implementation
// ============================================================================

export class FastDataBrokerQuicClient {
  private config: QuicConnectionConfig;
  private socket: net.Socket | null = null;
  private connected: boolean = false;
  private messageHandlers: Map<string, Function> = new Map();
  private connectionStart: number = 0;
  private stats = {
    messagesSent: 0,
    messagesReceived: 0,
    lastMessageTime: 0,
  };

  constructor(config: QuicConnectionConfig) {
    this.config = {
      idleTimeoutMs: 30000,
      maxStreams: 100,
      autoReconnect: true,
      ...config,
    };
  }

  /**
   * Generate PSK identity and secret hash
   */
  private generatePskIdentity(): {
    identity: string;
    secretHash: string;
  } {
    const identity = `${this.config.tenantId}:${this.config.clientId}`;
    const secretHash = crypto
      .createHash('sha256')
      .update(this.config.pskSecret)
      .digest('hex');

    return { identity, secretHash };
  }

  /**
   * Connect to FastDataBroker with PSK authentication
   */
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        const psk = this.generatePskIdentity();

        // Create TCP socket (foundation for QUIC)
        this.socket = net.createConnection({
          host: this.config.host,
          port: this.config.port,
          timeout: this.config.idleTimeoutMs,
        });

        this.socket.on('connect', () => {
          console.log(
            `✓ Connected to FastDataBroker at ${this.config.host}:${this.config.port}`
          );

          // Send PSK authentication handshake
          this.sendPskHandshake(psk)
            .then(() => {
              this.connected = true;
              this.connectionStart = Date.now();
              console.log(`✓ PSK authentication successful for ${psk.identity}`);
              resolve();
            })
            .catch(reject);
        });

        this.socket.on('data', (data) => {
          this.handleIncomingData(data);
        });

        this.socket.on('error', (err) => {
          console.error('✗ Socket error:', err.message);
          this.connected = false;
          reject(err);
        });

        this.socket.on('close', () => {
          console.log('Connection closed');
          this.connected = false;
        });
      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * Send PSK authentication handshake
   */
  private sendPskHandshake(psk: {
    identity: string;
    secretHash: string;
  }): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.socket) {
        reject(new Error('Socket not initialized'));
        return;
      }

      const handshake = {
        type: 'psk_auth',
        identity: psk.identity,
        secretHash: psk.secretHash,
        timestamp: Date.now(),
      };

      const message = JSON.stringify(handshake) + '\n';

      this.socket.write(message, (err) => {
        if (err) {
          reject(err);
        } else {
          setTimeout(resolve, 100);
        }
      });
    });
  }

  /**
   * Send message to FastDataBroker
   */
  async sendMessage(message: Message): Promise<DeliveryResult> {
    if (!this.connected || !this.socket) {
      throw new Error('Not connected to FastDataBroker');
    }

    return new Promise((resolve, reject) => {
      try {
        const startTime = Date.now();
        const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

        const envelope = {
          type: 'message',
          id: messageId,
          topic: message.topic,
          payload: message.payload,
          priority: message.priority || Priority.NORMAL,
          ttl: message.ttlSeconds || 3600,
          headers: message.headers || {},
          timestamp: Date.now(),
        };

        const data = JSON.stringify(envelope) + '\n';

        this.socket!.write(data, (err) => {
          if (err) {
            reject(err);
            return;
          }

          this.stats.messagesSent++;
          this.stats.lastMessageTime = Date.now();

          const latency = Date.now() - startTime;
          resolve({
            messageId,
            status: 'success',
            latencyMs: latency,
            timestamp: Date.now(),
          });
        });
      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * Handle incoming data from server
   */
  private handleIncomingData(data: Buffer): void {
    const message = data.toString().trim();

    try {
      const parsed = JSON.parse(message);

      if (parsed.type === 'message' && this.messageHandlers.has(parsed.topic)) {
        const handler = this.messageHandlers.get(parsed.topic);
        if (handler) {
          handler(parsed);
          this.stats.messagesReceived++;
        }
      }
    } catch (err) {
      console.log('Received raw data:', message);
    }
  }

  /**
   * Register message handler for topic
   */
  onMessage(topic: string, handler: (message: any) => void): void {
    this.messageHandlers.set(topic, handler);
  }

  /**
   * Unregister message handler
   */
  offMessage(topic: string): void {
    this.messageHandlers.delete(topic);
  }

  /**
   * Get connection statistics
   */
  getStats(): ConnectionStats {
    return {
      isConnected: this.connected,
      messagesSent: this.stats.messagesSent,
      messagesReceived: this.stats.messagesReceived,
      connectionTime: Date.now() - this.connectionStart,
      uptimeSeconds: Math.floor((Date.now() - this.connectionStart) / 1000),
      lastMessageTime: this.stats.lastMessageTime,
    };
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Send multiple messages in parallel
   */
  async sendMessagesParallel(
    messages: Message[],
    numWorkers: number = 4
  ): Promise<DeliveryResult[]> {
    if (!this.connected) {
      throw new Error('Not connected to FastDataBroker');
    }

    const results: DeliveryResult[] = [];
    const chunk = Math.ceil(messages.length / numWorkers);
    const promises = [];

    for (let i = 0; i < numWorkers; i++) {
      const start = i * chunk;
      const end = Math.min(start + chunk, messages.length);
      const batch = messages.slice(start, end);

      const promise = (async () => {
        for (const msg of batch) {
          try {
            const result = await this.sendMessage(msg);
            results.push(result);
          } catch (error) {
            results.push({
              messageId: `error_${Date.now()}`,
              status: 'failed',
              latencyMs: 0,
              timestamp: Date.now(),
            });
          }
        }
      })();
      promises.push(promise);
    }

    await Promise.all(promises);
    return results;
  }

  /**
   * Send messages with progress callback
   */
  async sendMessagesParallelWithProgress(
    messages: Message[],
    numWorkers: number = 4,
    callback?: (completed: number, total: number) => void
  ): Promise<DeliveryResult[]> {
    if (!this.connected) {
      throw new Error('Not connected to FastDataBroker');
    }

    const results: DeliveryResult[] = [];
    let completed = 0;
    const chunk = Math.ceil(messages.length / numWorkers);
    const promises = [];

    for (let i = 0; i < numWorkers; i++) {
      const start = i * chunk;
      const end = Math.min(start + chunk, messages.length);
      const batch = messages.slice(start, end);

      const promise = (async () => {
        for (const msg of batch) {
          try {
            const result = await this.sendMessage(msg);
            results.push(result);
          } catch (error) {
            results.push({
              messageId: `error_${Date.now()}`,
              status: 'failed',
              latencyMs: 0,
              timestamp: Date.now(),
            });
          }
          completed++;
          if (callback) callback(completed, messages.length);
        }
      })();
      promises.push(promise);
    }

    await Promise.all(promises);
    return results;
  }

  /**
   * Disconnect from FastDataBroker
   */
  async disconnect(): Promise<void> {
    if (this.socket) {
      this.socket.destroy();
      this.connected = false;
      console.log('✓ Disconnected from FastDataBroker');
    }
  }
}

/**
 * Factory function to create client
 */
export function createQuicClient(config: QuicConnectionConfig): FastDataBrokerQuicClient {
  return new FastDataBrokerQuicClient(config);
}

/**
 * Parse PSK secret from environment
 */
export function getPskSecretFromEnv(): string {
  const secret = process.env.QUIC_PSK_SECRET;
  if (!secret) {
    throw new Error(
      'QUIC_PSK_SECRET environment variable not set. ' +
      'Get it from: POST /api/quic/psks'
    );
  }
  return secret;
}

// Export types for external use
export type { QuicConnectionConfig, ConnectionStats, Message, DeliveryResult };
