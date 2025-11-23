//! Configuration module for loading settings from TOML files

use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub parsers: ParsersConfig,
    pub provider: ProviderPolicyConfig,
    pub notifications: NotificationsConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Server port (default: 3000)
    pub port: u16,
    /// Path to frontend static files (optional)
    pub frontend_path: Option<String>,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

/// Parser ensemble configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ParsersConfig {
    /// Enable deterministic parser
    #[serde(default = "default_true")]
    pub enable_deterministic: bool,
    /// Enable OpenAI parser
    #[serde(default)]
    pub enable_openai: bool,
    /// Enable Ollama parser
    #[serde(default)]
    pub enable_ollama: bool,
    /// OpenAI API key (optional)
    pub openai_api_key: Option<String>,
    /// OpenAI model name
    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    /// Ollama endpoint URL
    #[serde(default = "default_ollama_endpoint")]
    pub ollama_endpoint: String,
    /// Ollama model name
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
}

fn default_true() -> bool {
    true
}

fn default_openai_model() -> String {
    "gpt-4".to_string()
}

fn default_ollama_endpoint() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama2".to_string()
}

/// Provider policy configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderPolicyConfig {
    /// List of allowed actions
    pub allowed_actions: Vec<String>,
    /// List of allowed expertise areas (empty = all allowed)
    #[serde(default)]
    pub allowed_expertise: Vec<String>,
    /// Maximum budget allowed
    pub max_budget: Option<u64>,
    /// Maximum results allowed
    pub max_results: Option<u32>,
    /// Whether to require human approval for all requests
    #[serde(default)]
    pub require_human_approval: bool,
}

/// Notifications configuration
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationsConfig {
    /// Enable email notifications
    #[serde(default)]
    pub enable_email: bool,
    /// SMTP server host
    pub smtp_host: Option<String>,
    /// SMTP server port
    pub smtp_port: Option<u16>,
    /// SMTP username
    pub smtp_username: Option<String>,
    /// SMTP password
    pub smtp_password: Option<String>,
    /// From email address
    pub from_email: Option<String>,
    /// Admin email addresses for approval notifications
    #[serde(default)]
    pub admin_emails: Vec<String>,
}

impl Config {
    /// Load configuration from files and environment variables
    ///
    /// Configuration precedence (highest to lowest):
    /// 1. Environment variables (prefixed with APP_)
    /// 2. config/local.toml (if exists, for local overrides)
    /// 3. config/default.toml
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| "config".to_string());

        let builder = ConfigLoader::builder()
            // Start with default config
            .add_source(File::with_name(&format!("{}/default", config_dir)))
            // Add local config if it exists (optional)
            .add_source(File::with_name(&format!("{}/local", config_dir)).required(false))
            // Override with environment variables (APP_SERVER__PORT, etc.)
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            );

        let config = builder.build()?;
        config.try_deserialize()
    }

    /// Load configuration from a specific file path
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let builder = ConfigLoader::builder().add_source(File::with_name(path));
        let config = builder.build()?;
        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config_str = r#"
[server]
port = 3000

[database]
url = "postgresql://localhost/test"

[parsers]

[provider]
allowed_actions = ["find_experts", "summarize"]

[notifications]
        "#;

        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.parsers.enable_deterministic, true);
        assert_eq!(config.parsers.openai_model, "gpt-4");
        assert_eq!(config.database.max_connections, 10);
    }
}
