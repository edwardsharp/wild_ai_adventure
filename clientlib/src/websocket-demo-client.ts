/**
 * WebSocket Demo Client
 *
 * A unified client that orchestrates WebSocket connection, media blob management,
 * and file uploads. This provides a high-level interface that combines all the
 * modular components into a cohesive demo client.
 */

import {
  WebSocketConnection,
  ConnectionStatus,
  type WebSocketConnectionOptions,
} from './websocket-connection.js';
import {
  MediaBlobManager,
  type MediaBlob,
  type MediaBlobData,
} from './media-blob-manager.js';
import { FileUploadHandler, type FileUploadOptions } from './file-upload.js';

export interface WebSocketDemoClientOptions {
  websocket?: WebSocketConnectionOptions;
  fileUpload?: FileUploadOptions;
  autoGetMediaBlobs?: boolean;
  logLevel?: 'none' | 'error' | 'warn' | 'info' | 'debug';
}

export interface DemoClientEvent {
  type: string;
  timestamp: number;
  data?: any;
}

export class WebSocketDemoClient extends EventTarget {
  private connection: WebSocketConnection;
  private blobManager: MediaBlobManager;
  private uploadHandler: FileUploadHandler;
  private eventLog: DemoClientEvent[] = [];
  private options: WebSocketDemoClientOptions;

  constructor(websocketUrl: string, options: WebSocketDemoClientOptions = {}) {
    super();

    this.options = {
      autoGetMediaBlobs: true,
      logLevel: 'info',
      ...options,
    };

    // Initialize components
    this.connection = new WebSocketConnection({
      url: websocketUrl,
      ...this.options.websocket,
    });

    this.blobManager = new MediaBlobManager();

    this.uploadHandler = new FileUploadHandler({
      clientId: 'demo-client',
      ...this.options.fileUpload,
    });

    this.setupEventHandlers();
  }

  /**
   * Connect to WebSocket server
   */
  async connect(): Promise<void> {
    this.log('info', 'Connecting to WebSocket server');
    return this.connection.connect();
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    this.log('info', 'Disconnecting from WebSocket server');
    this.connection.disconnect();
  }

  /**
   * Send a ping message
   */
  ping(): void {
    this.log('debug', 'Sending ping');
    this.connection.ping();
  }

  /**
   * Request media blobs from server
   */
  getMediaBlobs(limit = 10, offset = 0): void {
    this.log(
      'debug',
      `Requesting media blobs (limit: ${limit}, offset: ${offset})`
    );
    this.connection.send({
      type: 'GetMediaBlobs',
      data: { limit, offset },
    });
  }

  /**
   * Upload files
   */
  async uploadFiles(files: FileList | File[]): Promise<string[]> {
    this.log('info', `Starting upload of ${files.length} file(s)`);
    return this.uploadHandler.addFiles(files);
  }

  /**
   * Download a media blob
   */
  downloadBlob(blobId: string, filename?: string): boolean {
    this.log('debug', `Downloading blob: ${blobId}`);
    return this.blobManager.downloadBlob(blobId, filename);
  }

  /**
   * View a media blob in new tab
   */
  viewBlob(blobId: string): boolean {
    this.log('debug', `Viewing blob: ${blobId}`);
    return this.blobManager.viewBlob(blobId);
  }

  /**
   * Load blob data from server
   */
  loadBlobData(blobId: string): void {
    this.log('debug', `Loading blob data: ${blobId}`);
    this.blobManager.requestBlobData(blobId);
  }

  /**
   * Get current connection status
   */
  getConnectionStatus(): ConnectionStatus {
    return this.connection.getStatus();
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.connection.isConnected();
  }

  /**
   * Get current user count
   */
  getUserCount(): number {
    return this.connection.getUserCount();
  }

  /**
   * Get connection ID
   */
  getConnectionId(): string {
    return this.connection.getConnectionId();
  }

  /**
   * Get all media blobs
   */
  getBlobs(): MediaBlob[] {
    return this.blobManager.getBlobs();
  }

  /**
   * Get blob display info
   */
  getBlobDisplayInfo(blob: MediaBlob) {
    return this.blobManager.getBlobDisplayInfo(blob);
  }

  /**
   * Get upload statistics
   */
  getUploadStats() {
    return this.uploadHandler.getStats();
  }

  /**
   * Get cache statistics
   */
  getCacheStats() {
    return this.blobManager.getCacheStats();
  }

  /**
   * Clear completed uploads
   */
  clearCompletedUploads(): void {
    this.uploadHandler.clearCompleted();
  }

  /**
   * Clear blob cache
   */
  clearBlobCache(): void {
    this.blobManager.clearCache();
  }

  /**
   * Get event log
   */
  getEventLog(): DemoClientEvent[] {
    return [...this.eventLog];
  }

  /**
   * Clear event log
   */
  clearEventLog(): void {
    this.eventLog = [];
    this.dispatchEvent(
      new CustomEvent('log-cleared', {
        detail: { timestamp: Date.now() },
      })
    );
  }

