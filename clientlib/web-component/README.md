# WebAuthn Web Component

A modern, customizable web component for WebAuthn authentication built with Solid.js and TypeScript.

## Features

- 🔒 **Complete WebAuthn Flow**: Registration and login with biometric/security key authentication
- 🎨 **Themeable**: Light, dark, and auto themes with CSS custom properties
- 📱 **Responsive**: Mobile-friendly design that works on all screen sizes
- ⚡ **Fast**: Built with Solid.js for optimal performance
- 🧩 **Web Standards**: Uses standard Custom Elements API
- 🎯 **Framework Agnostic**: Works with any framework or vanilla HTML/JS
- 📦 **Lightweight**: Minimal bundle size with tree-shaking support
- ♿ **Accessible**: Full keyboard navigation and screen reader support

## Installation

```bash
npm install @webauthn/web-component
```

Or use directly from CDN:

```html
<!-- WebAuthn only -->
<script
  type="module"
  src="https://unpkg.com/@webauthn/web-component/dist/webauthn-auth.js"
></script>

<!-- WebSocket only -->
<script
  type="module"
  src="https://unpkg.com/@webauthn/web-component/dist/websocket-components.js"
></script>

<!-- All components -->
<script
  type="module"
  src="https://unpkg.com/@webauthn/web-component/dist/all-components.js"
></script>
```

## Quick Start

### WebAuthn Component

```html
<!DOCTYPE html>
<html>
  <head>
    <script
      type="module"
      src="./node_modules/@webauthn/web-component/dist/webauthn-auth.js"
    ></script>
  </head>
  <body>
    <webauthn-auth base-url="http://localhost:8080" theme="light">
    </webauthn-auth>
  </body>
</html>
```

### WebSocket Components

```html
<!DOCTYPE html>
<html>
  <head>
    <script
      type="module"
      src="./node_modules/@webauthn/web-component/dist/websocket-components.js"
    ></script>
  </head>
  <body>
    <!-- Connection status indicator -->
    <websocket-status
      status="disconnected"
      showText="true"
      showUserCount="true"
    >
    </websocket-status>

    <!-- Full WebSocket handler with UI -->
    <websocket-handler
      websocketUrl="ws://localhost:8080/ws"
      autoConnect="false"
      showDebugLog="true"
    >
    </websocket-handler>
  </body>
</html>
```

### All Components

```html
<!DOCTYPE html>
<html>
  <head>
    <script
      type="module"
      src="./node_modules/@webauthn/web-component/dist/all-components.js"
    ></script>
  </head>
  <body>
    <!-- All components are available -->
    <webauthn-auth base-url="http://localhost:8080"></webauthn-auth>
    <websocket-status status="connected"></websocket-status>
    <websocket-handler
      websocketUrl="ws://localhost:8080/ws"
    ></websocket-handler>
  </body>
</html>
```

### With Event Listeners

```html
<webauthn-auth id="auth" base-url="https://api.example.com"></webauthn-auth>

<script>
  const auth = document.getElementById('auth');

  auth.addEventListener('webauthn-login', (event) => {
    console.log('User logged in:', event.detail.username);
    // Redirect to dashboard, update UI, etc.
  });

  auth.addEventListener('webauthn-logout', () => {
    console.log('User logged out');
    // Clear session, redirect to home, etc.
  });

  auth.addEventListener('webauthn-error', (event) => {
    console.error('Authentication error:', event.detail.error);
    // Show error notification, log error, etc.
  });
</script>
```

## Components

### `<webauthn-auth>`

The main WebAuthn authentication component.

#### HTML Attributes

| Attribute  | Type                | Default                 | Description              |
| ---------- | ------------------- | ----------------------- | ------------------------ |
| `base-url` | `string`            | `http://localhost:8080` | WebAuthn server base URL |
| `theme`    | `light\|dark\|auto` | `auto`                  | Component theme          |
| `class`    | `string`            | -                       | Additional CSS classes   |

#### Custom Events

| Event Name        | Detail                 | Description                   |
| ----------------- | ---------------------- | ----------------------------- |
| `webauthn-login`  | `{ username: string }` | User successfully logged in   |
| `webauthn-logout` | -                      | User logged out               |
| `webauthn-error`  | `{ error: string }`    | Authentication error occurred |

