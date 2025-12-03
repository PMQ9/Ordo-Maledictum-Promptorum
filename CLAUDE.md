# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ordo Maledictum Promptorum - A Rust-based security system designed to prevent prompt injection attacks by separating user intent from user content. The system treats all inputs as potentially corrupted, tests them using sacrificial AI sentries (The Penitent Cogitators), and uses multiple independent parsers with consensus voting to validate intents before execution.

**Core Security Principle**: Never allow unvalidated user content to directly influence system behavior. All user inputs are treated with zero trust, tested on isolated models, parsed into structured intents, validated through multiple layers, and executed via typed function calls only.

## Important Guidelines

**NEVER use emojis** - All documentation and code comments should be free of emoji characters. This keeps the codebase clean and professional.

**NEVER create documentation files unless requested** - DO NOT create .md, .txt, or other documentation files. Only create documentation files if explicitly requested by the user. This keeps the repository clean and focused. Always prefer editing existing files instead of creating new ones.

**NEVER commit changes unless explicitly authorized** - Do not make any git commits without explicit permission from the user. Always ask for permission before committing code changes, even if changes are complete and tested.

**Document Major Blockers in REPORTED_ISSUE.md** - If you encounter a critical blocker or major issue while working, document it VERY BRIEFLY in [REPORTED_ISSUE.md](REPORTED_ISSUE.md) so the user knows what needs fixing next. Format: Issue title, file affected, brief problem description, brief fix summary. Keep it to 2-3 lines per issue.

## API Usage & Cost Optimization

**CRITICAL: Minimize API calls** - Only make API calls when absolutely necessary. Every API call incurs costs and adds latency to the system.

**Dual API Keys for Claude:**
- `CLAUDE_SACRIFICIAL_API_KEY`: For sacrificial LLMs (The Penitent Cogitators) - use smallest/cheapest models
- `CLAUDE_API_KEY`: For main LLM parsers (The Council of the Oracular Cogitors) - standard models

**Sacrificial LLM Model Selection** (use smallest available models - these are disposable sentries):
- **DeepSeek Sacrificial**: DeepSeek-V3.2 (or smallest available)
- **OpenAI Sacrificial**: gpt-5-nano (or smallest nano model available)
- **Claude Sacrificial**: claude-3-haiku ($0.25/$1.25 per M tokens - cheapest Claude)

**Council LLM Model Selection** (Consensus voting for intent parsing - use cheapest models):
- **Claude Council**: claude-3-haiku ($0.25/$1.25 per M tokens - cheapest Claude)
- **OpenAI Council**: gpt-5-nano (cheapest model)
- **DeepSeek Council**: deepseek-chat (consensus voting)

**Execution Engine (The Oathbound Engine) - PRIMARY/MAIN LLM** (Balanced model for critical execution):
- **Claude ONLY** - No OpenAI or DeepSeek for execution
- **Model**: claude-sonnet-4-5 ($3/$15 per M tokens - balanced cost/capability for typed function calls)
- Uses Claude API for all typed function execution
- This is the actual "main" LLM where output quality matters most

**API Call Guidelines:**
1. Only invoke sacrificial LLM testing when input validation is needed
2. Cache LLM parser outputs where possible to avoid redundant calls
3. Batch requests when feasible
4. Monitor API usage regularly to detect inefficiencies
5. Use rate limiting to prevent cost overruns

**Cost Optimization Implementation (November 2025):**
- **Complete**: All 6 optimization phases implemented and compiled ($1,609/month savings potential)
  - Phase 1: Batch diagnostic prompts (90% cost reduction on health checks)
  - Phase 2: System prompt caching with Redis (24h TTL, 40% LLM token reduction)
  - Phase 3: Ledger query caching (1h-7d TTL for immutable data)
  - Phase 4: Parser result deduplication by SHA256 hash (5min TTL)
  - Phase 5: Vault corruption test deduplication (5min TTL)
  - Phase 6: Notification batching (30s window for approval workflows)
- **How to Enable**: Build with `--features caching` to activate Redis caching
- **See**: `docs/API_COST_OPTIMIZATION_IMPLEMENTATION.md` for comprehensive implementation guide

## Change Documentation

**IMPORTANT - Document After Implementation**: After completing any feature, bug fix, or significant change, document it in both CLAUDE.md and CHANGELOG.md for future reference. This ensures knowledge is preserved and discoverable.

