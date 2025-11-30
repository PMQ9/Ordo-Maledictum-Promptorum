use crate::cache_helper;
use crate::config::OpenAIConfig;
use crate::types::{IntentParser, ParserError, ParserResult};
use chrono::Utc;
use intent_schema::{Intent, IntentMetadata, ParsedIntent, cache::cache_keys};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    response_format: ResponseFormat,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

/// Structured JSON output from OpenAI
#[derive(Debug, Deserialize)]
struct OpenAIIntent {
    action: String,
    topic_id: Option<String>,
    expertise: Option<Vec<String>>,
    #[serde(default)]
    constraints: HashMap<String, serde_json::Value>,
    #[serde(default)]
    confidence: f32,
}

/// Parser that uses OpenAI API
/// - Calls OpenAI API (gpt-4o-mini by default)
/// - Uses JSON mode for structured output
/// - Temperature 0 for consistency
/// - Medium-high trust level (0.8)
pub struct OpenAIParser {
    config: OpenAIConfig,
    client: reqwest::Client,
    parser_id: String,
}

impl OpenAIParser {
    pub fn new(config: OpenAIConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            parser_id: "openai_v1".to_string(),
        }
    }

    /// Build the system prompt for intent extraction
    fn build_system_prompt(&self) -> String {
        r#"You are an intent extraction system. Parse user input and extract structured intent information.

Return ONLY a valid JSON object with this exact structure:
{
  "action": "find_experts|summarize|draft_proposal|research|query|other",
  "topic_id": "brief_snake_case_topic_id",
  "expertise": ["ml", "embedded", "security", "cloud", "blockchain"],
  "constraints": {
    "max_budget": 0,
    "max_results": 10
  },
  "confidence": 0.0-1.0
}

Rules:
- action must be a snake_case string like: find_experts, summarize, draft_proposal, research, query
- topic_id should be a brief snake_case identifier for the topic
- expertise should include areas like: ml, embedded, security, cloud, blockchain
- constraints is optional, include only if found in input
- confidence should reflect how certain you are about the parsing (0.0 to 1.0)
- Return ONLY the JSON, no other text"#
            .to_string()
    }

    /// Get system prompt with caching (24 hour TTL in Redis)
    async fn get_system_prompt_cached(&self) -> String {
        cache_helper::get_cached_system_prompt(
            cache_keys::PARSER_SYSTEM_PROMPT_KEY,
            cache_keys::PARSER_SYSTEM_PROMPT_TTL_SECS,
            || self.build_system_prompt(),
        )
        .await
    }

    /// Parse the OpenAI response into an Intent
    fn parse_openai_response(&self, content: &str) -> Result<OpenAIIntent, ParserError> {
        serde_json::from_str::<OpenAIIntent>(content)
            .map_err(|e| ParserError::ParseError(format!("Failed to parse OpenAI JSON: {}", e)))
    }
}

#[async_trait::async_trait]
impl IntentParser for OpenAIParser {
    async fn parse(
        &self,
        user_input: &str,
        user_id: &str,
        session_id: &str,
    ) -> ParserResult<ParsedIntent> {
        let start = Instant::now();

        if user_input.trim().is_empty() {
            return Err(ParserError::InvalidInput("Empty input".to_string()));
        }

        if self.config.api_key.is_empty() {
            return Err(ParserError::ConfigError(
                "OpenAI API key is not configured".to_string(),
            ));
        }

        // Build request with cached system prompt
        let system_prompt = self.get_system_prompt_cached().await;
        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_input.to_string(),
                },
            ],
            temperature: self.config.temperature,
            response_format: ResponseFormat {
                format_type: "json_object".to_string(),
            },
        };

        // Call OpenAI API
        let url = format!("{}/chat/completions", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("OpenAI API request failed: {}", e);
                ParserError::ApiError(format!("Failed to call OpenAI API: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("OpenAI API error: {} - {}", status, error_text);
            return Err(ParserError::ApiError(format!(
                "OpenAI API returned error: {} - {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        // Extract content from response
        let content = openai_response
            .choices
            .first()
            .ok_or_else(|| ParserError::ParseError("No choices in OpenAI response".to_string()))?
            .message
            .content
            .clone();

        // Parse the structured output
        let openai_intent = self.parse_openai_response(&content)?;

        // Create metadata
        let metadata = IntentMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        };

        // Convert to Intent
        let intent = Intent {
            action: openai_intent.action,
            topic_id: openai_intent
                .topic_id
                .unwrap_or_else(|| format!("topic_{}", Uuid::new_v4())),
            expertise: openai_intent.expertise.unwrap_or_default(),
            constraints: openai_intent.constraints,
            content_refs: Vec::new(),
            metadata,
        };

        let confidence = openai_intent.confidence.clamp(0.0, 1.0);

        tracing::info!(
            "OpenAI parser completed in {}ms with confidence {}",
            start.elapsed().as_millis(),
            confidence
        );

        Ok(ParsedIntent {
            parser_id: self.parser_id(),
            intent,
            confidence,
        })
    }

    fn parser_id(&self) -> String {
        self.parser_id.clone()
    }

    fn trust_level(&self) -> f64 {
        0.8 // Medium-high trust for OpenAI
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() {
        let config = OpenAIConfig::default();
        let parser = OpenAIParser::new(config);

        let prompt = parser.build_system_prompt();
        assert!(prompt.contains("intent extraction"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_openai_response_parsing() {
        let config = OpenAIConfig::default();
        let parser = OpenAIParser::new(config);

        let json_response = r#"{
            "action": "research",
            "topic_id": "quantum_computing",
            "expertise": ["ml", "quantum"],
            "constraints": {"max_results": 10},
            "confidence": 0.92
        }"#;

        let result = parser.parse_openai_response(json_response);
        assert!(result.is_ok());

        let intent = result.unwrap();
        assert_eq!(intent.action, "research");
        assert_eq!(intent.confidence, 0.92);
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        let config = OpenAIConfig::default(); // No API key
        let parser = OpenAIParser::new(config);

        let result = parser.parse("test input", "user", "session").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParserError::ConfigError(_)));
    }
}
