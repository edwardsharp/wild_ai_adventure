-- Add account link code support to existing invite codes table
-- This allows invite codes to be used for account linking (linking new credentials to existing users)

-- First, expand the code column to support longer account link codes
DO $$
BEGIN
    -- Check current length and expand if needed
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'code'
        AND character_maximum_length < 16
    ) THEN
        ALTER TABLE invite_codes ALTER COLUMN code TYPE VARCHAR(16);
        -- Also update the foreign key reference in users table
        ALTER TABLE users ALTER COLUMN invite_code_used TYPE VARCHAR(16);
    END IF;
END $$;

-- Add account link fields to existing invite_codes table
DO $$
BEGIN
    -- Add link_for_user_id column if it doesn't exist
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'link_for_user_id'
    ) THEN
        ALTER TABLE invite_codes ADD COLUMN link_for_user_id UUID;
        ALTER TABLE invite_codes ADD CONSTRAINT fk_invite_codes_link_user
            FOREIGN KEY (link_for_user_id) REFERENCES users(id) ON DELETE CASCADE;
    END IF;

    -- Add link_expires_at column for shorter expiry on account link codes
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'link_expires_at'
    ) THEN
        ALTER TABLE invite_codes ADD COLUMN link_expires_at TIMESTAMPTZ;
    END IF;

    -- Add code_type to distinguish between regular invites and account link codes
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'code_type'
    ) THEN
        ALTER TABLE invite_codes ADD COLUMN code_type VARCHAR(20) NOT NULL DEFAULT 'invite';
        ALTER TABLE invite_codes ADD CONSTRAINT invite_codes_type_check
            CHECK (code_type IN ('invite', 'account-link'));
    END IF;
END $$;

-- Add indexes for account link functionality
DO $$
BEGIN
    -- Index for account link codes by user
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_link_user') THEN
        CREATE INDEX idx_invite_codes_link_user ON invite_codes(link_for_user_id)
        WHERE link_for_user_id IS NOT NULL;
    END IF;

    -- Index for active account link codes
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_link_active') THEN
        CREATE INDEX idx_invite_codes_link_active ON invite_codes(code_type, is_active)
        WHERE code_type = 'account-link' AND is_active = true;
    END IF;

    -- Index for account link expiration
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_link_expires') THEN
        CREATE INDEX idx_invite_codes_link_expires ON invite_codes(link_expires_at)
        WHERE link_expires_at IS NOT NULL;
    END IF;
END $$;

-- Add constraints to ensure account link codes are properly configured
DO $$
BEGIN
    -- Account link codes must have a target user
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.check_constraints
        WHERE constraint_name = 'link_code_has_user'
    ) THEN
        ALTER TABLE invite_codes ADD CONSTRAINT link_code_has_user
            CHECK (
                (code_type = 'invite' AND link_for_user_id IS NULL) OR
                (code_type = 'account-link' AND link_for_user_id IS NOT NULL)
            );
    END IF;

    -- Account link codes should have expiration set
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.check_constraints
        WHERE constraint_name = 'link_code_has_expiry'
    ) THEN
        ALTER TABLE invite_codes ADD CONSTRAINT link_code_has_expiry
            CHECK (
                (code_type = 'invite') OR
                (code_type = 'account-link' AND link_expires_at IS NOT NULL)
            );
    END IF;
END $$;

-- Add comments for documentation
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'invite_codes'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'invite_codes'::regclass AND attname = 'link_for_user_id')
    ) THEN
        COMMENT ON COLUMN invite_codes.link_for_user_id IS 'User ID this account link code is for (NULL for regular invite codes)';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'invite_codes'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'invite_codes'::regclass AND attname = 'link_expires_at')
    ) THEN
        COMMENT ON COLUMN invite_codes.link_expires_at IS 'Expiration time for account link codes (shorter than regular invites)';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'invite_codes'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'invite_codes'::regclass AND attname = 'code_type')
    ) THEN
        COMMENT ON COLUMN invite_codes.code_type IS 'Type of code: invite (new users) or account-link (existing users)';
    END IF;
END $$;
