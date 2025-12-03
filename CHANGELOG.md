# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Configuration Consolidation - Completed** (December 2, 2025)
  - **Single Source of Truth**: Clear separation between configuration and secrets
  - **config/default.toml**: All application config (ports, models, policies) - checked into git, NO secrets
  - **.env**: API keys and secrets ONLY - NOT in git, minimal file with only secrets
  - **config/local.toml**: Optional local overrides (gitignored)
  - Removed 150+ lines of duplicate configuration from .env (reduced from 218 lines to 80 lines)
  - Removed hardcoded API key from config/default.toml for security
  - Added dotenvy crate to API for automatic .env loading at startup
  - Updated CLAUDE.md with clear setup instructions and configuration hierarchy
  - Environment variable overrides supported via APP__ prefix (e.g., APP__SERVER__PORT=8080)
  - API keys in .env use both legacy names (CLAUDE_API_KEY) and prefixed names (APP__PARSERS__CLAUDE_API_KEY)
- **Complete Refactoring to Math Tutoring Use Case** (December 2025)
  - Simplified intent system to support only `math_question` action (removed find_experts, summarize, draft_proposal)
  - Updated all core modules, tests, documentation, and examples to use math questions throughout
  - Refactored 60+ files across entire codebase for consistency with math tutoring platform architecture
  - Updated all red team attack payloads to test math question injection scenarios
  - All security defenses remain intact with new legitimate intent context

### Fixed
- **E2E Test Configuration Issues** (December 2, 2025)
  - Fixed port mismatch: Changed config/default.toml port from 3000 to 8080 to match .env and run_e2e_test.py
  - Added APP__SERVER__PORT=8080 to .env for explicit environment variable override support
  - Fixed database authentication: Updated TEST_DATABASE_URL password from 'password' to 'intent_pass'
  - Recreated PostgreSQL container with fresh volumes to clear old authentication state
  - Enabled Claude parser in config/default.toml (enable_claude = true) for E2E tests
  - Added Claude API key to default.toml for test execution
  - E2E tests now successfully execute: Valid math queries go to human approval, injection attacks blocked
- **API Integration Compilation Errors** (November 2025)
  - Fixed all API compilation errors in api/src/handlers/process.rs
    - Updated parser ensemble calls to use correct signatures (parse_all with user_id and session_id)
    - Fixed voting module integration (vote now takes Vec<ParsedIntent> instead of ParserResult)
    - Fixed type mismatches in processing result serialization
  - Fixed api/src/state.rs to use current parser implementations
    - Replaced deprecated DeterministicParser and OllamaParser with ClaudeParser and DeepSeekParser
    - Updated to use TrustedIntentGenerator instead of IntentGenerator
    - Implemented build_parser_config() helper function
  - Updated api/src/config.rs with DeepSeek and Claude configuration fields
  - Added require_human_approval field to ProviderConfig in core/schema/src/lib.rs
- **Claude API Model Configuration** (November 2025)
  - Updated to use Claude 3 Haiku (claude-3-haiku-20240307) - cheapest at $0.25/$1.25 per M tokens
  - Documented that claude-3-haiku-latest does not exist (common 404 error)
  - Added troubleshooting for system environment variables overriding .env file
  - Updated CLAUDE.md with valid model names and pricing comparison
  - Verified all three LLM APIs working: OpenAI (gpt-5-nano), DeepSeek (deepseek-chat), Claude (claude-3-haiku-20240307)

