import { z } from 'zod';

// Error handling
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public responseText: string,
    public endpoint?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }

  static async fromResponse(
    response: Response,
    endpoint?: string
  ): Promise<ApiError> {
    const responseText = await response.text();
    return new ApiError(
      `HTTP ${response.status}: ${responseText}`,
      response.status,
      responseText,
      endpoint
    );
  }
}

// WebAuthn specific schemas
export const WebAuthnPublicKeyCredentialCreationOptionsSchema = z.object({
  challenge: z.string(),
  rp: z.object({
    id: z.string(),
    name: z.string(),
  }),
  user: z.object({
    id: z.string(),
    name: z.string(),
    displayName: z.string(),
  }),
  pubKeyCredParams: z.array(z.any()),
  timeout: z.number(),
  excludeCredentials: z.array(z.any()),
  authenticatorSelection: z.object({
    residentKey: z.string(),
    requireResidentKey: z.boolean(),
    userVerification: z.string(),
  }),
  attestation: z.string(),
  extensions: z.object({}).optional(),
});

export const WebAuthnPublicKeyCredentialRequestOptionsSchema = z.object({
  challenge: z.string(),
  timeout: z.number(),
  rp_id: z.string(),
  allow_credentials: z.array(z.any()),
  user_verification: z.string(),
});

export const WebAuthnCredentialSchema = z.object({
  id: z.string(),
  rawId: z.string(),
  response: z.object({
    attestationObject: z.string(),
    clientDataJSON: z.string(),
  }),
  type: z.string(),
});

export const WebAuthnAssertionSchema = z.object({
  id: z.string(),
  rawId: z.string(),
  response: z.object({
    authenticatorData: z.string().optional(),
    clientDataJSON: z.string(),
    signature: z.string().optional(),
    userHandle: z.string().optional(),
  }),
  type: z.string(),
});

// API response schemas
const RegisterStartResponseSchema = z.object({
  publicKey: WebAuthnPublicKeyCredentialCreationOptionsSchema,
});

const LoginStartResponseSchema = z.object({
  publicKey: WebAuthnPublicKeyCredentialRequestOptionsSchema,
});

const AuthStatusResponseSchema = z.object({
  authenticated: z.boolean(),
  username: z.string().optional(),
});

// Configuration interface
export interface ApiClientConfig {
  baseUrl?: string;
  defaultHeaders?: Record<string, string>;
  timeout?: number;
  credentials?: RequestCredentials;
}

// Main API Client class
export class ApiClient {
  private baseUrl: string;
  private defaultHeaders: Record<string, string>;
  private timeout: number;
  private credentials: RequestCredentials;

  constructor(config: ApiClientConfig = {}) {
    this.baseUrl = config.baseUrl ?? 'http://localhost:8080';
    this.defaultHeaders = config.defaultHeaders ?? {};
    this.timeout = config.timeout ?? 30000;
    this.credentials = config.credentials ?? 'include';
  }

  // Header management
  setHeader(key: string, value: string): void {
    this.defaultHeaders[key] = value;
  }

  removeHeader(key: string): void {
    delete this.defaultHeaders[key];
  }

  // Configuration updates
  setBaseUrl(baseUrl: string): void {
    this.baseUrl = baseUrl;
  }

  setTimeout(timeout: number): void {
    this.timeout = timeout;
  }

  setCredentials(credentials: RequestCredentials): void {
    this.credentials = credentials;
  }

  // Private method to build URL with path parameters and query parameters
  private buildUrl(
    path: string,
    pathParams?: Record<string, string>,
    queryParams?: Record<string, any>
  ): string {
    let url = path;

    // Replace path parameters
    if (pathParams) {
      Object.entries(pathParams).forEach(([key, value]) => {
        url = url.replace(`{${key}}`, encodeURIComponent(value));
      });
    }

    // Add query parameters
    if (queryParams) {
      const searchParams = new URLSearchParams();
      Object.entries(queryParams).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      const queryString = searchParams.toString();
      if (queryString) {
        url += `?${queryString}`;
      }
    }

    return `${this.baseUrl}${url}`;
  }

