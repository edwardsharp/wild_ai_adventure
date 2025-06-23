/**
 * WebSocket message types and Zod schemas for type-safe communication
 *
 * These types mirror the Rust serde types on the server side to ensure
 * consistent message format between client and server.
 */

import { z } from 'zod';

// Base schemas
const UuidSchema = z.string().uuid();
const DateTimeSchema = z.string().datetime();

/**
 * Media blob data structure matching the server-side MediaBlob
 */
export const MediaBlobSchema = z.object({
  id: UuidSchema,
  data: z.array(z.number()).optional(), // Vec<u8> as number array, often omitted
  sha256: z.string(),
  size: z.number().int().optional(),
  mime: z.string().optional(),
  source_client_id: z.string().optional(),
  local_path: z.string().optional(),
  metadata: z.record(z.any()).default({}), // JSONB as Record<string, any>
  created_at: DateTimeSchema,
  updated_at: DateTimeSchema,
});

export type MediaBlob = z.infer<typeof MediaBlobSchema>;

/**
 * Messages sent from client to server
 */
export const WebSocketMessageSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('Ping'),
  }),
  z.object({
    type: z.literal('GetMediaBlobs'),
    data: z.object({
      limit: z.number().int().positive().optional(),
      offset: z.number().int().min(0).optional(),
    }),
  }),
  z.object({
    type: z.literal('UploadMediaBlob'),
    data: z.object({
      blob: MediaBlobSchema,
    }),
  }),
  z.object({
    type: z.literal('GetMediaBlob'),
    data: z.object({
      id: UuidSchema,
    }),
  }),
]);

export type WebSocketMessage = z.infer<typeof WebSocketMessageSchema>;

/**
 * Messages sent from server to client
 */
export const WebSocketResponseSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('Welcome'),
    data: z.object({
      message: z.string(),
      user_id: UuidSchema.optional(),
      connection_id: z.string(),
    }),
  }),
  z.object({
    type: z.literal('Pong'),
  }),
  z.object({
    type: z.literal('MediaBlobs'),
    data: z.object({
      blobs: z.array(MediaBlobSchema),
      total_count: z.number().int().min(0),
    }),
  }),
  z.object({
    type: z.literal('MediaBlob'),
    data: z.object({
      blob: MediaBlobSchema,
    }),
  }),
  z.object({
    type: z.literal('Error'),
    data: z.object({
      message: z.string(),
      code: z.string().optional(),
    }),
  }),
  z.object({
    type: z.literal('ConnectionStatus'),
    data: z.object({
      connected: z.boolean(),
      user_count: z.number().int().min(0),
    }),
  }),
]);

export type WebSocketResponse = z.infer<typeof WebSocketResponseSchema>;

/**
 * Connection status for presence indication
 */
export enum ConnectionStatus {
  Disconnected = 'disconnected', // Red light
  Connecting = 'connecting', // Yellow light
  Connected = 'connected', // Green light
  Error = 'error', // Red light with error
}

/**
 * Helper functions for message creation and validation
 */
export const createMessage = {
  ping: (): WebSocketMessage => ({ type: 'Ping' }),

  getMediaBlobs: (limit?: number, offset?: number): WebSocketMessage => ({
    type: 'GetMediaBlobs',
    data: { limit, offset },
  }),

  getMediaBlob: (id: string): WebSocketMessage => ({
    type: 'GetMediaBlob',
    data: { id },
  }),

  uploadMediaBlob: (blob: MediaBlob): WebSocketMessage => ({
    type: 'UploadMediaBlob',
    data: { blob },
  }),
};

/**
 * Utility functions for message parsing and validation
 */
export const parseWebSocketMessage = (data: unknown): WebSocketMessage => {
  return WebSocketMessageSchema.parse(data);
};

export const parseWebSocketResponse = (data: unknown): WebSocketResponse => {
  return WebSocketResponseSchema.parse(data);
};

/**
 * Safe message parsing that returns error instead of throwing
 */
export const safeParseWebSocketResponse = (
  data: unknown
):
  | { success: true; data: WebSocketResponse }
  | { success: false; error: z.ZodError } => {
  const result = WebSocketResponseSchema.safeParse(data);
  return result;
};

/**
 * Type guards for response types
 */
export const isWelcomeMessage = (
  response: WebSocketResponse
): response is Extract<WebSocketResponse, { type: 'Welcome' }> => {
  return response.type === 'Welcome';
};

export const isMediaBlobsMessage = (
  response: WebSocketResponse
): response is Extract<WebSocketResponse, { type: 'MediaBlobs' }> => {
  return response.type === 'MediaBlobs';
};

export const isErrorMessage = (
  response: WebSocketResponse
): response is Extract<WebSocketResponse, { type: 'Error' }> => {
  return response.type === 'Error';
};

export const isConnectionStatusMessage = (
  response: WebSocketResponse
): response is Extract<WebSocketResponse, { type: 'ConnectionStatus' }> => {
  return response.type === 'ConnectionStatus';
};
