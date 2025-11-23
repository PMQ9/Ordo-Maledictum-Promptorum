-- Initial schema for intent ledger
-- This migration creates the append-only ledger_entries table with proper constraints

-- Create the ledger_entries table
CREATE TABLE IF NOT EXISTS ledger_entries (
    -- Primary identifiers
    id UUID PRIMARY KEY,
    session_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- User input
    user_input TEXT NOT NULL,
    user_input_hash VARCHAR(64) NOT NULL,

    -- Malicious detection
    malicious_score DOUBLE PRECISION,
    malicious_blocked BOOLEAN NOT NULL DEFAULT FALSE,

    -- Parsing phase (stored as JSONB for flexibility and queryability)
    voting_result JSONB NOT NULL,

    -- Comparison phase
    comparison_result JSONB NOT NULL,

    -- Elevation (optional)
    elevation_event JSONB,

    -- Trusted intent (optional)
    trusted_intent JSONB,

    -- Processing phase (optional)
    processing_output JSONB,

    -- Metadata
    ip_address VARCHAR(45),  -- Supports IPv6
    user_agent TEXT
);

-- Create indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_ledger_user_id ON ledger_entries(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ledger_session_id ON ledger_entries(session_id, timestamp ASC);
CREATE INDEX IF NOT EXISTS idx_ledger_timestamp ON ledger_entries(timestamp);
CREATE INDEX IF NOT EXISTS idx_ledger_malicious ON ledger_entries(malicious_blocked) WHERE malicious_blocked = TRUE;

-- Create a GIN index on JSON fields for efficient querying
CREATE INDEX IF NOT EXISTS idx_ledger_voting_result ON ledger_entries USING GIN(voting_result);
CREATE INDEX IF NOT EXISTS idx_ledger_comparison_result ON ledger_entries USING GIN(comparison_result);

-- Create an index for elevation events
CREATE INDEX IF NOT EXISTS idx_ledger_elevation ON ledger_entries(timestamp DESC) WHERE elevation_event IS NOT NULL;

-- Create an index on user_input_hash for duplicate detection
CREATE INDEX IF NOT EXISTS idx_ledger_input_hash ON ledger_entries(user_input_hash);

-- Add a partial index for recent entries (last 30 days) for faster queries
CREATE INDEX IF NOT EXISTS idx_ledger_recent ON ledger_entries(timestamp DESC)
    WHERE timestamp > NOW() - INTERVAL '30 days';

-- ============================================================================
-- APPEND-ONLY ENFORCEMENT
-- ============================================================================
-- Create a trigger function that prevents UPDATE and DELETE operations
-- This ensures the ledger remains immutable and append-only

CREATE OR REPLACE FUNCTION prevent_ledger_modifications()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'ledger_entries is append-only: % operations are not allowed', TG_OP
        USING HINT = 'The ledger is immutable. Only INSERT operations are permitted.';
END;
$$ LANGUAGE plpgsql;

-- Create triggers to enforce append-only behavior
CREATE TRIGGER prevent_ledger_update
    BEFORE UPDATE ON ledger_entries
    FOR EACH ROW
    EXECUTE FUNCTION prevent_ledger_modifications();

CREATE TRIGGER prevent_ledger_delete
    BEFORE DELETE ON ledger_entries
    FOR EACH ROW
    EXECUTE FUNCTION prevent_ledger_modifications();

-- ============================================================================
-- AUDIT VIEWS
-- ============================================================================
-- Create helpful views for common audit queries

-- View for blocked malicious entries
CREATE OR REPLACE VIEW v_blocked_entries AS
SELECT
    id,
    user_id,
    session_id,
    timestamp,
    malicious_score,
    user_input,
    comparison_result->>'decision' as comparator_decision
FROM ledger_entries
WHERE malicious_blocked = TRUE
ORDER BY timestamp DESC;

-- View for elevation events
CREATE OR REPLACE VIEW v_elevation_events AS
SELECT
    id,
    user_id,
    session_id,
    timestamp,
    elevation_event->>'status' as elevation_status,
    elevation_event->>'reason' as elevation_reason,
    elevation_event->>'approved_by' as approved_by,
    (elevation_event->>'approved_at')::TIMESTAMPTZ as approved_at,
    user_input
FROM ledger_entries
WHERE elevation_event IS NOT NULL
ORDER BY timestamp DESC;

-- View for successful vs failed processing
CREATE OR REPLACE VIEW v_processing_stats AS
SELECT
    DATE_TRUNC('hour', timestamp) as time_bucket,
    COUNT(*) as total_requests,
    SUM(CASE WHEN processing_output->>'success' = 'true' THEN 1 ELSE 0 END) as successful,
    SUM(CASE WHEN processing_output->>'success' = 'false' THEN 1 ELSE 0 END) as failed,
    AVG((processing_output->>'execution_time_ms')::NUMERIC) as avg_execution_ms
FROM ledger_entries
WHERE processing_output IS NOT NULL
GROUP BY time_bucket
ORDER BY time_bucket DESC;

-- View for parser agreement analysis
CREATE OR REPLACE VIEW v_parser_agreement AS
SELECT
    DATE_TRUNC('hour', timestamp) as time_bucket,
    voting_result->>'agreement_level' as agreement_level,
    COUNT(*) as count,
    AVG((voting_result->>'confidence')::NUMERIC) as avg_confidence
FROM ledger_entries
GROUP BY time_bucket, agreement_level
ORDER BY time_bucket DESC, agreement_level;

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to get ledger statistics
CREATE OR REPLACE FUNCTION get_ledger_stats()
RETURNS TABLE (
    total_entries BIGINT,
    total_users BIGINT,
    total_sessions BIGINT,
    blocked_entries BIGINT,
    elevation_events BIGINT,
    oldest_entry TIMESTAMPTZ,
    newest_entry TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(*) as total_entries,
        COUNT(DISTINCT user_id) as total_users,
        COUNT(DISTINCT session_id) as total_sessions,
        SUM(CASE WHEN malicious_blocked THEN 1 ELSE 0 END) as blocked_entries,
        SUM(CASE WHEN elevation_event IS NOT NULL THEN 1 ELSE 0 END) as elevation_events,
        MIN(timestamp) as oldest_entry,
        MAX(timestamp) as newest_entry
    FROM ledger_entries;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE ledger_entries IS 'Append-only ledger storing all intent processing flows. Immutable once written.';
COMMENT ON COLUMN ledger_entries.id IS 'Unique identifier for each ledger entry';
COMMENT ON COLUMN ledger_entries.session_id IS 'User session identifier for grouping related requests';
COMMENT ON COLUMN ledger_entries.user_id IS 'User identifier';
COMMENT ON COLUMN ledger_entries.user_input_hash IS 'SHA-256 hash of user input for duplicate detection';
COMMENT ON COLUMN ledger_entries.voting_result IS 'JSON containing parser voting results and agreement level';
COMMENT ON COLUMN ledger_entries.comparison_result IS 'JSON containing comparator decision and mismatches';
COMMENT ON COLUMN ledger_entries.elevation_event IS 'JSON containing privilege elevation request details (null if not needed)';
COMMENT ON COLUMN ledger_entries.trusted_intent IS 'JSON containing the validated and sanitized intent';
COMMENT ON COLUMN ledger_entries.processing_output IS 'JSON containing execution results and errors';
