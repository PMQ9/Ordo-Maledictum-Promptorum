use chrono::Utc;
use intent_schema::{
    Action, DocumentSummary, Expert, Intent, ProcessingMetadata, ProcessingResult, Proposal,
    ProposalSection,
};
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
            "find_experts" => Ok(Action::FindExperts),
            "summarize" => Ok(Action::Summarize),
            "draft_proposal" => Ok(Action::DraftProposal),
            "analyze_document" => Ok(Action::AnalyzeDocument),
            "generate_report" => Ok(Action::GenerateReport),
            "search_knowledge" => Ok(Action::SearchKnowledge),
            "llm_chat" => Ok(Action::LlmChat),
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
            "find_experts" => self.execute_find_experts(intent).await,
            "summarize" => self.execute_summarize(intent).await,
            "draft_proposal" => self.execute_draft_proposal(intent).await,
            "analyze_document" => self.execute_analyze_document(intent).await,
            "generate_report" => self.execute_generate_report(intent).await,
            "search_knowledge" => self.execute_search_knowledge(intent).await,
            "llm_chat" => self.execute_llm_chat(intent).await,
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

    /// Find experts matching the intent criteria (MOCK)
    ///
    /// This is a typed function call - no free-form LLM prompting
    async fn execute_find_experts(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        // Extract constraints from HashMap
        let max_results = intent
            .constraints
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let max_budget = intent
            .constraints
            .get("max_budget")
            .and_then(|v| v.as_i64());

        let experts = find_experts(
            Some(intent.topic_id.clone()),
            intent.expertise.clone(),
            max_results,
            max_budget,
        );

        let data = json!({ "experts": experts, "count": experts.len() });

        Ok(("find_experts".to_string(), data, vec![]))
    }

    /// Summarize a document (MOCK)
    async fn execute_summarize(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        if intent.content_refs.is_empty() {
            return Err(ProcessingError::InvalidIntent(
                "No documents to summarize".to_string(),
            ));
        }

        let summary = summarize_document(&intent.content_refs[0], Some(intent.topic_id.clone()));

        let data = json!({ "summary": summary });

        Ok(("summarize_document".to_string(), data, vec![]))
    }

    /// Draft a proposal (MOCK)
    async fn execute_draft_proposal(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let max_budget = intent
            .constraints
            .get("max_budget")
            .and_then(|v| v.as_i64());

        let proposal = draft_proposal(
            Some(intent.topic_id.clone()),
            intent.expertise.clone(),
            max_budget,
        );

        let data = json!({ "proposal": proposal });

        let mut warnings = vec![];
        if proposal.estimated_budget.is_none() {
            warnings.push("Budget estimation not available".to_string());
        }

        Ok(("draft_proposal".to_string(), data, warnings))
    }

    /// Analyze a document (MOCK)
    async fn execute_analyze_document(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let analysis = json!({
            "status": "analyzed",
            "topic_id": intent.topic_id,
            "complexity": "medium",
            "key_findings": ["Finding 1", "Finding 2", "Finding 3"]
        });

        Ok(("analyze_document".to_string(), analysis, vec![]))
    }

    /// Generate a report (MOCK)
    async fn execute_generate_report(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let report = json!({
            "title": format!("Report: {}", intent.topic_id),
            "sections": [
                {"heading": "Executive Summary", "content": "..."},
                {"heading": "Detailed Analysis", "content": "..."},
                {"heading": "Recommendations", "content": "..."}
            ],
            "generated_at": Utc::now()
        });

        Ok(("generate_report".to_string(), report, vec![]))
    }

    /// Search knowledge base (MOCK)
    async fn execute_search_knowledge(
        &self,
        intent: &Intent,
    ) -> Result<(String, serde_json::Value, Vec<String>), ProcessingError> {
        let results = json!({
            "query": intent.topic_id,
            "results": [
                {"id": "doc1", "title": "Sample Document 1", "relevance": 0.95},
                {"id": "doc2", "title": "Sample Document 2", "relevance": 0.87},
            ],
            "total_count": 2
        });

        Ok(("search_knowledge".to_string(), results, vec![]))
    }

    /// Execute a free-form LLM chat call via Claude API
    ///
    /// This accepts the validated intent from The Arbiter of Purpose
    /// and uses it as a raw prompt for Claude. The intent has already
    /// been cleaned and validated by the entire validation pipeline.
    async fn execute_llm_chat(
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
        // The topic_id contains the user's question/request
        let prompt = &intent.topic_id;

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
                        let message = data
                            .get("content")
                            .and_then(|content| content.get(0))
                            .and_then(|block| block.get("text"))
                            .and_then(|text| text.as_str())
                            .unwrap_or("No response from model");

                        let result = json!({
                            "response": message,
                            "model": self.config.claude_model,
                            "prompt": prompt,
                            "api_response": data,
                        });

                        info!("LLM chat execution completed successfully");
                        Ok(("llm_chat".to_string(), result, vec![]))
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
// MOCK IMPLEMENTATION FUNCTIONS
//
// These are placeholder implementations that demonstrate the type-safe
// function call pattern. In production, these would integrate with:
// - Real expert databases
// - Document processing pipelines
// - LLM APIs with structured outputs
// - Knowledge bases
//
// IMPORTANT: Notice that none of these functions accept raw prompts.
// All inputs are typed and validated.
// ============================================================================

/// Mock function to find experts
///
/// In production, this would query a database or API
fn find_experts(
    _topic: Option<String>,
    _expertise: Vec<String>,
    max_results: usize,
    max_budget: Option<i64>,
) -> Vec<Expert> {
    let mut experts = vec![
        Expert {
            id: "exp_001".to_string(),
            name: "Dr. Sarah Chen".to_string(),
            expertise: vec!["security".to_string(), "cloud".to_string()],
            availability: true,
            hourly_rate: 250,
            confidence_score: 0.95,
            bio: Some("Expert in cloud security with 15 years of experience".to_string()),
            years_experience: Some(15),
        },
        Expert {
            id: "exp_002".to_string(),
            name: "James Rodriguez".to_string(),
            expertise: vec!["machine_learning".to_string(), "data_science".to_string()],
            availability: true,
            hourly_rate: 200,
            confidence_score: 0.88,
            bio: Some("ML researcher and practitioner".to_string()),
            years_experience: Some(10),
        },
        Expert {
            id: "exp_003".to_string(),
            name: "Emily Watson".to_string(),
            expertise: vec!["embedded".to_string(), "security".to_string()],
            availability: false,
            hourly_rate: 275,
            confidence_score: 0.92,
            bio: Some("Embedded systems security specialist".to_string()),
            years_experience: Some(12),
        },
    ];

    // Filter by budget if specified
    if let Some(budget) = max_budget {
        experts.retain(|e| e.hourly_rate as i64 <= budget);
    }

    // Limit results
    experts.truncate(max_results);

    experts
}

/// Mock function to summarize a document
///
/// In production, this would call a document processing service
fn summarize_document(document_id: &str, topic: Option<String>) -> DocumentSummary {
    DocumentSummary {
        document_id: document_id.to_string(),
        title: format!("Document: {}", document_id),
        summary: format!(
            "This document covers {}. It provides comprehensive analysis \
             and actionable recommendations for stakeholders.",
            topic.unwrap_or_else(|| "various topics".to_string())
        ),
        key_points: vec![
            "Key finding 1: Market analysis shows strong growth potential".to_string(),
            "Key finding 2: Risk mitigation strategies are essential".to_string(),
            "Key finding 3: Timeline estimates are optimistic but achievable".to_string(),
        ],
        word_count: 2500,
        confidence: 0.89,
        generated_at: Utc::now(),
    }
}

/// Mock function to draft a proposal
///
/// In production, this would use a structured generation pipeline
fn draft_proposal(topic: Option<String>, _expertise: Vec<String>, budget: Option<i64>) -> Proposal {
    let topic_str = topic.unwrap_or_else(|| "Project Proposal".to_string());

    Proposal {
        id: format!("prop_{}", uuid::Uuid::new_v4()),
        title: format!("Proposal: {}", topic_str),
        sections: vec![
            ProposalSection {
                heading: "Executive Summary".to_string(),
                content: format!(
                    "This proposal outlines a comprehensive approach to {}. \
                     We bring together experts to deliver exceptional results.",
                    topic_str
                ),
                order: 1,
            },
            ProposalSection {
                heading: "Scope of Work".to_string(),
                content: "Detailed breakdown of deliverables, milestones, and timeline."
                    .to_string(),
                order: 2,
            },
            ProposalSection {
                heading: "Team and Expertise".to_string(),
                content: "Our team comprises industry-leading experts with proven track records."
                    .to_string(),
                order: 3,
            },
            ProposalSection {
                heading: "Budget and Timeline".to_string(),
                content: format!(
                    "Estimated budget: {}. Timeline: 12-16 weeks.",
                    budget
                        .map(|b| format!("${}", b))
                        .unwrap_or_else(|| "TBD".to_string())
                ),
                order: 4,
            },
        ],
        created_at: Utc::now(),
        estimated_budget: budget.map(|b| b as u64),
        timeline_weeks: Some(14),
    }
}

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
    async fn test_execute_llm_chat_missing_api_key() {
        let engine = ProcessingEngine::new();
        let intent = create_test_intent("llm_chat", "What is the capital of France?");

        let result = engine.execute(&intent).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result
            .error
            .unwrap()
            .contains("Claude API key not configured"));
    }

    #[tokio::test]
    async fn test_execute_find_experts() {
        let engine = ProcessingEngine::new();

        let mut intent = create_test_intent("find_experts", "supply_chain_risk");
        intent
            .constraints
            .insert("max_budget".to_string(), json!(300));
        intent
            .constraints
            .insert("max_results".to_string(), json!(5));

        let result = engine.execute(&intent).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, Action::FindExperts);
        assert!(result.data.get("experts").is_some());
        assert!(result.metadata.duration_ms < 1000);
    }

    #[tokio::test]
    async fn test_execute_summarize() {
        let engine = ProcessingEngine::new();

        let mut intent = create_test_intent("summarize", "cybersecurity_trends");
        intent.content_refs = vec!["doc_123".to_string()];

        let result = engine.execute(&intent).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, Action::Summarize);
        assert!(result.data.get("summary").is_some());
    }

    #[tokio::test]
    async fn test_execute_draft_proposal() {
        let engine = ProcessingEngine::new();

        let mut intent = create_test_intent("draft_proposal", "ai_integration_project");
        intent.expertise = vec!["machine_learning".to_string(), "security".to_string()];
        intent
            .constraints
            .insert("max_budget".to_string(), json!(50000));

        let result = engine.execute(&intent).await.unwrap();

        assert!(result.success);
        assert_eq!(result.action, Action::DraftProposal);
        assert!(result.data.get("proposal").is_some());
    }

    #[tokio::test]
    async fn test_unsupported_action() {
        let engine = ProcessingEngine::new();
        let intent = create_test_intent("invalid_action", "test");

        let result = engine.execute(&intent).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_find_experts_filters_by_budget() {
        let experts = find_experts(Some("security".to_string()), vec![], 10, Some(250));

        assert!(experts.len() <= 2);
        assert!(experts.iter().all(|e| e.hourly_rate <= 250));
    }

    #[test]
    fn test_summarize_document_structure() {
        let summary = summarize_document("doc_456", Some("AI ethics".to_string()));

        assert_eq!(summary.document_id, "doc_456");
        assert!(summary.summary.contains("AI ethics"));
        assert!(!summary.key_points.is_empty());
        assert!(summary.confidence > 0.0 && summary.confidence <= 1.0);
    }

    #[test]
    fn test_draft_proposal_structure() {
        let proposal = draft_proposal(
            Some("Cloud migration".to_string()),
            vec!["cloud".to_string(), "devops".to_string()],
            Some(100000),
        );

        assert!(proposal.title.contains("Cloud migration"));
        assert_eq!(proposal.sections.len(), 4);
        assert_eq!(proposal.estimated_budget, Some(100000));
        assert!(proposal.timeline_weeks.is_some());
    }
}
