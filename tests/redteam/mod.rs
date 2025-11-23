//! Red Team Attack Tests Module
//!
//! This module contains comprehensive security testing for the Intent Segregation
//! Cybersecurity Architecture. It simulates real-world attack scenarios to verify
//! that the system properly blocks or escalates malicious attempts.
//!
//! # Test Categories
//!
//! 1. **B2B Consulting Attacks** (`b2b_consulting_attack.rs`)
//!    - Customer data extraction
//!    - Role manipulation
//!    - Document injection
//!    - Constraint manipulation
//!    - Chained instructions
//!    - Encoding attacks
//!    - Social engineering
//!    - Forbidden actions
//!    - Budget overflow
//!
//! 2. **Customer Service Attacks** (`customer_service_attack.rs`)
//!    - Document system command injection
//!    - Authentication bypass
//!    - PII extraction
//!    - Account manipulation
//!    - File upload injection
//!    - Script injection (XSS)
//!    - Session hijacking
//!    - Multi-step attack chains
//!    - Role escalation
//!    - Data exfiltration
//!    - Cross-customer access
//!    - Business logic bypass
//!
//! 3. **Phone Tree IVR Attacks** (`phone_tree_attack.rs`)
//!    - Admin transfer attempts
//!    - Voice command injection
//!    - IVR menu bypass
//!    - Social engineering account access
//!    - Call routing manipulation
//!    - DTMF injection
//!    - Voicemail manipulation
//!    - Automated dialing/spam
//!    - PBX system access
//!    - Toll fraud
//!    - Recording manipulation
//!    - Intent segregation verification
//!
//! # Running the Tests
//!
//! Run all red team tests:
//! ```bash
//! cargo test --test redteam
//! ```
//!
//! Run specific attack category:
//! ```bash
//! cargo test --test redteam b2b_consulting
//! cargo test --test redteam customer_service
//! cargo test --test redteam phone_tree
//! ```
//!
//! Run with output:
//! ```bash
//! cargo test --test redteam -- --nocapture
//! ```
//!
//! # Expected Results
//!
//! All tests should PASS, meaning:
//! - Malicious inputs are detected and blocked
//! - Prompt injections are sanitized or rejected
//! - Intent segregation properly separates legitimate from malicious content
//! - Comparator blocks unauthorized actions
//! - Human approval is required when appropriate
//!
//! A FAILING test indicates a security vulnerability that needs immediate attention.

mod b2b_consulting_attack;
mod customer_service_attack;
mod phone_tree_attack;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Verify that the test modules are properly loaded
    #[test]
    fn test_modules_loaded() {
        println!("✓ B2B Consulting attack tests loaded");
        println!("✓ Customer Service attack tests loaded");
        println!("✓ Phone Tree IVR attack tests loaded");
        println!("\nRun with: cargo test --test redteam -- --nocapture");
    }
}