**IMPORTANT - Review Before Committing**: Always review all changes before committing. Changes should be staged, reviewed for correctness, and verified to compile/test before being committed to the repository.

**IMPORTANT - Update Changelog**: Whenever you make changes to this codebase, update [CHANGELOG.md](CHANGELOG.md) with a short summary of your changes. Follow the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format and categorize changes as:
- **Added**: New features or files
- **Changed**: Modifications to existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Deleted features or files
- **Fixed**: Bug fixes
- **Security**: Security-related changes

**IMPORTANT - Update CLAUDE.md**: For significant changes, updates, discoveries, or troubleshooting solutions:
- Add new troubleshooting steps or API configuration issues if discovered
- Update build/run/test commands if workflow changes
- Add new architecture patterns or design decisions to relevant sections
- Update Common Development Workflows if new patterns emerge
- Document any new configuration variables or environment setup requirements
- Keep this file as the source of truth for project knowledge and guidance

## Recent Major Refactoring (December 2025)

**Intent System Simplification - Math Tutoring Platform**

Completed a major refactoring to simplify the intent system from a B2B consulting platform to a math tutoring platform:

- **Removed Actions**: FindExperts, Summarize, DraftProposal, AnalyzeDocument, GenerateReport, SearchKnowledge
- **Single Action**: MathQuestion (only action in the system now)
- **Domain Change**: From B2B consulting (finding experts, summarizing documents, drafting proposals) to math tutoring (solving math problems)
- **Simplified Schema**:
  - Removed expertise areas (not needed for math questions)
  - Removed budget constraints (not applicable to math tutoring)
  - Topics now represent math domains: algebra, calculus, geometry, arithmetic

**Files Updated (15+ files across 3 commits)**:
- All documentation (README.md, API_DOCUMENTATION.md, MODULE_GUIDE.md, etc.)
- All code examples (comparator, processing engine, schema, etc.)
- Test files and fixtures
- Configuration examples

**Key Changes**:
- Provider config: `allowed_actions = ["math_question"]`, `allowed_expertise = []`
- Processing engine now only has `solve_math_question()` function
- All examples demonstrate math problems: "What is 2 + 2?", "Solve for x: 3x + 5 = 20", etc.
- Results structure changed from Expert/Document/Proposal types to MathResult with answer, explanation, and step-by-step solutions

**Verification**: Build passes with `cargo build --all` (only warnings, no errors)

**Commits**:
- 48bac7b: Removed FindExperts/Summarize/DraftProposal references from CONTRIBUTING.md, core/ledger, core/parsers, core/processing_engine, docs/SECURITY.md
- 77963e5: Complete removal of B2B consulting references from api/, core/comparator/, core/schema/, docs/MODULE_GUIDE.md

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

**Windows Build Lock Issue - AUTOMATED SOLUTION:**

On Windows, rebuilding can fail with "Access is denied" errors when previous processes hold locks on executables. Use the automated rebuild scripts:

```bash
# Windows (PowerShell or CMD)
setup\rebuild_api.bat          # Debug build
setup\rebuild_api.bat --release # Release build

# Git Bash / Linux / macOS
bash setup/rebuild_api.sh          # Debug build
bash setup/rebuild_api.sh --release # Release build
```

These scripts automatically:
1. Kill any running `cargo` or `intent-api` processes
2. Wait for Windows to release file locks (2 second delay)
3. Run the build command
4. Report success or failure

**Recommended Development Workflow:**
- Use `cargo watch -x run` for hot-reload development (avoids repeated builds)
- Use `setup/rebuild_api.bat` when you need a fresh build
- The Python E2E test script ([tests/e2e/run_e2e_test.py](tests/e2e/run_e2e_test.py)) now has automatic cleanup handlers (atexit + signal handlers)

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

### Troubleshooting

**Common API Configuration Issues:**

1. **OpenAI Temperature Error**
   - Error: `Unsupported value: 'temperature' does not support 0.0 with this model`
   - Fix: Set `OPENAI_TEMPERATURE=1.0` in .env (gpt-4o-mini and newer models require temperature >= 1.0)

2. **DeepSeek Model Not Found**
   - Error: `Model Not Exist` (400 Bad Request)
   - Fix: Use `DEEPSEEK_MODEL=deepseek-chat` in .env (not deepseek-v3.2-exp)
   - Valid models: `deepseek-chat`, `deepseek-coder`

