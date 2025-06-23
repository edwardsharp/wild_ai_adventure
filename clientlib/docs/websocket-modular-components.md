# WebSocket Modular Components

This document explains the new modular WebSocket client library components that provide a clean, event-driven interface for WebSocket connections, media blob management, and file uploads.

## Overview

The WebSocket functionality has been refactored into several modular components:

- **WebSocketConnection** - Core WebSocket connection management
- **MediaBlobManager** - Media blob handling, caching, and thumbnails
- **FileUploadHandler** - File upload processing and validation
- **WebSocketDemoClient** - Unified client orchestrating all components

## Components

### WebSocketConnection

Manages WebSocket connections with automatic reconnection and status tracking.

```typescript
import { WebSocketConnection } from '@webauthn/clientlib';

const connection = new WebSocketConnection({
  url: 'ws://localhost:8080/ws',
  autoReconnect: true,
  reconnectDelay: 3000,
  maxReconnectAttempts: 5,
  pingInterval: 30000
});

// Event listeners
connection.addEventListener('status-change', (e) => {
  console.log('Status:', e.detail.status);
});

connection.addEventListener('message', (e) => {
  console.log('Message:', e.detail.message);
});

// Connect and send messages
await connection.connect();
connection.send({ type: 'Ping' });
```

### MediaBlobManager

Handles media blob data management, caching, and thumbnail generation.

```typescript
import { MediaBlobManager } from '@webauthn/clientlib';

const blobManager = new MediaBlobManager();

// Event listeners
blobManager.addEventListener('blobs-updated', (e) => {
  console.log('Blobs updated:', e.detail.blobs);
});

blobManager.addEventListener('blob-data-requested', (e) => {
  // Handle requests to load blob data
  loadBlobDataFromServer(e.detail.id);
});

// Update blobs from server
blobManager.updateBlobs(blobsFromServer);

// Generate display info
const blob = blobManager.getBlob('blob-id');
const displayInfo = blobManager.getBlobDisplayInfo(blob);
```

### FileUploadHandler

Processes file uploads with validation, SHA256 calculation, and progress tracking.

```typescript
import { FileUploadHandler } from '@webauthn/clientlib';

const uploadHandler = new FileUploadHandler({
  maxFileSize: 10 * 1024 * 1024, // 10MB
  clientId: 'my-client'
});

// Event listeners
uploadHandler.addEventListener('upload-completed', (e) => {
  const { file, blob } = e.detail;
  console.log('Upload completed:', file.name);
  // Send blob to server
});

// Process files
const fileInput = document.querySelector('input[type="file"]');
fileInput.addEventListener('change', async (e) => {
  const uploadIds = await uploadHandler.addFiles(e.target.files);
  console.log('Started uploads:', uploadIds);
});
```

### WebSocketDemoClient

Unified client that orchestrates all components together.

```typescript
import { WebSocketDemoClient } from '@webauthn/clientlib';

const client = new WebSocketDemoClient('ws://localhost:8080/ws', {
  autoGetMediaBlobs: true,
  logLevel: 'info'
});

// Event listeners
client.addEventListener('status-change', (e) => {
  console.log('Connection status:', e.detail.status);
});

client.addEventListener('blobs-updated', (e) => {
  console.log('Media blobs:', e.detail.blobs);
});

// Connect and use
await client.connect();
client.ping();
client.getMediaBlobs();

// Upload files
const files = document.querySelector('input[type="file"]').files;
await client.uploadFiles(files);

// Download/view blobs
client.downloadBlob('blob-id', 'filename.jpg');
client.viewBlob('blob-id');
```

## Web Components

### WebSocketDemo Component

A complete demo component that showcases all the modular functionality:

```html
<websocket-demo
  websocketUrl="ws://localhost:8080/ws"
  autoConnect="false"
  showDebugLog="true">
</websocket-demo>
```

### Standalone Files

The build process generates standalone HTML files you can use directly:

- `websocket-demo-standalone.html` - Complete demo with modular components
- `websocket-components-standalone.html` - Basic WebSocket components
- `webauthn-auth-standalone.html` - WebAuthn authentication component

## Usage Examples

### Basic WebSocket Connection

```typescript
import { WebSocketConnection } from '@webauthn/clientlib';

const ws = new WebSocketConnection({ url: 'ws://localhost:8080/ws' });

ws.addEventListener('status-change', (e) => {
  document.getElementById('status').textContent = e.detail.status;
});

ws.addEventListener('message', (e) => {
  console.log('Received:', e.detail.message);
});

document.getElementById('connect').onclick = () => ws.connect();
document.getElementById('disconnect').onclick = () => ws.disconnect();
```

### File Upload with Progress

```typescript
import { FileUploadHandler } from '@webauthn/clientlib';

const uploader = new FileUploadHandler();

uploader.addEventListener('upload-started', (e) => {
  console.log(`Starting upload: ${e.detail.file.name}`);
});

uploader.addEventListener('upload-completed', (e) => {
  console.log(`Completed: ${e.detail.file.name}`);
  // Send e.detail.blob to your WebSocket server
});

// Handle file input
document.getElementById('file-input').onchange = (e) => {
  uploader.addFiles(e.target.files);
};
```

### Media Blob Display

```typescript
import { MediaBlobManager } from '@webauthn/clientlib';

const blobManager = new MediaBlobManager();

blobManager.addEventListener('blobs-updated', (e) => {
  const container = document.getElementById('blobs');
  container.innerHTML = '';

  e.detail.blobs.forEach(blob => {
    const info = blobManager.getBlobDisplayInfo(blob);
    const div = document.createElement('div');
    div.innerHTML = `
      <h3>${blob.id}</h3>
      <p>${info.mime} • ${info.size}</p>
      <div>${info.thumbnailHtml}</div>
      <button onclick="downloadBlob('${blob.id}')">Download</button>
    `;
    container.appendChild(div);
  });
});

window.downloadBlob = (id) => {
  blobManager.downloadBlob(id);
};
```

## Event Reference

### WebSocketConnection Events

- `status-change` - Connection status changed
- `message` - Raw message received
- `connection-error` - Connection error occurred
- `pong` - Pong response received
- `reconnecting` - Attempting to reconnect

### MediaBlobManager Events

- `blobs-updated` - Blob list updated
- `blob-data-cached` - Blob data loaded and cached
- `blob-data-requested` - Request to load blob data
- `blob-downloaded` - Blob downloaded
- `blob-viewed` - Blob opened in new tab

### FileUploadHandler Events

- `upload-started` - Upload processing started
- `upload-completed` - Upload processing completed
- `upload-error` - Upload failed
- `upload-cancelled` - Upload cancelled

### WebSocketDemoClient Events

- All events from individual components
- `server-error` - Server error message received
- `log` - Debug log entry added

## Benefits

1. **Modular Design** - Use only the components you need
2. **Event-Driven** - Clean separation of concerns
3. **No Styling Dependencies** - Pure logic, minimal CSS
4. **TypeScript Support** - Full type safety
5. **Easy Testing** - Each component can be tested independently
6. **Reusable** - Components work in any JavaScript environment

## Migration from websocket-demo.html

If you were using the original `websocket-demo.html`, you can now:

1. Import the modular components instead of copying HTML
2. Use the `WebSocketDemoClient` for the same functionality
3. Build custom UIs using individual components
4. Keep your existing styles separate from the logic

The modular approach gives you the same functionality with better maintainability and reusability.
