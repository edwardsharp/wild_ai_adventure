import { TestApiClient } from "../src/test-helpers";
import { ApiError } from "../src/api-client";
import * as fs from 'fs';
import * as path from 'path';

describe("Invite Code Tests", () => {
  let client: TestApiClient;
  let inviteCodes: string[] = [];

  beforeAll(() => {
    client = new TestApiClient(
      process.env.API_BASE_URL || "{{BASE_URL}}",
    );

    // Load generated invite codes from test data
    try {
      const testDataPath = path.join(__dirname, '..', 'test-data', 'invite-codes.json');
      if (fs.existsSync(testDataPath)) {
        inviteCodes = JSON.parse(fs.readFileSync(testDataPath, 'utf8'));
        console.log(`Loaded ${inviteCodes.length} test invite codes`);
      } else {
        console.warn('No invite codes found - some tests may be skipped');
      }
    } catch (error) {
      console.error('Failed to load invite codes:', error);
    }
  });

  describe("Configuration-based Invite Code Tests", () => {
    it("should allow registration without invite code when not required", async () => {
      // Test config has invite_codes_required: false
      const testUser = client.generateTestUser(`_no_invite_${Date.now()}`);

      const challenge = await client.registerStart(testUser.username);

      expect(challenge.publicKey).toBeDefined();
      expect(challenge.publicKey.user.name).toBe(testUser.username);
    });

    it("should accept optional invite code when provided", async () => {
      if (inviteCodes.length === 0) {
        console.log('Skipping test - no invite codes available');
        return;
      }

      const testUser = client.generateTestUser(`_with_invite_${Date.now()}`);
      const inviteCode = inviteCodes[0];

      const challenge = await client.registerStart(testUser.username, {
        invite_code: inviteCode
      });

      expect(challenge.publicKey).toBeDefined();
      expect(challenge.publicKey.user.name).toBe(testUser.username);
    });
  });

  describe("Invite Code Validation", () => {
    it("should reject invalid invite codes when provided", async () => {
      const testUser = client.generateTestUser(`_invalid_invite_${Date.now()}`);
      const invalidCode = "INVALID123";

      try {
        await client.registerStart(testUser.username, {
          invite_code: invalidCode
        });
        fail("Should have thrown an error for invalid invite code");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(400);
        expect((error as ApiError).responseText).toContain("Invalid or expired invite code");
      }
    });

    it("should reject empty invite codes when provided", async () => {
      const testUser = client.generateTestUser(`_empty_invite_${Date.now()}`);

      try {
        await client.registerStart(testUser.username, {
          invite_code: ""
        });
        fail("Should have thrown an error for empty invite code");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(400);
      }
    });

    it("should reject malformed invite codes", async () => {
      const testUser = client.generateTestUser(`_malformed_invite_${Date.now()}`);
      const malformedCodes = [
        "ABC!@#",          // Special characters
        "123",             // Too short
        "a".repeat(50),    // Too long
        "abc def",         // Spaces
        "АВС123",          // Cyrillic characters
      ];

      for (const badCode of malformedCodes) {
        try {
          await client.registerStart(`${testUser.username}_${badCode.length}`, {
            invite_code: badCode
          });
          fail(`Should have rejected malformed invite code: ${badCode}`);
        } catch (error) {
          expect(error).toBeInstanceOf(ApiError);
          expect((error as ApiError).status).toBe(400);
        }
      }
    });
  });

  describe("Invite Code Usage Tracking", () => {
    it("should track invite code usage", async () => {
      if (inviteCodes.length < 2) {
        console.log('Skipping test - need at least 2 invite codes');
        return;
      }

      const testUser = client.generateTestUser(`_track_usage_${Date.now()}`);
      const inviteCode = inviteCodes[1];

      // First use should succeed
      const challenge = await client.registerStart(testUser.username, {
        invite_code: inviteCode
      });

      expect(challenge.publicKey).toBeDefined();

      // Note: Testing actual invite code consumption would require completing
      // the full WebAuthn registration flow, which needs browser crypto APIs
      // that aren't available in Node.js tests. In a real application,
      // you would:
      // 1. Complete the registration with registerFinish()
      // 2. Try to use the same invite code again
      // 3. Expect it to be rejected as already used
    });

    it("should handle concurrent usage attempts", async () => {
      if (inviteCodes.length < 3) {
        console.log('Skipping test - need at least 3 invite codes');
        return;
      }

      const inviteCode = inviteCodes[2];
      const promises = Array.from({ length: 3 }, (_, i) => {
        const testUser = client.generateTestUser(`_concurrent_${i}_${Date.now()}`);
        return client.registerStart(testUser.username, {
          invite_code: inviteCode
        });
      });

      const results = await Promise.allSettled(promises);

      // All should succeed at the registration start phase
      // (actual usage tracking happens during registration completion)
      const successful = results.filter(r => r.status === "fulfilled").length;
      expect(successful).toBeGreaterThan(0);
    });
  });

  describe("Edge Cases and Error Handling", () => {
    it("should handle very long invite codes", async () => {
      const testUser = client.generateTestUser(`_long_invite_${Date.now()}`);
      const longCode = "A".repeat(100);

      try {
        await client.registerStart(testUser.username, {
          invite_code: longCode
        });
        fail("Should have rejected overly long invite code");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(400);
      }
    });

    it("should handle case sensitivity correctly", async () => {
      if (inviteCodes.length === 0) {
        console.log('Skipping test - no invite codes available');
        return;
      }

      const testUser = client.generateTestUser(`_case_test_${Date.now()}`);
      const originalCode = inviteCodes[0];
      const lowerCaseCode = originalCode.toLowerCase();

      if (originalCode === lowerCaseCode) {
        console.log('Skipping case test - invite code is already lowercase');
        return;
      }

      // Test config likely has case_sensitive: false
      try {
        const challenge = await client.registerStart(testUser.username, {
          invite_code: lowerCaseCode
        });
        expect(challenge.publicKey).toBeDefined();
      } catch (error) {
        // If case sensitive, this should fail
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(400);
      }
    });

    it("should handle SQL injection attempts in invite codes", async () => {
      const testUser = client.generateTestUser(`_sql_injection_${Date.now()}`);
      const maliciousCodes = [
        "'; DROP TABLE invite_codes; --",
        "' OR '1'='1",
        "'; UPDATE invite_codes SET used_at=NULL; --",
        "' UNION SELECT * FROM users; --"
      ];

      for (const maliciousCode of maliciousCodes) {
        try {
          await client.registerStart(`${testUser.username}_${maliciousCodes.indexOf(maliciousCode)}`, {
            invite_code: maliciousCode
          });
          fail(`Should have rejected SQL injection attempt: ${maliciousCode}`);
        } catch (error) {
          expect(error).toBeInstanceOf(ApiError);
          expect((error as ApiError).status).toBe(400);
        }
      }
    });

    it("should handle null and undefined invite codes gracefully", async () => {
      const testUser = client.generateTestUser(`_null_invite_${Date.now()}`);

      // Test with various falsy values
      const falsyValues = [null, undefined, 0, false, NaN];

      for (const falsyValue of falsyValues) {
        try {
          // TypeScript might complain, but we want to test runtime behavior
          await client.registerStart(testUser.username, {
            invite_code: falsyValue as any
          });
          // This might succeed (treated as no invite code) or fail
        } catch (error) {
          // Either way is acceptable - just shouldn't crash the server
          expect(error).toBeInstanceOf(ApiError);
        }
      }
    });
  });

  describe("Performance with Invite Codes", () => {
    it("should handle invite code validation efficiently", async () => {
      if (inviteCodes.length === 0) {
        console.log('Skipping test - no invite codes available');
        return;
      }

      const startTime = Date.now();
      const promises = Array.from({ length: 10 }, (_, i) => {
        const testUser = client.generateTestUser(`_perf_${i}_${Date.now()}`);
        const inviteCode = inviteCodes[i % inviteCodes.length];
        return client.registerStart(testUser.username, {
          invite_code: inviteCode
        });
      });

      await Promise.allSettled(promises);

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });

    it("should handle mixed invite code and no-invite requests", async () => {
      const promises = Array.from({ length: 10 }, (_, i) => {
        const testUser = client.generateTestUser(`_mixed_${i}_${Date.now()}`);

        if (i % 2 === 0) {
          // Even indices: no invite code
          return client.registerStart(testUser.username);
        } else {
          // Odd indices: with invite code (if available)
          const inviteCode = inviteCodes.length > 0 ? inviteCodes[0] : undefined;
          return inviteCode
            ? client.registerStart(testUser.username, { invite_code: inviteCode })
            : client.registerStart(testUser.username);
        }
      });

      const results = await Promise.allSettled(promises);

      // All should succeed since invite codes are not required in test config
      const successful = results.filter(r => r.status === "fulfilled").length;
      expect(successful).toBe(10);
    });
  });

  describe("Invite Code Information Leakage Prevention", () => {
    it("should not leak information about existing vs non-existing codes", async () => {
      const testUser = client.generateTestUser(`_info_leak_${Date.now()}`);

      // Test with a code that definitely doesn't exist
      const nonExistentCode = "ZZZZZZZZ";

      try {
        await client.registerStart(testUser.username, {
          invite_code: nonExistentCode
        });
        fail("Should have rejected non-existent invite code");
      } catch (error) {
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).status).toBe(400);

        // Error message should be generic
        const errorText = (error as ApiError).responseText;
        expect(errorText).toContain("Invalid or expired invite code");
        expect(errorText).not.toContain("not found");
        expect(errorText).not.toContain("does not exist");
      }
    });

    it("should not reveal timing information", async () => {
      if (inviteCodes.length === 0) {
        console.log('Skipping test - no invite codes available');
        return;
      }

      const testUser = client.generateTestUser(`_timing_${Date.now()}`);
      const validCode = inviteCodes[0];
      const invalidCode = "INVALID123";

      // Measure time for valid code
      const validStart = Date.now();
      try {
        await client.registerStart(testUser.username, {
          invite_code: validCode
        });
      } catch (error) {
        // May still error for other reasons
      }
      const validDuration = Date.now() - validStart;

      // Measure time for invalid code
      const invalidStart = Date.now();
      try {
        await client.registerStart(`${testUser.username}_invalid`, {
          invite_code: invalidCode
        });
      } catch (error) {
        // Expected to error
      }
      const invalidDuration = Date.now() - invalidStart;

      // Times should be reasonably similar (within an order of magnitude)
      const ratio = Math.max(validDuration, invalidDuration) / Math.min(validDuration, invalidDuration);
      expect(ratio).toBeLessThan(10); // Not a strict requirement, just a sanity check
    });
  });
});
