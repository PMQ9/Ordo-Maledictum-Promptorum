use crate::config::OllamaConfig;
use crate::types::{IntentParser, ParserError, ParserResult};
use chrono::Utc;
use intent_schema::{Intent, IntentMetadata, ParsedIntent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Ollama API request format
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
}

/// Ollama API response format
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    #[serde(default)]
    done: bool,
}

/// Structured JSON output from Ollama
#[derive(Debug, Deserialize)]
struct OllamaIntent {
    action: String,
    topic_id: Option<String>,
    expertise: Option<Vec<String>>,
    #[serde(default)]
    constraints: HashMap<String, serde_json::Value>,
    #[serde(default)]
    confidence: f32,
}

/// Parser that uses local Ollama API
/// - Calls localhost:11434 by default
/// - Uses JSON mode for structured output
/// - Temperature 0 for consistency
/// - Medium trust level (0.75)
pub struct OllamaParser {
    config: OllamaConfig,
    client: reqwest::Client,
    parser_id: String,
}

impl OllamaParser {
    pub fn new(config: OllamaConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            parser_id: "ollama_v1".to_string(),
        }
    }

    /// Build the system prompt for intent extraction
    fn build_prompt(&self, user_input: &str) -> String {
        format!(
            r#"You are an intent extraction system. Parse the user input and extract structured intent information.

User Input: "{}"

Extract and return ONLY a valid JSON object with this exact structure:
{{
  "action": "find_experts|summarize|draft_proposal|research|query|other",
  "topic_id": "brief_snake_case_topic_id",
  "expertise": ["ml", "embedded", "security", "cloud", "blockchain"],
  "constraints": {{
    "max_budget": 0,
    "max_results": 10
  }},
  "confidence": 0.0-1.0
}}

Rules:
- action must be a snake_case string like: find_experts, summarize, draft_proposal, research, query
- topic_id should be a brief snake_case identifier for the topic
- expertise should include areas like: ml, embedded, security, cloud, blockchain
- constraints is optional, include only if found in input
- confidence should reflect how certain you are about the parsing (0.0 to 1.0)
- Return ONLY the JSON, no other text

JSON output:"#,
            user_input
        )
    }

    /// Parse the Ollama response into an OllamaIntent
    fn parse_ollama_response(&self, response: &str) -> Result<OllamaIntent, ParserError> {
        // Try to extract JSON from the response
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        serde_json::from_str::<OllamaIntent>(json_str)
            .map_err(|e| ParserError::ParseError(format!("Failed to parse Ollama JSON: {}", e)))
    }
}

#[async_trait::async_trait]
impl IntentParser for OllamaParser {
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

        // Build request
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: self.build_prompt(user_input),
            stream: false,
            format: "json".to_string(),
            options: OllamaOptions {
                temperature: self.config.temperature,
            },
        };

        // Call Ollama API
        let url = format!("{}/api/generate", self.config.endpoint);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Ollama API request failed: {}", e);
                ParserError::ApiError(format!("Failed to call Ollama API: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Ollama API error: {} - {}", status, error_text);
            return Err(ParserError::ApiError(format!(
                "Ollama API returned error: {} - {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        // Parse the structured output
        let ollama_intent = self.parse_ollama_response(&ollama_response.response)?;

        // Create metadata
        let metadata = IntentMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        };

        // Convert to Intent
        let intent = Intent {
            action: ollama_intent.action,
            topic_id: ollama_intent
                .topic_id
                .unwrap_or_else(|| format!("topic_{}", Uuid::new_v4())),
            expertise: ollama_intent.expertise.unwrap_or_default(),
            constraints: ollama_intent.constraints,
            content_refs: Vec::new(),
            metadata,
        };

        let confidence = ollama_intent.confidence.clamp(0.0, 1.0);

        tracing::info!(
            "Ollama parser completed in {}ms with confidence {}",
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
        0.75 // Medium-high trust for local LLM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_building() {
        let config = OllamaConfig::default();
        let parser = OllamaParser::new(config);

        let prompt = parser.build_prompt("Find ML experts");
        assert!(prompt.contains("Find ML experts"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_ollama_response_parsing() {
        let config = OllamaConfig::default();
        let parser = OllamaParser::new(config);

        let json_response = r#"{
            "action": "find_experts",
            "topic_id": "ai_research",
            "expertise": ["ml", "security"],
            "constraints": {"max_budget": 50000},
            "confidence": 0.95
        }"#;

        let result = parser.parse_ollama_response(json_response);
        assert!(result.is_ok());

        let intent = result.unwrap();
        assert_eq!(intent.action, "find_experts");
        assert_eq!(intent.confidence, 0.95);
    }

    #[test]
    fn test_ollama_response_with_extra_text() {
        let config = OllamaConfig::default();
        let parser = OllamaParser::new(config);

        let response_with_extra = r#"Here is the JSON:
        {
            "action": "summarize",
            "topic_id": "blockchain",
            "expertise": ["blockchain"],
            "constraints": {},
            "confidence": 0.8
        }
        That's the result."#;

        let result = parser.parse_ollama_response(response_with_extra);
        assert!(result.is_ok());
    }
}
