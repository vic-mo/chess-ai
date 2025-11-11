/**
 * WebSocket Connection Manager
 *
 * Manages a persistent WebSocket connection with:
 * - Auto-reconnection with exponential backoff
 * - Heartbeat/ping to detect silent disconnections
 * - Request queuing during reconnection
 * - Connection state tracking
 */

import { logger } from '../utils/logger';

export type ConnectionState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'failed';

export interface ConnectionStatusEvent {
  state: ConnectionState;
  error?: string;
}

type MessageHandler = (data: any) => void;
type StatusHandler = (event: ConnectionStatusEvent) => void;

export class WebSocketConnectionManager {
  private ws: WebSocket | null = null;
  private url: string;
  private state: ConnectionState = 'disconnected';
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000; // Start with 1s
  private maxReconnectDelay = 8000; // Max 8s
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private heartbeatInterval = 30000; // 30s
  private heartbeatTimeout = 5000; // 5s to respond
  private awaitingPong = false;

  private messageHandlers = new Set<MessageHandler>();
  private statusHandlers = new Set<StatusHandler>();
  private pendingMessages: string[] = [];

  constructor(url: string) {
    this.url = url;
  }

  /**
   * Connect to WebSocket server
   */
  connect(): Promise<void> {
    if (this.state === 'connected' || this.state === 'connecting') {
      logger.log('[ConnectionManager] Already connected or connecting');
      return Promise.resolve();
    }

    return new Promise((resolve, reject) => {
      this.setState('connecting');

      try {
        logger.log('[ConnectionManager] Connecting to:', this.url);
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
          logger.log('[ConnectionManager] Connected');
          this.reconnectAttempts = 0;
          this.reconnectDelay = 1000;
          this.setState('connected');
          this.startHeartbeat();
          this.flushPendingMessages();
          resolve();
        };

        this.ws.onmessage = (ev) => {
          // Check if this is a pong response
          if (ev.data === 'pong') {
            logger.debug('[ConnectionManager] Received pong');
            this.awaitingPong = false;
            return;
          }

          // Handle regular messages
          try {
            const data = JSON.parse(ev.data);
            this.notifyMessageHandlers(data);
          } catch (e) {
            logger.error('[ConnectionManager] Failed to parse message:', e);
          }
        };

        this.ws.onerror = (error) => {
          logger.error('[ConnectionManager] WebSocket error:', error);
        };

        this.ws.onclose = (event) => {
          logger.log('[ConnectionManager] Connection closed:', event.code, event.reason);
          this.stopHeartbeat();

          // Don't reconnect if close was clean or if we're shutting down
          if (event.code === 1000 || this.state === 'disconnected') {
            this.setState('disconnected');
          } else {
            this.attemptReconnect();
          }
        };
      } catch (error) {
        logger.error('[ConnectionManager] Failed to create WebSocket:', error);
        this.setState('failed', error instanceof Error ? error.message : String(error));
        reject(error);
      }
    });
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    logger.log('[ConnectionManager] Disconnecting...');
    this.setState('disconnected');
    this.stopHeartbeat();
    this.clearReconnectTimer();

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
  }

  /**
   * Send a message (queues if not connected)
   */
  send(data: any): void {
    const message = JSON.stringify(data);

    if (this.state === 'connected' && this.ws?.readyState === WebSocket.OPEN) {
      logger.debug('[ConnectionManager] Sending message:', data);
      this.ws.send(message);
    } else {
      logger.log('[ConnectionManager] Queueing message (not connected):', data);
      this.pendingMessages.push(message);

      // Try to connect if not already connecting
      if (this.state === 'disconnected' || this.state === 'failed') {
        this.connect().catch((error) => {
          logger.error('[ConnectionManager] Auto-connect failed:', error);
        });
      }
    }
  }

  /**
   * Get current connection state
   */
  getState(): ConnectionState {
    return this.state;
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.state === 'connected' && this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Register message handler
   */
  onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  /**
   * Register status handler
   */
  onStatusChange(handler: StatusHandler): () => void {
    this.statusHandlers.add(handler);
    // Immediately notify of current state
    handler({ state: this.state });
    return () => this.statusHandlers.delete(handler);
  }

  // Private methods

  private setState(state: ConnectionState, error?: string): void {
    this.state = state;
    logger.log('[ConnectionManager] State changed to:', state, error ? `(${error})` : '');
    this.notifyStatusHandlers({ state, error });
  }

  private notifyMessageHandlers(data: any): void {
    this.messageHandlers.forEach((handler) => {
      try {
        handler(data);
      } catch (e) {
        logger.error('[ConnectionManager] Message handler error:', e);
      }
    });
  }

  private notifyStatusHandlers(event: ConnectionStatusEvent): void {
    this.statusHandlers.forEach((handler) => {
      try {
        handler(event);
      } catch (e) {
        logger.error('[ConnectionManager] Status handler error:', e);
      }
    });
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      logger.error('[ConnectionManager] Max reconnect attempts reached');
      this.setState('failed', 'Max reconnect attempts reached');
      return;
    }

    this.setState('reconnecting');
    this.reconnectAttempts++;

    // Exponential backoff with jitter
    const jitter = Math.random() * 1000;
    const delay = Math.min(this.reconnectDelay + jitter, this.maxReconnectDelay);

    logger.log(
      `[ConnectionManager] Reconnecting in ${Math.round(delay)}ms (attempt ${this.reconnectAttempts})`,
    );

    this.reconnectTimer = setTimeout(() => {
      this.connect().catch((error) => {
        logger.error('[ConnectionManager] Reconnect failed:', error);
        // attemptReconnect will be called again via onclose
      });
    }, delay);

    // Increase delay for next attempt
    this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();

    this.heartbeatTimer = setInterval(() => {
      if (!this.isConnected()) {
        this.stopHeartbeat();
        return;
      }

      // Check if previous ping was answered
      if (this.awaitingPong) {
        logger.warn('[ConnectionManager] Heartbeat timeout - no pong received');
        this.ws?.close(1000, 'Heartbeat timeout');
        return;
      }

      // Send ping
      logger.debug('[ConnectionManager] Sending ping');
      this.awaitingPong = true;
      this.ws?.send('ping');

      // Set timeout for pong response
      setTimeout(() => {
        if (this.awaitingPong) {
          logger.warn('[ConnectionManager] Pong timeout');
          this.ws?.close(1000, 'Pong timeout');
        }
      }, this.heartbeatTimeout);
    }, this.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    this.awaitingPong = false;
  }

  private flushPendingMessages(): void {
    if (this.pendingMessages.length === 0) {
      return;
    }

    logger.log(`[ConnectionManager] Flushing ${this.pendingMessages.length} pending messages`);

    while (this.pendingMessages.length > 0 && this.isConnected()) {
      const message = this.pendingMessages.shift();
      if (message && this.ws) {
        this.ws.send(message);
      }
    }
  }
}
