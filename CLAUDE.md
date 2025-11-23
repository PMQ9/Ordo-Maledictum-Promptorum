# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Intent Segregation Cybersecurity Architecture - A Rust-based security system designed to prevent prompt injection attacks by separating user intent from user content. The system uses multiple independent parsers with consensus voting to validate intents before execution.

**Core Security Principle**: Never allow unvalidated user content to directly influence system behavior. All user inputs are parsed into structured intents, validated through multiple layers, and executed via typed function calls only.

## Change Documentation

**IMPORTANT**: Whenever you make changes to this codebase, update [CHANGELOG.md](CHANGELOG.md) with a short summary of your changes. Follow the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format and categorize changes as:
- **Added**: New features or files
- **Changed**: Modifications to existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Deleted features or files
- **Fixed**: Bug fixes
- **Security**: Security-related changes

## Build & Run Commands

### Building
```bash
# Build all workspace members
cargo build

# Build optimized release version
cargo build --release

# Build specific package
cargo build -p intent-api
cargo build -p intent-parsers

# Clean build artifacts
cargo clean
```

### Running
```bash
# Quick start (runs API + frontend + checks services)
./run_local.sh

# Run API server manually
cargo run --bin intent-api

# Run with hot-reload
cargo install cargo-watch
cargo watch -x run

# Frontend dev server (separate terminal)
cd frontend && npm run dev
```

### Testing
```bash
# Run all tests
cargo test

# Test specific package
cargo test -p intent-parsers
cargo test -p intent-voting

# Run integration tests
cargo test --test integration

# Run red-team security tests
cargo test --test redteam

# Run with output visible
cargo test -- --nocapture

# Run ignored tests (requires API keys)
cargo test -- --ignored
```

### Linting & Formatting
```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check

# Lint with clippy
cargo clippy

# Clippy with warnings as errors
cargo clippy -- -D warnings

# Auto-fix clippy suggestions
cargo clippy --fix
```

### Database Management
```bash
# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add <migration_name>

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate without dependencies
cargo doc --no-deps

# Generate for specific package
cargo doc -p intent-parsers --open
```

## Architecture Overview

### Workspace Structure

This is a Cargo workspace with the following organization:

```
core/                   # Core security modules (independent libraries)
├── schema/             # Shared types and structures (Intent, Action, etc.)
├── malicious_detector/ # First defense: regex-based attack detection
├── parsers/            # Multi-parser ensemble (deterministic, Ollama, OpenAI)
├── voting/             # Consensus voting on parser outputs
├── comparator/         # Policy validation against provider config
├── intent_generator/   # Creates signed, trusted intent objects
├── processing_engine/  # Executes intents via typed function calls
├── ledger/             # Immutable append-only audit log
├── supervision/        # Human-in-the-loop approval workflow
└── notifications/      # Email/Slack alerts

api/                    # REST API server (Axum)
frontend/               # React/TypeScript UI
tests/                  # Integration and red-team tests
config/                 # Provider policies and configuration
```

### Request Flow Pipeline

All user inputs follow this sequential validation pipeline:

1. **Malicious Detection** (`core/malicious_detector/`) - Fast regex-based checks for SQL injection, command injection, XSS, path traversal

2. **Parser Ensemble** (`core/parsers/`) - Multiple independent parsers extract structured intent:
   - `DeterministicParser`: Rule-based, zero hallucination (trust: 1.0)
   - `OllamaParser`: Local LLM (trust: 0.75)
   - `OpenAIParser`: Cloud LLM (trust: 0.8)

3. **Voting Module** (`core/voting/`) - Compare parser outputs, select canonical intent:
   - High Confidence (≥95% similarity): Auto-approve
   - Low Confidence (75-95%): Use deterministic fallback
   - Conflict (<75%): Escalate to human

4. **Intent Comparator** (`core/comparator/`) - Validate against provider policies:
   - Check action is in `allowed_actions`
   - Validate expertise areas
   - Enforce budget/parameter constraints

5. **Supervision** (`core/supervision/`) - If needed, create human approval request:
   - Store in `approval_requests` table
   - Notify admins via email/Slack
   - Wait for decision

6. **Intent Generator** (`core/intent_generator/`) - Create signed, trusted intent object

7. **Processing Engine** (`core/processing_engine/`) - Execute via typed functions (NOT free-form LLM):
   - `find_experts()`
   - `summarize()`
   - `draft_proposal()`
   - All operations logged to ledger

8. **Ledger** (`core/ledger/`) - Write immutable audit entry with full pipeline data

### Database Schema

PostgreSQL with 4 main tables:

- `ledger_entries`: Immutable audit log (append-only, enforced by DB rules)
- `approval_requests`: Human approval workflow tracking
- `provider_policies`: Runtime policy storage
- `parser_health`: Parser monitoring and circuit breaker state

The ledger is **immutable by design** - database rules prevent UPDATE and DELETE operations.

