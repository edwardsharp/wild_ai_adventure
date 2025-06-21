/**
 * WebSocket Message Handler Web Component
 *
 * A web component that manages WebSocket connections, handles incoming messages,
 * and provides methods for sending messages. Integrates with the WebSocket client
 * and provides a simple interface for media blob handling.
 */

import { LitElement, html, css, PropertyValueMap } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';

// Import types from the clientlib
interface MediaBlob {
  id: string;
  data?: number[];
  sha256: string;
  size?: number;
  mime?: string;
  source_client_id?: string;
  local_path?: string;
  metadata: Record<string, any>;
  created_at: string;
  updated_at: string;
}

interface WebSocketMessage {
  type: 'Ping' | 'GetMediaBlobs' | 'UploadMediaBlob' | 'GetMediaBlob';
  data?: any;
}

interface WebSocketResponse {
  type: 'Welcome' | 'Pong' | 'MediaBlobs' | 'MediaBlob' | 'Error' | 'ConnectionStatus';
  data?: any;
}

enum ConnectionStatus {
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Connected = 'connected',
  Error = 'error',
}

@customElement('websocket-handler')
export class WebSocketHandlerComponent extends LitElement {
  static styles = css`
    :host {
      display: block;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }

    .container {
      padding: 16px;
      border: 1px solid #e5e7eb;
      border-radius: 8px;
      background: #f9fafb;
    }

    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 16px;
    }

    .title {
      font-size: 18px;
      font-weight: 600;
      color: #111827;
    }

    .controls {
      display: flex;
      gap: 8px;
    }

    button {
      padding: 6px 12px;
      border: 1px solid #d1d5db;
      border-radius: 6px;
      background: white;
      color: #374151;
      font-size: 14px;
      cursor: pointer;
      transition: all 0.2s;
    }

    button:hover {
      background: #f3f4f6;
      border-color: #9ca3af;
    }

    button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    button.primary {
      background: #3b82f6;
      color: white;
      border-color: #3b82f6;
    }

    button.primary:hover {
      background: #2563eb;
      border-color: #2563eb;
    }

    .status-section {
      margin-bottom: 16px;
    }

    .debug-log {
      background: #1f2937;
      color: #f3f4f6;
      padding: 12px;
      border-radius: 6px;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      font-size: 12px;
      max-height: 300px;
      overflow-y: auto;
      white-space: pre-wrap;
      word-break: break-all;
    }

    .media-blobs {
      margin-top: 16px;
    }

    .media-blob {
      padding: 12px;
      border: 1px solid #e5e7eb;
      border-radius: 6px;
      margin-bottom: 8px;
      background: white;
    }

    .media-blob-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
    }

    .media-blob-id {
      font-family: monospace;
      font-size: 12px;
      color: #6b7280;
    }

    .media-blob-info {
      font-size: 14px;
      color: #374151;
    }

    .media-blob-meta {
      font-size: 12px;
      color: #6b7280;
      margin-top: 4px;
    }

    .empty-state {
      text-align: center;
      color: #6b7280;
      font-style: italic;
      padding: 32px;
    }

    .error-message {
      background: #fef2f2;
      border: 1px solid #fecaca;
      color: #dc2626;
      padding: 12px;
      border-radius: 6px;
      margin-bottom: 16px;
    }
  `;

  @property({ type: String })
  websocketUrl: string = '';

  @property({ type: Boolean })
  autoConnect: boolean = true;

  @property({ type: Boolean })
  showDebugLog: boolean = true;

  @state()
  private status: ConnectionStatus = ConnectionStatus.Disconnected;

  @state()
  private socket: WebSocket | null = null;

  @state()
  private debugLog: string[] = [];

  @state()
  private mediaBlobs: MediaBlob[] = [];

  @state()
  private errorMessage: string = '';

  @state()
  private userCount: number = 0;

  @state()
  private connectionId: string = '';

