use chrono::Utc;
use intent_schema::{Action, Intent, ProcessingMetadata, ProcessingResult};
use serde_json::json;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, warn};

/// Errors that can occur during processing
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Unsupported action: {0}")]
    UnsupportedAction(String),

    #[error("Invalid intent: {0}")]
    InvalidIntent(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// The main processing engine that executes trusted intents
///
/// This engine ensures that:
/// - All operations are type-safe and use typed function calls
/// - No free-form LLM calls can execute privileged actions
/// - All intents are validated before execution
/// - Results are structured and auditable
pub struct ProcessingEngine {
    /// Configuration for the engine
    #[allow(dead_code)]
    config: EngineConfig,
}

/// Configuration for the processing engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Enable verbose logging
    pub verbose: bool,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Claude API key for free-form LLM calls
    pub claude_api_key: Option<String>,
    /// Claude model to use (default: claude-3-5-sonnet-20241022)
    pub claude_model: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            max_execution_time_ms: 30_000, // 30 seconds
            claude_api_key: None,
            claude_model: "claude-3-haiku-20240307".to_string(), // Cheapest Claude model
        }
    }
}

impl ProcessingEngine {
    /// Create a new processing engine with default configuration
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Create a new processing engine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Convert string action to Action enum
    fn parse_action(action: &str) -> Result<Action, ProcessingError> {
        match action {
            "math_question" => Ok(Action::MathQuestion),
            _ => Err(ProcessingError::UnsupportedAction(action.to_string())),
        }
    }

    /// Execute a trusted intent and return a structured result
    ///
    /// This is the main entry point for processing. It dispatches to
    /// type-safe functions based on the intent action.
    ///
    /// # Security Guarantees
    /// - Only predefined actions can be executed
    /// - All parameters are validated
    /// - No raw prompts or unstructured commands are accepted
    /// - All operations are logged and traceable
    pub async fn execute(&self, intent: &Intent) -> Result<ProcessingResult, ProcessingError> {
        let start_time = Instant::now();
        let started_at = Utc::now();

        info!(
            "Executing intent: action={:?}, topic_id={:?}",
            intent.action, intent.topic_id
        );

        // Dispatch to the appropriate function based on the action
        let result = match intent.action.as_str() {
            "math_question" => self.execute_math_question(intent).await,
            _ => return Err(ProcessingError::UnsupportedAction(intent.action.clone())),
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let completed_at = Utc::now();

        let action_enum = Self::parse_action(&intent.action)?;

        match result {
            Ok((function_name, data, warnings)) => {
                info!("Intent executed successfully in {}ms", duration_ms);

                Ok(ProcessingResult::success(
                    action_enum,
                    data,
                    ProcessingMetadata {
                        started_at,
                        completed_at,
                        duration_ms,
                        function_called: function_name,
                        warnings,
                    },
                ))
            }
            Err(e) => {
                warn!("Intent execution failed: {}", e);

                Ok(ProcessingResult::failure(
                    action_enum,
                    e.to_string(),
                    ProcessingMetadata {
                        started_at,
                        completed_at,
                        duration_ms,
                        function_called: "unknown".to_string(),
                        warnings: vec![],
                    },
                ))
            }
        }
    }

    /// Execute a math question via Claude API
    ///
    /// This accepts the validated intent from The Arbiter of Purpose
    /// and uses it to answer a math question via Claude. The intent has already
    /// been cleaned and validated by the entire validation pipeline.
    async fn execute_math_question(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let api_key = match &self.config.claude_api_key {
            Some(key) => key,
            None => {
                return Err(ProcessingError::ProcessingFailed(
                    "Claude API key not configured".to_string(),
                ))
            }
        };

        // Build the prompt from the intent
        // The topic_id contains the user's math question
        let math_question = &intent.topic_id;
        let prompt = format!("Please solve this math problem: {}", math_question);

        let client = reqwest::Client::new();
        let endpoint = "https://api.anthropic.com/v1/messages";

        let payload = json!({
            "model": self.config.claude_model,
            "max_tokens": 2000,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
        });

        match client
            .post(endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        // Extract the assistant's response from the Claude API response
                        let answer = data
                            .get("content")
                            .and_then(|content| content.get(0))
                            .and_then(|block| block.get("text"))
                            .and_then(|text| text.as_str())
                            .unwrap_or("No response from model");

                        let result = json!({
                            "question": math_question,
                            "answer": answer,
                            "model": self.config.claude_model,
                            "api_response": data,
                        });

                        info!("Math question execution completed successfully");
                        Ok(("math_question".to_string(), result, vec![]))
                    }
                    Err(e) => Err(ProcessingError::ProcessingFailed(format!(
                        "Failed to parse Claude API response: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(ProcessingError::ProcessingFailed(format!(
                "Claude API call failed: {}",
                e
            ))),
        }
    }
}

impl Default for ProcessingEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// IMPLEMENTATION NOTES
//
// The execute_math_question function is the only action handler.
// It uses Claude API to answer math questions after full validation.
//
// IMPORTANT: The input has already been validated by the entire security
// pipeline before reaching this function.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use intent_schema::IntentMetadata;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_intent(action: &str, topic_id: &str) -> Intent {
        Intent {
            action: action.to_string(),
            topic_id: topic_id.to_string(),
            expertise: vec!["security".to_string()],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: IntentMetadata {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                user_id: "test_user".to_string(),
                session_id: "test_session".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_execute_math_question_missing_api_key() {
        let engine = ProcessingEngine::new();
        let intent = create_test_intent("math_question", "What is 2+2?");

        let result = engine.execute(&intent).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result
            .error
            .unwrap()
            .contains("Claude API key not configured"));
    }

    #[tokio::test]
    async fn test_unsupported_action() {
        let engine = ProcessingEngine::new();
        let intent = create_test_intent("invalid_action", "test");

        let result = engine.execute(&intent).await;
        assert!(result.is_err());
    }
}
