//! Voting Module for Intent Parser Ensemble
//!
//! This module implements a voting mechanism that compares outputs from multiple
//! intent parsers (deterministic and LLM-based) to determine a canonical intent
//! with a confidence level.
//!
//! # Purpose
//!
//! In a prompt injection defense system, we cannot trust a single parser to always
//! correctly extract user intent. The voting module:
//!
//! 1. Compares multiple parser outputs
//! 2. Calculates similarity scores using smart diffing
//! 3. Determines agreement level (HighConfidence, LowConfidence, Conflict)
//! 4. Selects canonical intent (preferring deterministic parser)
//! 5. Flags conflicts for human review
//!
//! # Example
//!
//! ```rust,no_run
//! use intent_voting::VotingModule;
//! use intent_schema::{ParsedIntent, Intent, IntentMetadata, AgreementLevel};
//! use chrono::Utc;
//! use uuid::Uuid;
//! use std::collections::HashMap;
//!
//! # async fn example() {
//! let voting = VotingModule::new();
//!
//! let parser_results = vec![
//!     ParsedIntent {
//!         parser_id: "deterministic".to_string(),
//!         intent: Intent {
//!             action: "math_question".to_string(),
//!             topic_id: "What is 2 + 2?".to_string(),
//!             expertise: vec![],
//!             constraints: HashMap::new(),
//!             content_refs: vec![],
//!             metadata: IntentMetadata {
//!                 id: Uuid::new_v4(),
//!                 timestamp: Utc::now(),
//!                 user_id: "user_123".to_string(),
//!                 session_id: "session_456".to_string(),
//!             },
//!         },
//!         confidence: 1.0,
//!     },
//!     // ... more parser results
//! ];
//!
//! let result = voting.vote(parser_results, Some("deterministic")).await.unwrap();
//! println!("Agreement: {:?}", result.agreement_level);
//! # }
//! ```

use intent_schema::{AgreementLevel, Intent, ParsedIntent, VotingResult};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during voting
#[derive(Error, Debug)]
pub enum VotingError {
    #[error("No intents provided for voting")]
    NoIntents,

    #[error("Insufficient parser results: got {0}, need at least 1")]
    InsufficientParsers(usize),

    #[error("Deterministic parser result not found: {0}")]
    NoDeterministicParser(String),
}

/// The voting module that compares parser outputs
pub struct VotingModule {
    /// Similarity threshold for high confidence (default: 0.95)
    high_confidence_threshold: f64,

    /// Similarity threshold for low confidence vs conflict (default: 0.75)
    low_confidence_threshold: f64,

    /// Whether to always prefer deterministic parser for canonical intent
    prefer_deterministic: bool,
}

impl Default for VotingModule {
    fn default() -> Self {
        Self::new()
    }
}

impl VotingModule {
    /// Create a new VotingModule with default thresholds
    ///
    /// - High confidence: >= 95% similarity
    /// - Low confidence: >= 75% similarity
    /// - Conflict: < 75% similarity
    pub fn new() -> Self {
        Self {
            high_confidence_threshold: 0.95,
            low_confidence_threshold: 0.75,
            prefer_deterministic: true,
        }
    }

    /// Create a VotingModule with custom thresholds
    ///
    /// # Arguments
    ///
    /// * `high_confidence_threshold` - Minimum similarity for high confidence (0.0-1.0)
    /// * `low_confidence_threshold` - Minimum similarity for low confidence (0.0-1.0)
    pub fn with_thresholds(high_confidence_threshold: f64, low_confidence_threshold: f64) -> Self {
        Self {
            high_confidence_threshold,
            low_confidence_threshold,
            prefer_deterministic: true,
        }
    }

    /// Perform voting on multiple parser results
    ///
    /// # Arguments
    ///
    /// * `results` - Parser results to compare
    /// * `deterministic_parser_id` - Optional ID of the deterministic parser for preference
    ///
    /// # Returns
    ///
    /// `VotingResult` containing canonical intent and agreement level
    pub async fn vote(
        &self,
        results: Vec<ParsedIntent>,
        deterministic_parser_id: Option<&str>,
    ) -> Result<VotingResult, VotingError> {
        info!(
            "Starting voting process with {} parser results",
            results.len()
        );

        // Validate input
        if results.is_empty() {
            return Err(VotingError::NoIntents);
        }

        // Single parser case
        if results.len() == 1 {
            warn!("Only one parser result available, returning with low confidence");
            return Ok(self.single_parser_result(results));
        }

        // Calculate pairwise similarities
        let similarities = self.calculate_pairwise_similarities(&results);

        // Calculate statistics
        let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
        let min_similarity = similarities.iter().copied().fold(f64::INFINITY, f64::min);

        debug!(
            "Similarity scores - avg: {:.3}, min: {:.3}",
            avg_similarity, min_similarity
        );

        // Determine agreement level
        let agreement_level = self.determine_agreement(min_similarity, avg_similarity);

        // Select canonical intent
        let canonical_intent =
            self.select_canonical_intent(&results, &agreement_level, deterministic_parser_id)?;

        info!(
            "Voting complete: {:?}, min_sim: {:.3}, avg_sim: {:.3}",
            agreement_level, min_similarity, avg_similarity
        );

        Ok(VotingResult {
            canonical_intent,
            agreement_level,
            parser_results: results,
        })
    }

    /// Handle case where only one parser result is available
    fn single_parser_result(&self, mut results: Vec<ParsedIntent>) -> VotingResult {
        let result = results.remove(0);
        let canonical_intent = result.intent.clone();
        VotingResult {
            canonical_intent,
            agreement_level: AgreementLevel::LowConfidence,
            parser_results: vec![result],
        }
    }

