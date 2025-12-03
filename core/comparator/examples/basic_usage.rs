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

    // Create a provider configuration for a math tutor service
    let config = ProviderConfig {
        allowed_actions: vec!["math_question".to_string()],
        allowed_expertise: vec![],
        max_budget: None,
        allowed_domains: vec![],
        require_human_approval: false,
    };

    println!("Provider Configuration:");
    println!("  Allowed Actions: {:?}", config.allowed_actions);
    println!("  Allowed Expertise: {:?}", config.allowed_expertise);
    println!("  Max Budget: None\n");

    // Create the comparator
    let comparator = IntentComparator::new();

    // Example 1: Valid math question - should be approved
    println!("--- Example 1: Valid Math Question ---");
    let intent1 = create_intent("math_question", "What is 2 + 2?");
    check_intent(&comparator, &intent1, &config, 1).await;

    // Example 2: Another valid math question
    println!("\n--- Example 2: Complex Math Question ---");
    let intent2 = create_intent("math_question", "Solve for x: 3x + 5 = 20");
    check_intent(&comparator, &intent2, &config, 2).await;

    // Example 3: Disallowed action
    println!("\n--- Example 3: Disallowed Action ---");
    let intent3 = create_intent("find_experts", "security");
    check_intent(&comparator, &intent3, &config, 3).await;

    // Example 4: Valid calculus question
    println!("\n--- Example 4: Calculus Question ---");
    let intent4 = create_intent(
        "math_question",
        "Calculate the derivative of f(x) = x^2 + 3x",
    );
    check_intent(&comparator, &intent4, &config, 4).await;
}

fn create_intent(action: &str, topic: &str) -> Intent {
    let constraints = HashMap::new();

    Intent {
        action: action.to_string(),
        topic_id: topic.to_string(),
        expertise: vec![],
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
