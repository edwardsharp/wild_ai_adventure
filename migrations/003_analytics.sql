-- Analytics table for request tracking and monitoring
-- This migration is idempotent and can be run multiple times safely

-- Analytics table for request tracking
CREATE TABLE IF NOT EXISTS request_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id VARCHAR(36) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id UUID,
    method VARCHAR(10) NOT NULL,
    path TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    duration_ms INTEGER,
    user_agent TEXT,
    ip_address TEXT,
    request_data JSONB,
    response_size BIGINT,
    error_message TEXT,
    trace_id VARCHAR(32),
    span_id VARCHAR(16)
);

-- Indexes for efficient querying (using idempotent approach)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_analytics_timestamp') THEN
        CREATE INDEX idx_analytics_timestamp ON request_analytics(timestamp DESC);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_analytics_user_id') THEN
        CREATE INDEX idx_analytics_user_id ON request_analytics(user_id) WHERE user_id IS NOT NULL;
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_analytics_path') THEN
        CREATE INDEX idx_analytics_path ON request_analytics(path);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_analytics_status') THEN
        CREATE INDEX idx_analytics_status ON request_analytics(status_code);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_analytics_trace_id') THEN
        CREATE INDEX idx_analytics_trace_id ON request_analytics(trace_id) WHERE trace_id IS NOT NULL;
    END IF;
END $$;

-- Comments for documentation (idempotent)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = 0
    ) THEN
        COMMENT ON TABLE request_analytics IS 'Analytics and monitoring data for HTTP requests';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'request_id')
    ) THEN
        COMMENT ON COLUMN request_analytics.request_id IS 'Unique identifier for the HTTP request';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'timestamp')
    ) THEN
        COMMENT ON COLUMN request_analytics.timestamp IS 'When the request was made';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'user_id')
    ) THEN
        COMMENT ON COLUMN request_analytics.user_id IS 'User ID if request was authenticated';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'method')
    ) THEN
        COMMENT ON COLUMN request_analytics.method IS 'HTTP method (GET, POST, etc.)';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'path')
    ) THEN
        COMMENT ON COLUMN request_analytics.path IS 'Request path/endpoint';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'status_code')
    ) THEN
        COMMENT ON COLUMN request_analytics.status_code IS 'HTTP response status code';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'duration_ms')
    ) THEN
        COMMENT ON COLUMN request_analytics.duration_ms IS 'Request processing time in milliseconds';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'user_agent')
    ) THEN
        COMMENT ON COLUMN request_analytics.user_agent IS 'Client user agent string';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'ip_address')
    ) THEN
        COMMENT ON COLUMN request_analytics.ip_address IS 'Client IP address';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'request_data')
    ) THEN
        COMMENT ON COLUMN request_analytics.request_data IS 'Additional request metadata as JSON';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'response_size')
    ) THEN
        COMMENT ON COLUMN request_analytics.response_size IS 'Response size in bytes';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'error_message')
    ) THEN
        COMMENT ON COLUMN request_analytics.error_message IS 'Error message if request failed';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'trace_id')
    ) THEN
        COMMENT ON COLUMN request_analytics.trace_id IS 'Distributed tracing trace ID';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_description
        WHERE objoid = 'request_analytics'::regclass
        AND objsubid = (SELECT attnum FROM pg_attribute WHERE attrelid = 'request_analytics'::regclass AND attname = 'span_id')
    ) THEN
        COMMENT ON COLUMN request_analytics.span_id IS 'Distributed tracing span ID';
    END IF;
END $$;
