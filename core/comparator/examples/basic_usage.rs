//! Basic usage example for the Intent Comparator
//!
//! Run with: cargo run --example basic_usage

use intent_comparator::{ComparisonResult, IntentComparator};
use intent_schema::{Intent, IntentMetadata, ProviderConfig};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Set up logging
    tracing_subscriber::fmt::init();

    println!("=== Intent Comparator Examples ===\n");

    // Create a provider configuration for a B2B consulting platform
    let config = ProviderConfig {
        allowed_actions: vec![
            "find_experts".to_string(),
            "summarize".to_string(),
            "analyze_document".to_string(),
        ],
        allowed_expertise: vec![
            "security".to_string(),
            "ml".to_string(),
            "cloud".to_string(),
        ],
        max_budget: Some(50000),
        allowed_domains: vec![],
    };

    println!("Provider Configuration:");
    println!("  Allowed Actions: {:?}", config.allowed_actions);
    println!("  Allowed Expertise: {:?}", config.allowed_expertise);
    println!("  Max Budget: ${}\n", config.max_budget.unwrap());

    // Create the comparator
    let comparator = IntentComparator::new();

    // Example 1: Valid intent - should be approved
    println!("--- Example 1: Valid Intent ---");
    let intent1 = create_intent(
        "find_experts",
        "supply_chain_security",
        vec!["security"],
        20000,
    );
    check_intent(&comparator, &intent1, &config, 1).await;

    // Example 2: Disallowed action
    println!("\n--- Example 2: Disallowed Action ---");
    let intent2 = create_intent("draft_proposal", "new_project", vec!["security"], 10000);
    check_intent(&comparator, &intent2, &config, 2).await;

    // Example 3: Disallowed expertise
    println!("\n--- Example 3: Disallowed Expertise ---");
    let intent3 = create_intent("find_experts", "web_development", vec!["frontend"], 15000);
    check_intent(&comparator, &intent3, &config, 3).await;

    // Example 4: Budget exceeded
    println!("\n--- Example 4: Budget Exceeded ---");
    let intent4 = create_intent("find_experts", "ai_research", vec!["ml"], 100000);
    check_intent(&comparator, &intent4, &config, 4).await;

    // Example 5: Multiple violations
    println!("\n--- Example 5: Multiple Violations ---");
    let intent5 = create_intent("draft_proposal", "test", vec!["frontend"], 200000);
    check_intent(&comparator, &intent5, &config, 5).await;

    // Example 6: Valid intent with multiple expertise areas
    println!("\n--- Example 6: Multiple Valid Expertise Areas ---");
    let intent6 = create_intent(
        "find_experts",
        "cloud_ml_project",
        vec!["cloud", "ml"],
        30000,
    );
    check_intent(&comparator, &intent6, &config, 6).await;
}

fn create_intent(action: &str, topic: &str, expertise: Vec<&str>, budget: i64) -> Intent {
    let mut constraints = HashMap::new();
    constraints.insert("max_budget".to_string(), json!(budget));

    Intent {
        action: action.to_string(),
        topic_id: topic.to_string(),
        expertise: expertise.iter().map(|s| s.to_string()).collect(),
        constraints,
        content_refs: vec![],
        metadata: IntentMetadata {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_id: "user_demo".to_string(),
            session_id: "session_demo".to_string(),
        },
    }
}

async fn check_intent(
    comparator: &IntentComparator,
    intent: &Intent,
    config: &ProviderConfig,
    example_num: usize,
) {
    println!("Intent {}:", example_num);
    println!("  Action: {}", intent.action);
    println!("  Topic: {}", intent.topic_id);
    println!("  Expertise: {:?}", intent.expertise);
    if let Some(budget) = intent.constraints.get("max_budget") {
        println!("  Budget: ${}", budget.as_i64().unwrap());
    }

    let result = comparator
        .compare(intent, config)
        .await
        .expect("Comparison failed");

    match result {
        ComparisonResult::Approved { message } => {
            println!("\n  ✓ APPROVED");
            println!("  {}", message);
        }
        ComparisonResult::SoftMismatch { message, reasons } => {
            println!("\n  ⚠ SOFT MISMATCH");
            println!("  {}", message);
            println!("  Reasons:");
            for reason in reasons {
                println!("    - [{:?}] {}", reason.severity, reason.description);
            }
        }
        ComparisonResult::HardMismatch { message, reasons } => {
            println!("\n  ✗ HARD MISMATCH");
            println!("  {}", message);
            println!("  Reasons:");
            for reason in reasons {
                println!("    - [{:?}] {}", reason.severity, reason.description);
            }
        }
    }
}
