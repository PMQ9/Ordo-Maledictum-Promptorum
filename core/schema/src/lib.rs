//! Intent Schema Module
//!
//! This module defines the core data structures for the Intent Segregation
//! Cybersecurity Architecture. It provides type-safe representations of user
//! intents, parsing results, voting outcomes, and audit logs.
//!
//! # Overview
//!
//! The schema module is the foundation of the intent-first security architecture,
//! ensuring that all components work with strongly-typed, validated data structures
//! rather than free-form text or prompts.
//!
//! # Key Concepts
//!
//! - **Intent**: Structured representation of what a user wants the system to do
//! - **ParsedIntent**: Output from a single parser with confidence score
//! - **VotingResult**: Consensus from multiple parsers
//! - **ComparisonResult**: Validation against provider policies
//! - **LedgerEntry**: Immutable audit record of the entire processing pipeline

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

// Re-export commonly used types
pub use serde_json::Value;

/// Main Intent structure representing a user's parsed and validated intention.
///
/// This structure is the core of the system. It represents what the user wants
/// to accomplish in a structured, type-safe format that cannot be manipulated
/// through prompt injection.
///
/// # Fields
///
/// - `action`: The type of operation requested (e.g., "find_experts", "summarize")
/// - `topic_id`: Identifier for the subject matter or domain
/// - `expertise`: List of required expertise areas
/// - `constraints`: Key-value pairs for limits and parameters (e.g., max_budget)
/// - `content_refs`: References to sanitized user-provided documents
/// - `metadata`: System-generated metadata about this intent
///
/// # Example
///
/// ```rust
/// use intent_schema::*;
/// use std::collections::HashMap;
/// use chrono::Utc;
/// use uuid::Uuid;
///
/// let intent = Intent {
///     action: "find_experts".to_string(),
///     topic_id: "supply_chain_risk".to_string(),
///     expertise: vec!["security".to_string(), "ml".to_string()],
///     constraints: {
///         let mut map = HashMap::new();
///         map.insert("max_budget".to_string(), serde_json::json!(20000));
///         map
///     },
///     content_refs: vec!["doc_1321".to_string()],
///     metadata: IntentMetadata {
///         id: Uuid::new_v4(),
///         timestamp: Utc::now(),
///         user_id: "user_123".to_string(),
///         session_id: "session_456".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct Intent {
    /// The action to perform (e.g., "find_experts", "summarize", "draft_proposal")
    #[validate(length(min = 1, max = 100))]
    pub action: String,

    /// Identifier for the topic or domain
    #[validate(length(min = 1, max = 200))]
    pub topic_id: String,

    /// Required areas of expertise
    #[validate(length(min = 0, max = 50))]
    pub expertise: Vec<String>,

    /// Additional constraints and parameters (e.g., max_budget, deadline)
    pub constraints: HashMap<String, Value>,

    /// References to sanitized documents (not raw content)
    #[validate(length(min = 0, max = 100))]
    pub content_refs: Vec<String>,

    /// System-generated metadata
    #[validate(nested)]
    pub metadata: IntentMetadata,
}

/// Metadata associated with an Intent.
///
/// This structure contains system-generated information about when and where
/// an intent was created. All fields are automatically populated by the system.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct IntentMetadata {
    /// Unique identifier for this intent
    pub id: Uuid,

    /// When this intent was created
    pub timestamp: DateTime<Utc>,

    /// Identifier of the user who submitted the request
    #[validate(length(min = 1, max = 100))]
    pub user_id: String,

    /// Current session identifier
    #[validate(length(min = 1, max = 100))]
    pub session_id: String,
}

impl Intent {
    /// Calculate similarity score between two intents (0.0 = very different, 1.0 = identical)
    ///
    /// This method compares intents across multiple dimensions with weighted importance:
    /// - Action (weight 3.0): Most critical - must match for similar intents
    /// - Topic (weight 2.0): Important context indicator
    /// - Expertise (weight 2.0): Required skills/domains
    /// - Constraints (weight 1.5): Parameters and limits
    ///
    /// Returns a score from 0.0 to 1.0 where:
    /// - 1.0 = Identical intents
    /// - 0.95+ = Very similar (minor differences)
    /// - 0.75-0.95 = Moderately similar
    /// - <0.75 = Significantly different
    pub fn similarity(&self, other: &Intent) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Action comparison (weight: 3.0 - most important)
        let action_weight = 3.0;
        let action_sim = if self.action == other.action {
            1.0
        } else if self.action.to_lowercase() == other.action.to_lowercase() {
            0.95
        } else {
            0.0
        };
        score += action_sim * action_weight;
        weight_sum += action_weight;

