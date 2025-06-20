-- Initial database schema for WebAuthn with invite codes
-- This migration is idempotent and can be run multiple times safely

-- Invite codes table
CREATE TABLE IF NOT EXISTS invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(8) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used_at TIMESTAMPTZ,
    used_by_user_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Indexes for invite_codes (using IF NOT EXISTS)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_code') THEN
        CREATE INDEX idx_invite_codes_code ON invite_codes(code);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_active') THEN
        CREATE INDEX idx_invite_codes_active ON invite_codes(is_active) WHERE is_active = TRUE;
    END IF;
END $$;

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) NOT NULL UNIQUE,
    role VARCHAR(20) NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    invite_code_used VARCHAR(8),
    CONSTRAINT fk_users_invite_code FOREIGN KEY (invite_code_used) REFERENCES invite_codes(code),
    CONSTRAINT users_role_check CHECK (role IN ('admin', 'member'))
);

-- Indexes for users
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_users_username') THEN
        CREATE INDEX idx_users_username ON users(username);
    END IF;
END $$;

-- WebAuthn credentials table
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    credential_id BYTEA NOT NULL UNIQUE,
    credential_data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    CONSTRAINT fk_webauthn_credentials_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for webauthn_credentials
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_webauthn_credentials_user_id') THEN
        CREATE INDEX idx_webauthn_credentials_user_id ON webauthn_credentials(user_id);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_webauthn_credentials_credential_id') THEN
        CREATE INDEX idx_webauthn_credentials_credential_id ON webauthn_credentials(credential_id);
    END IF;
END $$;

-- Sessions table (for tower-sessions-sqlx-store)
CREATE TABLE IF NOT EXISTS tower_sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

-- Index for tower_sessions
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_tower_sessions_expiry') THEN
        CREATE INDEX idx_tower_sessions_expiry ON tower_sessions(expiry_date);
    END IF;
END $$;
