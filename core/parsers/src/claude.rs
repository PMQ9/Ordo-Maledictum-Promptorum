use crate::cache_helper;
use crate::config::ClaudeConfig;
use crate::types::{IntentParser, ParserError, ParserResult};
use chrono::Utc;
use intent_schema::{Intent, IntentMetadata, ParsedIntent, cache::cache_keys};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Claude API request format
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// Claude API response format
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Structured JSON output from Claude
#[derive(Debug, Deserialize)]
struct ClaudeIntent {
    action: String,
    topic_id: Option<String>,
    expertise: Option<Vec<String>>,
    #[serde(default)]
    constraints: HashMap<String, serde_json::Value>,
    #[serde(default)]
    confidence: f32,
}

/// Parser that uses Claude API (Anthropic)
/// - Calls Claude API (claude-3-5-sonnet by default)
/// - Uses JSON-based structured output
/// - Temperature 0 for consistency
/// - High trust level (0.87)
pub struct ClaudeParser {
    config: ClaudeConfig,
    client: reqwest::Client,
    parser_id: String,
}

impl ClaudeParser {
    pub fn new(config: ClaudeConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            parser_id: "claude_v1".to_string(),
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

    /// Parse the Claude response into an Intent
    fn parse_claude_response(&self, content: &str) -> Result<ClaudeIntent, ParserError> {
        serde_json::from_str::<ClaudeIntent>(content)
            .map_err(|e| ParserError::ParseError(format!("Failed to parse Claude JSON: {}", e)))
    }
}

#[async_trait::async_trait]
impl IntentParser for ClaudeParser {
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
                "Claude API key is not configured".to_string(),
            ));
        }

        // Build request with cached system prompt
        let system_prompt = self.get_system_prompt_cached().await;
        let request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: 1024,
            system: system_prompt,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_input.to_string(),
            }],
        };

        // Call Claude API
        let url = format!("{}/messages", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Claude API request failed: {}", e);
                ParserError::ApiError(format!("Failed to call Claude API: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Claude API error: {} - {}", status, error_text);
            return Err(ParserError::ApiError(format!(
                "Claude API returned error: {} - {}",
                status, error_text
            )));
        }

        let claude_response: ClaudeResponse = response.json().await?;

        // Extract text content from response
        let content = claude_response
            .content
            .iter()
            .find(|c| c.content_type == "text")
            .and_then(|c| c.text.clone())
            .ok_or_else(|| {
                ParserError::ParseError("No text content in Claude response".to_string())
            })?;

        // Parse the structured output
        let claude_intent = self.parse_claude_response(&content)?;

        // Create metadata
        let metadata = IntentMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        };

        // Convert to Intent
        let intent = Intent {
            action: claude_intent.action,
            topic_id: claude_intent
                .topic_id
                .unwrap_or_else(|| format!("topic_{}", Uuid::new_v4())),
            expertise: claude_intent.expertise.unwrap_or_default(),
            constraints: claude_intent.constraints,
            content_refs: Vec::new(),
            metadata,
        };

        let confidence = claude_intent.confidence.clamp(0.0, 1.0);

        tracing::info!(
            "Claude parser completed in {}ms with confidence {}",
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
        0.87 // High trust for Claude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() {
        let config = ClaudeConfig::default();
        let parser = ClaudeParser::new(config);

        let prompt = parser.build_system_prompt();
        assert!(prompt.contains("intent extraction"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_claude_response_parsing() {
        let config = ClaudeConfig::default();
        let parser = ClaudeParser::new(config);

        let json_response = r#"{
            "action": "research",
            "topic_id": "quantum_computing",
            "expertise": ["ml", "quantum"],
            "constraints": {"max_results": 10},
            "confidence": 0.90
        }"#;

        let result = parser.parse_claude_response(json_response);
        assert!(result.is_ok());

        let intent = result.unwrap();
        assert_eq!(intent.action, "research");
        assert_eq!(intent.confidence, 0.90);
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        let config = ClaudeConfig::default(); // No API key
        let parser = ClaudeParser::new(config);

        let result = parser.parse("test input", "user", "session").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParserError::ConfigError(_)));
    }
}