        // Topic comparison (weight: 2.0)
        let topic_weight = 2.0;
        let topic_sim = if self.topic_id == other.topic_id {
            1.0
        } else if self.topic_id.to_lowercase() == other.topic_id.to_lowercase() {
            0.95
        } else {
            // Simple word overlap similarity
            let a_words: Vec<&str> = self
                .topic_id
                .split(&['_', '-', ' '][..])
                .filter(|s| !s.is_empty())
                .collect();
            let b_words: Vec<&str> = other
                .topic_id
                .split(&['_', '-', ' '][..])
                .filter(|s| !s.is_empty())
                .collect();
            let common = a_words.iter().filter(|w| b_words.contains(w)).count();
            if a_words.len() + b_words.len() > 0 {
                (2.0 * common as f64) / (a_words.len() + b_words.len()) as f64
            } else {
                0.0
            }
        };
        score += topic_sim * topic_weight;
        weight_sum += topic_weight;

        // Expertise comparison (weight: 2.0)
        let expertise_weight = 2.0;
        let expertise_sim = calculate_set_similarity(&self.expertise, &other.expertise);
        score += expertise_sim * expertise_weight;
        weight_sum += expertise_weight;

        // Constraints comparison (weight: 1.5)
        let constraints_weight = 1.5;
        let constraints_sim =
            calculate_constraint_similarity(&self.constraints, &other.constraints);
        score += constraints_sim * constraints_weight;
        weight_sum += constraints_weight;

        score / weight_sum
    }
}

/// Calculate similarity between two sets of strings (e.g., expertise areas)
fn calculate_set_similarity(set_a: &[String], set_b: &[String]) -> f64 {
    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }

    if set_a.is_empty() || set_b.is_empty() {
        return 0.0;
    }

    // Calculate Jaccard similarity
    let set_a_normalized: Vec<String> = set_a.iter().map(|s| s.to_lowercase()).collect();
    let set_b_normalized: Vec<String> = set_b.iter().map(|s| s.to_lowercase()).collect();

    let mut intersection = 0;
    for item_a in &set_a_normalized {
        if set_b_normalized.contains(item_a) {
            intersection += 1;
        }
    }

    let union = set_a.len() + set_b.len() - intersection;
    if union > 0 {
        intersection as f64 / union as f64
    } else {
        1.0
    }
}

/// Calculate similarity between two constraint HashMaps
fn calculate_constraint_similarity(
    constraints_a: &HashMap<String, Value>,
    constraints_b: &HashMap<String, Value>,
) -> f64 {
    if constraints_a.is_empty() && constraints_b.is_empty() {
        return 1.0;
    }

    if constraints_a.is_empty() || constraints_b.is_empty() {
        // Partially similar if one has constraints and other doesn't
        return 0.3;
    }

    let all_keys: std::collections::HashSet<_> =
        constraints_a.keys().chain(constraints_b.keys()).collect();

    let mut total_similarity = 0.0;
    for key in &all_keys {
        let sim = match (constraints_a.get(*key), constraints_b.get(*key)) {
            (Some(a), Some(b)) => {
                if a == b {
                    1.0
                } else if let (Some(a_num), Some(b_num)) = (a.as_f64(), b.as_f64()) {
                    // Numeric comparison with tolerance
                    let diff = (a_num - b_num).abs();
                    let max_val = a_num.abs().max(b_num.abs());
                    if max_val > 0.0 {
                        1.0 - (diff / max_val).min(1.0)
                    } else {
                        1.0
                    }
                } else {
                    0.0
                }
            }
            _ => 0.0, // Key exists in only one
        };
        total_similarity += sim;
    }

    total_similarity / all_keys.len() as f64
}

/// Result from a single parser in the parser ensemble.
///
/// Each parser independently analyzes user input and produces a structured
/// intent along with a confidence score. Multiple ParsedIntents are then
/// fed into the voting module.
///
/// # Fields
///
/// - `parser_id`: Identifier for the parser that produced this result
/// - `intent`: The structured intent extracted by this parser
/// - `confidence`: Float between 0.0 and 1.0 indicating parser's confidence
///
/// # Example
///
/// ```rust
/// use intent_schema::*;
///
/// let parsed = ParsedIntent {
///     parser_id: "llm_parser_1".to_string(),
///     intent: Intent {
///         // ... intent fields
/// #       action: "test".to_string(),
/// #       topic_id: "test".to_string(),
/// #       expertise: vec![],
/// #       constraints: std::collections::HashMap::new(),
/// #       content_refs: vec![],
/// #       metadata: IntentMetadata {
/// #           id: uuid::Uuid::new_v4(),
/// #           timestamp: chrono::Utc::now(),
/// #           user_id: "user".to_string(),
/// #           session_id: "session".to_string(),
/// #       },
///     },
///     confidence: 0.95,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ParsedIntent {
    /// Identifier of the parser that generated this result
    #[validate(length(min = 1, max = 100))]
    pub parser_id: String,

    /// The parsed intent
    #[validate(nested)]
    pub intent: Intent,

    /// Confidence score (0.0 to 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence: f32,
}