  // Generic request method with timeout and validation
  private async request<T>(
    method: string,
    url: string,
    options: {
      body?: any;
      headers?: Record<string, string>;
      responseSchema?: z.ZodSchema<T>;
      requestSchema?: z.ZodSchema<any>;
      endpoint?: string;
    } = {}
  ): Promise<T> {
    const {
      body,
      headers = {},
      responseSchema,
      requestSchema,
      endpoint,
    } = options;

    // Validate request body if schema provided
    if (requestSchema && body !== undefined) {
      requestSchema.parse(body);
    }

    const requestHeaders = {
      'Content-Type': 'application/json',
      ...this.defaultHeaders,
      ...headers,
    };

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        method,
        headers: requestHeaders,
        body: body !== undefined ? JSON.stringify(body) : null,
        credentials: this.credentials,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw await ApiError.fromResponse(response, endpoint);
      }

      // Handle void responses
      if (!responseSchema) {
        return undefined as T;
      }

      let data: any;
      const contentType = response.headers.get('content-type');

      if (contentType?.includes('application/json')) {
        data = await response.json();
      } else {
        const text = await response.text();
        data = text || undefined;
      }

      // Validate response if schema provided
      if (responseSchema) {
        return responseSchema.parse(data);
      }

      return data;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof ApiError) {
        throw error;
      }

      if (error instanceof DOMException && error.name === 'AbortError') {
        throw new ApiError(
          `Request timeout after ${this.timeout}ms`,
          408,
          'Request Timeout',
          endpoint
        );
      }

      throw new ApiError(
        `Network error: ${error instanceof Error ? error.message : 'Unknown error'}`,
        0,
        String(error),
        endpoint
      );
    }
  }

  // WebAuthn Registration Flow
  async registerStart(
    username: string,
    queryParams?: { invite_code?: string }
  ): Promise<z.infer<typeof RegisterStartResponseSchema>> {
    const url = this.buildUrl('/register_start/{username}', { username }, queryParams);

    return this.request('POST', url, {
      responseSchema: RegisterStartResponseSchema,
      endpoint: 'registerStart',
    });
  }

  async registerFinish(
    request: z.infer<typeof WebAuthnCredentialSchema>
  ): Promise<{ message?: string } | undefined> {
    const url = this.buildUrl('/register_finish');

    return this.request('POST', url, {
      body: request,
      requestSchema: WebAuthnCredentialSchema,
      endpoint: 'registerFinish',
    });
  }

  // WebAuthn Login Flow
  async loginStart(
    username: string
  ): Promise<z.infer<typeof LoginStartResponseSchema>> {
    const url = this.buildUrl('/login_start/{username}', { username });

    return this.request('POST', url, {
      responseSchema: LoginStartResponseSchema,
      endpoint: 'loginStart',
    });
  }

  async loginFinish(
    request: z.infer<typeof WebAuthnAssertionSchema>
  ): Promise<{ message?: string } | undefined> {
    const url = this.buildUrl('/login_finish');

    return this.request('POST', url, {
      body: request,
      requestSchema: WebAuthnAssertionSchema,
      endpoint: 'loginFinish',
    });
  }

  // Authentication Management
  async logout(): Promise<{ message?: string } | undefined> {
    const url = this.buildUrl('/logout');

    return this.request('POST', url, {
      endpoint: 'logout',
    });
  }

  async authStatus(): Promise<z.infer<typeof AuthStatusResponseSchema>> {
    const url = this.buildUrl('/auth/status');

    return this.request('GET', url, {
      responseSchema: AuthStatusResponseSchema,
      endpoint: 'authStatus',
    });
  }

  // Health Check
  async health(): Promise<void> {
    const url = this.buildUrl('/health');

    return this.request('GET', url, {
      endpoint: 'health',
    });
  }
}

// Types for external use
export type RegisterStartResponse = z.infer<typeof RegisterStartResponseSchema>;
export type LoginStartResponse = z.infer<typeof LoginStartResponseSchema>;
export type AuthStatusResponse = z.infer<typeof AuthStatusResponseSchema>;
export type WebAuthnCredential = z.infer<typeof WebAuthnCredentialSchema>;
export type WebAuthnAssertion = z.infer<typeof WebAuthnAssertionSchema>;
