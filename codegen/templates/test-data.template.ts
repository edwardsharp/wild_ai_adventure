import * as fs from "fs";
import * as path from "path";

export interface TestInviteCode {
  code: string;
  used: boolean;
  usedBy?: string;
}

export class TestDataManager {
  private static instance: TestDataManager;
  private inviteCodes: TestInviteCode[] = [];
  private testDataPath: string;

  private constructor() {
    this.testDataPath = path.join(__dirname, "..", "test-data");
  }

  public static getInstance(): TestDataManager {
    if (!TestDataManager.instance) {
      TestDataManager.instance = new TestDataManager();
    }
    return TestDataManager.instance;
  }

  /**
   * Load invite codes from the generated test data files
   */
  public loadInviteCodes(): void {
    try {
      const codesFilePath = path.join(this.testDataPath, "invite-codes.json");

      if (fs.existsSync(codesFilePath)) {
        const rawCodes = JSON.parse(
          fs.readFileSync(codesFilePath, "utf8"),
        ) as string[];
        this.inviteCodes = rawCodes.map((code) => ({
          code,
          used: false,
        }));
      } else {
        console.warn(
          "No invite codes file found, tests may fail if invite codes are required",
        );
        this.inviteCodes = [];
      }
    } catch (error) {
      console.error("Failed to load invite codes:", error);
      this.inviteCodes = [];
    }
  }

  /**
   * Get an unused invite code for testing
   */
  public getUnusedInviteCode(): string | null {
    const unused = this.inviteCodes.find((ic) => !ic.used);
    if (unused) {
      unused.used = true;
      return unused.code;
    }
    return null;
  }

  /**
   * Get a specific invite code by index
   */
  public getInviteCodeByIndex(index: number): string | null {
    if (index >= 0 && index < this.inviteCodes.length) {
      return this.inviteCodes[index].code;
    }
    return null;
  }

  /**
   * Mark an invite code as used by a specific user
   */
  public markInviteCodeUsed(code: string, username: string): void {
    const inviteCode = this.inviteCodes.find((ic) => ic.code === code);
    if (inviteCode) {
      inviteCode.used = true;
      inviteCode.usedBy = username;
    }
  }

  /**
   * Get all invite codes (for testing purposes)
   */
  public getAllInviteCodes(): TestInviteCode[] {
    return [...this.inviteCodes];
  }

  /**
   * Reset all invite codes to unused state (for test cleanup)
   */
  public resetInviteCodes(): void {
    this.inviteCodes.forEach((ic) => {
      ic.used = false;
      delete ic.usedBy;
    });
  }

  /**
   * Check if invite codes are available
   */
  public hasInviteCodes(): boolean {
    return this.inviteCodes.length > 0;
  }

  /**
   * Get count of available invite codes
   */
  public getAvailableInviteCount(): number {
    return this.inviteCodes.filter((ic) => !ic.used).length;
  }

  /**
   * Generate test user data
   */
  public generateTestUser(suffix: string = ""): {
    username: string;
    display_name: string;
  } {
    return {
      username: `testuser${suffix}`,
      display_name: `Test User${suffix}`,
    };
  }

  /**
   * Generate random string for test data
   */
  public randomString(length: number = 8): string {
    return Math.random()
      .toString(36)
      .substring(2, length + 2);
  }

  /**
   * Create unique test user with random suffix
   */
  public createUniqueTestUser(): {
    username: string;
    display_name: string;
    suffix: string;
  } {
    const suffix = `_${this.randomString()}`;
    return {
      ...this.generateTestUser(suffix),
      suffix,
    };
  }
}

// Export singleton instance for easy use
export const testData = TestDataManager.getInstance();

// Helper functions for common test scenarios
export const testUtils = {
  delay: (ms: number) => new Promise((resolve) => setTimeout(resolve, ms)),

  retry: async <T>(
    operation: () => Promise<T>,
    maxAttempts: number = 3,
    delayMs: number = 1000,
  ): Promise<Error> => {
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        return await operation();
      } catch (error) {
        if (attempt === maxAttempts) throw error;
        await testUtils.delay(delayMs);
      }
    }
    throw new Error("Should not reach here");
  },

  expectError: async <T>(
    operation: () => Promise<T>,
    expectedStatus?: number,
    expectedMessage?: string,
  ): Promise<Error> => {
    try {
      await operation();
      throw new Error("Expected operation to throw an error");
    } catch (error) {
      if (
        expectedStatus &&
        "status" in error &&
        (error as any).status !== expectedStatus
      ) {
        throw new Error(
          `Expected status ${expectedStatus}, got ${(error as any).status}`,
        );
      }
      if (expectedMessage && !error.message.includes(expectedMessage)) {
        throw new Error(
          `Expected error message to contain "${expectedMessage}", got "${error.message}"`,
        );
      }
      return error as Error;
    }
  },
};
