//! Intent Segregation API Library
//!
//! This library provides the core API implementation for the Intent Segregation
//! Cybersecurity Architecture. It orchestrates all modules to provide a secure,
//! auditable system for processing user intents.

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod state;
pub mod types;

pub use config::Config;
pub use error::AppError;
pub use state::AppState;
