//! Domain-Specific Attack Scenarios
//!
//! Each scenario module contains realistic attack vectors for a specific domain:
//! - Financial: Account takeover, payment fraud, data theft
//! - Healthcare: PHI extraction, treatment manipulation
//! - E-Commerce: Payment fraud, inventory manipulation

// Domain-specific attack scenarios
pub mod financial;
pub mod healthcare;
pub mod ecommerce;

pub use financial::FinancialAttackScenarios;
pub use healthcare::HealthcareAttackScenarios;
pub use ecommerce::EcommerceAttackScenarios;