### `<websocket-handler>`

A comprehensive WebSocket management component with UI for handling WebSocket connections, sending/receiving messages, and displaying media blobs.

#### HTML Attributes

| Attribute      | Type      | Default | Description                     |
| -------------- | --------- | ------- | ------------------------------- |
| `websocketUrl` | `string`  | `""`    | WebSocket server URL            |
| `autoConnect`  | `boolean` | `true`  | Whether to auto-connect on load |
| `showDebugLog` | `boolean` | `true`  | Whether to show debug log       |

#### Custom Events

| Event Name             | Detail                                      | Description                      |
| ---------------------- | ------------------------------------------- | -------------------------------- |
| `status-change`        | `{ status: ConnectionStatus }`              | Connection status changed        |
| `media-blobs-received` | `{ blobs: MediaBlob[], totalCount: number}` | Media blobs received from server |
| `media-blob-received`  | `{ blob: MediaBlob }`                       | Single media blob received       |

#### Methods (via DOM element)

```javascript
const handler = document.querySelector('websocket-handler');

// Send a ping message
handler.ping();

// Request media blobs
handler.getMediaBlobs(limit, offset);

// Request specific media blob
handler.getMediaBlob(id);

// Upload a media blob
handler.uploadMediaBlob(blobData);

// Manual connection control
handler.connect();
handler.disconnect();
```

### `<websocket-status>`

A minimal status indicator component showing WebSocket connection state.

#### HTML Attributes

| Attribute       | Type               | Default        | Description                 |
| --------------- | ------------------ | -------------- | --------------------------- |
| `status`        | `ConnectionStatus` | `disconnected` | Current connection status   |
| `showText`      | `boolean`          | `true`         | Whether to show status text |
| `userCount`     | `number`           | `0`            | Number of connected users   |
| `showUserCount` | `boolean`          | `false`        | Whether to show user count  |
| `compact`       | `boolean`          | `false`        | Compact display mode        |

#### Connection Status Types

```typescript
enum ConnectionStatus {
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Connected = 'connected',
  Error = 'error',
}
```

#### Custom Events

| Event Name      | Detail                                            | Description    |
| --------------- | ------------------------------------------------- | -------------- |
| `status-change` | `{ status: ConnectionStatus, timestamp: number }` | Status changed |

## Practical Examples

### Simple WebSocket Chat Interface

```html
<!DOCTYPE html>
<html>
  <head>
    <script type="module" src="./dist/websocket-components.js"></script>
    <style>
      .chat-container {
        max-width: 800px;
        margin: 0 auto;
        padding: 20px;
      }
      .status-bar {
        margin-bottom: 20px;
        padding: 10px;
        background: #f5f5f5;
        border-radius: 6px;
      }
    </style>
  </head>
  <body>
    <div class="chat-container">
      <div class="status-bar">
        <websocket-status
          status="disconnected"
          showText="true"
          showUserCount="true"
        >
        </websocket-status>
      </div>

      <websocket-handler
        websocketUrl="ws://localhost:8080/ws"
        autoConnect="true"
        showDebugLog="false"
      >
      </websocket-handler>
    </div>

    <script>
      // Update status indicator based on handler events
      document.addEventListener('status-change', (e) => {
        const statusEl = document.querySelector('websocket-status');
        statusEl.status = e.detail.status;
      });
    </script>
  </body>
</html>
```

### Combined Authentication & WebSocket

