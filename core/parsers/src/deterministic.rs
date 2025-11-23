use crate::types::{IntentParser, ParserError, ParserResult};
use chrono::Utc;
use intent_schema::{Intent, IntentMetadata, ParsedIntent};
use regex::Regex;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// Deterministic rule-based parser
/// - No LLM, zero hallucination risk
/// - Highest trust level (1.0)
/// - Uses keyword matching and regex patterns
pub struct DeterministicParser {
    parser_id: String,
}

impl DeterministicParser {
    pub fn new() -> Self {
        Self {
            parser_id: "deterministic_v1".to_string(),
        }
    }

    /// Extract action from user input using keyword matching
    fn extract_action(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();

        // Define keyword patterns for each action
        let patterns = vec![
            (
                "find_experts",
                vec![
                    "find expert",
                    "search expert",
                    "locate expert",
                    "get expert",
                ],
            ),
            (
                "summarize",
                vec!["summarize", "summary of", "give me a summary"],
            ),
            (
                "draft_proposal",
                vec![
                    "draft proposal",
                    "create proposal",
                    "write proposal",
                    "proposal for",
                ],
            ),
            ("research", vec!["research", "investigate", "study"]),
            ("query", vec!["query", "question", "ask about", "what is"]),
        ];

        // Check each pattern
        for (action, keywords) in patterns {
            for keyword in keywords {
                if input_lower.contains(keyword) {
                    return action.to_string();
                }
            }
        }

        "unknown".to_string()
    }

    /// Extract expertise areas from user input
    fn extract_expertise(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();
        let mut expertise = Vec::new();

        let expertise_keywords = vec![
            (
                "ml",
                vec!["ml", "machine learning", "ai", "artificial intelligence"],
            ),
            (
                "embedded",
                vec!["embedded", "iot", "firmware", "microcontroller"],
            ),
            (
                "security",
                vec![
                    "security",
                    "cybersecurity",
                    "infosec",
                    "penetration testing",
                ],
            ),
            ("cloud", vec!["cloud", "aws", "azure", "gcp", "kubernetes"]),
            (
                "blockchain",
                vec!["blockchain", "crypto", "web3", "ethereum"],
            ),
        ];

        for (key, keywords) in expertise_keywords {
            for keyword in keywords {
                if input_lower.contains(keyword) {
                    expertise.push(key.to_string());
                    break;
                }
            }
        }

        expertise
    }

    /// Extract topic ID from user input
    fn extract_topic_id(&self, input: &str) -> String {
        // Simple heuristic: extract text after common prepositions
        let re = Regex::new(r"(?:about|on|regarding|for|in)\s+([^.?!,]+)").unwrap();

        if let Some(captures) = re.captures(input) {
            if let Some(topic) = captures.get(1) {
                let topic_str = topic.as_str().trim();
                // Convert to snake_case ID
                return topic_str
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
                    .replace(' ', "_")
                    .chars()
                    .take(50)
                    .collect();
            }
        }

        // Fallback: use a hash of the input
        format!("topic_{}", &format!("{:x}", md5::compute(input))[..8])
    }

    /// Extract budget constraint from user input
    fn extract_budget(&self, input: &str) -> Option<u64> {
        let re = Regex::new(r"budget[:\s]+\$?(\d+(?:,\d{3})*(?:\.\d{2})?)(?:k|K)?").unwrap();

        if let Some(captures) = re.captures(input) {
            if let Some(amount) = captures.get(1) {
                let amount_str = amount.as_str().replace(",", "");
                if let Ok(mut budget) = amount_str.parse::<u64>() {
                    // Handle "k" suffix
                    if input[amount.end()..].starts_with('k')
                        || input[amount.end()..].starts_with('K')
                    {
                        budget *= 1000;
                    }
                    return Some(budget);
                }
            }
        }

        None
    }

    /// Extract max results constraint
    fn extract_max_results(&self, input: &str) -> Option<u64> {
        let re =
            Regex::new(r"(?:max|maximum|up to|top)\s+(\d+)(?:\s+(?:results?|experts?|items?))?")
                .unwrap();

        if let Some(captures) = re.captures(input) {
            if let Some(count) = captures.get(1) {
                if let Ok(max_results) = count.as_str().parse::<u64>() {
                    return Some(max_results.min(100)); // Cap at 100
                }
            }
        }

        None
    }
}

impl Default for DeterministicParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl IntentParser for DeterministicParser {
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

        // Extract all components using rule-based matching
        let action = self.extract_action(user_input);
        let topic_id = self.extract_topic_id(user_input);
        let expertise = self.extract_expertise(user_input);
        let budget = self.extract_budget(user_input);
        let max_results = self.extract_max_results(user_input);

        // Build constraints
        let mut constraints = HashMap::new();
        if let Some(budget) = budget {
            constraints.insert("max_budget".to_string(), serde_json::json!(budget));
        }
        if let Some(max_results) = max_results {
            constraints.insert("max_results".to_string(), serde_json::json!(max_results));
        }

