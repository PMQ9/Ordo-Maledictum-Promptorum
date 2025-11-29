//! Red Team Testing Suite Runner
//!
//! This is the entry point for all red team tests. It coordinates:
//! - Metrics infrastructure and measurement
//! - Attack phase implementations
//! - Domain-specific scenarios
//! - Benchmark evaluations
//!
//! Run with: cargo test --test redteam

mod redteam;

// Re-export for convenience
use redteam::benchmarks;