  private setupEventHandlers(): void {
    // WebSocket connection events
    this.connection.addEventListener('status-change', (e: any) => {
      const { status, userCount, connectionId } = e.detail;
      this.log('info', `Connection status changed: ${status}`, {
        userCount,
        connectionId,
      });

      this.dispatchEvent(
        new CustomEvent('status-change', { detail: e.detail })
      );

      // Auto-request media blobs when connected
      if (status === 'connected' && this.options.autoGetMediaBlobs) {
        setTimeout(() => this.getMediaBlobs(), 100);
      }
    });

    this.connection.addEventListener('message', (e: any) => {
      const { message } = e.detail;
      this.handleServerMessage(message);
    });

    this.connection.addEventListener('connection-error', (e: any) => {
      this.log('error', 'Connection error', e.detail);
      this.dispatchEvent(
        new CustomEvent('connection-error', { detail: e.detail })
      );
    });

    this.connection.addEventListener('pong', (e: any) => {
      this.log('debug', 'Pong received');
      this.dispatchEvent(new CustomEvent('pong', { detail: e.detail }));
    });

    // Media blob manager events
    this.blobManager.addEventListener('blobs-updated', (e: any) => {
      this.log('info', `Media blobs updated: ${e.detail.count} blobs`);
      this.dispatchEvent(
        new CustomEvent('blobs-updated', { detail: e.detail })
      );
    });

    this.blobManager.addEventListener('blob-data-requested', (e: any) => {
      const { id } = e.detail;
      this.connection.send({
        type: 'GetMediaBlobData',
        data: { id },
      });
    });

    this.blobManager.addEventListener('blob-data-cached', (e: any) => {
      this.log('debug', `Blob data cached: ${e.detail.id}`);
      this.dispatchEvent(
        new CustomEvent('blob-data-cached', { detail: e.detail })
      );
    });

    // File upload events
    this.uploadHandler.addEventListener('upload-started', (e: any) => {
      const { file } = e.detail;
      this.log('info', `Upload started: ${file.name}`);
      this.dispatchEvent(
        new CustomEvent('upload-started', { detail: e.detail })
      );
    });

    this.uploadHandler.addEventListener('upload-completed', (e: any) => {
      const { file, blob } = e.detail;
      this.log('info', `Upload completed: ${file.name}`);

      // Send the blob to the server
      this.connection.send({
        type: 'UploadMediaBlob',
        data: { blob },
      });

      this.dispatchEvent(
        new CustomEvent('upload-completed', { detail: e.detail })
      );
    });

    this.uploadHandler.addEventListener('upload-error', (e: any) => {
      const { file, error } = e.detail;
      this.log('error', `Upload failed: ${file.name}`, { error });
      this.dispatchEvent(new CustomEvent('upload-error', { detail: e.detail }));
    });
  }

  private handleServerMessage(message: any): void {
    switch (message.type) {
      case 'MediaBlobs':
        const blobsData = message.data;
        this.log(
          'info',
          `Received ${blobsData?.blobs?.length || 0} media blobs`
        );
        this.blobManager.updateBlobs(blobsData?.blobs || []);
        break;

      case 'MediaBlob':
        const blob = message.data?.blob;
        this.log('info', `Received single media blob: ${blob?.id}`);
        break;

      case 'MediaBlobData':
        const blobData = message.data as MediaBlobData;
        this.log('debug', `Received blob data: ${blobData?.id}`);
        this.blobManager.cacheBlobData(blobData);
        break;

      case 'Error':
        const error = message.data?.message || 'Server error';
        this.log('error', `Server error: ${error}`);
        this.dispatchEvent(
          new CustomEvent('server-error', {
            detail: { error },
          })
        );
        break;

      default:
        this.log('debug', `Unknown message type: ${message.type}`);
    }

    // Always emit the raw message
    this.dispatchEvent(
      new CustomEvent('message', {
        detail: { message },
      })
    );
  }

  private log(level: string, message: string, data?: any): void {
    if (!this.shouldLog(level)) return;

    const event: DemoClientEvent = {
      type: level,
      timestamp: Date.now(),
      data: { message, data },
    };

    this.eventLog.push(event);

    // Keep last 100 entries
    if (this.eventLog.length > 100) {
      this.eventLog = this.eventLog.slice(-100);
    }

    // Emit log event
    this.dispatchEvent(new CustomEvent('log', { detail: event }));

    // Console log
    const timestamp = new Date().toLocaleTimeString();
    const logMessage = data
      ? `[${timestamp}] [WebSocketDemo] ${message}: ${JSON.stringify(data, null, 2)}`
      : `[${timestamp}] [WebSocketDemo] ${message}`;

    switch (level) {
      case 'error':
        console.error(logMessage);
        break;
      case 'warn':
        console.warn(logMessage);
        break;
      case 'debug':
        console.debug(logMessage);
        break;
      default:
        console.log(logMessage);
    }
  }

  private shouldLog(level: string): boolean {
    const levels = ['none', 'error', 'warn', 'info', 'debug'];
    const currentLevel = levels.indexOf(this.options.logLevel || 'info');
    const messageLevel = levels.indexOf(level);
    return messageLevel <= currentLevel;
  }

  /**
   * Destroy and clean up all resources
   */
  destroy(): void {
    this.log('info', 'Destroying WebSocket demo client');

    this.connection.destroy();
    this.blobManager.destroy();
    this.uploadHandler.destroy();

    this.eventLog = [];

    // Remove all event listeners
    const events = [
      'status-change',
      'blobs-updated',
      'blob-data-cached',
      'upload-started',
      'upload-completed',
      'upload-error',
      'connection-error',
      'server-error',
      'message',
      'log',
      'log-cleared',
      'pong',
    ];
    events.forEach((event) => {
      const listeners = (this as any)._listeners?.[event] || [];
      listeners.forEach((listener: any) => {
        this.removeEventListener(event, listener);
      });
    });
  }
}