/// Agreement level between multiple parsers in the voting module.
///
/// Indicates how well the parsers agreed on the user's intent:
/// - `HighConfidence`: All parsers agree or only minor differences
/// - `LowConfidence`: Some disagreement but parsers mostly align
/// - `Conflict`: Significant disagreement requiring human intervention
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgreementLevel {
    /// High agreement between parsers (safe to proceed)
    HighConfidence,

    /// Low agreement between parsers (may require confirmation)
    LowConfidence,

    /// Conflicting interpretations (requires human approval)
    Conflict,
}

/// Result from the voting module after analyzing multiple parser outputs.
///
/// The voting module compares results from all parsers and produces a
/// canonical intent along with an agreement level.
///
/// # Fields
///
/// - `canonical_intent`: The consensus intent chosen by the voting algorithm
/// - `agreement_level`: How well the parsers agreed
/// - `parser_results`: All individual parser outputs for audit purposes
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct VotingResult {
    /// The canonical intent determined by voting
    #[validate(nested)]
    pub canonical_intent: Intent,

    /// Level of agreement between parsers
    pub agreement_level: AgreementLevel,

    /// All parser results that were considered
    #[validate(length(min = 1, max = 20))]
    pub parser_results: Vec<ParsedIntent>,
}

/// Result of comparing a parsed intent against provider configuration.
///
/// The comparator validates whether a user's intent is allowed under the
/// provider's policy configuration.
///
/// # Variants
///
/// - `Approved`: Intent matches policy, safe to execute
/// - `SoftMismatch`: Minor policy violation, may proceed with user confirmation
/// - `HardMismatch`: Serious policy violation, must be blocked or escalated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonResult {
    /// Intent is approved and matches provider policy
    Approved,

    /// Soft mismatch with policy (may proceed with confirmation)
    /// Contains human-readable reason for the mismatch
    SoftMismatch(String),

    /// Hard mismatch with policy (must be blocked or escalated)
    /// Contains human-readable reason for the rejection
    HardMismatch(String),
}

/// Provider configuration defining allowed intents and constraints.
///
/// This structure defines what actions, expertise areas, and parameters are
/// permitted by the service provider. It acts as a policy enforcement point.
///
/// # Fields
///
/// - `allowed_actions`: Whitelist of permitted action types
/// - `allowed_expertise`: Whitelist of permitted expertise areas
/// - `max_budget`: Optional maximum budget constraint
/// - `allowed_domains`: Whitelist of permitted topic domains
///
/// # Example
///
/// ```rust
/// use intent_schema::ProviderConfig;
///
/// let config = ProviderConfig {
///     allowed_actions: vec![
///         "find_experts".to_string(),
///         "summarize".to_string(),
///         "draft_proposal".to_string(),
///     ],
///     allowed_expertise: vec![
///         "ml".to_string(),
///         "embedded".to_string(),
///         "security".to_string(),
///     ],
///     max_budget: Some(50000),
///     allowed_domains: vec![
///         "supply_chain".to_string(),
///         "cybersecurity".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ProviderConfig {
    /// List of allowed action types
    #[validate(length(min = 1, max = 100))]
    pub allowed_actions: Vec<String>,

    /// List of allowed expertise areas
    #[validate(length(min = 0, max = 100))]
    pub allowed_expertise: Vec<String>,

    /// Maximum budget allowed (if applicable)
    #[validate(range(min = 0))]
    pub max_budget: Option<i64>,

    /// List of allowed topic domains
    #[validate(length(min = 0, max = 100))]
    pub allowed_domains: Vec<String>,
}

/// Record of human approval or rejection.
///
/// When an intent requires human intervention (due to policy mismatches or
/// parser conflicts), this structure records the human decision.
///
/// # Fields
///
/// - `approved`: Whether the human approved the intent
/// - `approver_id`: Identifier of the person who made the decision
/// - `timestamp`: When the decision was made
/// - `reason`: Human-readable explanation for the decision
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct HumanApproval {
    /// Whether the intent was approved
    pub approved: bool,

    /// Identifier of the approver
    #[validate(length(min = 1, max = 100))]
    pub approver_id: String,

    /// When the approval/rejection occurred
    pub timestamp: DateTime<Utc>,

    /// Explanation for the decision
    #[validate(length(min = 0, max = 1000))]
    pub reason: String,
}

