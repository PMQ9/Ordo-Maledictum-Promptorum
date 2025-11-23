//! Regression Tests Module
//!
//! This module contains all regression tests for the Intent Segregation system.

// Shared test helpers
#[path = "../test_helpers.rs"]
mod test_helpers;

// Regression test modules
pub mod known_issues_test;
pub mod performance_regression_test;
pub mod security_regression_test;
