-- Session storage migration for tower-sessions PostgreSQL backend
-- This creates the table needed for tower-sessions-sqlx-store

-- Sessions table for tower-sessions-sqlx-store
CREATE TABLE IF NOT EXISTS tower_sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

-- Index for efficient cleanup of expired sessions
CREATE INDEX IF NOT EXISTS idx_tower_sessions_expiry ON tower_sessions(expiry_date);

-- Additional indexes for performance
CREATE INDEX IF NOT EXISTS idx_tower_sessions_id ON tower_sessions(id);

-- Add cleanup function for expired sessions (optional but recommended)
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM tower_sessions WHERE expiry_date < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$;

-- Comment on the table
COMMENT ON TABLE tower_sessions IS 'Session storage for tower-sessions with PostgreSQL backend';
COMMENT ON COLUMN tower_sessions.id IS 'Unique session identifier';
COMMENT ON COLUMN tower_sessions.data IS 'Serialized session data';
COMMENT ON COLUMN tower_sessions.expiry_date IS 'Session expiration timestamp';
COMMENT ON FUNCTION cleanup_expired_sessions() IS 'Utility function to clean up expired sessions';