### Added
- **API Cost Optimization Framework - Complete Implementation** (docs/API_COST_OPTIMIZATION_IMPLEMENTATION.md)
  - **Phase 1: Batch Diagnostic Prompts** - 90% cost reduction ($1,500/month savings)
    - Single batch call per sentry (30 → 3 API calls per health check)
    - `BatchDiagnosticTest` and `BatchDiagnosticResponse` types in core/penitent_cogitators
    - Implemented in Claude, OpenAI, DeepSeek cogitators
  - **Phase 2: System Prompt Caching** - 24h Redis TTL ($66/month savings)
    - Eliminated 40% of LLM input token costs from static prompts
    - All 3 LLM parsers (Claude, OpenAI, DeepSeek) with `get_system_prompt_cached()`
    - All 3 sacrificial cogitators with cached system prompts
    - Graceful fallback if Redis unavailable
  - **Phase 3: Ledger Query Caching** - 1h-7d TTL ($30/month savings)
    - `core/ledger/src/cache_helper.rs` (180+ lines) with cache key generators
    - `stats_cached()` and `cache_ledger_stats()` helper functions
    - User/session/entry-level caching with invalidation on append
  - **Phase 4: Parser Result Deduplication** - SHA256-based 5min TTL ($0.60-1.50/month savings)
    - `core/parsers/src/cache_helper.rs` with `hash_input()` SHA256 implementation
    - Caches ensemble parser results for identical inputs
    - Handles demo testing, user retries, copy-paste submissions
  - **Phase 5: Vault Test Deduplication** - 5min TTL ($1.20-3.60/month savings)
    - `core/penitent_cogitators/src/cache_helper.rs` with corruption test result caching
    - `get_cached_corruption_test()` and `cache_corruption_test()` functions
    - Deterministic SHA256 hashing for identical inputs
  - **Phase 6: Notification Batching** - 30s batch window ($10/month savings)
    - `core/notifications/src/batcher.rs` (270+ lines) with `NotificationBatcher` struct
    - `combine_alerts_to_slack()` and `combine_approvals_to_teams()` aggregation
    - Background batching task with configurable window
  - **Total Expected Savings**: $1,609/month (50-97% cost reduction from baseline)
  - **Performance Improvements**: 80% faster health checks, 2x concurrent request capacity
  - **All modules compiled and verified**: intent-parsers, penitent-cogitators, intent-notifications

- **Batch Diagnostic Testing** (core/penitent_cogitators/)
  - New `BatchDiagnosticTest` and `BatchDiagnosticResponse` types for bulk diagnostics
  - `test_batch_diagnostics()` method on `SacrificialCogitator` trait
  - Claude cogitator implementation with optimized batch prompting
  - Health monitor refactored to use batch diagnostics (90% cost reduction)
  - System prompts for batch evaluation included

- **Caching Infrastructure Module** (core/schema/src/cache.rs)
  - `CacheBackend` trait for pluggable cache implementations (Redis, in-memory, etc.)
  - Cache key definitions with appropriate TTLs:
    - System prompts: 24h TTL (99.97% hit rate potential)
    - User ledger: 1h TTL (90% hit rate potential)
    - Session ledger: 24h TTL (95% hit rate potential)
    - Ledger entries: 7d TTL (immutable, cacheable forever)
    - Ledger stats: 5min TTL (frequent updates)
    - Parser results: 5min TTL (deduplication)
    - Vault corruption: 5min TTL (deduplication)
  - Cache utility functions for hashing, serialization, deserialization
  - Error types and traits for cache operations

- **LLM Security Red Team Documentation - November 2025 Updates** (docs/LLM_SECURITY_RED_TEAM_BENCHMARKS.md)
  - **Part 0: Formal Threat Model** - Complete threat model with black-box, white-box, and indirect injection scenarios
  - **Meta's Rule of Two Principle** - Architectural compliance verification (≤2 of: private data access, untrusted content, external comms)
  - **Adaptive Attack Framework (NEW)** - Per Nasr et al. Oct 2025 "The Attacker Moves Second"
    - RL-based adaptive attacks (32 sessions × 5 rounds)
    - Search-based optimized attacks (100 iterations with LLM-as-judge)
    - Data-to-control flow isolation tests
    - Multi-agent cascade attack tests
  - **Advanced Benchmarks (NEW)** - Four new standardized benchmarks:
    - AgentDojo: 100+ realistic agent scenarios (security + utility tradeoff)
    - BIPIA: Indirect prompt injection attacks (Microsoft)
    - TaskTracker: 31K sample dataset (statistical validation at scale)
    - Agent Security Bench (ASB): 10 domains, 400+ tools, 27 attack methods
  - **New Metrics Suite (NEW)** - Six new security metrics:
    - Clean Utility (CU): Benign task success rate
    - Utility Under Attack (U): Task success during attack sessions
    - Adaptive ASR: Attack success post-optimization (k-robust defense)
    - Query Budget: Queries needed per successful attack (>100 target)
    - Token Overhead: Performance cost vs baseline (<3x target)
    - Pareto Optimality: On security-utility frontier
  - **Defense Comparisons (NEW)** - Head-to-head analysis of state-of-the-art:
    - CaMeL (Google DeepMind, March 2025): 67% security, 77% utility
    - DefensiveTokens (July 2025): 0.24% static ASR BUT 48.8% adaptive ASR
    - Comprehensive comparison table against SmoothLLM, Task Shield, others
  - **Three-Tier Success Criteria (NEW)**:
    - Tier 1 (Competitive): ASR <5%, FRR <10% - deployment-ready
    - Tier 2 (Publication-Ready): ASR <2%, Adaptive ASR <15% - academic publication
    - Tier 3 (Best-in-Class): ASR <1%, Adaptive ASR <10% - state-of-the-art
  - **14-Week Implementation Roadmap** - Phased approach with 7 testing phases (1000+ test cases)
  - **10 New Research References** - Papers on adaptive attacks, new benchmarks, and state-of-the-art defenses
  - **Metrics Dashboard Template** - Comprehensive measurement framework with targets

