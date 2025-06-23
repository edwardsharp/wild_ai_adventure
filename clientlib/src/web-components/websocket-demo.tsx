/**
 * WebSocket Demo Component
 *
 * A simple demo that showcases the modular WebSocket client library
 * components without heavy styling or complex UI logic.
 */

/* @jsxImportSource solid-js */
import { customElement } from "solid-element";
import { createSignal, createEffect, For, Show, onCleanup } from "solid-js";
import { WebSocketDemoClient } from "../lib/websocket-demo-client.js";
import type { MediaBlob } from "../lib/media-blob-manager.js";

export interface WebSocketDemoProps {
  websocketUrl?: string;
  autoConnect?: boolean;
  showDebugLog?: boolean;
}

const WebSocketDemo = (props: WebSocketDemoProps) => {
  const [client, setClient] = createSignal<WebSocketDemoClient | null>(null);
  const [status, setStatus] = createSignal("disconnected");
  const [userCount, setUserCount] = createSignal(0);
  const [blobs, setBlobs] = createSignal<MediaBlob[]>([]);
  const [logs, setLogs] = createSignal<string[]>([]);
  const [thumbnailRefresh, setThumbnailRefresh] = createSignal(0);
  const [url, setUrl] = createSignal("ws://localhost:8080/ws");

  // Initialize URL from props
  createEffect(() => {
    const initialUrl = props.websocketUrl;
    if (initialUrl) {
      setUrl(initialUrl);
    }
  });

  // File upload ref
  let fileInputRef: HTMLInputElement | undefined;

  // Global function for loading blob data (called from thumbnail onclick)

  (
    window as unknown as { loadBlobData: (blobId: string) => void }
  ).loadBlobData = // eslint-disable-next-line solid/reactivity
    (blobId: string) => {
      client()?.loadBlobData(blobId);
    };

  // Initialize client
  createEffect(() => {
    const currentUrl = url();

    const wsClient = new WebSocketDemoClient(currentUrl, {
      logLevel: "info",
      autoGetMediaBlobs: true,
    });

    // Set up event listeners

    wsClient.addEventListener("status-change", (e: Event) => {
      const { status: newStatus, userCount: newUserCount } = (e as CustomEvent)
        .detail;
      setStatus(newStatus);
      setUserCount(newUserCount || 0);
    });

    wsClient.addEventListener("blobs-updated", (e: Event) => {
      setBlobs((e as CustomEvent).detail.blobs);
    });

    wsClient.addEventListener("blob-data-cached", () => {
      // Trigger thumbnail refresh
      setThumbnailRefresh((prev) => prev + 1);
    });

    wsClient.addEventListener("log", (e: Event) => {
      const { message, data } = (e as CustomEvent).detail.data;
      const logEntry = data ? `${message}: ${JSON.stringify(data)}` : message;

      setLogs((prev) => [...prev.slice(-49), logEntry]); // Keep last 50 entries
    });

    setClient(wsClient);

    // Auto-connect if requested
    if (props.autoConnect) {
      wsClient.connect().catch(console.error);
    }

    // Cleanup on component unmount
    onCleanup(() => {
      wsClient.destroy();
    });
  });

  const handleConnect = () => {
    client()?.connect().catch(console.error);
  };

  const handleDisconnect = () => {
    client()?.disconnect();
  };

  const handlePing = () => {
    client()?.ping();
  };

  const handleGetBlobs = () => {
    client()?.getMediaBlobs();
  };

  const handleFileUpload = (event: Event) => {
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (files && files.length > 0) {
      client()?.uploadFiles(files);
      target.value = ""; // Reset input
    }
  };

  const handleUploadClick = () => {
    fileInputRef?.click();
  };

  const handleDownload = (blobId: string, filename?: string) => {
    client()?.downloadBlob(blobId, filename);
  };

  const handleView = (blobId: string) => {
    client()?.viewBlob(blobId);
  };

  const handleLoadData = (blobId: string) => {
    client()?.loadBlobData(blobId);
  };

  const clearLogs = () => {
    setLogs([]);
    client()?.clearEventLog();
  };

  const getStatusColor = () => {
    switch (status()) {
      case "connected":
        return "#10b981";
      case "connecting":
        return "#f59e0b";
      case "error":
        return "#ef4444";
      default:
        return "#6b7280";
    }
  };

  return (
    <div style={{ padding: "1rem", "font-family": "sans-serif" }}>
      <style>{`
        .demo-section { margin-bottom: 2rem; }
        .controls { display: flex; gap: 0.75rem; margin-bottom: 1rem; flex-wrap: wrap; align-items: center; }
        button {
          padding: 0.5rem 1rem;
          border: 1px solid #ccc;
          background: white;
          cursor: pointer;
          border-radius: 4px;
          font-size: 0.875rem;
          font-weight: 500;
        }
        button:hover:not(:disabled) { background: #f0f0f0; }
        button:disabled { opacity: 0.5; cursor: not-allowed; }
        button.primary { background: #3b82f6; color: white; border-color: #3b82f6; }
        button.primary:hover:not(:disabled) { background: #2563eb; }
        input[type="text"] {
          padding: 0.5rem;
          border: 1px solid #ccc;
          border-radius: 4px;
          min-width: 300px;
          font-size: 0.875rem;
        }
        .status-indicator {
          display: inline-block;
          width: 12px;
          height: 12px;
          border-radius: 50%;
          margin-right: 0.5rem;
        }
        .log-container {
          background: #f8f9fa;
          border: 1px solid #e9ecef;
          border-radius: 4px;
          padding: 1rem;
          max-height: 300px;
          overflow-y: auto;
          font-family: monospace;
          font-size: 0.875rem;
          white-space: pre-wrap;
        }
        .blob-list { display: grid; gap: 1rem; }
        .blob-item {
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 1rem;
          background: white;
        }
        .blob-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 0.5rem;
        }
        .blob-actions { display: flex; gap: 0.5rem; margin-top: 0.5rem; }
        .blob-actions button {
          font-size: 0.75rem;
          padding: 0.25rem 0.5rem;
          font-weight: normal;
        }
        .section-title {
          margin: 0 0 1rem 0;
          color: #374151;
          font-weight: 600;
        }
        .empty-state {
          text-align: center;
          padding: 2rem;
          color: #6b7280;
          font-style: italic;
        }
      `}</style>

      <h1>WebSocket Demo (Modular Components)</h1>

      <div class="demo-section">
        <h2 class="section-title">Connection</h2>
        <div class="controls">
          <input
            type="text"
            value={url()}
            onInput={(e) => setUrl(e.target.value)}
            placeholder="WebSocket URL"
            disabled={status() === "connected" || status() === "connecting"}
          />
          <button
            class="primary"
            onClick={handleConnect}
            disabled={status() === "connected" || status() === "connecting"}
          >
            Connect
          </button>
          <button
            onClick={handleDisconnect}
            disabled={status() === "disconnected"}
          >
            Disconnect
          </button>
        </div>

        <div style={{ "margin-bottom": "1rem" }}>
          <span
            class="status-indicator"
            style={{ "background-color": getStatusColor() }}
          />
          Status: {status()}
          <Show when={userCount() > 0}>
            {" "}
            ({userCount()} user{userCount() !== 1 ? "s" : ""} online)
          </Show>
        </div>
      </div>

      <div class="demo-section">
        <h2 class="section-title">Actions</h2>
        <div class="controls">
          <button onClick={handlePing} disabled={status() !== "connected"}>
            Ping
          </button>
          <button onClick={handleGetBlobs} disabled={status() !== "connected"}>
            Get Media Blobs
          </button>
          <button
            onClick={handleUploadClick}
            disabled={status() !== "connected"}
          >
            Upload Files
          </button>
          <button onClick={clearLogs}>Clear Log</button>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            onChange={handleFileUpload}
            disabled={status() !== "connected"}
            style={{ display: "none" }}
          />
        </div>
      </div>

      <div class="demo-section">
        <h2 class="section-title">Media Blobs ({blobs().length})</h2>
        <Show
          when={blobs().length > 0}
          fallback={
            <div class="empty-state">
              No media blobs yet. Upload a file or get blobs from server.
            </div>
          }
        >
          <div class="blob-list">
            <For each={blobs()}>
              {(blob) => {
                const displayInfo = () => {
                  // Include refresh signal to make this reactive
                  thumbnailRefresh();
                  return client()?.getBlobDisplayInfo(blob);
                };
                return (
                  <div class="blob-item">
                    <div class="blob-header">
                      <div>
                        <strong>{blob.id}</strong>
                        <br />
                        <small>
                          {displayInfo()?.mime} • {displayInfo()?.size}
                        </small>
                      </div>
                      {/* eslint-disable-next-line solid/no-innerhtml */}
                      <div innerHTML={displayInfo()?.thumbnailHtml} />
                    </div>
                    <div>
                      <small>
                        Path: {blob.local_path || "None"}
                        <br />
                        Created: {new Date(blob.created_at).toLocaleString()}
                      </small>
                    </div>
                    <div class="blob-actions">
                      <button
                        onClick={() => handleDownload(blob.id, blob.local_path)}
                      >
                        Download
                      </button>
                      <button onClick={() => handleView(blob.id)}>View</button>
                      <button onClick={() => handleLoadData(blob.id)}>
                        Load Data
                      </button>
                    </div>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      </div>

      <Show when={props.showDebugLog}>
        <div class="demo-section">
          <h2 class="section-title">Debug Log</h2>
          <div class="controls">
            <button onClick={clearLogs}>Clear Log</button>
          </div>
          <div class="log-container">
            <For each={logs()}>{(log) => <div>{log}</div>}</For>
            <Show when={logs().length === 0}>
              <div style={{ color: "#6b7280", "font-style": "italic" }}>
                No log entries yet...
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};

// Register as custom element
customElement(
  "websocket-demo",
  {
    websocketUrl: "ws://localhost:8080/ws",
    autoConnect: false,
    showDebugLog: true,
  },
  WebSocketDemo
);

export { WebSocketDemo };

/* eslint-disable @typescript-eslint/no-namespace */
declare global {
  namespace JSX {
    interface IntrinsicElements {
      "websocket-demo": {
        websocketUrl?: string;
        autoConnect?: boolean;
        showDebugLog?: boolean;
      };
    }
  }
}
/* eslint-enable @typescript-eslint/no-namespace */
