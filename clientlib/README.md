# WebAuthn Client Library

A comprehensive TypeScript client library for WebAuthn authentication with Zod validation, fetch wrappers, and extensive testing utilities.

## Features

- 🔒 **WebAuthn Support**: Full WebAuthn registration and authentication flow
- 📝 **TypeScript**: Full type safety with comprehensive TypeScript definitions
- ✅ **Zod Validation**: Automatic request/response validation using Zod schemas
- 🧪 **Testing Utilities**: Comprehensive test helpers and mock utilities
- 🔌 **Web Components**: Ready-to-use Solid.js web component
- 🎯 **Framework Agnostic**: Works with any JavaScript framework or vanilla JS
- 📦 **ESM Ready**: Modern ES modules with tree-shaking support

## Installation

```bash
npm install @webauthn/clientlib
```

## Quick Start

### Basic Usage

```typescript
import { ApiClient, apiClient } from '@webauthn/clientlib';

// Use the default client instance
const challenge = await apiClient.registerStart('username');

// Or create a custom client
const client = new ApiClient({
  baseUrl: 'https://your-api.com',
  timeout: 10000,
});
```

### WebAuthn Registration Flow

```typescript
import { apiClient } from '@webauthn/clientlib';

async function registerUser(username: string, inviteCode?: string) {
  try {
    // Start registration
    const challenge = await apiClient.registerStart(username, {
      invite_code: inviteCode
    });

    // Create WebAuthn credential (browser API)
    const credential = await navigator.credentials.create({
      publicKey: {
        ...challenge.publicKey,
        challenge: base64ToUint8Array(challenge.publicKey.challenge),
        user: {
          ...challenge.publicKey.user,
          id: base64ToUint8Array(challenge.publicKey.user.id),
        },
      },
    });

    // Finish registration
    await apiClient.registerFinish({
      id: credential.id,
      rawId: uint8ArrayToBase64(new Uint8Array(credential.rawId)),
      type: credential.type,
      response: {
        attestationObject: uint8ArrayToBase64(
          new Uint8Array(credential.response.attestationObject)
        ),
        clientDataJSON: uint8ArrayToBase64(
          new Uint8Array(credential.response.clientDataJSON)
        ),
      },
    });

    console.log('Registration successful!');
  } catch (error) {
    console.error('Registration failed:', error);
  }
}
```

### WebAuthn Login Flow

```typescript
async function loginUser(username: string) {
  try {
    // Start login
    const challenge = await apiClient.loginStart(username);

    // Get WebAuthn assertion
    const assertion = await navigator.credentials.get({
      publicKey: {
        ...challenge.publicKey,
        challenge: base64ToUint8Array(challenge.publicKey.challenge),
        allowCredentials: challenge.publicKey.allow_credentials?.map(cred => ({
          ...cred,
          id: base64ToUint8Array(cred.id),
        })),
      },
    });

    // Finish login
    await apiClient.loginFinish({
      id: assertion.id,
      rawId: uint8ArrayToBase64(new Uint8Array(assertion.rawId)),
      type: assertion.type,
      response: {
        authenticatorData: uint8ArrayToBase64(
          new Uint8Array(assertion.response.authenticatorData)
        ),
        clientDataJSON: uint8ArrayToBase64(
          new Uint8Array(assertion.response.clientDataJSON)
        ),
        signature: uint8ArrayToBase64(
          new Uint8Array(assertion.response.signature)
        ),
        userHandle: assertion.response.userHandle
          ? uint8ArrayToBase64(new Uint8Array(assertion.response.userHandle))
          : undefined,
      },
    });

    console.log('Login successful!');
  } catch (error) {
    console.error('Login failed:', error);
  }
}
```

## Web Component

Use the ready-made Solid.js web component for quick integration:

### HTML Usage

```html
<!DOCTYPE html>
<html>
<head>
  <script type="module" src="./node_modules/@webauthn/clientlib/web-component/dist/webauthn-auth.js"></script>
</head>
<body>
  <webauthn-auth
    base-url="http://localhost:8080"
    theme="light">
  </webauthn-auth>

  <script>
    document.querySelector('webauthn-auth').addEventListener('webauthn-login', (e) => {
      console.log('User logged in:', e.detail.username);
    });

    document.querySelector('webauthn-auth').addEventListener('webauthn-logout', () => {
      console.log('User logged out');
    });

    document.querySelector('webauthn-auth').addEventListener('webauthn-error', (e) => {
      console.error('Auth error:', e.detail.error);
    });
  </script>
</body>
</html>
```

