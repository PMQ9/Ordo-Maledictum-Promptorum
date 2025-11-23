use serde::{Deserialize, Serialize};

/// Configuration for the Ollama parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama API endpoint (default: http://localhost:11434)
    pub endpoint: String,

    /// Model to use (e.g., "llama2", "mistral")
    pub model: String,

    /// Temperature (should be 0 for deterministic output)
    pub temperature: f32,

    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama2".to_string(),
            temperature: 0.0,
            timeout_secs: 30,
        }
    }
}

/// Configuration for the OpenAI parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// OpenAI API key
    pub api_key: String,

    /// Model to use (e.g., "gpt-4o-mini")
    pub model: String,

    /// Temperature (should be 0 for deterministic output)
    pub temperature: f32,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Base URL for API (default: https://api.openai.com/v1)
    pub base_url: String,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.0,
            timeout_secs: 30,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

impl OpenAIConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

/// Overall parser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Enable deterministic parser
    pub enable_deterministic: bool,

    /// Enable Ollama parser
    pub enable_ollama: bool,

    /// Enable OpenAI parser
    pub enable_openai: bool,

    /// Ollama configuration
    pub ollama: OllamaConfig,

    /// OpenAI configuration
    pub openai: OpenAIConfig,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            enable_deterministic: true,
            enable_ollama: true,
            enable_openai: true,
            ollama: OllamaConfig::default(),
            openai: OpenAIConfig::default(),
        }
    }
}

impl ParserConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let openai_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

        let ollama_endpoint = std::env::var("OLLAMA_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let ollama_model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());

        let openai_model =
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        Ok(Self {
            enable_deterministic: true,
            enable_ollama: std::env::var("ENABLE_OLLAMA")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_openai: std::env::var("ENABLE_OPENAI")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            ollama: OllamaConfig {
                endpoint: ollama_endpoint,
                model: ollama_model,
                ..Default::default()
            },
            openai: OpenAIConfig {
                api_key: openai_api_key,
                model: openai_model,
                ..Default::default()
            },
        })
    }
}
