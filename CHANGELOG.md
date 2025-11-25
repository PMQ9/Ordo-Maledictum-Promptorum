# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