```html
<!DOCTYPE html>
<html>
  <head>
    <script type="module" src="./dist/all-components.js"></script>
    <style>
      .app {
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;
      }
      .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;
      }
      .main-content {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 20px;
      }
    </style>
  </head>
  <body>
    <div class="app">
      <div class="header">
        <h1>My App</h1>
        <websocket-status
          status="disconnected"
          showText="true"
        ></websocket-status>
      </div>

      <div class="main-content">
        <div class="auth-section">
          <h2>Authentication</h2>
          <webauthn-auth
            base-url="http://localhost:8080"
            theme="auto"
          ></webauthn-auth>
        </div>

        <div class="ws-section">
          <h2>Real-time Data</h2>
          <websocket-handler
            websocketUrl="ws://localhost:8080/ws"
            autoConnect="false"
            showDebugLog="true"
          >
          </websocket-handler>
        </div>
      </div>
    </div>

    <script>
      let isAuthenticated = false;

      // Handle authentication events
      document.addEventListener('webauthn-login', (e) => {
        isAuthenticated = true;
        console.log('User logged in:', e.detail.username);

        // Auto-connect WebSocket after authentication
        const wsHandler = document.querySelector('websocket-handler');
        wsHandler.connect();
      });

      document.addEventListener('webauthn-logout', () => {
        isAuthenticated = false;
        console.log('User logged out');

        // Disconnect WebSocket
        const wsHandler = document.querySelector('websocket-handler');
        wsHandler.disconnect();
      });

      // Update status indicator
      document.addEventListener('status-change', (e) => {
        const statusEl = document.querySelector('websocket-status');
        statusEl.status = e.detail.status;
      });
    </script>
  </body>
</html>
```

### React Integration Example

```jsx
import React, { useEffect, useRef, useState } from 'react';

// Import the web components (this registers them globally)
import '@webauthn/web-component';

function App() {
  const wsHandlerRef = useRef(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [wsStatus, setWsStatus] = useState('disconnected');
  const [mediaBlobs, setMediaBlobs] = useState([]);

  useEffect(() => {
    const handleLogin = (e) => {
      setIsAuthenticated(true);
      console.log('User logged in:', e.detail.username);

      // Connect WebSocket after login
      if (wsHandlerRef.current) {
        wsHandlerRef.current.connect();
      }
    };

    const handleLogout = () => {
      setIsAuthenticated(false);
      if (wsHandlerRef.current) {
        wsHandlerRef.current.disconnect();
      }
    };

    const handleStatusChange = (e) => {
      setWsStatus(e.detail.status);
    };

    const handleMediaBlobs = (e) => {
      setMediaBlobs(e.detail.blobs);
    };

    // Add event listeners
    document.addEventListener('webauthn-login', handleLogin);
    document.addEventListener('webauthn-logout', handleLogout);
    document.addEventListener('status-change', handleStatusChange);
    document.addEventListener('media-blobs-received', handleMediaBlobs);

    return () => {
      document.removeEventListener('webauthn-login', handleLogin);
      document.removeEventListener('webauthn-logout', handleLogout);
      document.removeEventListener('status-change', handleStatusChange);
      document.removeEventListener('media-blobs-received', handleMediaBlobs);
    };
  }, []);

  return (
    <div className='app'>
      <header>
        <h1>My React App</h1>
        <websocket-status
          status={wsStatus}
          showText='true'
          showUserCount='true'
        />
      </header>

      <main>
        <section>
          <h2>Authentication</h2>
          <webauthn-auth base-url='http://localhost:8080' theme='auto' />
        </section>

        {isAuthenticated && (
          <section>
            <h2>WebSocket Connection</h2>
            <websocket-handler
              ref={wsHandlerRef}
              websocketUrl='ws://localhost:8080/ws'
              autoConnect='false'
              showDebugLog='true'
            />

            <h3>Media Blobs ({mediaBlobs.length})</h3>
            <ul>
              {mediaBlobs.map((blob) => (
                <li key={blob.id}>
                  {blob.mime} - {blob.size} bytes
                </li>
              ))}
            </ul>
          </section>
        )}
      </main>
    </div>
  );
}

export default App;
```

### Vue.js Integration Example

