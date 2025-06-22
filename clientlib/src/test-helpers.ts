import { vi } from 'vitest';
import { ApiClient, ApiError } from './api-client.js';

export class TestApiClient extends ApiClient {
  constructor(baseUrl: string = 'http://localhost:8080') {
    super({ baseUrl });
  }

  // Helper method for testing error responses
  async expectError(
    operation: () => Promise<any>,
    expectedStatus?: number
  ): Promise<ApiError> {
    try {
      await operation();
      throw new Error('Expected operation to throw an error');
    } catch (error) {
      if (!(error instanceof ApiError)) {
        throw error;
      }
      if (expectedStatus && error.status !== expectedStatus) {
        throw new Error(
          `Expected status ${expectedStatus}, got ${error.status}`
        );
      }
      return error;
    }
  }

  // Helper for generating test data
  generateTestUser(suffix: string = '') {
    return {
      username: `testuser${suffix}`,
      displayName: `Test User${suffix}`,
    };
  }

  // Helper for making authenticated requests
  async withAuth<T>(operation: () => Promise<T>): Promise<T> {
    // This would typically handle authentication tokens
    // For WebAuthn, this might involve managing session cookies
    return await operation();
  }

  // Helper to check if server is running
  async isServerRunning(): Promise<boolean> {
    try {
      await this.health();
      return true;
    } catch {
      return false;
    }
  }

  // Helper to wait for server to be ready
  async waitForServer(timeoutMs: number = 10000): Promise<void> {
    const startTime = Date.now();
    while (Date.now() - startTime < timeoutMs) {
      if (await this.isServerRunning()) {
        return;
      }
      await testUtils.delay(500);
    }
    throw new Error(`Server not ready after ${timeoutMs}ms`);
  }
}

// Test utilities
export const testUtils = {
  randomString: (length: number = 8): string =>
    Math.random()
      .toString(36)
      .substring(2, length + 2),

  delay: (ms: number): Promise<void> =>
    new Promise((resolve) => setTimeout(resolve, ms)),

  retry: async <T>(
    operation: () => Promise<T>,
    maxAttempts: number = 3,
    delayMs: number = 1000
  ): Promise<T> => {
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        return await operation();
      } catch (error) {
        if (attempt === maxAttempts) throw error;
        await testUtils.delay(delayMs);
      }
    }
    throw new Error('Should not reach here');
  },

  // Generate test WebAuthn credential data
  generateMockCredential: () => ({
    id: testUtils.randomString(32),
    rawId: testUtils.randomString(32),
    response: {
      attestationObject: testUtils.randomString(64),
      clientDataJSON: testUtils.randomString(128),
    },
    type: 'public-key' as const,
  }),

  // Generate test WebAuthn assertion data
  generateMockAssertion: () => ({
    id: testUtils.randomString(32),
    rawId: testUtils.randomString(32),
    response: {
      authenticatorData: testUtils.randomString(64),
      clientDataJSON: testUtils.randomString(128),
      signature: testUtils.randomString(64),
      userHandle: testUtils.randomString(16),
    },
    type: 'public-key' as const,
  }),

  // Create a test environment setup
  setupTestEnvironment: () => {
    const originalConsoleError = console.error;
    const errorLogs: string[] = [];

    // Capture console.error during tests
    console.error = (...args: any[]) => {
      errorLogs.push(args.join(' '));
      originalConsoleError(...args);
    };

    return {
      getErrorLogs: () => errorLogs,
      cleanup: () => {
        console.error = originalConsoleError;
      },
    };
  },
};

// Mock WebAuthn API for browser environment testing
export const mockWebAuthn = {
  // Mock navigator.credentials.create
  mockCreate: (mockResponse?: any) => {
    const originalCreate = navigator.credentials?.create;
    const mockFn = vi
      .fn()
      .mockResolvedValue(mockResponse || testUtils.generateMockCredential());

    if (navigator.credentials) {
      navigator.credentials.create = mockFn;
    }

    return {
      mockFn,
      restore: () => {
        if (navigator.credentials && originalCreate) {
          navigator.credentials.create = originalCreate;
        }
      },
    };
  },

  // Mock navigator.credentials.get
  mockGet: (mockResponse?: any) => {
    const originalGet = navigator.credentials?.get;
    const mockFn = vi
      .fn()
      .mockResolvedValue(mockResponse || testUtils.generateMockAssertion());

    if (navigator.credentials) {
      navigator.credentials.get = mockFn;
    }

    return {
      mockFn,
      restore: () => {
        if (navigator.credentials && originalGet) {
          navigator.credentials.get = originalGet;
        }
      },
    };
  },
};
