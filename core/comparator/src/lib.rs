use intent_schema::{Intent, ProviderConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Result of comparing an intent against provider configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonResult {
    /// Intent is fully approved and within all constraints
    Approved { message: String },
    /// Intent has minor issues that might be acceptable with user confirmation
    SoftMismatch {
        reasons: Vec<MismatchReason>,
        message: String,
    },
    /// Intent violates hard constraints and should be blocked or escalated
    HardMismatch {
        reasons: Vec<MismatchReason>,
        message: String,
    },
}

/// Detailed reason for a mismatch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MismatchReason {
    pub severity: Severity,
    pub category: MismatchCategory,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MismatchCategory {
    ActionNotAllowed,
    ExpertiseNotAllowed,
    BudgetExceeded,
    CustomConstraintViolation,
}

/// Errors that can occur during comparison
#[derive(Debug, Error)]
pub enum ComparatorError {
    #[error("Invalid intent: {0}")]
    InvalidIntent(String),

    #[error("Invalid provider config: {0}")]
    InvalidConfig(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Intent comparator that validates user intents against provider configurations
pub struct IntentComparator {
    /// Whether to use strict mode (treat medium/high mismatches as hard mismatches)
    strict_mode: bool,
}

impl IntentComparator {
    /// Creates a new IntentComparator with default settings
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Creates a new IntentComparator with strict mode enabled
    pub fn new_strict() -> Self {
        Self { strict_mode: true }
    }

    /// Compares a user intent against provider configuration
    ///
    /// # Arguments
    /// * `intent` - The parsed user intent to validate
    /// * `config` - The provider configuration defining allowed intents
    ///
    /// # Returns
    /// A `ComparisonResult` indicating whether the intent is approved, has soft mismatches, or hard mismatches
    pub async fn compare(
        &self,
        intent: &Intent,
        config: &ProviderConfig,
    ) -> Result<ComparisonResult, ComparatorError> {
        info!(
            action = %intent.action,
            "Starting intent comparison"
        );

        let mut reasons = Vec::new();

        // Check if action is allowed
        self.check_action(intent, config, &mut reasons);

        // Check if expertise is allowed
        self.check_expertise(intent, config, &mut reasons);

        // Check budget constraints
        self.check_budget(intent, config, &mut reasons);

        // Categorize the result based on severity
        let result = self.categorize_result(reasons);

        debug!(result = ?result, "Comparison completed");

        Ok(result)
    }

    /// Checks if the requested action is in the allowed actions list
    fn check_action(
        &self,
        intent: &Intent,
        config: &ProviderConfig,
        reasons: &mut Vec<MismatchReason>,
    ) {
        if !config.allowed_actions.contains(&intent.action) {
            warn!(
                action = %intent.action,
                allowed_actions = ?config.allowed_actions,
                "Action not in allowed list"
            );

            reasons.push(MismatchReason {
                severity: Severity::Critical,
                category: MismatchCategory::ActionNotAllowed,
                description: format!(
                    "Action '{}' is not in the allowed actions list. Allowed actions: {:?}",
                    intent.action, config.allowed_actions
                ),
            });
        }
    }

    /// Checks if the requested expertise areas are allowed
    fn check_expertise(
        &self,
        intent: &Intent,
        config: &ProviderConfig,
        reasons: &mut Vec<MismatchReason>,
    ) {
        // If provider doesn't specify allowed expertise, skip this check
        if config.allowed_expertise.is_empty() {
            return;
        }

        let allowed_set: HashSet<&String> = config.allowed_expertise.iter().collect();
        let requested_set: HashSet<&String> = intent.expertise.iter().collect();

        let unauthorized: Vec<&String> = requested_set.difference(&allowed_set).copied().collect();

        if !unauthorized.is_empty() {
            warn!(
                unauthorized = ?unauthorized,
                "Requested expertise not allowed"
            );

            reasons.push(MismatchReason {
                severity: Severity::Critical,
                category: MismatchCategory::ExpertiseNotAllowed,
                description: format!(
                    "Requested expertise areas not allowed: {:?}. Allowed expertise: {:?}",
                    unauthorized, config.allowed_expertise
                ),
            });
        }
    }

    /// Checks if the budget constraint is within limits
    fn check_budget(
        &self,
        intent: &Intent,
        config: &ProviderConfig,
        reasons: &mut Vec<MismatchReason>,
    ) {
        // Try to extract max_budget from constraints HashMap
        if let Some(budget_value) = intent.constraints.get("max_budget") {
            if let Some(requested_budget) = budget_value.as_i64() {
                if let Some(max_allowed_budget) = config.max_budget {
                    if requested_budget > max_allowed_budget {
                        warn!(
                            requested_budget,
                            max_allowed_budget, "Budget exceeds maximum allowed"
                        );

                        reasons.push(MismatchReason {
                            severity: Severity::Critical,
                            category: MismatchCategory::BudgetExceeded,
                            description: format!(
                                "Requested budget ${} exceeds maximum allowed budget ${}",
                                requested_budget, max_allowed_budget
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Categorizes the comparison result based on mismatch reasons
    fn categorize_result(&self, reasons: Vec<MismatchReason>) -> ComparisonResult {
        if reasons.is_empty() {
            return ComparisonResult::Approved {
                message: "Intent approved - all checks passed".to_string(),
            };
        }

        // Check if any reasons are critical
        let has_critical = reasons
            .iter()
            .any(|r| matches!(r.severity, Severity::Critical));

        // In strict mode, High and Medium severity also become hard mismatches
        let has_elevated = self.strict_mode
            && reasons
                .iter()
                .any(|r| matches!(r.severity, Severity::High | Severity::Medium));

        if has_critical || has_elevated {
            ComparisonResult::HardMismatch {
                message: format!("Intent denied - {} violation(s) found", reasons.len()),
                reasons,
            }
        } else {
            ComparisonResult::SoftMismatch {
                message: format!(
                    "Intent requires review - {} minor issue(s) found",
                    reasons.len()
                ),
                reasons,
            }
        }
    }
}

impl Default for IntentComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonResult {
    /// Returns true if the intent is approved
    pub fn is_approved(&self) -> bool {
        matches!(self, ComparisonResult::Approved { .. })
    }

    /// Returns true if the intent has a soft mismatch
    pub fn is_soft_mismatch(&self) -> bool {
        matches!(self, ComparisonResult::SoftMismatch { .. })
    }

    /// Returns true if the intent has a hard mismatch
    pub fn is_hard_mismatch(&self) -> bool {
        matches!(self, ComparisonResult::HardMismatch { .. })
    }

    /// Returns the reasons for mismatch, if any
    pub fn reasons(&self) -> Vec<MismatchReason> {
        match self {
            ComparisonResult::Approved { .. } => Vec::new(),
            ComparisonResult::SoftMismatch { reasons, .. }
            | ComparisonResult::HardMismatch { reasons, .. } => reasons.clone(),
        }
    }

    /// Returns the message
    pub fn message(&self) -> &str {
        match self {
            ComparisonResult::Approved { message }
            | ComparisonResult::SoftMismatch { message, .. }
            | ComparisonResult::HardMismatch { message, .. } => message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intent_schema::IntentMetadata;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_config() -> ProviderConfig {
        ProviderConfig {
            allowed_actions: vec!["math_question".to_string()],
            allowed_expertise: vec![],
            max_budget: None,
            allowed_domains: vec![],
            require_human_approval: false,
        }
    }

    fn create_test_intent_metadata() -> IntentMetadata {
        IntentMetadata {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_id: "user_123".to_string(),
            session_id: "session_456".to_string(),
        }
    }

    #[tokio::test]
    async fn test_approved_intent() {
        let comparator = IntentComparator::new();
        let config = create_test_config();

        let intent = Intent {
            action: "math_question".to_string(),
            topic_id: "What is 2 + 2?".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: create_test_intent_metadata(),
        };

        let result = comparator.compare(&intent, &config).await.unwrap();

        assert!(result.is_approved());
        assert_eq!(result.message(), "Intent approved - all checks passed");
    }

    #[tokio::test]
    async fn test_action_not_allowed() {
        let comparator = IntentComparator::new();
        let config = create_test_config();

        let intent = Intent {
            action: "delete_database".to_string(),
            topic_id: "What is 2 + 2?".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: create_test_intent_metadata(),
        };

        let result = comparator.compare(&intent, &config).await.unwrap();

        assert!(result.is_hard_mismatch());
        let reasons = result.reasons();
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].category, MismatchCategory::ActionNotAllowed);
    }

    #[tokio::test]
    async fn test_expertise_allowed_empty() {
        let comparator = IntentComparator::new();
        let config = create_test_config();

        let intent = Intent {
            action: "math_question".to_string(),
            topic_id: "Solve for x: 3x + 5 = 20".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: create_test_intent_metadata(),
        };

        let result = comparator.compare(&intent, &config).await.unwrap();

        // Should be approved since empty expertise list means no restriction
        assert!(result.is_approved());
    }

    #[tokio::test]
    async fn test_no_budget_constraint() {
        let comparator = IntentComparator::new();
        let config = create_test_config();

        let intent = Intent {
            action: "math_question".to_string(),
            topic_id: "What is the derivative of x^2?".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: create_test_intent_metadata(),
        };

        let result = comparator.compare(&intent, &config).await.unwrap();

        // Should be approved with no budget constraints
        assert!(result.is_approved());
    }

    #[tokio::test]
    async fn test_forbidden_action() {
        let comparator = IntentComparator::new();
        let config = create_test_config();

        let intent = Intent {
            action: "execute_code".to_string(),
            topic_id: "rm -rf /".to_string(),
            expertise: vec![],
            constraints: HashMap::new(),
            content_refs: vec![],
            metadata: create_test_intent_metadata(),
        };

        let result = comparator.compare(&intent, &config).await.unwrap();

        assert!(result.is_hard_mismatch());
        let reasons = result.reasons();
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].category, MismatchCategory::ActionNotAllowed);
    }

    #[tokio::test]
    async fn test_math_question_variants() {
        let comparator = IntentComparator::new();
        let config = create_test_config();

        let questions = vec![
            "What is 2 + 2?",
            "Solve for x: 3x + 5 = 20",
            "What is the derivative of x^2?",
            "Calculate the integral of 2x",
        ];

        for question in questions {
            let intent = Intent {
                action: "math_question".to_string(),
                topic_id: question.to_string(),
                expertise: vec![],
                constraints: HashMap::new(),
                content_refs: vec![],
                metadata: create_test_intent_metadata(),
            };

            let result = comparator.compare(&intent, &config).await.unwrap();
            assert!(result.is_approved(), "Failed for question: {}", question);
        }
    }

    #[tokio::test]
    async fn test_serialization() {
        let result = ComparisonResult::Approved {
            message: "Test".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ComparisonResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }
}