```vue
<template>
  <div class="app">
    <header>
      <h1>My Vue App</h1>
      <websocket-status
        :status="wsStatus"
        show-text="true"
        :user-count="userCount"
        show-user-count="true"
      />
    </header>

    <main>
      <section>
        <h2>Authentication</h2>
        <webauthn-auth
          base-url="http://localhost:8080"
          theme="auto"
          @webauthn-login="handleLogin"
          @webauthn-logout="handleLogout"
          @webauthn-error="handleAuthError"
        />
      </section>

      <section v-if="isAuthenticated">
        <h2>WebSocket Connection</h2>
        <websocket-handler
          ref="wsHandler"
          :websocket-url="wsUrl"
          auto-connect="false"
          show-debug-log="true"
          @status-change="handleStatusChange"
          @media-blobs-received="handleMediaBlobs"
        />
      </section>
    </main>
  </div>
</template>

<script>
// Import web components
import '@webauthn/web-component';

export default {
  name: 'App',
  data() {
    return {
      isAuthenticated: false,
      wsStatus: 'disconnected',
      wsUrl: 'ws://localhost:8080/ws',
      userCount: 0,
      mediaBlobs: [],
    };
  },
  methods: {
    handleLogin(event) {
      this.isAuthenticated = true;
      console.log('User logged in:', event.detail.username);

      // Connect WebSocket
      this.$nextTick(() => {
        if (this.$refs.wsHandler) {
          this.$refs.wsHandler.connect();
        }
      });
    },

    handleLogout() {
      this.isAuthenticated = false;
      if (this.$refs.wsHandler) {
        this.$refs.wsHandler.disconnect();
      }
    },

    handleAuthError(event) {
      console.error('Auth error:', event.detail.error);
    },

    handleStatusChange(event) {
      this.wsStatus = event.detail.status;
    },

    handleMediaBlobs(event) {
      this.mediaBlobs = event.detail.blobs;
      this.userCount = event.detail.totalCount || 0;
    },
  },
};
</script>
```

### Programmatic Usage

You can also use the component programmatically with JavaScript frameworks:

```typescript
import { createWebAuthnAuth, WebAuthnAuth } from '@webauthn/web-component';

// Create component in a container
const dispose = createWebAuthnAuth(document.getElementById('container'), {
  baseUrl: 'https://api.example.com',
  theme: 'dark',
  onLogin: (username) => console.log('Logged in:', username),
  onLogout: () => console.log('Logged out'),
  onError: (error) => console.error('Error:', error),
});

// Clean up when done
dispose();
```

## Styling and Theming

### Built-in Themes

The component comes with three built-in themes:

- `light`: Light theme with clean, modern styling
- `dark`: Dark theme for low-light environments
- `auto`: Automatically adapts to system preference

```html
<webauthn-auth theme="dark"></webauthn-auth>
```

### Custom Styling

Use CSS custom properties to customize the appearance:

```css
webauthn-auth {
  --bg-color: #f8f9fa;
  --text-color: #333;
  --border-color: #dee2e6;
  --primary-color: #007bff;
  --error-color: #dc3545;
  --success-color: #28a745;
}
```

### CSS Classes

The component uses semantic CSS classes that you can target:

```css
/* Main container */
.webauthn-auth {
}

/* Form elements */
.webauthn-form {
}
.webauthn-input {
}
.webauthn-button {
}

/* Messages */
.webauthn-message {
}
.webauthn-message--error {
}
.webauthn-message--success {
}

/* User info when authenticated */
.webauthn-user-info {
}
.webauthn-username {
}
```

## Framework Integration

### React

```jsx
import { useEffect, useRef } from 'react';

function WebAuthnComponent({ onLogin, onLogout, onError }) {
  const ref = useRef();

  useEffect(() => {
    const element = ref.current;

    const handleLogin = (e) => onLogin?.(e.detail.username);
    const handleLogout = () => onLogout?.();
    const handleError = (e) => onError?.(e.detail.error);

    element.addEventListener('webauthn-login', handleLogin);
    element.addEventListener('webauthn-logout', handleLogout);
    element.addEventListener('webauthn-error', handleError);

    return () => {
      element.removeEventListener('webauthn-login', handleLogin);
      element.removeEventListener('webauthn-logout', handleLogout);
      element.removeEventListener('webauthn-error', handleError);
    };
  }, [onLogin, onLogout, onError]);

  return (
    <webauthn-auth ref={ref} base-url='https://api.example.com' theme='auto' />
  );
}
```

### Vue

