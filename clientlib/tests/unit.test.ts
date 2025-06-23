import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { ApiClient, ApiError } from '../src/lib/api-client.js';

// Mock fetch globally
global.fetch = vi.fn();
const mockFetch = fetch as any;

describe('ApiClient Unit Tests', () => {
  let client: ApiClient;

  beforeEach(() => {
    client = new ApiClient({
      baseUrl: 'http://test.example.com',
      timeout: 5000,
    });
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  describe('Constructor and Configuration', () => {
    it('should initialize with default config', () => {
      const defaultClient = new ApiClient();
      expect(defaultClient).toBeDefined();
    });

    it('should initialize with custom config', () => {
      const customClient = new ApiClient({
        baseUrl: 'http://custom.example.com',
        defaultHeaders: { 'X-Custom': 'test' },
        timeout: 10000,
        credentials: 'same-origin',
      });
      expect(customClient).toBeDefined();
    });

    it('should allow updating base URL', () => {
      client.setBaseUrl('http://new.example.com');
      expect(client).toBeDefined();
    });

    it('should allow updating timeout', () => {
      client.setTimeout(15000);
      expect(client).toBeDefined();
    });

    it('should allow updating credentials', () => {
      client.setCredentials('omit');
      expect(client).toBeDefined();
    });
  });

  describe('Header Management', () => {
    it('should set headers', () => {
      client.setHeader('Authorization', 'Bearer token');
      const headers = client.getHeaders();
      expect(headers['Authorization']).toBe('Bearer token');
    });

    it('should remove headers', () => {
      client.setHeader('Authorization', 'Bearer token');
      client.removeHeader('Authorization');
      const headers = client.getHeaders();
      expect(headers['Authorization']).toBeUndefined();
    });

    it('should get all headers', () => {
      client.setHeader('X-Test', 'value');
      client.setHeader('X-Another', 'another');
      const headers = client.getHeaders();
      expect(headers['X-Test']).toBe('value');
      expect(headers['X-Another']).toBe('another');
    });
  });

  describe('Health Check', () => {
    it('should make successful health check', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'text/plain' }),
        text: async () => '',
      } as Response);

      await client.health();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/health',
        expect.objectContaining({
          method: 'GET',
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
          }),
          credentials: 'include',
        })
      );
    });

    it('should handle health check error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        text: async () => 'Internal Server Error',
      } as Response);

      await expect(client.health()).rejects.toThrow(ApiError);
    });
  });

  describe('Registration Flow', () => {
    const mockRegisterStartResponse = {
      publicKey: {
        challenge: 'test-challenge',
        rp: { id: 'test-rp', name: 'Test RP' },
        user: {
          id: 'user-id',
          name: 'testuser',
          displayName: 'testuser',
        },
        pubKeyCredParams: [],
        timeout: 60000,
        excludeCredentials: [],
        authenticatorSelection: {
          residentKey: 'preferred',
          requireResidentKey: false,
          userVerification: 'preferred',
        },
        attestation: 'none',
      },
    };

    it('should start registration successfully', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockRegisterStartResponse,
      } as Response);

      const result = await client.registerStart('testuser');

      expect(result.publicKey.challenge).toBe('test-challenge');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/register_start/testuser',
        expect.objectContaining({
          method: 'POST',
        })
      );
    });

    it('should start registration with invite code', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockRegisterStartResponse,
      } as Response);

      await client.registerStart('testuser', { invite_code: 'test-invite' });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/register_start/testuser?invite_code=test-invite',
        expect.objectContaining({
          method: 'POST',
        })
      );
    });

    it('should finish registration successfully', async () => {
      const mockCredential = {
        id: 'credential-id',
        rawId: 'raw-credential-id',
        response: {
          attestationObject: 'attestation-object',
          clientDataJSON: 'client-data-json',
        },
        type: 'public-key' as const,
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => ({ message: 'Registration successful' }),
      } as Response);

      await client.registerFinish(mockCredential);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/register_finish',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(mockCredential),
        })
      );
    });
  });

  describe('Login Flow', () => {
    const mockLoginStartResponse = {
      publicKey: {
        challenge: 'login-challenge',
        timeout: 60000,
        rpId: 'test-rp',
        allowCredentials: [],
        userVerification: 'preferred',
      },
    };

    it('should start login successfully', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockLoginStartResponse,
      } as Response);

      const result = await client.loginStart('testuser');

      expect(result.publicKey.challenge).toBe('login-challenge');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/login_start/testuser',
        expect.objectContaining({
          method: 'POST',
        })
      );
    });

    it('should finish login successfully', async () => {
      const mockAssertion = {
        id: 'assertion-id',
        rawId: 'raw-assertion-id',
        response: {
          authenticatorData: 'authenticator-data',
          clientDataJSON: 'client-data-json',
          signature: 'signature',
          userHandle: 'user-handle',
        },
        type: 'public-key' as const,
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => ({ message: 'Login successful' }),
      } as Response);

      await client.loginFinish(mockAssertion);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/login_finish',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(mockAssertion),
        })
      );
    });
  });

  describe('Authentication Status', () => {
    it('should check authentication status', async () => {
      const mockAuthStatus = {
        authenticated: true,
        user_id: 'test-user-id',
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockAuthStatus,
      } as Response);

      const result = await client.authStatus();

      expect(result.authenticated).toBe(true);
      expect(result.user_id).toBe('test-user-id');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/auth/status',
        expect.objectContaining({
          method: 'GET',
        })
      );
    });
  });

  describe('Logout', () => {
    it('should logout successfully', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => ({ message: 'Logged out successfully' }),
      } as Response);

      await client.logout();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/logout',
        expect.objectContaining({
          method: 'POST',
        })
      );
    });
  });

  describe('Error Handling', () => {
    it('should handle HTTP errors', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        text: async () => 'Not Found',
      } as Response);

      await expect(client.health()).rejects.toThrow(ApiError);
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(client.health()).rejects.toThrow(ApiError);
    });

    it('should handle timeout errors', async () => {
      // Mock fetch to simulate an aborted request
      mockFetch.mockImplementationOnce(() => {
        const abortError = new DOMException(
          'The operation was aborted.',
          'AbortError'
        );
        return Promise.reject(abortError);
      });

      const timeoutClient = new ApiClient({
        baseUrl: 'http://test.example.com',
        timeout: 100,
      });

      await expect(timeoutClient.health()).rejects.toThrow(
        'Request timeout after 100ms'
      );
    });

    it('should create ApiError with proper details', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 422,
        text: async () => 'Validation failed',
      } as Response);

      try {
        await client.health();
        expect.fail('Should have thrown an error');
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        const apiError = error as ApiError;
        expect(apiError.status).toBe(422);
        expect(apiError.responseText).toBe('Validation failed');
        expect(apiError.endpoint).toBe('health');
      }
    });
  });

  describe('URL Building', () => {
    it('should build URLs with path parameters', async () => {
      const mockRegisterResponse = {
        publicKey: {
          challenge: 'test-challenge',
          rp: { id: 'example.com', name: 'Test Site' },
          user: {
            id: 'user-id',
            name: 'test@user.com',
            displayName: 'Test User',
          },
          pubKeyCredParams: [],
          timeout: 60000,
          excludeCredentials: [],
          authenticatorSelection: {
            residentKey: 'preferred',
            requireResidentKey: false,
            userVerification: 'preferred',
          },
          attestation: 'none',
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockRegisterResponse,
      } as Response);

      await client.registerStart('test@user.com');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/register_start/test%40user.com',
        expect.any(Object)
      );
    });

    it('should build URLs with query parameters', async () => {
      const mockRegisterResponse = {
        publicKey: {
          challenge: 'test-challenge',
          rp: { id: 'example.com', name: 'Test Site' },
          user: { id: 'user-id', name: 'testuser', displayName: 'Test User' },
          pubKeyCredParams: [],
          timeout: 60000,
          excludeCredentials: [],
          authenticatorSelection: {
            residentKey: 'preferred',
            requireResidentKey: false,
            userVerification: 'preferred',
          },
          attestation: 'none',
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockRegisterResponse,
      } as Response);

      await client.registerStart('testuser', { invite_code: 'special-invite' });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/register_start/testuser?invite_code=special-invite',
        expect.any(Object)
      );
    });

    it('should handle undefined query parameters', async () => {
      const mockRegisterResponse = {
        publicKey: {
          challenge: 'test-challenge',
          rp: { id: 'example.com', name: 'Test Site' },
          user: { id: 'user-id', name: 'testuser', displayName: 'Test User' },
          pubKeyCredParams: [],
          timeout: 60000,
          excludeCredentials: [],
          authenticatorSelection: {
            residentKey: 'preferred',
            requireResidentKey: false,
            userVerification: 'preferred',
          },
          attestation: 'none',
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockRegisterResponse,
      } as Response);

      await client.registerStart('testuser', { invite_code: undefined });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test.example.com/register_start/testuser',
        expect.any(Object)
      );
    });
  });

  describe('Request Configuration', () => {
    it('should include custom headers', async () => {
      client.setHeader('X-Custom-Header', 'custom-value');

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'text/plain' }),
        text: async () => '',
      } as Response);

      await client.health();

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            'X-Custom-Header': 'custom-value',
          }),
        })
      );
    });

    it('should include credentials', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'text/plain' }),
        text: async () => '',
      } as Response);

      await client.health();

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          credentials: 'include',
        })
      );
    });
  });

  describe('Response Handling', () => {
    it('should handle JSON responses', async () => {
      const mockAuthStatus = {
        authenticated: false,
        user_id: undefined,
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: async () => mockAuthStatus,
      } as Response);

      const result = await client.authStatus();
      expect(result.authenticated).toBe(false);
    });

    it('should handle text responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'text/plain' }),
        text: async () => 'OK',
      } as Response);

      await client.health();
      // Should not throw an error
    });

    it('should handle empty responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'text/plain' }),
        text: async () => '',
      } as Response);

      await client.health();
      // Should not throw an error
    });
  });
});

describe('ApiError', () => {
  it('should create error with basic info', () => {
    const error = new ApiError('Test error', 500, 'Internal Server Error');
    expect(error.message).toBe('Test error');
    expect(error.status).toBe(500);
    expect(error.responseText).toBe('Internal Server Error');
    expect(error.name).toBe('ApiError');
  });

  it('should create error with endpoint info', () => {
    const error = new ApiError('Test error', 404, 'Not Found', 'testEndpoint');
    expect(error.endpoint).toBe('testEndpoint');
  });

  it('should create error from response', async () => {
    const mockResponse = {
      status: 422,
      text: async () => 'Validation Error',
    } as Response;

    const error = await ApiError.fromResponse(mockResponse, 'register');
    expect(error.status).toBe(422);
    expect(error.responseText).toBe('Validation Error');
    expect(error.endpoint).toBe('register');
    expect(error.message).toBe('HTTP 422: Validation Error');
  });
});