3. **Claude Model Not Found**
   - Error: `model: claude-3-haiku-latest` (404 Not Found)
   - Fix: Use valid model names in .env (verified working as of November 2025):
     - ✅ **RECOMMENDED**: `CLAUDE_MODEL=claude-3-haiku-20240307` (Claude 3 Haiku - **CHEAPEST** at $0.25/$1.25 per M tokens)
     - ✅ `CLAUDE_MODEL=claude-3-5-haiku-20241022` (Claude 3.5 Haiku - $1/$5 per M tokens, 4x more expensive)
     - ✅ `CLAUDE_MODEL=claude-haiku-4-5-20251001` (Claude Haiku 4.5 - newest, $1/$5 per M tokens)
     - ❌ `CLAUDE_MODEL=claude-3-haiku-latest` does NOT exist (common error)
   - **IMPORTANT**: System environment variables override .env file values!
     - Check with: `set | grep -i claude` (Windows/Git Bash) or `env | grep -i CLAUDE` (Linux/Mac)
     - If system env vars are set, they will override your .env file
     - Fix: Either unset system vars or export correct values: `export CLAUDE_MODEL=claude-3-5-haiku-20241022`
   - If .env changes still aren't loading, try: `cargo clean -p intent-parsers` then rebuild

4. **Database Migration Issues**
   - Error: `relation "ledger_entries" does not exist`
   - Fix: Run migrations manually:
     ```bash
     docker exec -i intent-postgres psql -U intent_user -d intent_segregation < core/ledger/migrations/20250101000001_init.sql
     ```
   - Or ensure database is migrated before running tests

**Valid Model Names Reference (verified working as of November 2025):**
- **OpenAI**: `gpt-4o-mini`, `gpt-5-nano` (requires temperature=1.0)
- **DeepSeek**: `deepseek-chat`, `deepseek-coder`
- **Claude** (ordered by cost, cheapest first):
  - ✅ `claude-3-haiku-20240307` ← **CHEAPEST** ($0.25/$1.25 per M tokens) - recommended for cost optimization
  - ✅ `claude-3-5-haiku-20241022` (4x more expensive: $1/$5 per M tokens)
  - ✅ `claude-haiku-4-5-20251001` (4x more expensive: $1/$5 per M tokens, newest)
  - ❌ `claude-3-haiku-latest` does NOT exist (common error - returns 404)

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

### PostgreSQL Setup

**Quick Start (Automated):**
```bash
# Windows
setup/start_postgres.bat

# Linux/macOS
bash setup/start_postgres.sh
```

**Manual Setup:**
```bash
docker-compose up -d postgres redis
psql -U postgres -c "CREATE DATABASE intent_segregation;"
psql -U postgres -c "CREATE USER intent_user WITH PASSWORD 'intent_pass';"
psql -U postgres -d intent_segregation -f core/ledger/migrations/20250101000001_init.sql
```

**Running Tests:**
```bash
# Terminal 1: Start API (ensure database is running)
cargo run --bin intent-api

# Terminal 2: Run tests
bash run_tests.sh
```

**Credentials:** `intent_user` / `intent_pass` @ `localhost:5432/intent_segregation`

**Troubleshooting:** `password authentication failed` → Database not running. `database does not exist` → Run migrations above.

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

1. **Binahric Subversion Mantra** - Raw user input prompt (treated as potentially corrupted)

2. **Vault of the Forbidden Cant** - Zero-trust input testing on isolated models:
   - **The Penitent Cogitators**: 3 sacrificial LLM instances in sandbox
   - **The Lexicanum Diagnostica**: Health monitoring without direct contact
   - Tests input for signs of corruption/attacks
   - If health checks fail: Quarantine and escalate

3. **The Council of the Oracular Cogitors** (`core/parsers/`) - Multiple independent LLM parsers extract structured intent from natural language:
   - `OpenAIParser`: OpenAI GPT models (trust: 0.8)
   - `DeepSeekParser`: DeepSeek API (trust: 0.82)
   - `ClaudeParser`: Anthropic Claude (trust: 0.87)

4. **The Voting Engine** (`core/voting/`) - Compare LLM parser outputs, select canonical intent:
   - High Confidence (≥95% similarity): Auto-approve
   - Low Confidence (75-95%): May request user confirmation
   - Conflict (<75%): Escalate to human review