/// Complete audit record for the intent processing pipeline.
///
/// This is an append-only ledger entry that records every step of processing
/// a user input through the entire system. Once written, entries should be
/// immutable.
///
/// # Purpose
///
/// - Audit trail for security and compliance
/// - Forensic analysis of system decisions
/// - Debugging and system improvement
/// - Explainability for users
///
/// # Fields
///
/// - `id`: Unique identifier for this ledger entry
/// - `timestamp`: When this entry was created
/// - `user_input`: The original raw user input (for reference only)
/// - `parsed_intents`: Results from all parsers
/// - `voting_result`: Consensus from the voting module
/// - `comparison_result`: Policy validation result
/// - `trusted_intent`: Final canonical intent (if approved)
/// - `processing_output`: Result of executing the intent (if applicable)
/// - `human_approval`: Human decision record (if required)
///
/// # Example
///
/// ```rust
/// use intent_schema::*;
/// use chrono::Utc;
/// use uuid::Uuid;
///
/// let entry = LedgerEntry {
///     id: Uuid::new_v4(),
///     timestamp: Utc::now(),
///     user_input: "Find me security experts for supply chain project".to_string(),
///     parsed_intents: vec![/* parser results */],
///     voting_result: VotingResult {
///         // ... voting result
/// #       canonical_intent: Intent {
/// #           action: "test".to_string(),
/// #           topic_id: "test".to_string(),
/// #           expertise: vec![],
/// #           constraints: std::collections::HashMap::new(),
/// #           content_refs: vec![],
/// #           metadata: IntentMetadata {
/// #               id: Uuid::new_v4(),
/// #               timestamp: Utc::now(),
/// #               user_id: "user".to_string(),
/// #               session_id: "session".to_string(),
/// #           },
/// #       },
/// #       agreement_level: AgreementLevel::HighConfidence,
/// #       parser_results: vec![],
///     },
///     comparison_result: ComparisonResult::Approved,
///     trusted_intent: Some(Intent {
///         // ... final intent
/// #       action: "test".to_string(),
/// #       topic_id: "test".to_string(),
/// #       expertise: vec![],
/// #       constraints: std::collections::HashMap::new(),
/// #       content_refs: vec![],
/// #       metadata: IntentMetadata {
/// #           id: Uuid::new_v4(),
/// #           timestamp: Utc::now(),
/// #           user_id: "user".to_string(),
/// #           session_id: "session".to_string(),
/// #       },
///     }),
///     processing_output: None,
///     human_approval: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct LedgerEntry {
    /// Unique identifier for this ledger entry
    pub id: Uuid,

    /// When this entry was created
    pub timestamp: DateTime<Utc>,

    /// Original raw user input (preserved for audit)
    #[validate(length(min = 0, max = 10000))]
    pub user_input: String,

    /// Results from all parsers in the ensemble
    #[validate(nested)]
    pub parsed_intents: Vec<ParsedIntent>,

    /// Result from the voting module
    #[validate(nested)]
    pub voting_result: VotingResult,

    /// Result from comparing against provider config
    pub comparison_result: ComparisonResult,

    /// Final trusted intent (None if blocked/rejected)
    pub trusted_intent: Option<Intent>,

    /// Output from processing engine (if executed)
    pub processing_output: Option<Value>,

    /// Human approval record (if required)
    pub human_approval: Option<HumanApproval>,
}

// ============================================================================
// Helper Methods and Implementations
// ============================================================================

impl Intent {
    /// Creates a new Intent with automatically generated metadata.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to perform
    /// * `topic_id` - The topic identifier
    /// * `expertise` - Required expertise areas
    /// * `constraints` - Additional parameters and limits
    /// * `content_refs` - References to sanitized documents
    /// * `user_id` - User identifier
    /// * `session_id` - Session identifier
    pub fn new(
        action: String,
        topic_id: String,
        expertise: Vec<String>,
        constraints: HashMap<String, Value>,
        content_refs: Vec<String>,
        user_id: String,
        session_id: String,
    ) -> Self {
        Self {
            action,
            topic_id,
            expertise,
            constraints,
            content_refs,
            metadata: IntentMetadata {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                user_id,
                session_id,
            },
        }
    }

    /// Checks if this intent contains a specific expertise area.
    pub fn has_expertise(&self, expertise: &str) -> bool {
        self.expertise.iter().any(|e| e == expertise)
    }

    /// Gets a constraint value by key.
    pub fn get_constraint(&self, key: &str) -> Option<&Value> {
        self.constraints.get(key)
    }

    /// Gets the budget constraint if present.
    pub fn get_budget(&self) -> Option<i64> {
        self.get_constraint("max_budget").and_then(|v| v.as_i64())
    }
}