  connectedCallback() {
    super.connectedCallback();
    if (this.autoConnect && this.websocketUrl) {
      this.connect();
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.disconnect();
  }

  private log(message: string, data?: any): void {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = data
      ? `[${timestamp}] ${message}: ${JSON.stringify(data, null, 2)}`
      : `[${timestamp}] ${message}`;

    this.debugLog = [...this.debugLog.slice(-99), logEntry]; // Keep last 100 entries
    console.log('[WebSocketHandler]', message, data);
  }

  private setStatus(status: ConnectionStatus): void {
    if (this.status !== status) {
      this.status = status;
      this.log(`Status changed to: ${status}`);
      this.dispatchEvent(new CustomEvent('status-change', {
        detail: { status },
        bubbles: true
      }));
    }
  }

  private setError(message: string): void {
    this.errorMessage = message;
    this.log(`Error: ${message}`);
  }

  private clearError(): void {
    this.errorMessage = '';
  }

  connect(): void {
    if (!this.websocketUrl) {
      this.setError('WebSocket URL not provided');
      return;
    }

    if (this.socket?.readyState === WebSocket.OPEN) {
      this.log('Already connected');
      return;
    }

    this.clearError();
    this.setStatus(ConnectionStatus.Connecting);
    this.log(`Connecting to ${this.websocketUrl}`);

    try {
      this.socket = new WebSocket(this.websocketUrl);
      this.setupSocketListeners();
    } catch (error) {
      this.setError(`Connection failed: ${error}`);
      this.setStatus(ConnectionStatus.Error);
    }
  }

  disconnect(): void {
    this.log('Disconnecting...');

    if (this.socket) {
      this.socket.close(1000, 'Client disconnect');
      this.socket = null;
    }

    this.setStatus(ConnectionStatus.Disconnected);
  }

  private setupSocketListeners(): void {
    if (!this.socket) return;

    this.socket.onopen = () => {
      this.log('Connected successfully');
      this.setStatus(ConnectionStatus.Connected);
      this.clearError();
    };

    this.socket.onclose = (event) => {
      this.log('Connection closed', { code: event.code, reason: event.reason });
      this.setStatus(ConnectionStatus.Disconnected);
    };

    this.socket.onerror = (error) => {
      this.log('Socket error', error);
      this.setStatus(ConnectionStatus.Error);
      this.setError('Connection error occurred');
    };

    this.socket.onmessage = (event) => {
      this.handleMessage(event.data);
    };
  }

  private handleMessage(rawMessage: string): void {
    this.log('Received raw message', rawMessage);

    try {
      const response: WebSocketResponse = JSON.parse(rawMessage);
      this.log('Parsed message', response);

      switch (response.type) {
        case 'Welcome':
          this.log('Welcome received', response.data);
          this.connectionId = response.data?.connection_id || '';
          break;

        case 'Pong':
          this.log('Pong received');
          break;

        case 'MediaBlobs':
          this.log('Media blobs received', response.data);
          this.mediaBlobs = response.data?.blobs || [];
          this.dispatchEvent(new CustomEvent('media-blobs-received', {
            detail: { blobs: this.mediaBlobs, totalCount: response.data?.total_count },
            bubbles: true
          }));
          break;

        case 'MediaBlob':
          this.log('Single media blob received', response.data);
          this.dispatchEvent(new CustomEvent('media-blob-received', {
            detail: { blob: response.data?.blob },
            bubbles: true
          }));
          break;

        case 'Error':
          this.log('Error message received', response.data);
          this.setError(response.data?.message || 'Server error');
          break;

        case 'ConnectionStatus':
          this.log('Connection status update', response.data);
          this.userCount = response.data?.user_count || 0;
          break;

        default:
          this.log('Unknown message type', response);
      }
    } catch (error) {
      this.log('Failed to parse message', { error: error.toString(), rawMessage });
      this.setError(`Message parse error: ${error}`);
    }
  }

  private sendMessage(message: WebSocketMessage): boolean {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      this.setError('Cannot send message: not connected');
      return false;
    }

    try {
      const json = JSON.stringify(message);
      this.socket.send(json);
      this.log('Sent message', message);
      return true;
    } catch (error) {
      this.setError(`Send error: ${error}`);
      return false;
    }
  }

  // Public API methods
  ping(): boolean {
    return this.sendMessage({ type: 'Ping' });
  }

  getMediaBlobs(limit?: number, offset?: number): boolean {
    return this.sendMessage({
      type: 'GetMediaBlobs',
      data: { limit, offset }
    });
  }

  getMediaBlob(id: string): boolean {
    return this.sendMessage({
      type: 'GetMediaBlob',
      data: { id }
    });
  }

  uploadMediaBlob(blob: MediaBlob): boolean {
    return this.sendMessage({
      type: 'UploadMediaBlob',
      data: { blob }
    });
  }

  private formatFileSize(bytes?: number): string {
    if (!bytes) return 'Unknown size';
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  render() {
    return html`
      <div class="container">
        <div class="header">
          <h2 class="title">WebSocket Handler</h2>
          <div class="controls">
            <button @click=${this.ping} ?disabled=${this.status !== ConnectionStatus.Connected}>
              Ping
            </button>
            <button @click=${() => this.getMediaBlobs()} ?disabled=${this.status !== ConnectionStatus.Connected}>
              Get Media Blobs
            </button>
            ${this.status === ConnectionStatus.Connected
              ? html`<button @click=${this.disconnect}>Disconnect</button>`
              : html`<button @click=${this.connect} class="primary">Connect</button>`
            }
          </div>
        </div>

        <div class="status-section">
          <websocket-status
            .status=${this.status}
            .userCount=${this.userCount}
            .showUserCount=${true}
          ></websocket-status>
        </div>

        ${this.errorMessage ? html`
          <div class="error-message">
            ${this.errorMessage}
          </div>
        ` : ''}

        ${this.showDebugLog ? html`
          <div class="debug-log">
            ${this.debugLog.join('\n')}
          </div>
        ` : ''}

        <div class="media-blobs">
          <h3>Media Blobs (${this.mediaBlobs.length})</h3>
          ${this.mediaBlobs.length === 0 ? html`
            <div class="empty-state">
              No media blobs received yet. Click "Get Media Blobs" to fetch from server.
            </div>
          ` : this.mediaBlobs.map(blob => html`
            <div class="media-blob">
              <div class="media-blob-header">
                <div class="media-blob-id">${blob.id}</div>
                <div class="media-blob-info">
                  ${blob.mime || 'Unknown type'} • ${this.formatFileSize(blob.size)}
                </div>
              </div>
              <div class="media-blob-meta">
                SHA256: ${blob.sha256}<br>
                Client: ${blob.source_client_id || 'Unknown'}<br>
                Path: ${blob.local_path || 'None'}<br>
                Created: ${new Date(blob.created_at).toLocaleString()}
                ${Object.keys(blob.metadata).length > 0 ? html`<br>Metadata: ${JSON.stringify(blob.metadata)}` : ''}
              </div>
            </div>
          `)}
        </div>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'websocket-handler': WebSocketHandlerComponent;
  }
}
