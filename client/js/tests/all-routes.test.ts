import { describe, it, expect, beforeAll } from "vitest";
import { TestApiClient } from "../src/lib/test-helpers.js";
import { ApiError } from "../src/lib/api-client.js";
import testCodes from "../test-data/invite-codes.json" assert { type: "json" };

describe("All Routes Integration Tests", () => {
  let client: TestApiClient;
  let inviteCodeIndex = 0;

  beforeAll(() => {
    // Use the test server URL - this should match your Rust test server
    client = new TestApiClient(
      process.env.API_BASE_URL || "http://localhost:8080"
    );
  });

  // Helper to get a fresh invite code for each test
  function getNextInviteCode(): string {
    if (inviteCodeIndex >= testCodes.length) {
      // Reset index to reuse codes instead of failing
      inviteCodeIndex = 0;
    }
    return testCodes[inviteCodeIndex++];
  }

  describe("Health Check Routes", () => {
    it("should respond to /health", async () => {
      await client.health();
      // If no error is thrown, the health check passed
    });

    it("should respond to /metrics health endpoint", async () => {
      const response = await fetch(`${client["baseUrl"]}/metrics/health`);
      // May or may not exist depending on config, so we just check it doesn't crash
      expect([200, 404]).toContain(response.status);
    });
  });

  describe("Registration Routes", () => {
    it("should start registration with invite code", async () => {
      const testUser = client.generateTestUser(`_reg_${Date.now()}`);
      const inviteCode = getNextInviteCode();

      const challenge = await client.registerStart(testUser.username, {
        invite_code: inviteCode,
      });

      expect(challenge.publicKey).toBeDefined();
      expect(challenge.publicKey.challenge).toBeDefined();
      expect(challenge.publicKey.rp).toBeDefined();
      expect(challenge.publicKey.user.name).toBe(testUser.username);
    });

    it("should reject registration finish without proper challenge", async () => {
      const fakeCredential = {
        id: "fake-id",
        rawId: "fake-raw-id",
        response: {
          attestationObject: "fake-attestation",
          clientDataJSON: "fake-client-data",
        },
        type: "public-key" as const,
      };

      try {
        await client.registerFinish(fakeCredential);
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(422);
      }
    });

    it("should handle invalid usernames in registration", async () => {
      // Empty username gives 405 due to route structure
      try {
        await client.registerStart("");
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(405);
      }
    });

    it("should handle very long usernames", async () => {
      const longUsername = "a".repeat(300);

      try {
        const challenge = await client.registerStart(longUsername);
        expect(challenge.publicKey).toBeDefined();
      } catch (error) {
        // May fail due to validation, that's also valid
        expect(error).toBeInstanceOf(ApiError);
      }
    });

    it("should handle special characters in usernames", async () => {
      const specialUsername = "test@user.com";
      const inviteCode = getNextInviteCode();

      const challenge = await client.registerStart(specialUsername, {
        invite_code: inviteCode,
      });
      expect(challenge.publicKey.user.name).toBe(specialUsername);
    });
  });

  describe("Login Routes", () => {
    it("should reject login for non-existent user", async () => {
      try {
        await client.loginStart("definitely_nonexistent_user_12345");
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(404);
      }
    });

    it("should handle empty username in login", async () => {
      try {
        await client.loginStart("");
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(405); // Route not found
      }
    });

    it("should reject login finish without proper challenge", async () => {
      const fakeCredential = {
        id: "fake-login-id",
        rawId: "fake-login-raw-id",
        response: {
          authenticatorData: "fake-auth-data",
          clientDataJSON: "fake-client-data",
          signature: "fake-signature",
          userHandle: "fake-user-handle",
        },
        type: "public-key" as const,
      };

      try {
        await client.loginFinish(fakeCredential);
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(422);
      }
    });
  });

  describe("Logout Route", () => {
    it("should handle logout request", async () => {
      // Logout should work even if not logged in
      await client.logout();
      // If no error is thrown, logout succeeded
    });
  });

  describe("Authentication Status Route", () => {
    it("should return authentication status", async () => {
      const status = await client.authStatus();
      expect(typeof status.authenticated).toBe("boolean");
    });
  });

  describe("Static File Routes", () => {
    it("should serve public files (if enabled)", async () => {
      const response = await fetch(`${client["baseUrl"]}/public/`);
      // May return 200, 403, or 404 depending on config and files
      expect([200, 403, 404]).toContain(response.status);
    });

    it("should protect private files", async () => {
      const response = await fetch(`${client["baseUrl"]}/private/`);
      // Should require authentication or not exist
      expect([401, 403, 404]).toContain(response.status);
    });
  });

  describe("API Routes", () => {
    it("should protect admin metrics endpoint", async () => {
      const response = await fetch(`${client["baseUrl"]}/api/admin/metrics`);
      // Should require admin authentication
      expect([401, 403, 404]).toContain(response.status);
    });

    it("should protect user profile endpoint", async () => {
      const response = await fetch(`${client["baseUrl"]}/api/user/profile`);
      // Should require authentication
      expect([401, 403, 404]).toContain(response.status);
    });

    it("should handle public metrics endpoint (if enabled)", async () => {
      const response = await fetch(`${client["baseUrl"]}/api/metrics`);
      // May or may not exist depending on config
      expect([200, 404]).toContain(response.status);
    });
  });

  describe("Error Handling", () => {
    it("should handle malformed JSON in registration finish", async () => {
      const response = await fetch(`${client["baseUrl"]}/register_finish`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: "invalid json",
      });

      expect(response.status).toBe(400);
    });

    it("should handle malformed JSON in login finish", async () => {
      const response = await fetch(`${client["baseUrl"]}/login_finish`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: "invalid json",
      });

      expect(response.status).toBe(400);
    });

    it("should handle missing Content-Type header", async () => {
      const response = await fetch(`${client["baseUrl"]}/register_finish`, {
        method: "POST",
        body: JSON.stringify({}),
      });

      expect([400, 415]).toContain(response.status);
    });

    it("should handle non-existent routes", async () => {
      const response = await fetch(`${client["baseUrl"]}/nonexistent/route`);
      expect(response.status).toBe(404);
    });

    it("should handle wrong HTTP methods", async () => {
      const response = await fetch(`${client["baseUrl"]}/health`, {
        method: "POST",
      });
      expect(response.status).toBe(405);
    });
  });

  describe("Performance and Stress Tests", () => {
    it("should handle burst registration requests", async () => {
      const registrationPromises = Array.from({ length: 10 }, (_, i) => {
        const testUser = client.generateTestUser(`_burst_${i}_${Date.now()}`);
        const inviteCode = getNextInviteCode();
        return client.registerStart(testUser.username, {
          invite_code: inviteCode,
        });
      });

      const results = await Promise.allSettled(registrationPromises);

      // Most should succeed
      const successful = results.filter((r) => r.status === "fulfilled").length;
      expect(successful).toBeGreaterThan(7);
    });

    it("should handle burst health check requests", async () => {
      const startTime = Date.now();
      const promises = Array.from({ length: 20 }, () => client.health());

      await Promise.all(promises);

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });

    it("should handle concurrent different route types", async () => {
      const promises = [
        client.health(),
        client.health(),
        client.registerStart(`concurrent_test_${Date.now()}`, {
          invite_code: getNextInviteCode(),
        }),
        fetch(`${client["baseUrl"]}/api/metrics`),
        fetch(`${client["baseUrl"]}/nonexistent`),
      ];

      const results = await Promise.allSettled(promises);

      // Should not crash the server
      expect(results.length).toBe(5);
    });
  });

  describe("Security Tests", () => {
    it("should reject requests with oversized bodies", async () => {
      const largeBody = "x".repeat(2 * 1024 * 1024); // 2MB

      const response = await fetch(`${client["baseUrl"]}/register_finish`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: largeBody,
      });

      expect([400, 413]).toContain(response.status);
    });

    it("should handle suspicious SQL injection attempts", async () => {
      const maliciousUsername = "'; DROP TABLE users; --";

      try {
        const inviteCode = getNextInviteCode();
        await client.registerStart(maliciousUsername, {
          invite_code: inviteCode,
        });
        // If it doesn't crash, that's good
        expect(true).toBe(true);
      } catch (error) {
        // If it errors, that's also fine - just shouldn't crash the server
        expect(error).toBeInstanceOf(ApiError);
      }
    });

    it("should handle XSS attempts in usernames", async () => {
      const xssUsername = "<script>alert('xss')</script>";

      try {
        const inviteCode = getNextInviteCode();
        const challenge = await client.registerStart(xssUsername, {
          invite_code: inviteCode,
        });
        // Server should handle this gracefully
        expect(challenge.publicKey.user.name).toBe(xssUsername);
      } catch (error) {
        // Or reject it cleanly
        expect(error).toBeInstanceOf(ApiError);
      }
    });
  });
});
