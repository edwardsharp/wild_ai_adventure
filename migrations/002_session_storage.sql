-- Session storage migration for tower-sessions PostgreSQL backend
-- This migration is idempotent and can be run multiple times safely

-- Sessions table for tower-sessions-sqlx-store
CREATE TABLE IF NOT EXISTS tower_sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

-- Index for efficient cleanup of expired sessions
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_tower_sessions_expiry') THEN
        CREATE INDEX idx_tower_sessions_expiry ON tower_sessions(expiry_date);
    END IF;
END $$;

-- Additional indexes for performance
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_tower_sessions_id') THEN
        CREATE INDEX idx_tower_sessions_id ON tower_sessions(id);
    END IF;
END $$;

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

-- Comments on the table (safe to run multiple times)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'tower_sessions'::regclass
        AND objsubid = 0
    ) THEN
        COMMENT ON TABLE tower_sessions IS 'Session storage for tower-sessions with PostgreSQL backend';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'tower_sessions'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'tower_sessions'::regclass AND attname = 'id')
    ) THEN
        COMMENT ON COLUMN tower_sessions.id IS 'Unique session identifier';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'tower_sessions'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'tower_sessions'::regclass AND attname = 'data')
    ) THEN
        COMMENT ON COLUMN tower_sessions.data IS 'Serialized session data';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'tower_sessions'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'tower_sessions'::regclass AND attname = 'expiry_date')
    ) THEN
        COMMENT ON COLUMN tower_sessions.expiry_date IS 'Session expiration timestamp';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'cleanup_expired_sessions'::regproc
    ) THEN
        COMMENT ON FUNCTION cleanup_expired_sessions() IS 'Utility function to clean up expired sessions';
    END IF;
END $$;
