/**
 * WebSocket Message Handler Web Component
 *
 * A web component that manages WebSocket connections, handles incoming messages,
 * and provides methods for sending messages. Integrates with the WebSocket client
 * and provides a simple interface for media blob handling.
 */

import { customElement } from 'solid-element';
import { createSignal, createEffect, Show, For } from 'solid-js';
import { ConnectionStatus, WebSocketStatus } from './websocket-status';
import './websocket-status';

// Import types from the clientlib
interface MediaBlob {
  id: string;
  data?: number[];
  sha256: string;
  size?: number;
  mime?: string;
  source_client_id?: string;
  local_path?: string;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

interface WebSocketMessage {
  type: 'Ping' | 'GetMediaBlobs' | 'UploadMediaBlob' | 'GetMediaBlob';
  data?: unknown;
}

interface WebSocketResponse {
  type:
    | 'Welcome'
    | 'Pong'
    | 'MediaBlobs'
    | 'MediaBlob'
    | 'Error'
    | 'ConnectionStatus';
  data?: unknown;
}

export interface WebSocketHandlerProps {
  websocketUrl?: string;
  autoConnect?: boolean;
  showDebugLog?: boolean;
}

const WebSocketHandler = (props: WebSocketHandlerProps) => {
  // Props with defaults
  const websocketUrl = () => props.websocketUrl ?? '';
  const autoConnect = () => props.autoConnect ?? true;
  const showDebugLog = () => props.showDebugLog ?? true;

  // State
  const [status, setStatus] = createSignal<ConnectionStatus>(
    ConnectionStatus.Disconnected
  );
  const [socket, setSocket] = createSignal<WebSocket | null>(null);
  const [debugLog, setDebugLog] = createSignal<string[]>([]);
  const [mediaBlobs, setMediaBlobs] = createSignal<MediaBlob[]>([]);
  const [errorMessage, setErrorMessage] = createSignal<string>('');
  const [userCount, setUserCount] = createSignal<number>(0);

  // Auto connect effect
  createEffect(() => {
    if (autoConnect() && websocketUrl()) {
      connect();
    }
  });

  const log = (message: string, ...args: unknown[]) => {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry =
      args.length > 0
        ? `[${timestamp}] ${message}: ${JSON.stringify(args, null, 2)}`
        : `[${timestamp}] ${message}`;

    setDebugLog((prev) => [...prev.slice(-99), logEntry]); // Keep last 100 entries
    console.log('[WebSocketHandler]', message, ...args);
  };

  const updateStatus = (newStatus: ConnectionStatus) => {
    if (status() !== newStatus) {
      setStatus(newStatus);
      log(`Status changed to: ${newStatus}`);

      // Dispatch status change event
      const event = new CustomEvent('status-change', {
        detail: { status: newStatus },
        bubbles: true,
      });

      setTimeout(() => {
        const host = document.querySelector('websocket-handler');
        if (host) {
          host.dispatchEvent(event);
        }
      }, 0);
    }
  };

  const setError = (message: string) => {
    setErrorMessage(message);
    log(`Error: ${message}`);
  };

  const clearError = () => {
    setErrorMessage('');
  };

  const connect = () => {
    if (!websocketUrl()) {
      setError('WebSocket URL not provided');
      return;
    }

    if (socket()?.readyState === WebSocket.OPEN) {
      log('Already connected');
      return;
    }

    clearError();
    updateStatus(ConnectionStatus.Connecting);
    log(`Connecting to ${websocketUrl()}`);

    try {
      const newSocket = new WebSocket(websocketUrl());
      setSocket(newSocket);
      setupSocketListeners(newSocket);
    } catch (error) {
      setError(`Connection failed: ${error}`);
      updateStatus(ConnectionStatus.Error);
    }
  };

  const disconnect = () => {
    log('Disconnecting...');

    const currentSocket = socket();
    if (currentSocket) {
      currentSocket.close(1000, 'Client disconnect');
      setSocket(null);
    }

    updateStatus(ConnectionStatus.Disconnected);
  };

  const setupSocketListeners = (ws: WebSocket) => {
    ws.onopen = () => {
      log('Connected successfully');
      updateStatus(ConnectionStatus.Connected);
      clearError();
    };

    ws.onclose = (event) => {
      log('Connection closed', { code: event.code, reason: event.reason });
      updateStatus(ConnectionStatus.Disconnected);
    };

    ws.onerror = (error) => {
      log('Socket error', error);
      updateStatus(ConnectionStatus.Error);
      setError('Connection error occurred');
    };

    ws.onmessage = (event) => {
      handleMessage(event.data);
    };
  };

  const handleMessage = (rawMessage: string) => {
    log('Received raw message', rawMessage);

    try {
      const response: WebSocketResponse = JSON.parse(rawMessage);
      log('Parsed message', response);

      switch (response.type) {
        case 'Welcome':
          log('Welcome received', response.data);
          break;

        case 'Pong':
          log('Pong received');
          break;

        case 'MediaBlobs': {
          log('Media blobs received', response.data);
          const blobsData = response.data as {
            blobs?: MediaBlob[];
            total_count?: number;
          };
          setMediaBlobs(blobsData?.blobs || []);

          // Dispatch event
          const blobsEvent = new CustomEvent('media-blobs-received', {
            detail: {
              blobs: mediaBlobs(),
              totalCount: blobsData?.total_count,
            },
            bubbles: true,
          });
          setTimeout(() => {
            const host = document.querySelector('websocket-handler');
            if (host) {
              host.dispatchEvent(blobsEvent);
            }
          }, 0);
          break;
        }

        case 'MediaBlob': {
          log('Single media blob received', response.data);
          const blobData = response.data as { blob?: MediaBlob };

          // Dispatch event
          const blobEvent = new CustomEvent('media-blob-received', {
            detail: { blob: blobData?.blob },
            bubbles: true,
          });
          setTimeout(() => {
            const host = document.querySelector('websocket-handler');
            if (host) {
              host.dispatchEvent(blobEvent);
            }
          }, 0);
          break;
        }

        case 'Error': {
          log('Error message received', response.data);
          const errorData = response.data as { message?: string };
          setError(errorData?.message || 'Server error');
          break;
        }

        case 'ConnectionStatus': {
          log('Connection status update', response.data);
          const statusData = response.data as { user_count?: number };
          setUserCount(statusData?.user_count || 0);
          break;
        }

        default:
          log('Unknown message type', response);
      }
    } catch (error) {
      log('Failed to parse message', {
        error: error instanceof Error ? error.toString() : String(error),
        rawMessage,
      });
      setError(`Message parse error: ${error}`);
    }
  };

  const sendMessage = (message: WebSocketMessage): boolean => {
    const currentSocket = socket();
    if (!currentSocket || currentSocket.readyState !== WebSocket.OPEN) {
      setError('Cannot send message: not connected');
      return false;
    }

    try {
      const json = JSON.stringify(message);
      currentSocket.send(json);
      log('Sent message', message);
      return true;
    } catch (error) {
      setError(`Send error: ${error}`);
      return false;
    }
  };

  // Public API methods
  const ping = () => sendMessage({ type: 'Ping' });

  const getMediaBlobs = (limit?: number, offset?: number) =>
    sendMessage({
      type: 'GetMediaBlobs',
      data: { limit, offset },
    });

  const getMediaBlob = (id: string) =>
    sendMessage({
      type: 'GetMediaBlob',
      data: { id },
    });

  const uploadMediaBlob = (blob: MediaBlob) =>
    sendMessage({
      type: 'UploadMediaBlob',
      data: { blob },
    });

  // Expose methods for external use
  const exposeMethods = () => {
    const element = document.querySelector('websocket-handler');
    if (element) {
      Object.assign(element, {
        ping,
        getMediaBlobs,
        getMediaBlob,
        uploadMediaBlob,
        connect,
        disconnect,
      });
    }
  };

  // Expose methods after mount
  createEffect(() => {
    setTimeout(exposeMethods, 0);
  });

  const formatFileSize = (bytes?: number): string => {
    if (!bytes) return 'Unknown size';
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  // Cleanup on unmount
  createEffect(() => {
    return disconnect;
  });

  return (
    <div
      style={{
        display: 'block',
        'font-family':
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      }}
    >
      <style>{`
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
      `}</style>

      <div class='container'>
        <div class='header'>
          <h2 class='title'>WebSocket Handler</h2>
          <div class='controls'>
            <button
              onClick={ping}
              disabled={status() !== ConnectionStatus.Connected}
            >
              Ping
            </button>
            <button
              onClick={() => getMediaBlobs()}
              disabled={status() !== ConnectionStatus.Connected}
            >
              Get Media Blobs
            </button>
            <Show
              when={status() === ConnectionStatus.Connected}
              fallback={
                <button onClick={connect} class='primary'>
                  Connect
                </button>
              }
            >
              <button onClick={disconnect}>Disconnect</button>
            </Show>
          </div>
        </div>

        <div class='status-section'>
          {WebSocketStatus({
            status: status(),
            userCount: userCount(),
            showUserCount: true,
            showText: true,
            compact: false,
          })}
        </div>

        <Show when={errorMessage()}>
          <div class='error-message'>{errorMessage()}</div>
        </Show>

        <Show when={showDebugLog()}>
          <div class='debug-log'>{debugLog().join('\n')}</div>
        </Show>

        <div class='media-blobs'>
          <h3>Media Blobs ({mediaBlobs().length})</h3>
          <Show
            when={mediaBlobs().length > 0}
            fallback={
              <div class='empty-state'>
                No media blobs received yet. Click "Get Media Blobs" to fetch
                from server.
              </div>
            }
          >
            <For each={mediaBlobs()}>
              {(blob) => (
                <div class='media-blob'>
                  <div class='media-blob-header'>
                    <div class='media-blob-id'>{blob.id}</div>
                    <div class='media-blob-info'>
                      {blob.mime || 'Unknown type'} •{' '}
                      {formatFileSize(blob.size)}
                    </div>
                  </div>
                  <div class='media-blob-meta'>
                    SHA256: {blob.sha256}
                    <br />
                    Client: {blob.source_client_id || 'Unknown'}
                    <br />
                    Path: {blob.local_path || 'None'}
                    <br />
                    Created: {new Date(blob.created_at).toLocaleString()}
                    <Show when={Object.keys(blob.metadata).length > 0}>
                      <br />
                      Metadata: {JSON.stringify(blob.metadata)}
                    </Show>
                  </div>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </div>
  );
};

// Register as custom element
customElement(
  'websocket-handler',
  {
    websocketUrl: '',
    autoConnect: true,
    showDebugLog: true,
  },
  WebSocketHandler
);

export { WebSocketHandler };

/* eslint-disable @typescript-eslint/no-namespace */
declare global {
  namespace JSX {
    interface IntrinsicElements {
      'websocket-handler': {
        websocketUrl?: string;
        autoConnect?: boolean;
        showDebugLog?: boolean;
      };
    }
  }
}
/* eslint-enable @typescript-eslint/no-namespace */
