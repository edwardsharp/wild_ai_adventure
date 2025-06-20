import { ApiClient, ApiError } from './api-client';
import { expect } from '@jest/globals';

export class TestApiClient extends ApiClient {
  constructor(baseUrl: string = '{{BASE_URL}}') {
    super(baseUrl);
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
        throw new Error(`Expected status ${expectedStatus}, got ${error.status}`);
      }
      return error;
    }
  }

  // Helper for generating test data
  generateTestUser(suffix: string = '') {
    return {
      username: `testuser${suffix}`,
      display_name: `Test User${suffix}`
    };
  }

  // Helper for making authenticated requests
  async withAuth<T>(operation: () => Promise<T>): Promise<T> {
    // This would typically handle authentication tokens
    // For WebAuthn, this might involve managing session cookies
    return await operation();
  }
}

// Test utilities
export const testUtils = {
  randomString: (length: number = 8) =>
    Math.random().toString(36).substring(2, length + 2),

  delay: (ms: number) => new Promise(resolve => setTimeout(resolve, ms)),

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
  }
};
