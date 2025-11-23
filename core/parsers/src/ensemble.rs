use crate::config::ParserConfig;
use crate::deterministic::DeterministicParser;
use crate::ollama::OllamaParser;
use crate::openai::OpenAIParser;
use crate::types::{IntentParser, ParserError};
use intent_schema::ParsedIntent;
use std::sync::Arc;
use std::time::Instant;

/// Result from running the parser ensemble
#[derive(Debug)]
pub struct EnsembleResult {
    /// Results from all parsers that completed successfully
    pub results: Vec<ParsedIntent>,

    /// Errors from parsers that failed
    pub errors: Vec<(String, ParserError)>,

    /// Total time taken to run all parsers in parallel
    pub total_time_ms: u64,

    /// Number of parsers that ran
    pub parsers_count: usize,

    /// Number of successful parses
    pub success_count: usize,
}

impl EnsembleResult {
    /// Get the result from the deterministic parser if available
    pub fn get_deterministic(&self) -> Option<&ParsedIntent> {
        self.results
            .iter()
            .find(|r| r.parser_id == "deterministic_v1")
    }

    /// Get the result from the Ollama parser if available
    pub fn get_ollama(&self) -> Option<&ParsedIntent> {
        self.results.iter().find(|r| r.parser_id == "ollama_v1")
    }

    /// Get the result from the OpenAI parser if available
    pub fn get_openai(&self) -> Option<&ParsedIntent> {
        self.results.iter().find(|r| r.parser_id == "openai_v1")
    }

    /// Get the highest confidence result
    pub fn get_highest_confidence(&self) -> Option<&ParsedIntent> {
        self.results
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }

    /// Get result by parser ID priority (deterministic > ollama > openai)
    pub fn get_by_priority(&self) -> Option<&ParsedIntent> {
        self.get_deterministic()
            .or_else(|| self.get_ollama())
            .or_else(|| self.get_openai())
    }
}

/// Parser ensemble that runs multiple parsers in parallel
pub struct ParserEnsemble {
    parsers: Vec<Arc<dyn IntentParser>>,
}

impl ParserEnsemble {
    /// Create a new ensemble from configuration
    pub fn new(config: ParserConfig) -> Self {
        let mut parsers: Vec<Arc<dyn IntentParser>> = Vec::new();

        // Add deterministic parser
        if config.enable_deterministic {
            parsers.push(Arc::new(DeterministicParser::new()));
        }

        // Add Ollama parser
        if config.enable_ollama {
            parsers.push(Arc::new(OllamaParser::new(config.ollama)));
        }

        // Add OpenAI parser
        if config.enable_openai {
            parsers.push(Arc::new(OpenAIParser::new(config.openai)));
        }

        Self { parsers }
    }

    /// Create ensemble with specific parsers
    pub fn with_parsers(parsers: Vec<Arc<dyn IntentParser>>) -> Self {
        Self { parsers }
    }

    /// Run all parsers in parallel
    pub async fn parse_all(
        &self,
        user_input: &str,
        user_id: &str,
        session_id: &str,
    ) -> EnsembleResult {
        let start = Instant::now();

        if self.parsers.is_empty() {
            return EnsembleResult {
                results: Vec::new(),
                errors: vec![(
                    "ensemble".to_string(),
                    ParserError::ConfigError("No parsers enabled in ensemble".to_string()),
                )],
                total_time_ms: 0,
                parsers_count: 0,
                success_count: 0,
            };
        }

        // Create tasks for all parsers
        let mut tasks = Vec::new();
        for parser in &self.parsers {
            let parser = Arc::clone(parser);
            let input = user_input.to_string();
            let uid = user_id.to_string();
            let sid = session_id.to_string();

            tasks.push(tokio::spawn(async move {
                let parser_id = parser.parser_id();
                match parser.parse(&input, &uid, &sid).await {
                    Ok(result) => Ok(result),
                    Err(e) => Err((parser_id, e)),
                }
            }));
        }

        // Wait for all parsers to complete
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for task in tasks {
            match task.await {
                Ok(Ok(parsed)) => {
                    tracing::debug!(
                        "Parser {} completed successfully with confidence {}",
                        parsed.parser_id,
                        parsed.confidence
                    );
                    results.push(parsed);
                }
                Ok(Err((parser_id, error))) => {
                    tracing::warn!("Parser {} failed: {}", parser_id, error);
                    errors.push((parser_id, error));
                }
                Err(e) => {
                    tracing::error!("Parser task panicked: {}", e);
                    errors.push((
                        "unknown".to_string(),
                        ParserError::ParseError(format!("Task panic: {}", e)),
                    ));
                }
            }
        }

        let total_time_ms = start.elapsed().as_millis() as u64;
        let parsers_count = self.parsers.len();
        let success_count = results.len();

        tracing::info!(
            "Ensemble completed in {}ms: {}/{} parsers succeeded",
            total_time_ms,
            success_count,
            parsers_count
        );

        EnsembleResult {
            results,
            errors,
            total_time_ms,
            parsers_count,
            success_count,
        }
    }

    /// Get number of parsers in ensemble
    pub fn parser_count(&self) -> usize {
        self.parsers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OllamaConfig;

    #[tokio::test]
    async fn test_ensemble_with_deterministic_only() {
        let config = ParserConfig {
            enable_deterministic: true,
            enable_ollama: false,
            enable_openai: false,
            ..Default::default()
        };

        let ensemble = ParserEnsemble::new(config);
        assert_eq!(ensemble.parser_count(), 1);

        let result = ensemble
            .parse_all(
                "Find experts in machine learning",
                "test_user",
                "test_session",
            )
            .await;

        assert_eq!(result.success_count, 1);
        assert!(result.get_deterministic().is_some());
    }

    #[tokio::test]
    async fn test_ensemble_result_methods() {
        let config = ParserConfig {
            enable_deterministic: true,
            enable_ollama: false,
            enable_openai: false,
            ..Default::default()
        };

        let ensemble = ParserEnsemble::new(config);
        let result = ensemble
            .parse_all("Summarize blockchain security", "test_user", "test_session")
            .await;

        assert!(result.get_highest_confidence().is_some());
        assert!(result.get_by_priority().is_some());
    }

    #[tokio::test]
    async fn test_empty_ensemble() {
        let ensemble = ParserEnsemble::with_parsers(Vec::new());
        assert_eq!(ensemble.parser_count(), 0);

        let result = ensemble.parse_all("test", "user", "session").await;
        assert_eq!(result.success_count, 0);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_ensemble_creation_from_config() {
        let config = ParserConfig {
            enable_deterministic: true,
            enable_ollama: true,
            enable_openai: true,
            ollama: OllamaConfig::default(),
            openai: crate::config::OpenAIConfig::new("test_key".to_string()),
        };

        let ensemble = ParserEnsemble::new(config);
        assert_eq!(ensemble.parser_count(), 3);
    }
}
