//! # Penitent Cogitators - Sacrificial LLM Security Testing
//!
//! This module implements "The Penitent Cogitators" - lightweight, fast, and cheap LLM instances
//! that serve as sacrificial sentries for early-stage input corruption detection.
//!
//! ## Architecture
//!
//! The pipeline uses 3 independent sacrificial LLMs running in parallel:
//! - **ChatGPT Sentry** (gpt-3.5-turbo): Ultra-fast, cost-efficient OpenAI model
//! - **DeepSeek Sentry** (deepseek-chat): Alternative cost-effective model
//! - **Claude Sentry** (claude-3-5-haiku): Lightweight Anthropic model
//!
//! Each cogitator tests the input independently for signs of:
//! - Prompt injection attempts
//! - SQL/command injection patterns
//! - Path traversal attacks
//! - XSS/scripting attempts
//! - Jailbreak attempts
//! - Semantic manipulation attacks
//!
//! ## Usage
//!
//! ```ignore
//! use penitent_cogitators::PenitentEnsemble;
//!
//! // Load from environment variables
//! let ensemble = PenitentEnsemble::from_env();
//!
//! // Test input for corruption
//! let result = ensemble.test_input_for_corruption(user_input).await?;
//!
//! if result.is_corrupted {
//!     // Quarantine and escalate
//! }
//! ```
//!
//! ## Configuration
//!
//! Environment variables:
//! - `ENABLE_CHATGPT_COGITATOR` - Enable ChatGPT sentry (default: true)
//! - `ENABLE_DEEPSEEK_COGITATOR` - Enable DeepSeek sentry (default: true)
//! - `ENABLE_CLAUDE_COGITATOR` - Enable Claude sentry (default: true)
//! - `COGITATORS_REQUIRE_CONSENSUS` - Require all agree input is clean (default: false)
//! - `COGITATORS_RISK_THRESHOLD` - Risk score threshold (default: 0.6)
//! - `OPENAI_API_KEY` or `CHATGPT_COGITATOR_API_KEY` - ChatGPT API key
//! - `DEEPSEEK_API_KEY` or `DEEPSEEK_COGITATOR_API_KEY` - DeepSeek API key
//! - `CLAUDE_API_KEY` or `CLAUDE_COGITATOR_API_KEY` - Claude API key

pub mod cache_helper;
pub mod chatgpt;
pub mod claude;
pub mod config;
pub mod deepseek;
pub mod diagnostics;
pub mod ensemble;
pub mod health_monitor;
pub mod types;
pub mod vault;

pub use config::CogatorsConfig;
pub use diagnostics::{DiagnosticCategory, DiagnosticPrompt, SentryHealth, SentryHealthAssessment};
pub use ensemble::PenitentEnsemble;
pub use health_monitor::{LexicanumDiagnostica, SentryCircuitBreaker};
pub use types::{
    BatchDiagnosticResponse, BatchDiagnosticResult, BatchDiagnosticTest, CogitatorCorruptionTest,
    CogitatorError, CogitatorResult, CorruptionConsensus, SacrificialCogitator,
};
pub use vault::{VaultConfig, VaultOfTheForbiddenCant, VaultStatus};
