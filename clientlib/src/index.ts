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

// Test utilities (useful for development and testing)
export { TestApiClient, testUtils, mockWebAuthn } from './test-helpers.js';

// Test data management
export { TestDataManager, testData } from './test-data.js';
export type { TestInviteCode } from './test-data.js';

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