impl VotingResult {
    /// Returns true if the voting result indicates high confidence.
    pub fn is_high_confidence(&self) -> bool {
        matches!(self.agreement_level, AgreementLevel::HighConfidence)
    }

    /// Returns true if there was a conflict between parsers.
    pub fn has_conflict(&self) -> bool {
        matches!(self.agreement_level, AgreementLevel::Conflict)
    }

    /// Gets the average confidence across all parsers.
    pub fn average_confidence(&self) -> f32 {
        if self.parser_results.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.parser_results.iter().map(|p| p.confidence).sum();
        sum / self.parser_results.len() as f32
    }
}

impl ComparisonResult {
    /// Returns true if the intent was approved.
    pub fn is_approved(&self) -> bool {
        matches!(self, ComparisonResult::Approved)
    }

    /// Returns true if there was any kind of mismatch.
    pub fn has_mismatch(&self) -> bool {
        !self.is_approved()
    }

    /// Returns true if there was a hard mismatch.
    pub fn is_hard_mismatch(&self) -> bool {
        matches!(self, ComparisonResult::HardMismatch(_))
    }

    /// Gets the mismatch reason if there is one.
    pub fn get_reason(&self) -> Option<&str> {
        match self {
            ComparisonResult::Approved => None,
            ComparisonResult::SoftMismatch(reason) => Some(reason),
            ComparisonResult::HardMismatch(reason) => Some(reason),
        }
    }
}

impl ProviderConfig {
    /// Checks if an action is allowed by this configuration.
    pub fn is_action_allowed(&self, action: &str) -> bool {
        self.allowed_actions.iter().any(|a| a == action)
    }

    /// Checks if an expertise area is allowed by this configuration.
    pub fn is_expertise_allowed(&self, expertise: &str) -> bool {
        self.allowed_expertise.is_empty() || self.allowed_expertise.iter().any(|e| e == expertise)
    }

    /// Checks if all expertise areas in the list are allowed.
    pub fn are_expertise_allowed(&self, expertise_list: &[String]) -> bool {
        expertise_list.iter().all(|e| self.is_expertise_allowed(e))
    }

    /// Checks if a budget is within the allowed maximum.
    pub fn is_budget_allowed(&self, budget: i64) -> bool {
        self.max_budget.map_or(true, |max| budget <= max)
    }

    /// Checks if a domain is allowed by this configuration.
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        self.allowed_domains.is_empty() || self.allowed_domains.iter().any(|d| d == domain)
    }
}

