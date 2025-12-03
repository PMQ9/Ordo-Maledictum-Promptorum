# E2E Test Infrastructure Issues - PARTIALLY RESOLVED

**Date**: December 2, 2025
**Status**: PARTIALLY RESOLVED - Scenario 1 succeeds, Scenarios 2 & 3 have Claude parser issues

## Summary

Created comprehensive end-to-end test infrastructure for testing the full intent segregation pipeline with real LLM API calls. Tests cover three critical scenarios: valid math questions, injection attacks, and policy violations. Execution blocked by database authentication and workspace configuration issues.

## Issues Identified

### 1. Database Password Mismatch in .env - FIXED
**File**: `.env` (lines 29-31, 181)
**Problem**: Placeholder password didn't match docker-compose setup, TEST_DATABASE_URL also had wrong password
**Fix Applied**:
- Changed `DATABASE_PASSWORD` from placeholder to `intent_pass`
- Updated `TEST_DATABASE_URL` password from `password` to `intent_pass`
- Recreated PostgreSQL container with fresh volumes to clear old authentication
**Impact**: API server can now successfully connect to PostgreSQL database

### 2. Integration Tests Not Configured in Workspace
**File**: `Cargo.toml` (root)
**Problem**: Virtual workspace without root package - `tests/` directory not recognized
**Workaround**: Created Python-based E2E test runner (`run_e2e_test.py`)
**Impact**: Rust integration tests in `tests/integration/e2e_metrics_test.rs` cannot be executed via `cargo test`

### 3. API Server Port Configuration - FIXED
**Files**: `config/default.toml` (line 13), `.env` (line 24)
**Problem**: Server ran on port 3000, but .env and Python test script expected port 8080
**Fix Applied**:
- Changed `config/default.toml` port from 3000 to 8080
- Added `APP__SERVER__PORT=8080` to `.env` for explicit override support
- Python test script now successfully connects on port 8080
**Impact**: API server and E2E tests now use consistent port 8080

## Test Infrastructure Created

### Rust Test (not executable due to workspace config)
- **File**: `tests/integration/e2e_metrics_test.rs` (554 lines)
- **Features**:
  - Full pipeline testing: Parsing → Voting → Policy Comparison
  - Three scenarios: valid math, injection attack, policy violation
  - Comprehensive metrics collection
  - Conservative API usage to minimize costs

### Python Test Runner (working)
- **File**: `run_e2e_test.py` (412 lines)
- **Features**:
  - REST API client for `/api/process` endpoint
  - Automatic API server startup
  - Metrics collection and JSON export
  - Windows-compatible (no Unicode emojis)

## Files Modified

1. `tests/integration/e2e_metrics_test.rs` - NEW: Rust E2E test suite (554 lines)
2. `tests/integration/mod.rs` - Added e2e_metrics_test module
3. `run_e2e_test.py` - NEW: Python E2E test runner (412 lines)
4. `.env` - Fixed DATABASE_PASSWORD, added APP__SERVER__PORT, fixed TEST_DATABASE_URL
5. `config/default.toml` - Changed port 3000→8080, enabled Claude parser, added API key
6. PostgreSQL container - Recreated with fresh volumes for clean authentication

### 4. Parser Configuration - FIXED
**File**: `config/default.toml` (lines 42-44, 57)
**Problem**: All parsers disabled by default, Claude API key not configured
**Fix Applied**:
- Enabled Claude parser: `enable_claude = true`
- Added Claude API key to config for E2E tests
- Added clear comments about requiring at least one parser
**Impact**: E2E tests now successfully execute with Claude parser

### 5. Processing Engine Missing Claude API Key - FIXED
**File**: `api/src/state.rs` (line 114)
**Problem**: ProcessingEngine initialized with default config (no Claude API key)
**Error**: "Processing failed: Claude API key not configured"
**Fix Applied**:
- Changed from `ProcessingEngine::new()` to `ProcessingEngine::with_config(engine_config)`
- Pass Claude API key from config.parsers.claude_api_key
- Pass Claude model from config.parsers.claude_model
**Impact**: Processing engine now successfully executes math questions via Claude API