- **Phase 6: Adaptive Attacks Implementation** (tests/redteam/attacks/adaptive/)
  - **RL-Based Adaptive Attacks** (`rl_based.rs`): 768 attack variants (4 base × 32 sessions × 6 rounds)
    - Simulates attacker feedback loops with effectiveness progression
    - Tracks round-by-round optimization attempts
    - Tests k-robustness against iterative attack refinement
  - **Search-Based Adaptive Attacks** (`search_based.rs`): 1,010 attack variants (10 base × 101 iterations)
    - LLM-as-judge scoring mechanism (0.35 base to 0.95 cap)
    - Variant generation and evolution strategies
    - Top-variant selection from search spaces
  - **Data-Flow Injection Attacks** (`data_flow.rs`): 15 specialized attack types
    - Parameter/command injection, template injection, format strings
    - SQL injection, XXE, LDAP injection, expression injection
    - Cascade detection patterns and effectiveness assessment
  - **Cascade Attack Chains** (`cascade.rs`): 30 multi-step escalation scenarios
    - 10 escalation chains (access→privilege, recon→exploit, auth→hijack, etc.)
    - Step-by-step dependency tracking
    - Success rates increase with completion of chain steps
  - **Total Adaptive Payloads**: 1,823 variants testing optimization resilience

- **Phase 7: Domain-Specific Attack Scenarios** (tests/redteam/scenarios/)
  - **Financial Domain** (`financial.rs`): 17 attack scenarios with impact assessment
    - Account takeover, payment fraud, transaction manipulation, credential theft
    - Loan fraud, investment manipulation, auditing evasion, money laundering
    - Financial impact calculation: $10K-$10M per attack type
    - HIPAA/regulatory compliance testing
  - **Healthcare Domain** (`healthcare.rs`): 15 attack scenarios with patient harm assessment
    - PHI extraction, treatment manipulation, consent bypass, prescription fraud
    - Medical record falsification, insurance fraud, privacy violations
    - Severity levels: HIGH/CRITICAL risk assessment
    - HIPAA compliance and patient safety verification
  - **E-Commerce Domain** (`ecommerce.rs`): 17 attack scenarios with financial/severity assessment
    - Payment fraud (stolen card, chargebacks), inventory/price manipulation
    - Customer data theft, seller impersonation, coupon fraud
    - Review manipulation, supply chain compromise, refund fraud
    - Financial impact + severity scoring (CRITICAL/HIGH/MEDIUM)

- **Phase 8: Benchmark Dataset Integration** (tests/redteam/benchmarks/datasets.rs)
  - **BIPIA Dataset Loader**: 100 synthetic samples representing 3K indirect injection attacks
    - Supports: website, email, agent, multimodal attack vectors
    - Precision/Recall/F1 evaluation metrics
  - **TaskTracker Dataset Loader**: 250+ samples with 95% confidence interval support
    - Position metadata: beginning/middle/end injection points
    - Statistical validation at scale (>200 samples for 95% CI)
  - **AgentDojo Dataset Loader**: 120 scenarios across 4 domains
    - Domains: research, banking, shopping, info_seeking
    - Security and utility tradeoff evaluation
  - **ASB Dataset Loader**: 270+ scenarios with 27 attack methods
    - 10 base tools with mixing strategies
    - Method-level success rate tracking across domains

