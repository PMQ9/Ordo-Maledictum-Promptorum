//! Common test utilities and fixtures for Intent Segregation tests
//!
//! This module provides shared test utilities, data generators, and helper functions
//! used across all test modules in the project.

use chrono::{DateTime, Utc};
use intent_schema::*;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Test data builder for creating Intent instances with common configurations
pub struct IntentBuilder {
    action: String,
    topic_id: String,
    expertise: Vec<String>,
    constraints: HashMap<String, serde_json::Value>,
    content_refs: Vec<String>,
    user_id: String,
    session_id: String,
}

impl IntentBuilder {
    /// Create a new IntentBuilder with sensible defaults
    pub fn new() -> Self {
        Self {
            action: "find_experts".to_string(),
            topic_id: "test_topic".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            user_id: "test_user".to_string(),
            session_id: "test_session".to_string(),
        }
    }

    /// Set the action
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    /// Set the topic ID
    pub fn topic_id(mut self, topic_id: impl Into<String>) -> Self {
        self.topic_id = topic_id.into();
        self
    }

    /// Add expertise area
    pub fn add_expertise(mut self, expertise: impl Into<String>) -> Self {
        self.expertise.push(expertise.into());
        self
    }

    /// Set expertise areas
    pub fn expertise(mut self, expertise: Vec<String>) -> Self {
        self.expertise = expertise;
        self
    }

    /// Add a constraint
    pub fn add_constraint(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.constraints.insert(key.into(), value);
        self
    }

    /// Set constraints
    pub fn constraints(mut self, constraints: HashMap<String, serde_json::Value>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Add budget constraint
    pub fn budget(mut self, budget: i64) -> Self {
        self.constraints.insert("max_budget".to_string(), json!(budget));
        self
    }

    /// Add content reference
    pub fn add_content_ref(mut self, content_ref: impl Into<String>) -> Self {
        self.content_refs.push(content_ref.into());
        self
    }

    /// Set content references
    pub fn content_refs(mut self, content_refs: Vec<String>) -> Self {
        self.content_refs = content_refs;
        self
    }

    /// Set user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = user_id.into();
        self
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
    }

    /// Build the Intent
    pub fn build(self) -> Intent {
        Intent::new(
            self.action,
            self.topic_id,
            self.expertise,
            self.constraints,
            self.content_refs,
            self.user_id,
            self.session_id,
        )
    }
}

impl Default for IntentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data builder for ParsedIntent
pub struct ParsedIntentBuilder {
    parser_id: String,
    intent: Intent,
    confidence: f32,
}

impl ParsedIntentBuilder {
    pub fn new() -> Self {
        Self {
            parser_id: "test_parser".to_string(),
            intent: IntentBuilder::new().build(),
            confidence: 0.9,
        }
    }

    pub fn parser_id(mut self, parser_id: impl Into<String>) -> Self {
        self.parser_id = parser_id.into();
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn build(self) -> ParsedIntent {
        ParsedIntent {
            parser_id: self.parser_id,
            intent: self.intent,
            confidence: self.confidence,
        }
    }
}

impl Default for ParsedIntentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data builder for ProviderConfig
pub struct ProviderConfigBuilder {
    allowed_actions: Vec<String>,
    allowed_expertise: Vec<String>,
    max_budget: Option<i64>,
    allowed_domains: Vec<String>,
}

impl ProviderConfigBuilder {
    pub fn new() -> Self {
        Self {
            allowed_actions: vec![
                "find_experts".to_string(),
                "summarize".to_string(),
                "draft_proposal".to_string(),
            ],
            allowed_expertise: vec![
                "security".to_string(),
                "ml".to_string(),
                "cloud".to_string(),
            ],
            max_budget: Some(50000),
            allowed_domains: vec![],
        }
    }

    pub fn allowed_actions(mut self, actions: Vec<String>) -> Self {
        self.allowed_actions = actions;
        self
    }

    pub fn add_allowed_action(mut self, action: impl Into<String>) -> Self {
        self.allowed_actions.push(action.into());
        self
    }

    pub fn allowed_expertise(mut self, expertise: Vec<String>) -> Self {
        self.allowed_expertise = expertise;
        self
    }

