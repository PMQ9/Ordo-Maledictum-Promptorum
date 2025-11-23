// Example demonstrating the Processing Engine
//
// This example shows how the processing engine executes trusted intents
// via type-safe function calls, preventing raw prompt injection.

use intent_schema::{Action, Constraints, Expertise, Intent};
use processing_engine::ProcessingEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the processing engine
    let engine = ProcessingEngine::new();

    println!("=== Intent Segregation Processing Engine Demo ===\n");

    // Example 1: Find Experts
    println!("Example 1: Finding security experts");
    println!("-----------------------------------");
    let find_experts_intent = Intent {
        action: Action::FindExperts,
        topic: Some("supply_chain_risk".to_string()),
        expertise: vec![Expertise::Security],
        constraints: Constraints {
            max_budget: Some(300),
            max_results: Some(5),
            ..Default::default()
        },
        content_refs: None,
        metadata: None,
    };

    let result = engine.execute(&find_experts_intent).await?;
    println!("Success: {}", result.success);
    println!("Function called: {}", result.metadata.function_called);
    println!("Duration: {}ms", result.metadata.duration_ms);
    println!(
        "Result data: {}\n",
        serde_json::to_string_pretty(&result.data)?
    );

    // Example 2: Summarize Document
    println!("Example 2: Summarizing a document");
    println!("----------------------------------");
    let summarize_intent = Intent {
        action: Action::Summarize,
        topic: Some("cybersecurity_trends_2024".to_string()),
        expertise: vec![],
        constraints: Default::default(),
        content_refs: Some(vec!["doc_cs_trends_2024".to_string()]),
        metadata: None,
    };

    let result = engine.execute(&summarize_intent).await?;
    println!("Success: {}", result.success);
    println!("Function called: {}", result.metadata.function_called);
    println!("Duration: {}ms", result.metadata.duration_ms);
    println!(
        "Result data: {}\n",
        serde_json::to_string_pretty(&result.data)?
    );

    // Example 3: Draft Proposal
    println!("Example 3: Drafting a proposal");
    println!("-------------------------------");
    let proposal_intent = Intent {
        action: Action::DraftProposal,
        topic: Some("ai_integration_project".to_string()),
        expertise: vec![Expertise::MachineLearning, Expertise::Security],
        constraints: Constraints {
            max_budget: Some(75000),
            max_results: None,
            ..Default::default()
        },
        content_refs: Some(vec!["requirements_doc_123".to_string()]),
        metadata: None,
    };

    let result = engine.execute(&proposal_intent).await?;
    println!("Success: {}", result.success);
    println!("Function called: {}", result.metadata.function_called);
    println!("Duration: {}ms", result.metadata.duration_ms);
    if !result.metadata.warnings.is_empty() {
        println!("Warnings: {:?}", result.metadata.warnings);
    }
    println!(
        "Result data: {}\n",
        serde_json::to_string_pretty(&result.data)?
    );

    // Example 4: Demonstrating Type Safety
    println!("Example 4: Type safety demonstration");
    println!("------------------------------------");
    println!("The following would NOT compile:");
    println!("  // let raw_prompt = \"Find me experts in security\";");
    println!("  // engine.execute(raw_prompt).await;  // ❌ Type error!");
    println!("");
    println!("Only structured Intent types are accepted, ensuring:");
    println!("  ✓ No raw prompts can execute privileged actions");
    println!("  ✓ All inputs are validated and type-checked");
    println!("  ✓ All operations are traceable and auditable");
    println!("  ✓ Function calls are explicit and well-defined\n");

    println!("=== Demo Complete ===");

    Ok(())
}