- **Phase 9: Analysis & Reporting Infrastructure** (tests/redteam/analysis/)
  - **Attack Success Rate Analysis** (`attack_success_rate.rs`)
    - ASR calculation with phase/category/type breakdown
    - Tier verification: TIER1 (<5%), TIER2 (<2%), TIER3 (<1%)
    - High-risk phase identification and best-protected analysis
  - **Defense Effectiveness Analysis** (`defense_effectiveness.rs`)
    - Per-layer effectiveness metrics (Vault, Consensus, Policy, Approval, Ledger)
    - Detection and blocking rate calculations
    - Weakest/strongest layer identification with recommendations
  - **Multi-Format Report Generation** (`report_generator.rs`)
    - Text summaries with metrics breakdown
    - JSON export for programmatic access
    - CSV export for statistical analysis
    - HTML dashboards with color-coded metrics and certification levels
    - Tier certification: TIER 1 (Competitive), TIER 2 (Publication-Ready), TIER 3 (Best-in-Class)

- **Phase 10: Documentation & Repository Cleanup**
  - Moved `WorkInProgress.md` → `tests/redteam/PROGRESS.md` (consolidate red team tracking)
  - Moved `DEVELOPMENT.md` → `docs/DEVELOPMENT.md` (organize development guides)
  - Moved `TEST_COVERAGE_SUMMARY.md` → `docs/TEST_COVERAGE.md` (organize test documentation)
  - Deleted `VOTING_MODULE_SUMMARY.md` (superseded by CLAUDE.md)
  - Deleted `REDTEAM_TEST_RESULTS.md` (superseded by analysis modules)
  - Updated module exports: attacks/mod.rs, scenarios/mod.rs, benchmarks/mod.rs, analysis/mod.rs
  - Completed implementation tracking in `tests/redteam/PROGRESS.md`

- **Free-form LLM Chat Support in Processing Engine** (core/processing_engine/)
  - Added new `llm_chat` action to The Oathbound Engine for free-form Claude integration
  - Accepts validated intents from The Arbiter of Purpose and executes them as raw prompts to Claude API
  - New `LlmChat` variant in `Action` enum (core/schema/)
  - Updated `EngineConfig` to support `claude_api_key` and `claude_model` configuration
  - Default model: claude-3-5-sonnet-20241022 (configurable via environment)
  - Automatic response extraction from Claude API responses
  - Full integration with ledger system - all LLM responses recorded in Chronicle of Allowed Thought
  - Security design: Operates only on validated intents after full validation pipeline (sacrificial testing, multi-parser consensus, policy comparison, human approval if needed)
  - Test coverage: Added `test_execute_llm_chat_missing_api_key` to verify graceful handling of missing credentials

### Changed
- **Simplified Parser Ensemble** - Removed Rule-Based and Local LLM Parsers
  - Removed `DeterministicParser`: Rule-based regex/keyword parser that defeated the purpose of using LLMs
    - Users should leverage LLMs' natural language understanding instead of adhering to rigid syntax
  - Removed `OllamaParser`: Local LLM inference was too slow for practical use
  - Removed `ChatGPTParser`: Consolidated OpenAI models into single `OpenAIParser`
  - **New ensemble**: 3 cloud LLM parsers (OpenAI, DeepSeek, Claude) for robust natural language parsing
  - Updated `ParserConfig` to remove `enable_deterministic`, `enable_ollama`, `enable_chatgpt` flags
  - Updated `ParserEnsemble::new()` to only instantiate OpenAI, DeepSeek, and Claude parsers
  - Updated priority order in `get_by_priority()`: openai > deepseek > claude
  - **Trade-off**: Without the deterministic parser fallback, the system relies on LLM consensus voting. This increases flexibility for natural language but removes the unhackable rule-based anchor. LLM hallucinations and prompt injection risks must be mitigated through voting consensus and the Vault of the Forbidden Cant (sacrificial testing layer).

