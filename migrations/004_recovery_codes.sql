-- Add recovery code support to existing invite codes table
-- This allows invite codes to be used for account recovery (linking new credentials to existing users)

-- Add recovery fields to existing invite_codes table
DO $$
BEGIN
    -- Add recovery_for_user_id column if it doesn't exist
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'recovery_for_user_id'
    ) THEN
        ALTER TABLE invite_codes ADD COLUMN recovery_for_user_id UUID;
        ALTER TABLE invite_codes ADD CONSTRAINT fk_invite_codes_recovery_user
            FOREIGN KEY (recovery_for_user_id) REFERENCES users(id) ON DELETE CASCADE;
    END IF;

    -- Add recovery_expires_at column for shorter expiry on recovery codes
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'recovery_expires_at'
    ) THEN
        ALTER TABLE invite_codes ADD COLUMN recovery_expires_at TIMESTAMPTZ;
    END IF;

    -- Add code_type to distinguish between regular invites and recovery codes
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'code_type'
    ) THEN
        ALTER TABLE invite_codes ADD COLUMN code_type VARCHAR(20) NOT NULL DEFAULT 'invite';
        ALTER TABLE invite_codes ADD CONSTRAINT invite_codes_type_check
            CHECK (code_type IN ('invite', 'recovery'));
    END IF;
END $$;

-- Add indexes for recovery functionality
DO $$
BEGIN
    -- Index for recovery codes by user
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_recovery_user') THEN
        CREATE INDEX idx_invite_codes_recovery_user ON invite_codes(recovery_for_user_id)
        WHERE recovery_for_user_id IS NOT NULL;
    END IF;

    -- Index for active recovery codes
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_recovery_active') THEN
        CREATE INDEX idx_invite_codes_recovery_active ON invite_codes(code_type, is_active)
        WHERE code_type = 'recovery' AND is_active = true;
    END IF;

    -- Index for recovery expiration
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_recovery_expires') THEN
        CREATE INDEX idx_invite_codes_recovery_expires ON invite_codes(recovery_expires_at)
        WHERE recovery_expires_at IS NOT NULL;
    END IF;
END $$;

-- Add constraints to ensure recovery codes are properly configured
DO $$
BEGIN
    -- Recovery codes must have a target user
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.check_constraints
        WHERE constraint_name = 'recovery_code_has_user'
    ) THEN
        ALTER TABLE invite_codes ADD CONSTRAINT recovery_code_has_user
            CHECK (
                (code_type = 'invite' AND recovery_for_user_id IS NULL) OR
                (code_type = 'recovery' AND recovery_for_user_id IS NOT NULL)
            );
    END IF;

    -- Recovery codes should have expiration set
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.check_constraints
        WHERE constraint_name = 'recovery_code_has_expiry'
    ) THEN
        ALTER TABLE invite_codes ADD CONSTRAINT recovery_code_has_expiry
            CHECK (
                (code_type = 'invite') OR
                (code_type = 'recovery' AND recovery_expires_at IS NOT NULL)
            );
    END IF;
END $$;

-- Add comments for documentation
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'invite_codes'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'invite_codes'::regclass AND attname = 'recovery_for_user_id')
    ) THEN
        COMMENT ON COLUMN invite_codes.recovery_for_user_id IS 'User ID this recovery code is for (NULL for regular invite codes)';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'invite_codes'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'invite_codes'::regclass AND attname = 'recovery_expires_at')
    ) THEN
        COMMENT ON COLUMN invite_codes.recovery_expires_at IS 'Expiration time for recovery codes (shorter than regular invites)';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'invite_codes'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'invite_codes'::regclass AND attname = 'code_type')
    ) THEN
        COMMENT ON COLUMN invite_codes.code_type IS 'Type of code: invite (new users) or recovery (existing users)';
    END IF;
END $$;