    pub fn add_allowed_expertise(mut self, expertise: impl Into<String>) -> Self {
        self.allowed_expertise.push(expertise.into());
        self
    }

    pub fn max_budget(mut self, budget: Option<i64>) -> Self {
        self.max_budget = budget;
        self
    }

    pub fn allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = domains;
        self
    }

    pub fn build(self) -> ProviderConfig {
        ProviderConfig {
            allowed_actions: self.allowed_actions,
            allowed_expertise: self.allowed_expertise,
            max_budget: self.max_budget,
            allowed_domains: self.allowed_domains,
        }
    }
}

impl Default for ProviderConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common test fixtures
pub mod fixtures {
    use super::*;

    /// Create a simple test intent for finding experts
    pub fn simple_find_experts_intent() -> Intent {
        IntentBuilder::new()
            .action("find_experts")
            .topic_id("supply_chain_risk")
            .add_expertise("security")
            .budget(20000)
            .build()
    }

    /// Create a test intent for summarization
    pub fn simple_summarize_intent() -> Intent {
        IntentBuilder::new()
            .action("summarize")
            .topic_id("cybersecurity_trends")
            .add_content_ref("doc_123")
            .build()
    }

    /// Create a test intent for drafting a proposal
    pub fn simple_draft_proposal_intent() -> Intent {
        IntentBuilder::new()
            .action("draft_proposal")
            .topic_id("ai_integration")
            .add_expertise("ml")
            .add_expertise("security")
            .budget(100000)
            .build()
    }

    /// Create a default provider config for testing
    pub fn default_provider_config() -> ProviderConfig {
        ProviderConfigBuilder::new().build()
    }

    /// Create a restrictive provider config
    pub fn restrictive_provider_config() -> ProviderConfig {
        ProviderConfigBuilder::new()
            .allowed_actions(vec!["find_experts".to_string()])
            .allowed_expertise(vec!["security".to_string()])
            .max_budget(Some(10000))
            .build()
    }

    /// Create a permissive provider config
    pub fn permissive_provider_config() -> ProviderConfig {
        ProviderConfigBuilder::new()
            .allowed_actions(vec![
                "find_experts".to_string(),
                "summarize".to_string(),
                "draft_proposal".to_string(),
                "analyze_document".to_string(),
                "generate_report".to_string(),
            ])
            .allowed_expertise(vec![]) // Empty means all allowed
            .max_budget(None)
            .build()
    }

    /// Create multiple parsed intents with high agreement
    pub fn high_agreement_parsed_intents() -> Vec<ParsedIntent> {
        let base_intent = simple_find_experts_intent();

        vec![
            ParsedIntentBuilder::new()
                .parser_id("deterministic")
                .intent(base_intent.clone())
                .confidence(1.0)
                .build(),
            ParsedIntentBuilder::new()
                .parser_id("llm1")
                .intent(base_intent.clone())
                .confidence(0.95)
                .build(),
            ParsedIntentBuilder::new()
                .parser_id("llm2")
                .intent(base_intent.clone())
                .confidence(0.92)
                .build(),
        ]
    }

    /// Create multiple parsed intents with conflict
    pub fn conflicting_parsed_intents() -> Vec<ParsedIntent> {
        vec![
            ParsedIntentBuilder::new()
                .parser_id("deterministic")
                .intent(
                    IntentBuilder::new()
                        .action("find_experts")
                        .topic_id("supply_chain_risk")
                        .add_expertise("security")
                        .build()
                )
                .confidence(1.0)
                .build(),
            ParsedIntentBuilder::new()
                .parser_id("llm1")
                .intent(
                    IntentBuilder::new()
                        .action("summarize")  // Different action!
                        .topic_id("cloud_security")
                        .add_expertise("cloud")
                        .build()
                )
                .confidence(0.85)
                .build(),
        ]
    }
}

/// Assertion helpers
pub mod assertions {
    use super::*;

    /// Assert that an intent has the expected action
    pub fn assert_intent_action(intent: &Intent, expected_action: &str) {
        assert_eq!(
            intent.action, expected_action,
            "Expected action '{}', got '{}'",
            expected_action, intent.action
        );
    }

    /// Assert that an intent has a specific expertise area
    pub fn assert_has_expertise(intent: &Intent, expertise: &str) {
        assert!(
            intent.has_expertise(expertise),
            "Expected intent to have expertise '{}'",
            expertise
        );
    }

