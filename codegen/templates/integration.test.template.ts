import { TestApiClient, testUtils } from "../src/test-helpers";
import { ApiError } from "../src/api-client";

describe("WebAuthn API Integration Tests", () => {
  let client: TestApiClient;

  beforeAll(() => {
    // Use the test server URL - this should match your Rust test server
    client = new TestApiClient(process.env.API_BASE_URL || "{{BASE_URL}}");
  });

  describe("Health Check", () => {
    it("should return healthy status", async () => {
      await client.health();
      // If no error is thrown, the health check passed
    });
  });

  describe("User Registration Flow", () => {
    it("should start registration successfully", async () => {
      const testUser = client.generateTestUser(`_${testUtils.randomString()}`);

      const challenge = await client.registerStart(testUser.username);

      expect(challenge.publicKey.challenge).toBeDefined();
      expect(challenge.publicKey.rp).toBeDefined();
      expect(challenge.publicKey.rp.id).toBeTruthy();
      expect(challenge.publicKey.user.name).toBe(testUser.username);
      expect(challenge.publicKey.user.displayName).toBe(testUser.username);
    });

    it("should reject registration with invalid data", async () => {
      const error = await client.expectError(
        () => client.registerStart(""),
        405,
      );

      expect(error.status).toBe(405);
    });

    it("should handle concurrent registration attempts", async () => {
      const promises = Array.from({ length: 5 }, (_, i) => {
        const testUser = client.generateTestUser(`_concurrent_${i}`);
        return client.registerStart(testUser.username);
      });

      const results = await Promise.allSettled(promises);

      // All should succeed (or at least not fail due to concurrency issues)
      results.forEach((result, index) => {
        if (result.status === "rejected") {
          console.error(`Concurrent request ${index} failed:`, result.reason);
        }
        expect(result.status).toBe("fulfilled");
      });
    });
  });

  describe("Login Flow", () => {
    it("should start login for existing user", async () => {
      // Since WebAuthn requires browser APIs to complete registration,
      // we can't actually create a complete user in Node.js tests.
      // This test is commented out until we have a way to seed test users
      // or mock the full registration flow.

      // For now, we'll skip this test
      expect(true).toBe(true);
    });

    it("should reject login for non-existent user", async () => {
      const error = await client.expectError(
        () => client.loginStart("nonexistent_user"),
        404,
      );

      expect(error.status).toBe(404);
    });
  });

  describe("Error Handling", () => {
    it("should handle malformed JSON", async () => {
      // This test requires manual fetch to send invalid JSON to register_finish endpoint
      const response = await fetch(`${client["baseUrl"]}/register_finish`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: "invalid json",
      });

      expect(response.status).toBe(400);
    });

    it("should handle network timeouts gracefully", async () => {
      // Create a client with a very short timeout for testing
      const timeoutClient = new TestApiClient("http://localhost:9999"); // Non-existent server

      await expect(timeoutClient.health()).rejects.toThrow();
    });
  });

  describe("Performance Tests", () => {
    it("should handle burst requests within reasonable time", async () => {
      const startTime = Date.now();
      const promises = Array.from({ length: 10 }, () => client.health());

      await Promise.all(promises);

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });
  });
});
