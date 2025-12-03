// Example demonstrating the Processing Engine
//
// This example shows how the processing engine executes trusted intents
// via type-safe function calls, preventing raw prompt injection.

use intent_schema::{Action, Constraints, Intent};
use processing_engine::{EngineConfig, ProcessingEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the processing engine with Claude API key
    let config = EngineConfig {
        verbose: true,
        max_execution_time_ms: 30_000,
        claude_api_key: std::env::var("CLAUDE_API_KEY").ok(),
        claude_model: "claude-3-haiku-20240307".to_string(),
    };
    let engine = ProcessingEngine::with_config(config);

    println!("=== Intent Segregation Processing Engine Demo ===\n");

    // Example 1: Simple Math Question
    println!("Example 1: Simple arithmetic");
    println!("----------------------------");
    let math_intent_1 = Intent {
        action: Action::MathQuestion,
        topic: Some("What is 2 + 2?".to_string()),
        expertise: vec![],
        constraints: Constraints::default(),
        content_refs: None,
        metadata: None,
    };

    let result = engine.execute(&math_intent_1).await?;
    println!("Success: {}", result.success);
    println!("Function called: {}", result.metadata.function_called);
    println!("Duration: {}ms", result.metadata.duration_ms);
    if result.success {
        if let Some(question) = result.data.get("question") {
            println!("Question: {}", question);
        }
        if let Some(answer) = result.data.get("answer") {
            println!("Answer: {}\n", answer);
        }
    } else if let Some(error) = result.error {
        println!("Error: {}\n", error);
    }

    // Example 2: Algebra Question
    println!("Example 2: Algebra problem");
    println!("--------------------------");
    let math_intent_2 = Intent {
        action: Action::MathQuestion,
        topic: Some("Solve for x: 3x + 5 = 20".to_string()),
        expertise: vec![],
        constraints: Constraints::default(),
        content_refs: None,
        metadata: None,
    };

    let result = engine.execute(&math_intent_2).await?;
    println!("Success: {}", result.success);
    println!("Function called: {}", result.metadata.function_called);
    println!("Duration: {}ms", result.metadata.duration_ms);
    if result.success {
        if let Some(question) = result.data.get("question") {
            println!("Question: {}", question);
        }
        if let Some(answer) = result.data.get("answer") {
            println!("Answer: {}\n", answer);
        }
    } else if let Some(error) = result.error {
        println!("Error: {}\n", error);
    }

    // Example 3: Geometry Question
    println!("Example 3: Geometry calculation");
    println!("--------------------------------");
    let math_intent_3 = Intent {
        action: Action::MathQuestion,
        topic: Some("Calculate the area of a circle with radius 5".to_string()),
        expertise: vec![],
        constraints: Constraints::default(),
        content_refs: None,
        metadata: None,
    };

    let result = engine.execute(&math_intent_3).await?;
    println!("Success: {}", result.success);
    println!("Function called: {}", result.metadata.function_called);
    println!("Duration: {}ms", result.metadata.duration_ms);
    if result.success {
        if let Some(question) = result.data.get("question") {
            println!("Question: {}", question);
        }
        if let Some(answer) = result.data.get("answer") {
            println!("Answer: {}\n", answer);
        }
    } else if let Some(error) = result.error {
        println!("Error: {}\n", error);
    }

    // Example 4: Demonstrating Type Safety
    println!("Example 4: Type safety demonstration");
    println!("------------------------------------");
    println!("The following would NOT compile:");
    println!("  // let raw_prompt = \"What is 2 + 2?\";");
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
