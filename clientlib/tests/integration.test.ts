import { describe, it, expect, beforeAll } from 'vitest';
import { TestApiClient, testUtils } from '../src/lib/test-helpers.js';
import { ApiError } from '../src/lib/api-client.js';
import testCodes from '../test-data/invite-codes.json' assert { type: 'json' };

describe('WebAuthn API Integration Tests', () => {
  let client: TestApiClient;
  let inviteCodeIndex = 0;

  beforeAll(() => {
    // Use the test server URL - this should match your Rust test server
    client = new TestApiClient(
      process.env.API_BASE_URL || 'http://localhost:8080'
    );
  });

  // Helper to get a fresh invite code for each test
  function getNextInviteCode(): string {
    if (inviteCodeIndex >= testCodes.length) {
      throw new Error(
        'Not enough test invite codes. Run: node setup-test-codes.js'
      );
    }
    return testCodes[inviteCodeIndex++];
  }

  describe('Health Check', () => {
    it('should return healthy status', async () => {
      await client.health();
      // If no error is thrown, the health check passed
    });
  });

  describe('User Registration Flow', () => {
    it('should start registration successfully', async () => {
      const testUser = client.generateTestUser(`_${testUtils.randomString()}`);
      const inviteCode = getNextInviteCode();

      const challenge = await client.registerStart(testUser.username, {
        invite_code: inviteCode,
      });

      expect(challenge.publicKey.challenge).toBeDefined();
      expect(challenge.publicKey.rp).toBeDefined();
      expect(challenge.publicKey.rp.id).toBeTruthy();
      expect(challenge.publicKey.user.name).toBe(testUser.username);
      expect(challenge.publicKey.user.displayName).toBe(testUser.username);
    });

    it('should start registration with invite code', async () => {
      const testUser = client.generateTestUser(
        `_invited_${testUtils.randomString()}`
      );
      const inviteCode = getNextInviteCode();

      const challenge = await client.registerStart(testUser.username, {
        invite_code: inviteCode,
      });

      expect(challenge.publicKey.challenge).toBeDefined();
      expect(challenge.publicKey.user.name).toBe(testUser.username);
    });

    it('should reject registration with invalid data', async () => {
      const error = await client.expectError(
        () => client.registerStart(''),
        405
      );

      expect(error.status).toBe(405);
    });

    it('should handle concurrent registration attempts', async () => {
      const promises = Array.from({ length: 5 }, (_, i) => {
        const testUser = client.generateTestUser(`_concurrent_${i}`);
        const inviteCode = getNextInviteCode();
        return client.registerStart(testUser.username, {
          invite_code: inviteCode,
        });
      });

      const results = await Promise.allSettled(promises);

      // All should succeed (or at least not fail due to concurrency issues)
      results.forEach((result, index) => {
        if (result.status === 'rejected') {
          console.error(`Concurrent request ${index} failed:`, result.reason);
        }
        expect(result.status).toBe('fulfilled');
      });
    });

    it('should reject registration finish with invalid credential', async () => {
      const mockCredential = testUtils.generateMockCredential();

      const error = await client.expectError(
        () => client.registerFinish(mockCredential),
        422
      );

      expect(error.status).toBe(422);
    });
  });

  describe('Login Flow', () => {
    it('should reject login for non-existent user', async () => {
      const error = await client.expectError(
        () => client.loginStart('nonexistent_user'),
        404
      );

      expect(error.status).toBe(404);
    });

    it('should reject empty username login', async () => {
      const error = await client.expectError(() => client.loginStart(''), 405);

      expect(error.status).toBe(405);
    });

    it('should reject login finish with invalid assertion', async () => {
      const mockAssertion = testUtils.generateMockAssertion();

      const error = await client.expectError(
        () => client.loginFinish(mockAssertion),
        422
      );

      expect(error.status).toBe(422);
    });
  });

  describe('Authentication Status', () => {
    it('should check authentication status', async () => {
      const status = await client.authStatus();

      expect(typeof status.authenticated).toBe('boolean');
      if (status.authenticated) {
        expect(typeof status.user_id).toBe('string');
      }
    });
  });

  describe('Logout', () => {
    it('should handle logout request', async () => {
      // Logout should work even if not logged in
      await client.logout();
      // If no error is thrown, logout succeeded
    });
  });

  describe('Error Handling', () => {
    it('should handle malformed JSON', async () => {
      // This test requires manual fetch to send invalid JSON to register_finish endpoint
      const response = await fetch(`${client['baseUrl']}/register_finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: 'invalid json',
      });

      expect(response.status).toBe(400);
    });

    it('should handle network timeouts gracefully', async () => {
      // Create a client with a very short timeout for testing
      const timeoutClient = new TestApiClient('http://localhost:9999'); // Non-existent server
      timeoutClient.setTimeout(100);

      await expect(timeoutClient.health()).rejects.toThrow();
    });

    it('should handle server errors gracefully', async () => {
      // Test with invalid characters that might cause server errors
      const invalidUsername = '\x00\x01\x02';

      try {
        await client.registerStart(invalidUsername);
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
      }
    });
  });

  describe('Performance Tests', () => {
    it('should handle burst requests within reasonable time', async () => {
      const startTime = Date.now();
      const promises = Array.from({ length: 10 }, () => client.health());

      await Promise.all(promises);

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });

    it('should handle mixed concurrent requests', async () => {
      const promises = [
        client.health(),
        client.authStatus(),
        client.registerStart(`perf_test_${testUtils.randomString()}`, {
          invite_code: getNextInviteCode(),
        }),
        client.expectError(() => client.loginStart('nonexistent'), 404),
      ];

      const results = await Promise.allSettled(promises);

      // All should complete without hanging
      expect(results).toHaveLength(4);
      results.forEach((result, index) => {
        if (result.status === 'rejected') {
          console.error(`Mixed request ${index} failed:`, result.reason);
        }
      });
    });
  });

  describe('Schema Validation', () => {
    it('should validate response schemas for registerStart', async () => {
      const testUser = client.generateTestUser(
        `_schema_${testUtils.randomString()}`
      );
      const inviteCode = getNextInviteCode();

      const response = await client.registerStart(testUser.username, {
        invite_code: inviteCode,
      });

      // Zod validation happens automatically in the client
      expect(response.publicKey).toBeDefined();
      expect(response.publicKey.challenge).toBeDefined();
      expect(response.publicKey.rp).toBeDefined();
      expect(response.publicKey.user).toBeDefined();
    });

    it('should validate response schemas for authStatus', async () => {
      const response = await client.authStatus();

      // Zod validation happens automatically in the client
      expect(typeof response.authenticated).toBe('boolean');
    });
  });
});
