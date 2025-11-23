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

    // Example 1: Find experts
    println!("--- Example 1: Find Experts ---");
    let input1 = "Find top 5 machine learning experts with budget $50,000";
    let result1 = ensemble
        .parse_all(input1, "demo_user", "demo_session_1")
        .await;
    display_results(&result1);

    // Example 2: Summarize
    println!("\n--- Example 2: Summarize ---");
    let input2 = "Summarize the latest research on blockchain security";
    let result2 = ensemble
        .parse_all(input2, "demo_user", "demo_session_2")
        .await;
    display_results(&result2);

    // Example 3: Draft proposal
    println!("\n--- Example 3: Draft Proposal ---");
    let input3 = "Draft a proposal for AI security audit with maximum 10 results";
    let result3 = ensemble
        .parse_all(input3, "demo_user", "demo_session_3")
        .await;
    display_results(&result3);

    // Example 4: Research
    println!("\n--- Example 4: Research ---");
    let input4 = "Research quantum computing applications in cybersecurity";
    let result4 = ensemble
        .parse_all(input4, "demo_user", "demo_session_4")
        .await;
    display_results(&result4);

    // Example 5: Complex query
    println!("\n--- Example 5: Complex Query ---");
    let input5 = "Find embedded systems experts specializing in IoT security, budget: $75,000, top 3 results";
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
