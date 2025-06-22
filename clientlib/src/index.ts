// Main API client exports
export { ApiClient, ApiError, apiClient } from './api-client.js';
export type { ApiClientConfig } from './api-client.js';

// API specification and types
export { API_SPEC } from './api-spec.js';
export type {
  ApiSpec,
  EndpointName,
  EndpointConfig,
  RegisterStartRequest,
  RegisterStartResponse,
  RegisterStartQueryParams,
  RegisterFinishRequest,
  RegisterFinishResponse,
  LoginStartRequest,
  LoginStartResponse,
  LoginFinishRequest,
  LoginFinishResponse,
  LogoutRequest,
  LogoutResponse,
  HealthRequest,
  HealthResponse,
  AuthStatusRequest,
  AuthStatusResponse,
  WebAuthnCredential,
  WebAuthnAssertion,
  WebAuthnPublicKeyCredentialCreationOptions,
  WebAuthnPublicKeyCredentialRequestOptions,
} from './api-spec.js';

// Note: Test utilities are available in individual files for testing environments
// but not exported from main index to avoid Node.js dependencies in browser builds

// WebSocket client and types
export { WebSocketClient } from './websocket-client.js';
export type {
  WebSocketClientConfig,
  WebSocketClientEvents,
} from './websocket-client.js';

// WebSocket message types and schemas
export {
  MediaBlobSchema,
  WebSocketMessageSchema,
  WebSocketResponseSchema,
  ConnectionStatus,
  createMessage,
  parseWebSocketMessage,
  parseWebSocketResponse,
  safeParseWebSocketResponse,
  isWelcomeMessage,
  isMediaBlobsMessage,
  isErrorMessage,
  isConnectionStatusMessage,
} from './websocket-types.js';
export type {
  MediaBlob,
  WebSocketMessage,
  WebSocketResponse,
} from './websocket-types.js';

// Zod schemas for external validation
export {
  WebAuthnPublicKeyCredentialCreationOptionsSchema,
  WebAuthnPublicKeyCredentialRequestOptionsSchema,
  WebAuthnCredentialSchema,
  WebAuthnAssertionSchema,
} from './api-spec.js';

// Re-export zod for convenience
export { z } from 'zod';

// Version info
export const VERSION = '1.0.0';

// Default exports for easy consumption
import { apiClient } from './api-client.js';
export { apiClient as default };
