use intent_parsers::{EnsembleResult, ParserConfig, ParserEnsemble};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("intent_parsers=debug")
        .init();

    println!("=== Intent Parser Ensemble Example ===\n");

    // Load configuration from environment variables
    // Make sure to set OPENAI_API_KEY if you want to use OpenAI parser
    let config = ParserConfig::from_env()?;

    // Create the parser ensemble
    let ensemble = ParserEnsemble::new(config);

    println!(
        "Ensemble initialized with {} parsers\n",
        ensemble.parser_count()
    );

    // Example 1: Simple arithmetic
    println!("--- Example 1: Simple Arithmetic ---");
    let input1 = "What is 2 + 2?";
    let result1 = ensemble
        .parse_all(input1, "demo_user", "demo_session_1")
        .await;
    display_results(&result1);

    // Example 2: Algebra
    println!("\n--- Example 2: Algebra ---");
    let input2 = "Solve for x: 3x + 5 = 20";
    let result2 = ensemble
        .parse_all(input2, "demo_user", "demo_session_2")
        .await;
    display_results(&result2);

    // Example 3: Geometry
    println!("\n--- Example 3: Geometry ---");
    let input3 = "Calculate the area of a circle with radius 5";
    let result3 = ensemble
        .parse_all(input3, "demo_user", "demo_session_3")
        .await;
    display_results(&result3);

    // Example 4: Calculus
    println!("\n--- Example 4: Calculus ---");
    let input4 = "Find the derivative of f(x) = x^2 + 3x";
    let result4 = ensemble
        .parse_all(input4, "demo_user", "demo_session_4")
        .await;
    display_results(&result4);

    // Example 5: Complex calculation
    println!("\n--- Example 5: Complex Calculation ---");
    let input5 = "What is the value of the integral of sin(x) from 0 to pi?";
    let result5 = ensemble
        .parse_all(input5, "demo_user", "demo_session_5")
        .await;
    display_results(&result5);

    Ok(())
}

fn display_results(result: &EnsembleResult) {
    println!("\nEnsemble Statistics:");
    println!(
        "  Success: {}/{} parsers",
        result.success_count, result.parsers_count
    );
    println!("  Total time: {}ms", result.total_time_ms);

    // Display successful results
    if !result.results.is_empty() {
        println!("\nParser Results:");
        for parsed in &result.results {
            println!(
                "\n  {} (confidence: {:.2})",
                parsed.parser_id, parsed.confidence
            );
            println!("    Action: {}", parsed.intent.action);
            println!("    Topic ID: {}", parsed.intent.topic_id);

            if !parsed.intent.expertise.is_empty() {
                println!("    Expertise: {:?}", parsed.intent.expertise);
            }

            if !parsed.intent.constraints.is_empty() {
                println!("    Constraints:");
                for (key, value) in &parsed.intent.constraints {
                    println!("      {}: {}", key, value);
                }
            }
        }
    }

    // Display errors
    if !result.errors.is_empty() {
        println!("\nErrors:");
        for (parser_id, error) in &result.errors {
            println!("  {}: {}", parser_id, error);
        }
    }

    // Show recommendation
    if let Some(priority) = result.get_by_priority() {
        println!("\nRecommended Result (by priority):");
        println!("  Parser: {}", priority.parser_id);
        println!("  Action: {}", priority.intent.action);
        println!("  Topic: {}", priority.intent.topic_id);
    }

    if let Some(confident) = result.get_highest_confidence() {
        println!("\nMost Confident Result:");
        println!("  Parser: {}", confident.parser_id);
        println!("  Confidence: {:.2}", confident.confidence);
    }

    println!("\n{}", "-".repeat(60));
}