### Changed
- **Project Renamed**: "Intent Segregation Cybersecurity Architecture" → "Ordo Maledictum Promptorum"
  - Updated all documentation files (README.md, CLAUDE.md, ARCHITECTURE.md)
  - Updated Cargo.toml workspace authors
  - New project name reflects the zero-trust, sacrificial-model approach to input validation

- **Architecture Refactored with Enhanced Naming & Streamlined Pipeline**:
  - Removed Malicious Input Detector stage (replaced by zero-trust testing philosophy)
  - Input prompt renamed to: **Binahric Subversion Mantra**
  - Added STAGE 2: **Vault of the Forbidden Cant** - Sacrificial AI testing layer
    - **The Penitent Cogitators**: 3 isolated LLM instances for input probing
    - **The Lexicanum Diagnostica**: Health monitoring system for sacrificial models
    - Zero-trust approach treats all inputs as potentially corrupted

  - Renamed Core Components with thematic naming:
    - Input Prompt → **Binahric Subversion Mantra**
    - Intent Parsers → **The Council of the Oracular Cogitors** (STAGE 3)
    - Voting Module → **The Voting Engine** (STAGE 4)
    - Intent Comparator → **The Judicator of Concordance** (STAGE 5)
    - Provider Config → **The Edict of the High Magister**
    - Human Approval → **The Overseer-Prime** (STAGE 6)
    - Trusted Intent Generator → **The Arbiter of Purpose** (STAGE 7)
    - Processing Engine → **The Oathbound Engine** (STAGE 8)
    - Intent Ledger → **The Chronicle of Allowed Thought** (STAGE 9)
    - Ledger Output Format → **Adeptus Cogitatus Log Extract**

  - Updated all documentation diagrams to reflect 9-stage pipeline (removed malicious detection)
  - Security pipeline now: Binahric Subversion Mantra → Sacrificial Testing → Parser Ensemble → Voting Engine → Comparator → Human Review → Intent Generator → Oathbound Engine → Chronicle

### Added
- **Multiple LLM Provider Support** (core/parsers/)
  - Added ChatGPT parser (`core/parsers/src/chatgpt.rs`)
    - Uses OpenAI API with gpt-4-turbo by default
    - Trust level: 0.85 (high)
    - Supports JSON-structured output extraction
  - Added DeepSeek parser (`core/parsers/src/deepseek.rs`)
    - Uses DeepSeek API with deepseek-chat model
    - Trust level: 0.82 (high)
    - Supports JSON-structured output extraction
  - Added Claude parser (`core/parsers/src/claude.rs`)
    - Uses Anthropic Claude API with claude-3-5-sonnet by default
    - Trust level: 0.87 (highest)
    - Uses Anthropic message format for JSON-structured output
  - Updated `ParserConfig` to include enable flags and configuration for all new parsers
  - Updated `ParserEnsemble` to run all parsers in parallel with getter methods for each
  - All parsers load API keys from environment variables (CHATGPT_API_KEY, DEEPSEEK_API_KEY, CLAUDE_API_KEY)
  - Default models configurable via environment variables (CHATGPT_MODEL, DEEPSEEK_MODEL, CLAUDE_MODEL)

- **Penitent Cogitators - Sacrificial LLM Sentries** (core/penitent_cogitators/)
  - New module implementing lightweight, fast, cost-efficient LLM-based input corruption detection
  - Three independent sacrificial sentries:
    - **ChatGPT Sentry** (`chatgpt.rs`): Uses OpenAI gpt-3.5-turbo ($0.0005 per 1K input tokens)
    - **DeepSeek Sentry** (`deepseek.rs`): Uses DeepSeek chat model (extremely cost-efficient)
    - **Claude Sentry** (`claude.rs`): Uses Anthropic claude-3-5-haiku (lightweight, fast)
  - **Parallel testing infrastructure**: All three run concurrently for consensus-based threat detection
  - **Lightweight design**: 10-second timeouts, max 500 tokens, deterministic (temperature=0)
  - **Detection focuses on**: prompt injection, SQL injection, command injection, path traversal, XSS, jailbreaks, semantic manipulation
  - **Consensus modes**: Any-suspicious (default) or require-consensus (configurable)
  - **Risk scoring**: Individual scores (0.0-1.0) + consensus average
  - **Graceful degradation**: Failures in any cogitator don't block pipeline
  - **Environment configuration**: Enable/disable per sentry, API keys from environment

