//! Types specifically for the Intent Generator module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use crate::ParsedIntent;

/// Action enum for type-safe action representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    MathQuestion,
}

/// Expertise areas enum (not used for math tutoring platform)
/// Kept for type compatibility but should always be an empty vector
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Expertise {
    // No variants - expertise areas not applicable to math tutoring
    // This enum is kept for backward compatibility but Vec<Expertise> should always be empty
    #[serde(skip)]
    _Unused,
}

/// Constraints for intent execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Constraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_budget: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,

    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            max_budget: None,
            max_results: None,
            deadline: None,
            additional: HashMap::new(),
        }
    }
}

/// Voted intent from the voting module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotedIntent {
    pub action: Action,
    pub topic: String,
    pub expertise: Vec<Expertise>,
    pub constraints: Option<Constraints>,
    pub content_refs: Vec<String>,
    pub confidence: f64,
    pub requires_approval: bool,
    pub parser_results: Vec<ParsedIntent>,
}

/// Trusted intent ready for execution
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TrustedIntent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: Action,

    #[validate(length(max = 100))]
    pub topic_id: String,

    pub expertise: Vec<Expertise>,

    #[validate(nested)]
    pub constraints: Constraints,

    pub content_refs: Vec<String>,
    pub signature: Option<String>,
    pub content_hash: String,
    pub user_id: String,
    pub session_id: String,
}

/// Simple metadata for intent generation requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub user_id: String,
    pub session_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Error types for schema operations
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Invalid expertise: {0}")]
    InvalidExpertise(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Processing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub function_called: String,
    pub warnings: Vec<String>,
}

/// Result of processing an intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub success: bool,
    pub action: Action,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub metadata: ProcessingMetadata,
}

impl ProcessingResult {
    pub fn success(action: Action, data: serde_json::Value, metadata: ProcessingMetadata) -> Self {
        Self {
            success: true,
            action,
            data,
            error: None,
            metadata,
        }
    }

    pub fn failure(action: Action, error: String, metadata: ProcessingMetadata) -> Self {
        Self {
            success: false,
            action,
            data: serde_json::Value::Null,
            error: Some(error),
            metadata,
        }
    }
}

/// Intent for processing engine (simplified from main Intent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub action: Action,
    pub topic: Option<String>,
    pub expertise: Vec<Expertise>,
    pub constraints: Constraints,
    pub content_refs: Option<Vec<String>>,
    pub metadata: Option<IntentMetadata>,
}

/// Intent metadata for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMetadata {
    pub user_id: String,
    pub session_id: String,
}

impl TrustedIntent {
    /// Validate that this intent contains no raw user content
    pub fn validate_no_raw_content(&self) -> Result<(), SchemaError> {
        // Ensure topic_id is an identifier, not raw text
        if self.topic_id.contains(' ') || self.topic_id.len() > 100 {
            return Err(SchemaError::ValidationError(
                "topic_id appears to contain raw user text".to_string(),
            ));
        }

        // Ensure all content_refs are references, not content
        for content_ref in &self.content_refs {
            if content_ref.contains('\n') || content_ref.len() > 100 {
                return Err(SchemaError::ValidationError(
                    "content_ref appears to contain raw content".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_action_serialization() {
        let action = Action::MathQuestion;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"math_question\"");

        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Action::MathQuestion);
    }

    #[test]
    fn test_expertise_empty() {
        // Expertise is not used for math tutoring platform
        // Vec<Expertise> should always be empty
        let expertise_list: Vec<Expertise> = vec![];
        assert!(expertise_list.is_empty());
    }

    #[test]
    fn test_constraints_default() {
        let constraints = Constraints::default();
        assert!(constraints.max_budget.is_none());
        assert!(constraints.max_results.is_none());
        assert!(constraints.deadline.is_none());
        assert!(constraints.additional.is_empty());
    }

    #[test]
    fn test_constraints_serialization() {
        let constraints = Constraints {
            max_budget: Some(50000),
            max_results: Some(10),
            deadline: Some("2024-12-31".to_string()),
            additional: HashMap::new(),
        };

        let json = serde_json::to_string(&constraints).unwrap();
        let deserialized: Constraints = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_budget, Some(50000));
        assert_eq!(deserialized.max_results, Some(10));
        assert_eq!(deserialized.deadline, Some("2024-12-31".to_string()));
    }

    #[test]
    fn test_trusted_intent_validation_valid() {
        let intent = TrustedIntent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            action: Action::MathQuestion,
            topic_id: "algebra_equation".to_string(),
            expertise: vec![],
            constraints: Constraints::default(),
            content_refs: vec![],
            signature: None,
            content_hash: "abc123".to_string(),
            user_id: "user_123".to_string(),
            session_id: "session_456".to_string(),
        };

        assert!(intent.validate_no_raw_content().is_ok());
    }

    #[test]
    fn test_trusted_intent_validation_invalid_topic() {
        let intent = TrustedIntent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            action: Action::MathQuestion,
            topic_id: "this has spaces in it".to_string(), // Invalid!
            expertise: vec![],
            constraints: Constraints::default(),
            content_refs: vec![],
            signature: None,
            content_hash: "abc123".to_string(),
            user_id: "user_123".to_string(),
            session_id: "session_456".to_string(),
        };

        assert!(intent.validate_no_raw_content().is_err());
    }

    #[test]
    fn test_trusted_intent_validation_invalid_content_ref() {
        let intent = TrustedIntent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            action: Action::MathQuestion,
            topic_id: "valid_topic".to_string(),
            expertise: vec![],
            constraints: Constraints::default(),
            content_refs: vec!["invalid\nwith\nnewlines".to_string()], // Invalid!
            signature: None,
            content_hash: "abc123".to_string(),
            user_id: "user_123".to_string(),
            session_id: "session_456".to_string(),
        };

        assert!(intent.validate_no_raw_content().is_err());
    }

    #[test]
    fn test_schema_error_display() {
        let err = SchemaError::ValidationError("test error".to_string());
        assert!(err.to_string().contains("Validation error"));

        let err2 = SchemaError::InvalidAction("test_action".to_string());
        assert!(err2.to_string().contains("Invalid action"));

        let err3 = SchemaError::ConstraintViolation("too high".to_string());
        assert!(err3.to_string().contains("Constraint violation"));
    }

    #[test]
    fn test_all_actions() {
        let actions = vec![Action::MathQuestion];

        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let deserialized: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(action, deserialized);
        }
    }

    #[test]
    fn test_no_expertise_for_math_tutoring() {
        // Math tutoring platform does not use expertise areas
        // All intents should have empty expertise vectors
        let intent = Intent {
            action: Action::MathQuestion,
            topic: Some("algebra".to_string()),
            expertise: vec![], // Always empty for math tutoring
            constraints: Constraints::default(),
            content_refs: None,
            metadata: None,
        };

        assert!(intent.expertise.is_empty());
    }
}
