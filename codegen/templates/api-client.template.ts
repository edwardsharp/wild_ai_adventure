import { z } from "zod";

// Error handling
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public responseText: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

// Zod schemas
{{SCHEMAS}}

// TypeScript types
{{TYPES}}

// API Client
export class ApiClient {
  constructor(
    private baseUrl: string = "{{BASE_URL}}",
    private defaultHeaders: Record<string, string> = {},
  ) {}

  setHeader(key: string, value: string) {
    this.defaultHeaders[key] = value;
  }

  removeHeader(key: string) {
    delete this.defaultHeaders[key];
  }

{{METHODS}}
}

// Default client instance
export const apiClient = new ApiClient();

// Convenience exports for re-use
export { ApiClient as default };
