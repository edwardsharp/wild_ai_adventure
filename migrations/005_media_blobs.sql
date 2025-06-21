-- Migration 005: Create media_blobs table for WebSocket file sharing
--
-- This table stores metadata and optionally data for media files
-- shared through the WebSocket system.

CREATE TABLE IF NOT EXISTS media_blobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data BYTEA,
    sha256 TEXT NOT NULL,
    size BIGINT,
    mime TEXT,
    source_client_id TEXT,
    local_path TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_media_blobs_sha256 ON media_blobs (sha256);
CREATE INDEX IF NOT EXISTS idx_media_blobs_client_id ON media_blobs (source_client_id);
CREATE INDEX IF NOT EXISTS idx_media_blobs_created_at ON media_blobs (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_media_blobs_local_path ON media_blobs (local_path);
CREATE INDEX IF NOT EXISTS idx_media_blobs_mime ON media_blobs (mime);

-- Optional: Add a trigger to automatically update updated_at
CREATE OR REPLACE FUNCTION update_media_blobs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER trigger_media_blobs_updated_at
    BEFORE UPDATE ON media_blobs
    FOR EACH ROW
    EXECUTE FUNCTION update_media_blobs_updated_at();

-- Comments for documentation
COMMENT ON TABLE media_blobs IS 'Stores media file metadata and optionally binary data for WebSocket file sharing';
COMMENT ON COLUMN media_blobs.data IS 'Optional binary data - may be stored externally and referenced by local_path';
COMMENT ON COLUMN media_blobs.sha256 IS 'SHA256 hash for deduplication and integrity verification';
COMMENT ON COLUMN media_blobs.source_client_id IS 'Identifier of the client that uploaded this blob';
COMMENT ON COLUMN media_blobs.local_path IS 'Local filesystem path if data is stored externally';
COMMENT ON COLUMN media_blobs.metadata IS 'Additional metadata as JSON (dimensions, duration, etc.)';