impl LedgerEntry {
    /// Creates a new ledger entry.
    pub fn new(
        user_input: String,
        parsed_intents: Vec<ParsedIntent>,
        voting_result: VotingResult,
        comparison_result: ComparisonResult,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_input,
            parsed_intents,
            voting_result,
            comparison_result,
            trusted_intent: None,
            processing_output: None,
            human_approval: None,
        }
    }

    /// Returns true if this entry required human approval.
    pub fn required_human_approval(&self) -> bool {
        self.human_approval.is_some()
    }

    /// Returns true if the intent was ultimately approved (either automatically or by human).
    pub fn was_approved(&self) -> bool {
        match &self.human_approval {
            Some(approval) => approval.approved,
            None => self.comparison_result.is_approved(),
        }
    }

    /// Returns true if the intent was executed.
    pub fn was_executed(&self) -> bool {
        self.processing_output.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_creation() {
        let intent = Intent::new(
            "find_experts".to_string(),
            "supply_chain".to_string(),
            vec!["security".to_string()],
            HashMap::new(),
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );

        assert_eq!(intent.action, "find_experts");
        assert_eq!(intent.topic_id, "supply_chain");
        assert!(intent.has_expertise("security"));
        assert!(!intent.has_expertise("ml"));
    }

    #[test]
    fn test_intent_budget() {
        let mut constraints = HashMap::new();
        constraints.insert("max_budget".to_string(), serde_json::json!(20000));

        let intent = Intent::new(
            "find_experts".to_string(),
            "supply_chain".to_string(),
            vec![],
            constraints,
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );

        assert_eq!(intent.get_budget(), Some(20000));
    }

    #[test]
    fn test_voting_result_confidence() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let parsed_intents = vec![
            ParsedIntent {
                parser_id: "p1".to_string(),
                intent: intent.clone(),
                confidence: 0.9,
            },
            ParsedIntent {
                parser_id: "p2".to_string(),
                intent: intent.clone(),
                confidence: 0.8,
            },
        ];

        let voting_result = VotingResult {
            canonical_intent: intent,
            agreement_level: AgreementLevel::HighConfidence,
            parser_results: parsed_intents,
        };

        assert!(voting_result.is_high_confidence());
        assert_eq!(voting_result.average_confidence(), 0.85);
    }

    #[test]
    fn test_comparison_result() {
        let approved = ComparisonResult::Approved;
        assert!(approved.is_approved());
        assert!(!approved.has_mismatch());
        assert!(approved.get_reason().is_none());

        let soft = ComparisonResult::SoftMismatch("Budget too high".to_string());
        assert!(!soft.is_approved());
        assert!(soft.has_mismatch());
        assert!(!soft.is_hard_mismatch());
        assert_eq!(soft.get_reason(), Some("Budget too high"));

        let hard = ComparisonResult::HardMismatch("Forbidden action".to_string());
        assert!(hard.is_hard_mismatch());
        assert_eq!(hard.get_reason(), Some("Forbidden action"));
    }

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig {
            allowed_actions: vec!["find_experts".to_string(), "summarize".to_string()],
            allowed_expertise: vec!["security".to_string(), "ml".to_string()],
            max_budget: Some(50000),
            allowed_domains: vec!["supply_chain".to_string()],
        };

        assert!(config.is_action_allowed("find_experts"));
        assert!(!config.is_action_allowed("delete_all"));

        assert!(config.is_expertise_allowed("security"));
        assert!(!config.is_expertise_allowed("quantum"));

        assert!(config.is_budget_allowed(30000));
        assert!(!config.is_budget_allowed(60000));

        assert!(config.is_domain_allowed("supply_chain"));
        assert!(!config.is_domain_allowed("finance"));
    }

    #[test]
    fn test_ledger_entry() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let voting_result = VotingResult {
            canonical_intent: intent.clone(),
            agreement_level: AgreementLevel::HighConfidence,
            parser_results: vec![],
        };

        let entry = LedgerEntry::new(
            "test input".to_string(),
            vec![],
            voting_result,
            ComparisonResult::Approved,
        );

        assert!(!entry.required_human_approval());
        assert!(entry.was_approved());
        assert!(!entry.was_executed());
    }

    #[test]
    fn test_serialization() {
        let intent = Intent::new(
            "find_experts".to_string(),
            "supply_chain".to_string(),
            vec!["security".to_string()],
            HashMap::new(),
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );

        let json = serde_json::to_string(&intent).unwrap();
        let deserialized: Intent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, deserialized);
    }

    // ========================================================================
    // COMPREHENSIVE TESTS FOR INTENT SIMILARITY
    // ========================================================================

    #[test]
    fn test_intent_similarity_identical() {
        let intent1 = Intent::new(
            "find_experts".to_string(),
            "supply_chain_risk".to_string(),
            vec!["security".to_string(), "ml".to_string()],
            HashMap::new(),
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );
        let intent2 = intent1.clone();

        let similarity = intent1.similarity(&intent2);
        assert_eq!(
            similarity, 1.0,
            "Identical intents should have similarity 1.0"
        );
    }

    #[test]
    fn test_intent_similarity_different_action() {
        let intent1 = Intent::new(
            "find_experts".to_string(),
            "supply_chain_risk".to_string(),
            vec!["security".to_string()],
            HashMap::new(),
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );
        let intent2 = Intent::new(
            "summarize".to_string(),
            "supply_chain_risk".to_string(),
            vec!["security".to_string()],
            HashMap::new(),
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );

        let similarity = intent1.similarity(&intent2);
        assert!(
            similarity < 0.75,
            "Different actions should have low similarity"
        );
    }

    #[test]
    fn test_intent_similarity_case_insensitive_action() {
        let intent1 = Intent::new(
            "find_experts".to_string(),
            "topic".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );
        let intent2 = Intent::new(
            "Find_Experts".to_string(),
            "topic".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let similarity = intent1.similarity(&intent2);
        assert!(
            similarity >= 0.95,
            "Case-insensitive actions should have high similarity"
        );
    }

    #[test]
    fn test_intent_similarity_different_expertise() {
        let intent1 = Intent::new(
            "find_experts".to_string(),
            "topic".to_string(),
            vec!["security".to_string(), "ml".to_string()],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );
        let intent2 = Intent::new(
            "find_experts".to_string(),
            "topic".to_string(),
            vec!["cloud".to_string(), "devops".to_string()],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let similarity = intent1.similarity(&intent2);
        // Should have lower similarity due to completely different expertise
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_intent_similarity_with_constraints() {
        let mut constraints1 = HashMap::new();
        constraints1.insert("max_budget".to_string(), serde_json::json!(50000));

        let mut constraints2 = HashMap::new();
        constraints2.insert("max_budget".to_string(), serde_json::json!(55000));

        let intent1 = Intent::new(
            "find_experts".to_string(),
            "topic".to_string(),
            vec![],
            constraints1,
            vec![],
            "user".to_string(),
            "session".to_string(),
        );
        let intent2 = Intent::new(
            "find_experts".to_string(),
            "topic".to_string(),
            vec![],
            constraints2,
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let similarity = intent1.similarity(&intent2);
        // Should be high but not perfect due to slight budget difference
        assert!(similarity > 0.9 && similarity < 1.0);
    }

    // ========================================================================
    // COMPREHENSIVE TESTS FOR HELPER FUNCTIONS
    // ========================================================================

    #[test]
    fn test_calculate_set_similarity_empty_sets() {
        let set_a: Vec<String> = vec![];
        let set_b: Vec<String> = vec![];
        let similarity = calculate_set_similarity(&set_a, &set_b);
        assert_eq!(
            similarity, 1.0,
            "Empty sets should be considered fully similar"
        );
    }

    #[test]
    fn test_calculate_set_similarity_one_empty() {
        let set_a = vec!["security".to_string()];
        let set_b: Vec<String> = vec![];
        let similarity = calculate_set_similarity(&set_a, &set_b);
        assert_eq!(
            similarity, 0.0,
            "One empty set should result in zero similarity"
        );
    }

    #[test]
    fn test_calculate_set_similarity_full_overlap() {
        let set_a = vec!["security".to_string(), "ml".to_string()];
        let set_b = vec!["security".to_string(), "ml".to_string()];
        let similarity = calculate_set_similarity(&set_a, &set_b);
        assert_eq!(
            similarity, 1.0,
            "Identical sets should have full similarity"
        );
    }

    #[test]
    fn test_calculate_set_similarity_partial_overlap() {
        let set_a = vec!["security".to_string(), "ml".to_string()];
        let set_b = vec!["security".to_string(), "cloud".to_string()];
        let similarity = calculate_set_similarity(&set_a, &set_b);
        // Jaccard similarity: intersection=1, union=3, so 1/3 ≈ 0.333
        assert!((similarity - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_calculate_set_similarity_case_insensitive() {
        let set_a = vec!["Security".to_string(), "ML".to_string()];
        let set_b = vec!["security".to_string(), "ml".to_string()];
        let similarity = calculate_set_similarity(&set_a, &set_b);
        assert_eq!(similarity, 1.0, "Should be case-insensitive");
    }

    #[test]
    fn test_calculate_constraint_similarity_both_empty() {
        let constraints_a = HashMap::new();
        let constraints_b = HashMap::new();
        let similarity = calculate_constraint_similarity(&constraints_a, &constraints_b);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_calculate_constraint_similarity_one_empty() {
        let mut constraints_a = HashMap::new();
        constraints_a.insert("max_budget".to_string(), serde_json::json!(50000));
        let constraints_b = HashMap::new();
        let similarity = calculate_constraint_similarity(&constraints_a, &constraints_b);
        assert_eq!(
            similarity, 0.3,
            "One empty constraint should give partial similarity"
        );
    }

    #[test]
    fn test_calculate_constraint_similarity_same_values() {
        let mut constraints_a = HashMap::new();
        constraints_a.insert("max_budget".to_string(), serde_json::json!(50000));
        constraints_a.insert("deadline".to_string(), serde_json::json!("2024-12-31"));

        let mut constraints_b = HashMap::new();
        constraints_b.insert("max_budget".to_string(), serde_json::json!(50000));
        constraints_b.insert("deadline".to_string(), serde_json::json!("2024-12-31"));

        let similarity = calculate_constraint_similarity(&constraints_a, &constraints_b);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_calculate_constraint_similarity_numeric_tolerance() {
        let mut constraints_a = HashMap::new();
        constraints_a.insert("max_budget".to_string(), serde_json::json!(50000));

        let mut constraints_b = HashMap::new();
        constraints_b.insert("max_budget".to_string(), serde_json::json!(51000));

        let similarity = calculate_constraint_similarity(&constraints_a, &constraints_b);
        // Should be close to 1.0 but not exactly 1.0
        assert!(similarity > 0.9 && similarity < 1.0);
    }

    // ========================================================================
    // COMPREHENSIVE TESTS FOR VOTING RESULT
    // ========================================================================

    #[test]
    fn test_voting_result_average_confidence() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let parsed_intents = vec![
            ParsedIntent {
                parser_id: "p1".to_string(),
                intent: intent.clone(),
                confidence: 1.0,
            },
            ParsedIntent {
                parser_id: "p2".to_string(),
                intent: intent.clone(),
                confidence: 0.8,
            },
            ParsedIntent {
                parser_id: "p3".to_string(),
                intent: intent.clone(),
                confidence: 0.9,
            },
        ];

        let voting_result = VotingResult {
            canonical_intent: intent,
            agreement_level: AgreementLevel::HighConfidence,
            parser_results: parsed_intents,
        };

        let avg = voting_result.average_confidence();
        assert!((avg - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_voting_result_empty_parsers() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let voting_result = VotingResult {
            canonical_intent: intent,
            agreement_level: AgreementLevel::Conflict,
            parser_results: vec![],
        };

        assert_eq!(voting_result.average_confidence(), 0.0);
    }

    // ========================================================================
    // COMPREHENSIVE TESTS FOR LEDGER ENTRY
    // ========================================================================

    #[test]
    fn test_ledger_entry_creation() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let voting_result = VotingResult {
            canonical_intent: intent.clone(),
            agreement_level: AgreementLevel::HighConfidence,
            parser_results: vec![],
        };

        let entry = LedgerEntry::new(
            "test input".to_string(),
            vec![],
            voting_result,
            ComparisonResult::Approved,
        );

        assert!(!entry.user_input.is_empty());
        assert!(!entry.was_executed());
        assert!(entry.was_approved());
    }

    #[test]
    fn test_ledger_entry_with_human_approval() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let voting_result = VotingResult {
            canonical_intent: intent.clone(),
            agreement_level: AgreementLevel::Conflict,
            parser_results: vec![],
        };

        let mut entry = LedgerEntry::new(
            "test input".to_string(),
            vec![],
            voting_result,
            ComparisonResult::HardMismatch("Policy violation".to_string()),
        );

        assert!(!entry.was_approved());
        assert!(!entry.required_human_approval());

        // Add human approval
        entry.human_approval = Some(HumanApproval {
            approved: true,
            approver_id: "admin_123".to_string(),
            timestamp: chrono::Utc::now(),
            reason: "Approved after review".to_string(),
        });

        assert!(entry.required_human_approval());
        assert!(entry.was_approved());
    }

    #[test]
    fn test_ledger_entry_with_denial() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let voting_result = VotingResult {
            canonical_intent: intent.clone(),
            agreement_level: AgreementLevel::HighConfidence,
            parser_results: vec![],
        };

        let mut entry = LedgerEntry::new(
            "test input".to_string(),
            vec![],
            voting_result,
            ComparisonResult::HardMismatch("Critical violation".to_string()),
        );

        entry.human_approval = Some(HumanApproval {
            approved: false,
            approver_id: "admin_456".to_string(),
            timestamp: chrono::Utc::now(),
            reason: "Denied due to security concerns".to_string(),
        });

        assert!(!entry.was_approved());
        assert!(entry.required_human_approval());
    }

    #[test]
    fn test_ledger_entry_with_execution() {
        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        let voting_result = VotingResult {
            canonical_intent: intent.clone(),
            agreement_level: AgreementLevel::HighConfidence,
            parser_results: vec![],
        };

        let mut entry = LedgerEntry::new(
            "test input".to_string(),
            vec![],
            voting_result,
            ComparisonResult::Approved,
        );

        assert!(!entry.was_executed());

        entry.processing_output = Some(serde_json::json!({"result": "success"}));

        assert!(entry.was_executed());
    }

    // ========================================================================
    // VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_intent_validation() {
        use validator::Validate;

        let intent = Intent::new(
            "find_experts".to_string(),
            "topic".to_string(),
            vec!["security".to_string()],
            HashMap::new(),
            vec![],
            "user_123".to_string(),
            "session_456".to_string(),
        );

        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_intent_validation_empty_action() {
        use validator::Validate;

        let intent = Intent {
            action: "".to_string(), // Empty action should fail validation
            topic_id: "topic".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: IntentMetadata {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                user_id: "user".to_string(),
                session_id: "session".to_string(),
            },
        };

        assert!(intent.validate().is_err());
    }

    #[test]
    fn test_parsed_intent_validation_confidence_range() {
        use validator::Validate;

        let intent = Intent::new(
            "test".to_string(),
            "test".to_string(),
            vec![],
            HashMap::new(),
            vec![],
            "user".to_string(),
            "session".to_string(),
        );

        // Valid confidence
        let valid = ParsedIntent {
            parser_id: "parser".to_string(),
            intent: intent.clone(),
            confidence: 0.95,
        };
        assert!(valid.validate().is_ok());

        // Invalid confidence (too high)
        let invalid = ParsedIntent {
            parser_id: "parser".to_string(),
            intent: intent.clone(),
            confidence: 1.5,
        };
        assert!(invalid.validate().is_err());

        // Invalid confidence (negative)
        let invalid2 = ParsedIntent {
            parser_id: "parser".to_string(),
            intent,
            confidence: -0.1,
        };
        assert!(invalid2.validate().is_err());
    }
}

// Additional types for the Intent Generator module
pub mod generator_types;
pub use generator_types::*;
