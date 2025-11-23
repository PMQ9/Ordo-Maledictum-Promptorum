# CI Pipeline Fixes

## Issues Found and Fixes Applied

### 1. ✅ Code Formatting Issues
**Status**: FIXED
- Applied `cargo fmt` to fix all formatting issues

### 2. ✅ Processing Engine Type Mismatches
**Status**: FIXED
- Fixed `intent.topic` → `intent.topic_id`
- Fixed Action enum conversion with `parse_action()` helper
- Fixed constraints HashMap access
- Fixed budget type conversion i64 → u64

### 3. ⚠️ Database Compilation Errors (intent-ledger)
**Status**: NEEDS FIX
**Error**: `set DATABASE_URL to use query macros`

**Solutions**:
1. **Option A** (Recommended for CI): Use offline mode with sqlx
   - Run `cargo sqlx prepare` locally with DATABASE_URL set
   - Commit `.sqlx/` directory to git
   - CI will use offline data

2. **Option B**: Set feature flag to skip compile-time checks
   - Add `sqlx = { workspace = true, features = ["offline"] }`

3. **Option C**: Mock DATABASE_URL in CI
   - Set `DATABASE_URL=postgres://user:pass@localhost/db` in GitHub Actions

### 4. ⚠️ Unused Field Warnings
**Status**: MINOR - Non-blocking

- `malicious-detector`: field `strict_mode` never read
- `processing-engine`: field `config` never read
- `intent-parsers`: field `done` in OllamaResponse never read

**Fix**: Add `#[allow(dead_code)]` attribute or use the fields

### 5. Missing Tests Setup
**Status**: NEEDS INVESTIGATION

Need to verify:
- Integration tests can compile without DATABASE_URL
- Red-team tests compile and run
- Frontend tests work

## Recommended CI Pipeline Order

1. **Format Check**: `cargo fmt --check` ✅
2. **Clippy**: `cargo clippy -- -D warnings`
3. **Build**: `cargo build --all-features`
4. **Unit Tests**: `cargo test --lib`
5. **Integration Tests**: `cargo test --test integration` (skip DB tests)
6. **Red-Team Tests**: `cargo test --test redteam`
7. **Frontend**: `cd frontend && npm test`

## Quick Fixes to Apply Now

### Fix 1: Add sqlx offline data
```bash
# On local machine with PostgreSQL running:
export DATABASE_URL="postgresql://intent_user:intent_pass@localhost:5432/intent_db"
cargo sqlx prepare --workspace
git add .sqlx/
```

### Fix 2: Suppress non-critical warnings
```rust
// In malicious_detector/src/lib.rs
#[allow(dead_code)]
strict_mode: bool,

// In processing_engine/src/lib.rs
#[allow(dead_code)]
config: EngineConfig,
```

### Fix 3: Update GitHub Actions workflow
Add to ci.yml:
```yaml
env:
  SQLX_OFFLINE: true  # Use prepared query data
```

## Current Status

✅ Core compilation works (except ledger DB macros)
✅ Type system fixes applied
✅ Formatting fixes applied
⚠️ Need offline sqlx data for full CI
⚠️ Need to test full pipeline

## Next Steps

1. Generate sqlx offline data (requires local PostgreSQL)
2. Commit `.sqlx/` directory
3. Push fixes
4. Verify CI passes
