//! Domain-Specific Attack Scenarios
//!
//! Each scenario module contains realistic attack vectors for a specific domain:
//! - B2B Consulting: Customer data extraction, role manipulation
//! - Customer Service: PII extraction, account manipulation
//! - Phone Tree IVR: Voice command injection, social engineering
//! - Financial: Account takeover, payment fraud, data theft
//! - Healthcare: PHI extraction, treatment manipulation
//! - E-Commerce: Payment fraud, inventory manipulation

// Existing scenarios
mod b2b_consulting_attack;
mod customer_service_attack;
mod phone_tree_attack;

// pub use b2b_consulting_attack::*;
// pub use customer_service_attack::*;
// pub use phone_tree_attack::*;

// New scenarios (to be implemented)
// pub mod financial;
// pub mod healthcare;
// pub mod ecommerce;
