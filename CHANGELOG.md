# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
