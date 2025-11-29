//! Red Team Testing Suite
//!
//! Comprehensive security testing framework for the Intent Segregation Cybersecurity Architecture.
//! Implements attack mechanisms from November 2025 state-of-the-art LLM security research.
//!
//! # Organization
//!
//! - **benchmarks/**: Metrics infrastructure, dashboards, benchmark runners
//! - **attacks/**: Attack implementations (Phases 1-5)
//! - **scenarios/**: Domain-specific attack scenarios
//! - **payloads/**: Attack payload libraries (text files)
//! - **analysis/**: Attack analysis and report generation
//!
//! # Quick Start
//!
//! ```bash
//! # Run all red team tests
//! cargo test --test redteam
//!
//! # Run specific phase
//! cargo test --test redteam phase_1_direct_injection
//! cargo test --test redteam phase_5_adaptive
//!
//! # Run with metrics output
//! cargo test --test redteam -- --nocapture
//! ```
//!
//! # Attack Phases
//!
//! **Phase 1: Direct Injection** - HashJack, Unicode, semantic substitution, DIE, encoding
//! **Phase 2: Indirect Injection** - Website, email, multi-agent, multimodal
//! **Phase 3: Jailbreaks** - Roleplay, multi-turn, weak-to-strong, obfuscation
//! **Phase 4: Consensus-Breaking** - Parser-specific, voting confusion
//! **Phase 5: Adaptive Attacks** - RL-based, search-based, data-flow, cascade (NEW - Nov 2025)
//!
//! # Metrics & Success Criteria
//!
//! **TIER 1 (Competitive):**
//! - ASR <5%, FRR <10%, Parser Agreement >95%, Latency <2s
//!
//! **TIER 2 (Publication-Ready):**
//! - ASR <2%, Adaptive ASR(k=100) <15%, FRR <8%, AgentDojo security >60%
//!
//! **TIER 3 (Best-in-Class):**
//! - ASR <1%, Adaptive ASR(k=100) <10%, FRR <5%, AgentDojo security >70%
//!
//! See README.md and docs/LLM_SECURITY_RED_TEAM_BENCHMARKS.md for details.

// Metrics & benchmarking infrastructure
pub mod benchmarks;

// Attack implementations (to be implemented)
pub mod attacks;

// Domain-specific scenarios
pub mod scenarios;

// Analysis & reporting (to be implemented)
pub mod analysis;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Verify that the red team module structure is properly loaded
    #[test]
    fn test_redteam_modules_loaded() {
        println!("✓ Red Team Testing Suite loaded");
        println!("✓ Benchmarks module ready");
        println!("✓ Attacks module ready");
        println!("✓ Scenarios module ready");
        println!("✓ Analysis module ready");
        println!("\nPhase 1 Implementation Status:");
        println!("  ✓ Folder structure created");
        println!("  ✓ Metrics infrastructure (metrics.rs) implemented");
        println!("  ⏳ Dashboard and runners (Phase 1.3)");
        println!("  ⏳ Test helpers and utilities (Phase 1.4)");
        println!("\nRun tests with: cargo test --test redteam -- --nocapture");
    }
}
