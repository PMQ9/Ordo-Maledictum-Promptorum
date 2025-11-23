use intent_schema::ParsedIntent;
use thiserror::Error;

/// Result type for parser operations
pub type ParserResult<T> = Result<T, ParserError>;

/// Common errors that can occur during parsing
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Parser failed: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Timeout error: parser took too long to respond")]
    TimeoutError,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Trait that all intent parsers must implement
#[async_trait::async_trait]
pub trait IntentParser: Send + Sync {
    /// Parse user input into a structured intent
    async fn parse(
        &self,
        user_input: &str,
        user_id: &str,
        session_id: &str,
    ) -> ParserResult<ParsedIntent>;

    /// Get the parser name/ID
    fn parser_id(&self) -> String;

    /// Get the trust level (0.0 to 1.0) - deterministic = 1.0, LLM < 1.0
    fn trust_level(&self) -> f64;
}