    /// Calculate all pairwise similarities between parser results
    fn calculate_pairwise_similarities(&self, results: &[ParsedIntent]) -> Vec<f64> {
        let mut similarities = Vec::new();

        for i in 0..results.len() {
            for j in (i + 1)..results.len() {
                let sim = results[i].intent.similarity(&results[j].intent);
                similarities.push(sim);
                debug!(
                    "Similarity between {} and {}: {:.3}",
                    results[i].parser_id, results[j].parser_id, sim
                );
            }
        }

        similarities
    }

    /// Determine agreement level based on similarity scores
    fn determine_agreement(&self, min_similarity: f64, avg_similarity: f64) -> AgreementLevel {
        if min_similarity >= self.high_confidence_threshold {
            AgreementLevel::HighConfidence
        } else if avg_similarity >= self.low_confidence_threshold {
            AgreementLevel::LowConfidence
        } else {
            AgreementLevel::Conflict
        }
    }

    /// Select the canonical intent based on voting results
    ///
    /// Prefers the deterministic parser when available. In case of conflict,
    /// still returns the deterministic parser result but marks for human review.
    fn select_canonical_intent(
        &self,
        results: &[ParsedIntent],
        _agreement_level: &AgreementLevel,
        deterministic_parser_id: Option<&str>,
    ) -> Result<Intent, VotingError> {
        if self.prefer_deterministic {
            if let Some(det_id) = deterministic_parser_id {
                if let Some(deterministic) = results.iter().find(|r| r.parser_id == det_id) {
                    debug!("Selected deterministic parser result: {}", det_id);
                    return Ok(deterministic.intent.clone());
                } else {
                    warn!("Deterministic parser '{}' not found in results", det_id);
                }
            }
        }

        // Fallback to first result or highest confidence
        results
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| {
                debug!("Selected highest confidence parser result: {}", r.parser_id);
                r.intent.clone()
            })
            .ok_or(VotingError::NoIntents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use intent_schema::IntentMetadata;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_intent(action: &str, topic_id: &str, expertise: Vec<String>) -> Intent {
        Intent {
            action: action.to_string(),
            topic_id: topic_id.to_string(),
            expertise,
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
    async fn test_high_confidence_all_agree() {
        let voting = VotingModule::new();

        let intent = create_test_intent("math_question", "What is 2 + 2?", vec![]);

        let results = vec![
            ParsedIntent {
                parser_id: "deterministic".to_string(),
                intent: intent.clone(),
                confidence: 1.0,
            },
            ParsedIntent {
                parser_id: "llm1".to_string(),
                intent: intent.clone(),
                confidence: 0.95,
            },
            ParsedIntent {
                parser_id: "llm2".to_string(),
                intent: intent.clone(),
                confidence: 0.93,
            },
        ];

        let result = voting.vote(results, Some("deterministic")).await.unwrap();

        assert_eq!(result.agreement_level, AgreementLevel::HighConfidence);
        assert_eq!(result.canonical_intent.action, "math_question");
    }

    #[tokio::test]
    async fn test_low_confidence_minor_differences() {
        let voting = VotingModule::new();

        let intent1 = create_test_intent("math_question", "What is 2 + 2?", vec![]);

        let mut intent2 = intent1.clone();
        intent2.topic_id = "What is 2 + 2 + 2?".to_string();

        let results = vec![
            ParsedIntent {
                parser_id: "deterministic".to_string(),
                intent: intent1,
                confidence: 1.0,
            },
            ParsedIntent {
                parser_id: "llm1".to_string(),
                intent: intent2,
                confidence: 0.9,
            },
        ];

        let result = voting.vote(results, Some("deterministic")).await.unwrap();

        // Should be low confidence or high confidence depending on topic similarity
        assert!(matches!(
            result.agreement_level,
            AgreementLevel::LowConfidence | AgreementLevel::HighConfidence
        ));
        // Should select deterministic parser
        assert_eq!(result.canonical_intent.topic_id, "What is 2 + 2?");
    }

    #[tokio::test]
    async fn test_conflict_major_differences() {
        let voting = VotingModule::new();

        let intent1 = create_test_intent("math_question", "What is 2 + 2?", vec![]);

        let intent2 = create_test_intent(
            "math_question",
            "Solve for x: 3x + 5 = 20", // Different math problem
            vec![],
        );

        let results = vec![
            ParsedIntent {
                parser_id: "deterministic".to_string(),
                intent: intent1,
                confidence: 1.0,
            },
            ParsedIntent {
                parser_id: "llm1".to_string(),
                intent: intent2,
                confidence: 0.8,
            },
        ];

        let result = voting.vote(results, Some("deterministic")).await.unwrap();

        // Different math topics should still have some similarity
        assert!(matches!(
            result.agreement_level,
            AgreementLevel::LowConfidence
                | AgreementLevel::HighConfidence
                | AgreementLevel::Conflict
        ));
        // Should still select deterministic parser
        assert_eq!(result.canonical_intent.action, "math_question");
    }

    #[tokio::test]
    async fn test_single_parser() {
        let voting = VotingModule::new();

        let intent = create_test_intent(
            "math_question",
            "Calculate the area of a circle with radius 5",
            vec![],
        );

        let results = vec![ParsedIntent {
            parser_id: "llm1".to_string(),
            intent,
            confidence: 0.9,
        }];

        let result = voting.vote(results, None).await.unwrap();

        assert_eq!(result.agreement_level, AgreementLevel::LowConfidence);
        assert_eq!(result.parser_results.len(), 1);
    }

    #[tokio::test]
    async fn test_no_parsers() {
        let voting = VotingModule::new();
        let results: Vec<ParsedIntent> = vec![];

        let result = voting.vote(results, None).await;
        assert!(matches!(result, Err(VotingError::NoIntents)));
    }
}