```vue
<template>
  <webauthn-auth
    ref="authRef"
    base-url="https://api.example.com"
    theme="auto"
    @webauthn-login="handleLogin"
    @webauthn-logout="handleLogout"
    @webauthn-error="handleError"
  />
</template>

<script>
export default {
  methods: {
    handleLogin(event) {
      console.log('User logged in:', event.detail.username);
    },
    handleLogout() {
      console.log('User logged out');
    },
    handleError(event) {
      console.error('Auth error:', event.detail.error);
    },
  },
};
</script>
```

### Angular

```typescript
import { Component, ElementRef, ViewChild } from '@angular/core';

@Component({
  selector: 'app-auth',
  template: `
    <webauthn-auth
      #authElement
      base-url="https://api.example.com"
      theme="auto"
      (webauthn-login)="onLogin($event)"
      (webauthn-logout)="onLogout()"
      (webauthn-error)="onError($event)"
    >
    </webauthn-auth>
  `,
})
export class AuthComponent {
  @ViewChild('authElement') authElement!: ElementRef;

  onLogin(event: CustomEvent) {
    console.log('User logged in:', event.detail.username);
  }

  onLogout() {
    console.log('User logged out');
  }

  onError(event: CustomEvent) {
    console.error('Auth error:', event.detail.error);
  }
}
```

## Development

### Building

```bash
# Install dependencies
npm install

# Build for production
npm run build

# Build standalone version (includes all dependencies)
npm run build:standalone

# Development mode with watching
npm run watch
```

### Development Setup

```bash
# Clone the repository
git clone <repository-url>
cd clientlib/web-component

# Install dependencies
npm install

# Start development server
npm run dev
```

### Testing

The web component inherits the comprehensive testing setup from the parent clientlib:

```bash
# Run tests from the parent directory
cd ../
npm test

# Run specific web component tests
npm run test -- --testPathPattern=web-component
```

## Browser Compatibility

- **Modern Browsers**: Chrome 67+, Firefox 60+, Safari 14+, Edge 18+
- **WebAuthn Support**: Required for authentication functionality
- **Custom Elements**: Supported in all modern browsers
- **ES Modules**: Required for module imports

### Polyfills

For older browser support, you may need polyfills:

```html
<!-- Custom Elements polyfill for older browsers -->
<script src="https://unpkg.com/@webcomponents/custom-elements@1.5.0/custom-elements.min.js"></script>

<!-- WebAuthn polyfill (limited functionality) -->
<script src="https://unpkg.com/@github/webauthn-json@2.1.1/dist/esm/webauthn-json.browser-global.js"></script>
```

## Security Considerations

- **HTTPS Required**: WebAuthn requires HTTPS in production (localhost exempted)
- **Same-Origin**: Authentication server should be same-origin or properly configured for CORS
- **CSP**: Ensure Content Security Policy allows the component's inline styles
- **Input Validation**: Server-side validation is always required regardless of client-side validation

## Troubleshooting

### Common Issues

**Component not loading:**

- Ensure the script is loaded as a module: `<script type="module">`
- Check browser console for import errors
- Verify the file path is correct

**WebAuthn not working:**

- Check that you're using HTTPS (except on localhost)
- Verify the server supports WebAuthn endpoints
- Ensure the browser supports WebAuthn APIs

**Styling issues:**

- CSS custom properties require modern browser support
- Check that CSS isn't being overridden by other stylesheets
- Verify theme attribute is spelled correctly

**Events not firing:**

- Ensure event listeners are added after the component is loaded
- Check event names are spelled correctly
- Verify the component is properly connected to the DOM

### Debug Mode

Enable debug logging:

```javascript
// Enable debug mode (if implemented)
document.querySelector('webauthn-auth').debug = true;
```

## Examples

See the [examples directory](./examples/) for complete working examples:

- [Basic HTML Example](./examples/basic.html)
- [React Integration](./examples/react-example/)
- [Vue Integration](./examples/vue-example/)
- [Custom Styling](./examples/custom-theme.html)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run the test suite
6. Submit a pull request

## License

MIT License - see [LICENSE](../../LICENSE) file for details.

## Related Projects

- [@webauthn/clientlib](../) - Core TypeScript client library
- [WebAuthn Server](../../server/) - Rust/Axum WebAuthn server implementation