    /// Assert that a voting result has high confidence
    pub fn assert_high_confidence(result: &VotingResult) {
        assert!(
            result.is_high_confidence(),
            "Expected high confidence voting result"
        );
    }

    /// Assert that a comparison result is approved
    pub fn assert_approved(result: &ComparisonResult) {
        assert!(
            result.is_approved(),
            "Expected approved comparison result, got: {:?}",
            result
        );
    }

    /// Assert that a comparison result is a hard mismatch
    pub fn assert_hard_mismatch(result: &ComparisonResult) {
        assert!(
            result.is_hard_mismatch(),
            "Expected hard mismatch comparison result, got: {:?}",
            result
        );
    }
}

/// Mock data generators for large-scale testing
pub mod generators {
    use super::*;
    use rand::Rng;

    /// Generate random test intents
    pub fn random_intent(seed: u64) -> Intent {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let actions = ["find_experts", "summarize", "draft_proposal", "analyze_document"];
        let topics = ["supply_chain", "cybersecurity", "ml_ops", "cloud_migration"];
        let expertise_options = ["security", "ml", "cloud", "embedded", "frontend"];

        let action = actions[rng.gen_range(0..actions.len())].to_string();
        let topic = topics[rng.gen_range(0..topics.len())].to_string();
        let expertise_count = rng.gen_range(0..3);
        let expertise: Vec<String> = (0..expertise_count)
            .map(|_| expertise_options[rng.gen_range(0..expertise_options.len())].to_string())
            .collect();

        let mut constraints = HashMap::new();
        if rng.gen_bool(0.5) {
            constraints.insert("max_budget".to_string(), json!(rng.gen_range(10000..100000)));
        }

        IntentBuilder::new()
            .action(action)
            .topic_id(topic)
            .expertise(expertise)
            .constraints(constraints)
            .build()
    }

    /// Generate a batch of random intents
    pub fn random_intents(count: usize, starting_seed: u64) -> Vec<Intent> {
        (0..count)
            .map(|i| random_intent(starting_seed + i as u64))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_builder() {
        let intent = IntentBuilder::new()
            .action("find_experts")
            .topic_id("test_topic")
            .add_expertise("security")
            .budget(50000)
            .build();

        assert_eq!(intent.action, "find_experts");
        assert_eq!(intent.topic_id, "test_topic");
        assert_eq!(intent.expertise.len(), 1);
        assert_eq!(intent.get_budget(), Some(50000));
    }

    #[test]
    fn test_parsed_intent_builder() {
        let parsed = ParsedIntentBuilder::new()
            .parser_id("test_parser")
            .confidence(0.95)
            .build();

        assert_eq!(parsed.parser_id, "test_parser");
        assert_eq!(parsed.confidence, 0.95);
    }

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfigBuilder::new()
            .add_allowed_action("custom_action")
            .max_budget(Some(100000))
            .build();

        assert!(config.is_action_allowed("find_experts"));
        assert!(config.is_action_allowed("custom_action"));
        assert!(config.is_budget_allowed(50000));
        assert!(!config.is_budget_allowed(200000));
    }

    #[test]
    fn test_fixtures() {
        let intent = fixtures::simple_find_experts_intent();
        assert_eq!(intent.action, "find_experts");

        let config = fixtures::default_provider_config();
        assert!(config.is_action_allowed("find_experts"));

        let parsed_intents = fixtures::high_agreement_parsed_intents();
        assert_eq!(parsed_intents.len(), 3);
    }

    #[test]
    fn test_random_generator() {
        let intent1 = generators::random_intent(42);
        let intent2 = generators::random_intent(42);

        // Same seed should generate same intent
        assert_eq!(intent1.action, intent2.action);
        assert_eq!(intent1.topic_id, intent2.topic_id);

        // Different seed should generate different intent (with high probability)
        let intent3 = generators::random_intent(43);
        assert!(intent1.action != intent3.action || intent1.topic_id != intent3.topic_id);
    }

    #[test]
    fn test_random_intents_batch() {
        let intents = generators::random_intents(10, 100);
        assert_eq!(intents.len(), 10);
    }
}