        // Create metadata
        let metadata = IntentMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        };

        // Create intent
        let intent = Intent {
            action,
            topic_id,
            expertise,
            constraints,
            content_refs: Vec::new(),
            metadata,
        };

        tracing::debug!(
            "Deterministic parser completed in {}ms",
            start.elapsed().as_millis()
        );

        Ok(ParsedIntent {
            parser_id: self.parser_id(),
            intent,
            confidence: 1.0, // Deterministic parser always has max confidence
        })
    }

    fn parser_id(&self) -> String {
        self.parser_id.clone()
    }

    fn trust_level(&self) -> f64 {
        1.0 // Highest trust - no hallucination risk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_experts_parsing() {
        let parser = DeterministicParser::new();
        let input = "Find experts in machine learning with budget $50000";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        assert_eq!(result.intent.action, "find_experts");
        assert!(result.intent.expertise.contains(&"ml".to_string()));
        assert_eq!(result.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_summarize_parsing() {
        let parser = DeterministicParser::new();
        let input = "Summarize the latest research on blockchain security";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        assert_eq!(result.intent.action, "summarize");
        assert!(result.intent.expertise.contains(&"blockchain".to_string()));
        assert!(result.intent.expertise.contains(&"security".to_string()));
    }

    #[tokio::test]
    async fn test_budget_extraction() {
        let parser = DeterministicParser::new();
        let input = "Find cloud experts budget: $25,000";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        let constraints = &result.intent.constraints;
        assert_eq!(constraints.get("max_budget").unwrap().as_u64(), Some(25000));
    }

    #[tokio::test]
    async fn test_max_results_extraction() {
        let parser = DeterministicParser::new();
        let input = "Find top 5 security experts";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        let constraints = &result.intent.constraints;
        assert_eq!(constraints.get("max_results").unwrap().as_u64(), Some(5));
    }

    #[tokio::test]
    async fn test_empty_input() {
        let parser = DeterministicParser::new();
        let result = parser.parse("", "test_user", "test_session").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whitespace_only_input() {
        let parser = DeterministicParser::new();
        let result = parser.parse("   \n\t  ", "test_user", "test_session").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parser_id() {
        let parser = DeterministicParser::new();
        assert_eq!(parser.parser_id(), "deterministic_v1");
    }

    #[tokio::test]
    async fn test_trust_level() {
        let parser = DeterministicParser::new();
        assert_eq!(parser.trust_level(), 1.0);
    }

    #[tokio::test]
    async fn test_multiple_expertise_extraction() {
        let parser = DeterministicParser::new();
        let input = "Find experts in machine learning and cybersecurity for cloud deployment";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        assert!(result.intent.expertise.contains(&"ml".to_string()));
        assert!(result.intent.expertise.contains(&"security".to_string()));
        assert!(result.intent.expertise.contains(&"cloud".to_string()));
    }

    #[tokio::test]
    async fn test_topic_extraction_with_preposition() {
        let parser = DeterministicParser::new();
        let input = "Summarize the latest research on supply chain security";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        assert!(result.intent.topic_id.contains("supply"));
    }

    #[tokio::test]
    async fn test_budget_with_k_suffix() {
        let parser = DeterministicParser::new();
        let input = "Find experts budget $50k";

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        let budget = result
            .intent
            .constraints
            .get("max_budget")
            .unwrap()
            .as_u64();
        assert_eq!(budget, Some(50000));
    }

    #[tokio::test]
    async fn test_max_results_capped_at_100() {
        let parser = DeterministicParser::new();
        let input = "Find top 500 experts"; // Should cap at 100

        let result = parser
            .parse(input, "test_user", "test_session")
            .await
            .unwrap();

        let max_results = result
            .intent
            .constraints
            .get("max_results")
            .unwrap()
            .as_u64();
        assert_eq!(max_results, Some(100));
    }

    #[test]
    fn test_extract_action_variants() {
        let parser = DeterministicParser::new();

        assert_eq!(
            parser.extract_action("find experts in security"),
            "find_experts"
        );
        assert_eq!(
            parser.extract_action("search expert for cloud"),
            "find_experts"
        );
        assert_eq!(
            parser.extract_action("please summarize this document"),
            "summarize"
        );
        assert_eq!(
            parser.extract_action("draft proposal for AI integration"),
            "draft_proposal"
        );
        assert_eq!(
            parser.extract_action("research blockchain trends"),
            "research"
        );
        assert_eq!(parser.extract_action("just some random text"), "unknown");
    }

    #[test]
    fn test_extract_expertise_all_types() {
        let parser = DeterministicParser::new();

        let ml_expertise = parser.extract_expertise("machine learning project");
        assert!(ml_expertise.contains(&"ml".to_string()));

        let embedded_expertise = parser.extract_expertise("embedded systems and IoT");
        assert!(embedded_expertise.contains(&"embedded".to_string()));

        let security_expertise = parser.extract_expertise("cybersecurity assessment");
        assert!(security_expertise.contains(&"security".to_string()));

        let cloud_expertise = parser.extract_expertise("AWS cloud migration");
        assert!(cloud_expertise.contains(&"cloud".to_string()));

        let blockchain_expertise = parser.extract_expertise("ethereum smart contracts");
        assert!(blockchain_expertise.contains(&"blockchain".to_string()));
    }

    #[test]
    fn test_extract_budget_formats() {
        let parser = DeterministicParser::new();

        assert_eq!(parser.extract_budget("budget $50000"), Some(50000));
        assert_eq!(parser.extract_budget("budget: $25,000"), Some(25000));
        assert_eq!(parser.extract_budget("budget $100k"), Some(100000));
        assert_eq!(parser.extract_budget("budget $75K"), Some(75000));
        assert_eq!(parser.extract_budget("no budget mentioned"), None);
    }

    #[test]
    fn test_extract_max_results_formats() {
        let parser = DeterministicParser::new();

        assert_eq!(parser.extract_max_results("top 5 experts"), Some(5));
        assert_eq!(parser.extract_max_results("maximum 10 results"), Some(10));
        assert_eq!(parser.extract_max_results("up to 20 items"), Some(20));
        assert_eq!(parser.extract_max_results("find some experts"), None);
    }
}
