/**
 * WebSocket Connection Manager
 *
 * Provides a clean, event-driven interface for WebSocket connections
 * with automatic reconnection, status tracking, and message handling.
 */

export type ConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'error';

export interface ConnectionStatusEvent {
  status: ConnectionStatus;
  userCount?: number;
  connectionId?: string;
  timestamp: number;
}

export interface WebSocketConnectionOptions {
  url: string;
  autoReconnect?: boolean;
  reconnectDelay?: number;
  maxReconnectAttempts?: number;
  pingInterval?: number;
}

export interface WebSocketMessage {
  type: string;
  data?: any;
}

export class WebSocketConnection extends EventTarget {
  private socket: WebSocket | null = null;
  private status: ConnectionStatus = 'disconnected';
  private options: Required<WebSocketConnectionOptions>;
  private reconnectAttempts = 0;
  private reconnectTimer?: number;
  private pingTimer?: number;
  private connectionId = '';
  private userCount = 0;

  constructor(options: WebSocketConnectionOptions) {
    super();

    this.options = {
      autoReconnect: true,
      reconnectDelay: 3000,
      maxReconnectAttempts: 5,
      pingInterval: 30000,
      ...options,
    };
  }

  /**
   * Connect to the WebSocket server
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.socket?.readyState === WebSocket.OPEN) {
        resolve();
        return;
      }

      this.setStatus('connecting');

      try {
        this.socket = new WebSocket(this.options.url);
        this.setupSocketListeners(resolve, reject);
      } catch (error) {
        this.setStatus('error');
        reject(error);
      }
    });
  }

  /**
   * Disconnect from the WebSocket server
   */
  disconnect(): void {
    this.clearTimers();
    this.options.autoReconnect = false; // Disable auto-reconnect for manual disconnect

    if (this.socket) {
      this.socket.close(1000, 'Manual disconnect');
    }
  }

  /**
   * Send a message to the server
   */
  send(message: WebSocketMessage): boolean {
    if (!this.isConnected()) {
      this.dispatchEvent(
        new CustomEvent('error', {
          detail: { error: 'Cannot send message: not connected' },
        })
      );
      return false;
    }

    try {
      const json = JSON.stringify(message);
      this.socket!.send(json);

      this.dispatchEvent(
        new CustomEvent('message-sent', {
          detail: { message },
        })
      );

      return true;
    } catch (error) {
      this.dispatchEvent(
        new CustomEvent('error', {
          detail: { error: `Send error: ${error}` },
        })
      );
      return false;
    }
  }

  /**
   * Send a ping message
   */
  ping(): void {
    this.send({ type: 'Ping' });
  }

  /**
   * Check if currently connected
   */
  isConnected(): boolean {
    return this.socket?.readyState === WebSocket.OPEN;
  }

  /**
   * Get current connection status
   */
  getStatus(): ConnectionStatus {
    return this.status;
  }

  /**
   * Get current user count
   */
  getUserCount(): number {
    return this.userCount;
  }

  /**
   * Get connection ID
   */
  getConnectionId(): string {
    return this.connectionId;
  }

  private setStatus(status: ConnectionStatus): void {
    if (this.status === status) return;

    this.status = status;

    const event: ConnectionStatusEvent = {
      status,
      userCount: this.userCount,
      connectionId: this.connectionId,
      timestamp: Date.now(),
    };

    this.dispatchEvent(
      new CustomEvent('status-change', {
        detail: event,
      })
    );
  }

  private setupSocketListeners(
    resolve: () => void,
    reject: (error: any) => void
  ): void {
    if (!this.socket) return;

    this.socket.onopen = () => {
      this.setStatus('connected');
      this.reconnectAttempts = 0;
      this.setupPingTimer();
      resolve();
    };

    this.socket.onclose = (event) => {
      this.clearTimers();
      this.setStatus('disconnected');
      this.socket = null;

      this.dispatchEvent(
        new CustomEvent('connection-closed', {
          detail: { code: event.code, reason: event.reason },
        })
      );

      // Attempt reconnection if enabled
      if (
        this.options.autoReconnect &&
        this.reconnectAttempts < this.options.maxReconnectAttempts
      ) {
        this.scheduleReconnect();
      }
    };

    this.socket.onerror = (error) => {
      this.setStatus('error');

      this.dispatchEvent(
        new CustomEvent('connection-error', {
          detail: { error },
        })
      );

      if (this.reconnectAttempts === 0) {
        reject(error);
      }
    };

    this.socket.onmessage = (event) => {
      this.handleMessage(event.data);
    };
  }

  private handleMessage(rawMessage: string): void {
    try {
      const response: WebSocketMessage = JSON.parse(rawMessage);

      // Handle built-in message types
      switch (response.type) {
        case 'Welcome':
          this.connectionId = response.data?.connection_id || '';
          break;

        case 'ConnectionStatus':
          this.userCount = response.data?.user_count || 0;
          // Re-emit status change with updated user count
          this.setStatus(this.status);
          break;

        case 'Pong':
          this.dispatchEvent(
            new CustomEvent('pong', {
              detail: { timestamp: Date.now() },
            })
          );
          break;

        case 'Error':
          this.dispatchEvent(
            new CustomEvent('server-error', {
              detail: { error: response.data?.message || 'Server error' },
            })
          );
          break;
      }

      // Always emit the raw message for custom handling
      this.dispatchEvent(
        new CustomEvent('message', {
          detail: { message: response, raw: rawMessage },
        })
      );
    } catch (error) {
      this.dispatchEvent(
        new CustomEvent('parse-error', {
          detail: { error, rawMessage, messageLength: rawMessage.length },
        })
      );
    }
  }

  private scheduleReconnect(): void {
    this.reconnectAttempts++;

    this.dispatchEvent(
      new CustomEvent('reconnecting', {
        detail: {
          attempt: this.reconnectAttempts,
          maxAttempts: this.options.maxReconnectAttempts,
          delay: this.options.reconnectDelay,
        },
      })
    );

    this.reconnectTimer = window.setTimeout(() => {
      this.connect().catch(() => {
        // Connection will be retried automatically if still under max attempts
      });
    }, this.options.reconnectDelay);
  }

  private setupPingTimer(): void {
    if (this.options.pingInterval > 0) {
      this.pingTimer = window.setInterval(() => {
        if (this.isConnected()) {
          this.ping();
        }
      }, this.options.pingInterval);
    }
  }

  private clearTimers(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = undefined;
    }

    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = undefined;
    }
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.disconnect();
    this.clearTimers();
    this.removeAllListeners();
  }

  private removeAllListeners(): void {
    const events = [
      'status-change',
      'message',
      'connection-closed',
      'connection-error',
      'message-sent',
      'error',
      'pong',
      'server-error',
      'parse-error',
      'reconnecting',
    ];
    events.forEach((event) => {
      // Remove all listeners for each event type
      const listeners = (this as any)._listeners?.[event] || [];
      listeners.forEach((listener: any) => {
        this.removeEventListener(event, listener);
      });
    });
  }
}
