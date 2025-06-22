-- Expand invite code length constraints
-- Remove the restrictive VARCHAR(16) limit and allow up to 128 characters
-- This enables more flexible invite code generation with longer, more secure codes

-- Expand the code column in invite_codes table
DO $$
BEGIN
    -- Check if we need to expand the code column
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'invite_codes'
        AND column_name = 'code'
        AND character_maximum_length < 128
    ) THEN
        ALTER TABLE invite_codes ALTER COLUMN code TYPE VARCHAR(128);
        RAISE NOTICE 'Expanded invite_codes.code column to VARCHAR(128)';
    ELSE
        RAISE NOTICE 'invite_codes.code column already has sufficient length';
    END IF;
END $$;

-- Expand the invite_code_used column in users table to match
DO $$
BEGIN
    -- Check if we need to expand the invite_code_used column
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users'
        AND column_name = 'invite_code_used'
        AND character_maximum_length < 128
    ) THEN
        ALTER TABLE users ALTER COLUMN invite_code_used TYPE VARCHAR(128);
        RAISE NOTICE 'Expanded users.invite_code_used column to VARCHAR(128)';
    ELSE
        RAISE NOTICE 'users.invite_code_used column already has sufficient length';
    END IF;
END $$;

-- Update any existing short codes to meet the new minimum requirement first
-- This is a safety measure in case there are existing codes shorter than 8 characters
DO $$
DECLARE
    short_code_count INTEGER;
BEGIN
    -- Count codes that are too short
    SELECT COUNT(*) INTO short_code_count
    FROM invite_codes
    WHERE char_length(code) < 8;

    IF short_code_count > 0 THEN
        RAISE NOTICE 'Found % invite codes shorter than 8 characters', short_code_count;

        -- For existing short codes, pad them with random alphanumeric characters
        -- to meet the minimum length requirement
        UPDATE invite_codes
        SET code = code || upper(substring(encode(gen_random_bytes(4), 'hex') from 1 for (8 - char_length(code))))
        WHERE char_length(code) < 8;

        RAISE NOTICE 'Updated % short invite codes to meet minimum length requirement', short_code_count;
    ELSE
        RAISE NOTICE 'All existing invite codes already meet minimum length requirement';
    END IF;
END $$;

-- Add a check constraint to ensure minimum code length (10 characters)
-- This replaces the database-level maximum constraint with an application-level minimum
DO $$
BEGIN
    -- Remove any existing length constraints that might conflict
    IF EXISTS (
        SELECT 1 FROM information_schema.check_constraints
        WHERE constraint_name = 'invite_codes_code_length_check'
    ) THEN
        ALTER TABLE invite_codes DROP CONSTRAINT invite_codes_code_length_check;
        RAISE NOTICE 'Removed existing invite_codes_code_length_check constraint';
    END IF;

    -- Add minimum length constraint
    ALTER TABLE invite_codes ADD CONSTRAINT invite_codes_code_min_length_check
        CHECK (char_length(code) >= 8);
    RAISE NOTICE 'Added minimum length check: invite codes must be at least 8 characters';
END $$;

-- Add helpful comments for future reference
COMMENT ON COLUMN invite_codes.code IS 'Invite or account link code (8-128 characters, alphanumeric, hyphens, underscores)';
COMMENT ON CONSTRAINT invite_codes_code_min_length_check ON invite_codes IS 'Ensures invite codes are at least 8 characters long for security';

-- Add an index for better performance on longer codes if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_invite_codes_code_hash') THEN
        CREATE INDEX idx_invite_codes_code_hash ON invite_codes USING hash(code);
        RAISE NOTICE 'Added hash index on invite_codes.code for improved lookup performance';
    END IF;
END $$;