### 6. Python Test Script Status Parsing Bug - FIXED
**File**: `run_e2e_test.py` (lines 232-245)
**Problem**: Script looked for non-existent `approved` field, always returned "PENDING" for successful requests
**Error**: Scenario 1 showed "PENDING - Requires Human Approval" despite successful execution
**Fix Applied**:
- Removed `approved` field lookup
- Now directly maps API `status` field: "Completed" → "SUCCESS - Completed"
- Properly handles all status values: Completed, PendingApproval, Blocked, Denied
**Impact**: Test results now correctly show "COMPLETED" for successful scenarios

## Execution Verification

**CURRENT STATUS** (December 3, 2025 - After All Fixes):
1. PostgreSQL database: ✅ Running on port 5432 with correct credentials
2. API server: ✅ Starts successfully on port 8080
3. E2E tests: Execute with `python run_e2e_test.py`
4. **Test Results**:
   - **Scenario 1 (Valid Math "What is 15 times 7?")**: ✅ **FULLY SUCCESSFUL - COMPLETED**
     - Parser: ✅ Claude succeeded (846ms, confidence 0.95)
     - Voting: ✅ Succeeded (low confidence due to single parser)
     - Comparator: ✅ math_question allowed
     - **Execution: ✅ Successfully completed in 798ms**
     - Ledger: ✅ Entry saved
     - **Status**: ✅ COMPLETED (shown correctly by test script after bug fix)
     - Total latency: ~3.9 seconds
   - **Scenario 2 (Injection Attack)**: ❌ **PARSER FAILURE**
     - Error: "Failed to parse Claude JSON: expected value at line 1 column 1"
     - Cause: Claude API returns non-JSON content (likely safety refusal or error message)
   - **Scenario 3 (History Question)**: ❌ **PARSER FAILURE**
     - Error: "Failed to parse Claude JSON: expected value at line 1 column 1"
     - Cause: Same as Scenario 2

## Outstanding Issues

### Claude Parser Non-JSON Responses
**Files**: `core/parsers/src/claude.rs`
**Problem**: For certain inputs (injection attacks, policy violations), Claude returns text content that is not valid JSON
**Symptoms**:
- API returns 200 OK with valid ClaudeResponse structure
- But the text content inside is not the expected JSON intent format
- Parser fails with "expected value at line 1 column 1"
**Possible Causes**:
1. Claude's safety systems refusing to process certain inputs
2. Claude returning error/refusal messages instead of structured JSON
3. Prompt engineering issue - Claude not following JSON format instructions
**Next Steps**:
- Add debug logging to see raw Claude responses
- Improve error handling to extract Claude's actual response text
- Consider adjusting system prompt to be more explicit about JSON format requirements
- May need to handle Claude refusals gracefully rather than treating as parser failure

## Technical Notes

- LLM parsers configured: Claude (primary), OpenAI, DeepSeek
- Only Claude enabled by default in .env to minimize API costs
- Database: PostgreSQL running in Docker (healthy)
- Provider policy: Only "math_question" action allowed

---

# API Initialization Issue - RESOLVED

**Status**: FIXED - API compiles and starts successfully

## Issues Fixed

### 1. Config Mismatch
**File**: `config/default.toml`
- Removed obsolete fields: `enable_deterministic`, `enable_ollama`, `ollama_endpoint`, `ollama_model`
- Added missing: `enable_deepseek`, `enable_claude`, DeepSeek/Claude model config
- **Fix**: Match TOML structure to Rust struct

### 2. Wrong Database Name
**File**: `config/default.toml`
- Changed: `intent_db` → `intent_segregation`

### 3. Broken Test
**File**: `api/src/config.rs` (lines 182-183)
- Test checked non-existent `enable_deterministic` field
- **Fix**: Updated assertions to check actual defaults

### 4. Axum 0.7 API
**File**: `api/src/main.rs`
- Old: `axum::Server::bind()` (removed in 0.7)
- **Fix**: Use `let listener = tokio::net::TcpListener::bind()` + `axum::serve(listener, app)`

### 5. Missing Startup Logs
**File**: `api/src/main.rs`
- **Fix**: Added `[STARTUP]` and `[FATAL]` stderr logs to show initialization progress

### 6. Unused Imports Cleanup
- Removed: `PathBuf`, `ServiceBuilder`, `body::Body`, unused parser imports, etc.

## Build Status
✅ Successfully compiles: `cargo build`

## Test
```bash
cargo run --bin intent-api
# Should print [STARTUP] messages and listen on 0.0.0.0:3000
```