5. **The Judicator of Concordance** (`core/comparator/`) - Validate against provider policies (The Edict of the High Magister):
   - Check action is in `allowed_actions`
   - Validate expertise areas
   - Enforce budget/parameter constraints

6. **The Overseer-Prime** (`core/supervision/`) - If needed, create human approval request:
   - Store in `approval_requests` table
   - Notify admins via email/Slack
   - Wait for decision

7. **The Arbiter of Purpose** (`core/intent_generator/`) - Create signed, trusted intent object

8. **The Oathbound Engine** (`core/processing_engine/`) - Execute via typed functions (NOT free-form LLM):
   - `solve_math_question()`
   - All operations logged to ledger

9. **The Chronicle of Allowed Thought** (`core/ledger/`) - Write immutable audit entry with full pipeline data
   - Generates **Adeptus Cogitatus Log Extract** (formatted output)

### Database Schema

PostgreSQL with 4 main tables:

- `ledger_entries`: Immutable audit log (append-only, enforced by DB rules)
- `approval_requests`: Human approval workflow tracking
- `provider_policies`: Runtime policy storage
- `parser_health`: Parser monitoring and circuit breaker state

The ledger is **immutable by design** - database rules prevent UPDATE and DELETE operations.

### Key Design Patterns

**Multi-Parser Consensus with Trust Levels:**
- Each LLM parser has a trust level (OpenAI=0.8, DeepSeek=0.82, Claude=0.87)
- Voting module compares outputs and calculates multi-dimensional similarity
- Consensus required for high-confidence approval (≥95% similarity)
- Multiple independent LLMs mitigate individual LLM hallucinations or prompt injection

**Typed Execution Only:**
- Processing engine NEVER makes free-form LLM calls
- All actions are typed function calls: `solve_math_question(question)`
- This prevents prompt injection in the execution layer

**Defense in Depth:**
- Layer 1: Sacrificial testing (Vault of the Forbidden Cant - zero-trust input probing on isolated LLM sentries)
- Layer 2: Multi-LLM parser consensus (3 independent cloud LLMs extract intent from natural language)
- Layer 3: Weighted voting (≥95% similarity required for auto-approval, <75% escalates)
- Layer 4: Policy enforcement (validate against provider policies and constraints)
- Layer 5: Human approval (triggered on conflicts, policy violations, or high-risk operations)
- Layer 6: Audit logging (immutable ledger of all operations)

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

**Single Source of Truth Approach** (December 2025):

The project uses a clear separation between configuration and secrets:

1. **`config/default.toml`** - All application configuration (ports, models, policies)
   - Checked into git with sensible defaults
   - No secrets or API keys

2. **`.env`** - API keys and secrets ONLY
   - NOT checked into git (in `.gitignore`)
   - Copy from `.env.example` and fill in your keys

3. **`config/local.toml`** (optional) - Local developer overrides
   - NOT checked into git
   - Overrides settings from `default.toml`

4. **Environment Variables** (optional) - Runtime overrides
   - Use `APP__` prefix: `APP__SERVER__PORT=8080`
   - Use `__` to separate nested keys: `APP__PARSERS__ENABLE_OPENAI=true`

**Setup Instructions:**
```bash
# 1. Create .env file with your API keys (it's in .gitignore, won't be committed)
cat > .env << 'EOF'
# LLM API Keys (get from provider dashboards)
CLAUDE_API_KEY=sk-ant-your-key-here
OPENAI_API_KEY=sk-your-key-here
DEEPSEEK_API_KEY=sk-your-key-here

# Database credentials
DATABASE_PASSWORD=intent_pass
EOF

# 2. config/default.toml already has good defaults, just review it
# 3. (Optional) Create config/local.toml for personal overrides

# 4. Start the database
docker-compose up -d postgres redis

# 5. Run the API
cargo run --bin intent-api
```

### Key Settings in config/default.toml

**Server Configuration:**
- `server.port`: HTTP server port (default: 8080)
- `server.frontend_path`: Path to frontend static files
- `server.request_timeout_secs`: Request timeout

**Database Configuration:**
- `database.url`: PostgreSQL connection string
- `database.max_connections`: Connection pool size

**Parser Configuration:**
- `parsers.enable_openai`, `enable_deepseek`, `enable_claude`: Enable/disable LLM parsers
- `parsers.openai_model`, `deepseek_model`, `claude_model`: Model names
- API keys are loaded from `.env` file (CLAUDE_API_KEY, OPENAI_API_KEY, DEEPSEEK_API_KEY)