### Key Design Patterns

**Multi-Parser Consensus with Trust Levels:**
- Each parser has a trust level (deterministic=1.0, LLMs=0.75-0.8)
- Voting module compares outputs and calculates similarity
- Deterministic parser is always trusted on conflicts
- Never rely on a single LLM for security decisions

**Typed Execution Only:**
- Processing engine NEVER makes free-form LLM calls
- All actions are typed function calls: `find_experts(topic, expertise, budget)`
- This prevents prompt injection in the execution layer

**Defense in Depth:**
- Layer 1: Malicious detection (regex patterns)
- Layer 2: Multi-parser validation
- Layer 3: Consensus voting
- Layer 4: Policy enforcement
- Layer 5: Human approval (when needed)
- Layer 6: Audit logging

**Human-in-the-Loop:**
- Triggered on parser conflicts, policy violations, or high-risk operations
- Creates `ApprovalRequest` with full context
- Notifies supervisors immediately
- Blocks execution until human decision

## Module Dependencies

Key dependency relationships:

```
intent-schema (base types)
    ↓
malicious-detector, intent-parsers
    ↓
intent-voting
    ↓
intent-comparator
    ↓
supervision (if needed) → notifications
    ↓
intent-generator
    ↓
processing-engine → ledger
```

All modules depend on `intent-schema` for shared types (`Intent`, `Action`, `Expertise`).

## Configuration

Environment variables are loaded from `.env` (copy `.env.example`):

**Critical settings:**
- `DATABASE_URL`: PostgreSQL connection
- `REDIS_HOST`, `REDIS_PORT`: Cache/session storage
- `ENABLE_DETERMINISTIC`, `ENABLE_OLLAMA`, `ENABLE_OPENAI`: Enable/disable parsers
- `OLLAMA_ENDPOINT`, `OLLAMA_MODEL`: Local LLM config
- `OPENAI_API_KEY`, `OPENAI_MODEL`: Cloud LLM config (optional)
- `ENABLE_HUMAN_APPROVAL`: Enable supervision module
- `SMTP_*` / `SLACK_*`: Notification configuration

Provider policies are stored in `config/default.toml` and can be loaded at runtime.

## Important Constraints

When modifying this codebase:

1. **Never bypass the validation pipeline** - All user inputs must flow through malicious detection → parsers → voting → comparison

2. **Preserve ledger immutability** - The `ledger_entries` table has DB rules preventing UPDATE/DELETE. Never circumvent this.

3. **Maintain parser independence** - Parsers must not share state or communicate. They operate in parallel.

4. **Use typed execution only** - Never add free-form LLM calls in the processing engine. All actions must be typed functions.

5. **Trust levels are security-critical** - The deterministic parser must always have trust level 1.0. Never lower it.

6. **Provider policies are security boundaries** - The comparator enforces these strictly. Changes to policies require careful review.

## Common Development Workflows

### Adding a New Parser

1. Create parser struct in `core/parsers/src/`
2. Implement `IntentParser` trait
3. Add to ensemble in `core/parsers/src/ensemble.rs`
4. Add configuration in `core/parsers/src/config.rs`
5. Set appropriate trust level (LLMs should be <1.0)

### Adding a New Action

1. Add to `Action` enum in `core/schema/src/lib.rs`
2. Add handler in `core/processing_engine/src/lib.rs`
3. Update provider config to allow the action
4. Add tests in `tests/integration/`

### Adding a New Test

Integration tests: `tests/integration/`
Red-team tests: `tests/redteam/`

Run specific test: `cargo test --test integration test_name`

## Security Considerations

This is a **security-focused codebase**. When making changes:

- Assume all user input is adversarial
- Never trust LLM outputs for security decisions (use consensus + deterministic fallback)
- All execution paths must be audited in the ledger
- Provider policies are security boundaries - enforce strictly
- High-risk operations must support human approval

The red-team tests (`tests/redteam/`) contain prompt injection attack scenarios. Run these after any parser or validation changes.

## Dependencies

Key external dependencies:
- **PostgreSQL 15+**: ACID guarantees, JSONB support
- **Redis 7**: Session storage, rate limiting
- **Ollama**: Local LLM inference (optional but recommended)
- **OpenAI API**: Cloud LLM parsing (optional)

All Rust dependencies are managed in the workspace `Cargo.toml` with shared versions.

## Additional Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md): Detailed system architecture with diagrams
- [DEVELOPMENT.md](DEVELOPMENT.md): Complete development guide with troubleshooting
- [docs/MODULE_GUIDE.md](docs/MODULE_GUIDE.md): Per-module API documentation
- [docs/SECURITY.md](docs/SECURITY.md): Security documentation
- [CONTRIBUTING.md](CONTRIBUTING.md): Contribution guidelines
- [frontend/README.md](frontend/README.md): Frontend-specific documentation