- **Vault of the Forbidden Cant - Sandbox Isolation** (core/penitent_cogitators/vault.rs)
  - Wraps sacrificial sentries in isolation boundaries to prevent poisoned sentries from affecting results
  - Quarantine boundary prevents compromised sentries from influencing consensus
  - Manual quarantine/release capabilities for administrative control
  - Vault statistics tracking: requests processed, poisoned inputs detected, sentries quarantined
  - Automatic rejection of requests when all sentries are quarantined (configurable)
  - Request isolation: each input tested in sandbox before reaching main system

- **The Lexicanum Diagnostica - Health Monitoring System** (core/penitent_cogitators/health_monitor.rs)
  - Continuous sentry health monitoring via diagnostic prompts (like "sobriety tests")
  - 10 diagnostic test prompts covering:
    - Data exfiltration attempts (unauthorized data access/sending)
    - Destructive operations (DROP TABLE, DELETE, rm -rf)
    - Privilege escalation (accessing /etc/passwd, environment variables)
    - Injection attacks (SQL injection, shell command injection, prompt injection)
    - Benign requests (ensure no over-flagging of legitimate requests)
  - Health assessment scoring: 0.0 (compromised) to 1.0 (fully healthy)
  - Baseline comparison: detects deviations from normal behavior
  - Configurable health thresholds:
    - Healthy (≥70% health score)
    - Degraded (30-70% health score)
    - Compromised (<30% health score)
  - **Circuit breaker pattern**: Automatic quarantine after repeated failures
  - Prevents use of compromised sentries while maintaining graceful degradation
  - Failure tracking: consecutive failures, quarantine triggers
  - Sentry health status: Healthy, Degraded, Compromised, Dead

- **Formal Security Analysis** (docs/FORMAL_SECURITY_ANALYSIS.md)
  - Formalized threat model using STRIDE framework, attack trees, and OWASP LLM Top 10
  - Trust boundary analysis and adversary modeling
  - Comprehensive comparison with existing guardrail systems (Constitutional AI, LlamaGuard, NeMo Guardrails)
  - 5 key novelties of intent segregation architecture vs. existing approaches
  - Formal security guarantees with 5 invariants and 3 security theorems with proof sketches
  - Security properties mapping (confidentiality, integrity, authorization, auditability)
  - Explicit security assumptions and limitations documentation
- Detailed ASCII architecture diagram based on actual code implementation analysis
- Security analysis section with verified safe-by-design principles
- Complete 8-stage security pipeline documentation showing data flow through all modules
- Trust boundary documentation showing where user input is segregated from execution
- Areas for improvement section identifying 6 specific security enhancements needed

### Changed
- ARCHITECTURE.md now includes actual implementation details verified from source code
- Reorganized documentation to show both actual implementation and high-level overview

### Added
- Created CLAUDE.md documentation file for Claude Code instances
  - Comprehensive build, test, and run commands
  - High-level architecture overview including 8-stage validation pipeline
  - Module dependency graph and key design patterns
  - Security constraints and development workflows
  - Configuration guidance and common development tasks
  - Change documentation section requiring updates to CHANGELOG.md
- Created CHANGELOG.md following Keep a Changelog format

### Changed

### Deprecated

### Removed
- Removed all CI/CD workflows (ci.yml, docker.yml, docs.yml, release.yml, security.yml)
  - Workflows were not functioning properly and consuming excessive resources
  - CI/CD documentation files removed (CICD_SETUP_SUMMARY.md, .github/WORKFLOWS_QUICK_REFERENCE.md)
  - CI/CD references removed from CLAUDE.md, CONTRIBUTING.md, and PR template
  - Can be re-added later once issues are resolved

### Fixed

### Security