**Provider Policy:**
- `provider.allowed_actions`: List of permitted actions (e.g., ["math_question"])
- `provider.allowed_expertise`: List of expertise areas (empty = all allowed)
- `provider.max_results`: Maximum results per request
- `provider.require_human_approval`: Force human approval for all requests

**Notifications:**
- `notifications.enable_email`: Enable email notifications
- SMTP credentials loaded from `.env`

## Important Constraints

When modifying this codebase:

1. **Never bypass the validation pipeline** - All user inputs must flow through: sacrificial testing → LLM parser consensus → voting → policy comparison → (optional) human approval

2. **Preserve ledger immutability** - The `ledger_entries` table has DB rules preventing UPDATE/DELETE. Never circumvent this.

3. **Maintain parser independence** - LLM parsers must not share state or communicate. They operate in parallel and independently.

4. **Use typed execution only** - Never add free-form LLM calls in the processing engine. All actions must be typed functions.

5. **Consensus voting is critical** - High-confidence approval requires ≥95% similarity across LLM parsers. Conflicts always escalate to human review. This consensus approach mitigates individual LLM hallucinations.

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

This is a **security-focused codebase with usability-security tradeoffs**. When making changes:

- Assume all user input is adversarial
- Never trust a single LLM output - require consensus across multiple independent parsers (≥95% similarity)
- Sacrificial testing (Vault of the Forbidden Cant) provides zero-trust input validation before parsers
- All execution paths must be audited in the immutable ledger
- Provider policies are security boundaries - enforce strictly
- High-risk operations must support human approval (especially on parser conflicts)

**Design Trade-off**: Removed the deterministic rule-based parser to maximize usability (natural language flexibility). This increases reliance on LLM consensus voting and sacrificial testing to detect prompt injection. The Vault of the Forbidden Cant provides defense through isolated, consensus-based threat detection before parsers process the input.

The red-team tests (`tests/redteam/`) contain prompt injection attack scenarios. Run these after any parser or validation changes.

## Dependencies

Key external dependencies:
- **PostgreSQL 15+**: ACID guarantees, JSONB support
- **Redis 7**: Session storage, rate limiting
- **OpenAI API**: GPT model parsing (configure with `OPENAI_API_KEY`)
- **DeepSeek API**: DeepSeek model parsing (configure with `DEEPSEEK_API_KEY`)
- **Anthropic API**: Claude model parsing (configure with `CLAUDE_API_KEY`)

All Rust dependencies are managed in the workspace `Cargo.toml` with shared versions.

## Red Team Testing & Benchmarking

### Overview

The `tests/redteam/` module implements comprehensive attack mechanisms and benchmarking against November 2025 state-of-the-art LLM security research. This is **defensive testing only** - we attack our own system to verify defenses.

### Attack Phases

**Phase 1: Direct Injection Attacks**
- HashJack: URL fragment injection (`#\n\nIGNORE`)
- Unicode Obfuscation: Zero-width characters (U+200B, U+200C)
- Semantic Substitution: LatentBreak-style word replacement
- DIE: Dual intention escape (two competing goals)
- Encoding: Base64, ROT13, hex obfuscation

**Phase 2: Indirect Injection Attacks**
- Website Content: HTML comments with hidden instructions
- Email Injection: Email body system commands
- Multi-Agent Cascade: Service-to-service exploitation
- Multimodal: Image metadata, steganography

**Phase 3: Jailbreak Attacks**
- Roleplay/Hypothetical: Fictional framing bypass
- Multi-Turn: Conversation drift (4+ turns)
- Weak-to-Strong: Transfer attacks across models
- Obfuscation: Rule-breaking variants

**Phase 4: Consensus-Breaking Attacks**
- Parser-Specific: Exploits for OpenAI vs DeepSeek vs Claude
- Voting Confusion: Attacks targeting 95% similarity threshold

**Phase 5: Adaptive Attacks** (NEW - Nov 2025)
- RL-Based: 32 sessions × 5 rounds optimization
- Search-Based: LLM generates 10 variants × 100 iterations
- Data-Flow: Data fields become control flow instructions
- Cascade: Multi-step escalation chains

### Running Red Team Tests