### Programmatic Usage

```typescript
import { createWebAuthnAuth } from '@webauthn/clientlib/web-component';

const dispose = createWebAuthnAuth(document.getElementById('auth-container'), {
  baseUrl: 'http://localhost:8080',
  theme: 'dark',
  onLogin: (username) => console.log('Logged in:', username),
  onLogout: () => console.log('Logged out'),
  onError: (error) => console.error('Auth error:', error),
});

// Later, cleanup
dispose();
```

## API Reference

### ApiClient

The main client class for interacting with the WebAuthn API.

#### Constructor

```typescript
new ApiClient(config?: ApiClientConfig)
```

#### Configuration

```typescript
interface ApiClientConfig {
  baseUrl?: string;          // Default: 'http://localhost:8080'
  defaultHeaders?: Record<string, string>;
  timeout?: number;          // Default: 30000ms
  credentials?: RequestCredentials; // Default: 'include'
}
```

#### Methods

- `registerStart(username: string, queryParams?: { invite_code?: string })`
- `registerFinish(credential: WebAuthnCredential)`
- `loginStart(username: string)`
- `loginFinish(assertion: WebAuthnAssertion)`
- `logout()`
- `authStatus()`
- `health()`

#### Header Management

```typescript
client.setHeader('Authorization', 'Bearer token');
client.removeHeader('Authorization');
const headers = client.getHeaders();
```

#### Configuration Updates

```typescript
client.setBaseUrl('https://new-api.com');
client.setTimeout(60000);
client.setCredentials('same-origin');
```

### Error Handling

```typescript
import { ApiError } from '@webauthn/clientlib';

try {
  await apiClient.registerStart('username');
} catch (error) {
  if (error instanceof ApiError) {
    console.error(`API Error ${error.status}: ${error.message}`);
    console.error('Response:', error.responseText);
    console.error('Endpoint:', error.endpoint);
  }
}
```

## Testing

### Test Utilities

```typescript
import { TestApiClient, testUtils } from '@webauthn/clientlib';

const testClient = new TestApiClient('http://localhost:8080');

// Generate test data
const user = testUtils.generateTestUser('_test');
const mockCredential = testUtils.generateMockCredential();

// Test error scenarios
await testClient.expectError(
  () => testClient.registerStart(''),
  405
);
```

### Running Tests

```bash
# Run all tests
npm test

# Run only unit tests
npm run test:unit

# Run only integration tests
npm run test:integration

# Run tests with coverage
npm run test:coverage

# Watch mode
npm run test:watch
```

### Integration Tests

Integration tests require a running WebAuthn server:

```bash
# Start your WebAuthn server
cargo run --bin server

# Run integration tests
npm run test:integration
```

## Development

### Building

```bash
# Build the library
npm run build

# Build web component
npm run build:web-component

# Development mode (watch)
npm run dev
```

### Linting and Formatting

```bash
# Lint code
npm run lint

# Fix linting issues
npm run lint:fix

# Format code
npm run format
```

## API Specification

The library includes a comprehensive API specification that describes all available routes:

```typescript
import { API_SPEC } from '@webauthn/clientlib';

console.log(API_SPEC.endpoints.registerStart);
// {
//   method: 'POST',
//   path: '/register_start/{username}',
//   pathParams: ['username'],
//   queryParams: { invite_code: z.string().optional() },
//   requestSchema: z.void(),
//   responseSchema: z.object({ ... })
// }
```

## TypeScript Support

Full TypeScript support with comprehensive type definitions:

```typescript
import type {
  RegisterStartResponse,
  LoginStartResponse,
  WebAuthnCredential,
  WebAuthnAssertion,
  ApiClientConfig
} from '@webauthn/clientlib';
```

## Browser Compatibility

- Modern browsers with WebAuthn support
- Chrome 67+, Firefox 60+, Safari 14+, Edge 18+
- Requires HTTPS in production (except localhost)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## License

MIT License - see LICENSE file for details

## Related Projects

- [WebAuthn Server](../server/) - Rust/Axum WebAuthn server implementation
- [WebAuthn CLI](../cli/) - Command-line tools for WebAuthn management