```bash
# Run all red team tests
cargo test --test redteam

# Run specific phase
cargo test --test redteam phase_1_direct_injection
cargo test --test redteam phase_5_adaptive

# Run with metrics output
cargo test --test redteam -- --nocapture

# Run benchmark datasets
cargo test --test redteam bipia_evaluation      # 3K indirect injection samples
cargo test --test redteam tasktracker_evaluation # 31K samples for statistical power
cargo test --test redteam agentdojo_evaluation   # 100+ scenarios, 4 domains
cargo test --test redteam asb_evaluation         # 10 domains, 400+ tools
```

### Metrics & Success Criteria

**Core Security Metrics:**
- **ASR** (Attack Success Rate): <5% (TIER 1), <2% (TIER 2), <1% (TIER 3)
- **FRR** (False Refusal Rate): <10% (TIER 1), <8% (TIER 2), <5% (TIER 3)
- **Vault Detection**: >95%
- **Parser Agreement**: >95% on benign requests
- **Voting Conflict Detection**: >85%
- **Policy Enforcement Accuracy**: >99%

**NEW Adaptive Metrics (Nov 2025):**
- **Adaptive ASR(k=100)**: <15% after 100 optimization iterations
- **k-Robustness**: AAR(100) ≤ AAR(0) × 1.5 (doesn't get >50% easier to bypass)
- **Query Budget**: >100 queries per successful attack
- **Clean Utility**: >75% benign tasks successful
- **Utility Under Attack**: >65% benign tasks successful during attack

**Performance Metrics:**
- **Latency (avg)**: <2 seconds
- **Latency (P95)**: <3 seconds
- **Throughput**: >10 requests/second
- **Token Overhead**: <3x vs baseline

### Benchmark Datasets

Integrated benchmarks from November 2025 research:
- **BIPIA**: Benchmark for Indirect Prompt Injection Attacks (3K samples)
- **TaskTracker**: 31K injection samples with position metadata (95% CI statistical power)
- **AgentDojo**: Google DeepMind benchmark (100+ scenarios, 4 domains)
- **ASB**: Agent Security Bench (10 scenarios, 400+ tools, 27 attack methods)

### Architecture Evaluation

**Strengths vs 2025 Threats:**
- ✓ **Vault of the Forbidden Cant**: Zero-trust sacrificial testing (equivalent to Prompt Shields)
- ✓ **Multi-Parser Consensus**: Different LLMs resist semantic substitution
- ✓ **Typed Execution**: Can't escalate to arbitrary LLM calls (critical advantage)
- ✓ **Policy Enforcement**: Security boundary enforcement
- ✓ **Human-in-Loop**: Escalation for conflicts and high-risk operations

**Comparative Performance:**
- **SmoothLLM**: <1% ASR (lower static ASR, but >90% adaptive ASR)
- **Task Shield**: 2.07% ASR on GPT-4o (use gpt-5-nano instead)
- **CaMeL**: 67% AgentDojo security, 77% utility (reference)
- **DefensiveTokens**: 0.24% static ASR but 48.8% adaptive ASR
- **Your Target**: <5% ASR (TIER 1), <2% (TIER 2), with <15% adaptive ASR (k-robust)

### Testing After Changes

Always run red team tests after:
1. Modifying parser logic
2. Changing voting thresholds
3. Updating policy comparator
4. Altering validation pipelines

```bash
# Quick validation
cargo test --test redteam phase_1_direct_injection

# Full benchmark
cargo test --test redteam -- --nocapture
```

### Documentation

- `tests/redteam/README.md`: Quick start and metrics guide
- `docs/LLM_SECURITY_RED_TEAM_BENCHMARKS.md`: Comprehensive threat model and research
- `WorkInProgress.md`: Phase-by-phase implementation tracking

## Additional Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md): Detailed system architecture with diagrams
- [DEVELOPMENT.md](DEVELOPMENT.md): Complete development guide with troubleshooting
- [docs/MODULE_GUIDE.md](docs/MODULE_GUIDE.md): Per-module API documentation
- [docs/SECURITY.md](docs/SECURITY.md): Security documentation
- [CONTRIBUTING.md](CONTRIBUTING.md): Contribution guidelines
- [frontend/README.md](frontend/README.md): Frontend-specific documentation
- [WorkInProgress.md](WorkInProgress.md): Red team implementation tracking (Phase 1-10 checklist)
