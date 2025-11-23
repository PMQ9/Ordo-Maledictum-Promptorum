# Intent Parsers Module

This module provides an ensemble of intent parsers for extracting structured intents from user input. The ensemble includes:

1. **DeterministicParser** - Rule-based, zero-hallucination parser
2. **OllamaParser** - Local LLM parser using Ollama
3. **OpenAIParser** - Cloud LLM parser using OpenAI API

## Architecture

All parsers implement the `IntentParser` trait:

```rust
#[async_trait]
pub trait IntentParser: Send + Sync {
    async fn parse(&self, user_input: &str) -> ParserResult<ParsedIntent>;
    fn parser_type(&self) -> ParserType;
    fn parser_id(&self) -> String;
    fn trust_level(&self) -> f64;
}
```

The `ParserEnsemble` runs all enabled parsers in parallel for maximum throughput and redundancy.

## Parsers

### 1. DeterministicParser

**Trust Level:** 1.0 (Highest)

A rule-based parser that uses keyword matching and regex patterns to extract intents. No LLM involvement means zero hallucination risk.

**Features:**
- Keyword-based action detection
- Regex-based topic extraction
- Budget and constraint parsing
- Expertise area detection
- Instant results (<1ms typically)

**Example:**
```rust
use intent_parsers::DeterministicParser;

let parser = DeterministicParser::new();
let result = parser.parse("Find experts in ML with budget $50000").await?;

println!("Action: {:?}", result.intent.action);
println!("Expertise: {:?}", result.intent.expertise);
```

### 2. OllamaParser

**Trust Level:** 0.75 (Medium-High)

Uses a local Ollama instance to parse intents using LLMs like Llama2 or Mistral.

**Features:**
- Local inference (no data leaves your infrastructure)
- JSON mode for structured output
- Temperature 0 for deterministic results
- Configurable model selection
- Typical response time: 500-2000ms

**Configuration:**
```rust
use intent_parsers::{OllamaParser, OllamaConfig};

let config = OllamaConfig {
    endpoint: "http://localhost:11434".to_string(),
    model: "llama2".to_string(),
    temperature: 0.0,
    timeout_secs: 30,
};

let parser = OllamaParser::new(config);
let result = parser.parse("Summarize blockchain security research").await?;
```

**Supported Models:**
- llama2
- mistral
- mixtral
- codellama
- Any model available in your Ollama instance

### 3. OpenAIParser

**Trust Level:** 0.8 (Medium-High)

Uses OpenAI's API for intent parsing with models like GPT-4o-mini.

**Features:**
- High-quality parsing with latest GPT models
- JSON mode for structured output
- Temperature 0 for consistency
- Fast response times (300-800ms)
- Requires API key

**Configuration:**
```rust
use intent_parsers::{OpenAIParser, OpenAIConfig};

let config = OpenAIConfig {
    api_key: "sk-your-api-key".to_string(),
    model: "gpt-4o-mini".to_string(),
    temperature: 0.0,
    timeout_secs: 30,
    base_url: "https://api.openai.com/v1".to_string(),
};

let parser = OpenAIParser::new(config);
let result = parser.parse("Draft a proposal for AI security audit").await?;
```

## ParserEnsemble

The `ParserEnsemble` runs multiple parsers in parallel and collects results from all of them.

### Basic Usage

```rust
use intent_parsers::{ParserEnsemble, ParserConfig};

// Create from configuration
let config = ParserConfig::from_env()?;
let ensemble = ParserEnsemble::new(config);

// Parse with all enabled parsers
let result = ensemble.parse_all("Find cloud security experts").await;

println!("Success: {}/{} parsers", result.success_count, result.parsers_count);
println!("Total time: {}ms", result.total_time_ms);

// Access specific parser results
if let Some(det_result) = result.get_deterministic() {
    println!("Deterministic: {:?}", det_result.intent.action);
}

if let Some(ollama_result) = result.get_ollama() {
    println!("Ollama: {:?}", ollama_result.intent.action);
}

if let Some(openai_result) = result.get_openai() {
    println!("OpenAI: {:?}", openai_result.intent.action);
}

// Get highest confidence result
if let Some(best) = result.get_highest_confidence() {
    println!("Best result from: {}", best.parser_id);
    println!("Confidence: {}", best.confidence);
}

// Get highest trust result (usually deterministic)
if let Some(most_trusted) = result.get_highest_trust() {
    println!("Most trusted: {}", most_trusted.parser_id);
}
```

### Configuration from Environment

```rust
use intent_parsers::ParserConfig;

// Load from environment variables
let config = ParserConfig::from_env()?;
let ensemble = ParserEnsemble::new(config);
```

Environment variables:
- `ENABLE_DETERMINISTIC=true|false`
- `ENABLE_OLLAMA=true|false`
- `ENABLE_OPENAI=true|false`
- `OLLAMA_ENDPOINT=http://localhost:11434`
- `OLLAMA_MODEL=llama2`
- `OPENAI_API_KEY=sk-...`
- `OPENAI_MODEL=gpt-4o-mini`

### Custom Configuration

```rust
use intent_parsers::{ParserConfig, OllamaConfig, OpenAIConfig, ParserEnsemble};

let config = ParserConfig {
    enable_deterministic: true,
    enable_ollama: true,
    enable_openai: false,  // Disable OpenAI
    ollama: OllamaConfig {
        endpoint: "http://localhost:11434".to_string(),
        model: "mistral".to_string(),
        temperature: 0.0,
        timeout_secs: 45,
    },
    openai: OpenAIConfig::default(),
};

let ensemble = ParserEnsemble::new(config);
```

## Complete Example

```rust
use intent_parsers::{ParserEnsemble, ParserConfig};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = ParserConfig::from_env()?;

    // Create ensemble
    let ensemble = ParserEnsemble::new(config);

    // User input
    let user_input = "Find top 5 machine learning experts with budget $50,000";

    // Parse with all parsers in parallel
    let result = ensemble.parse_all(user_input).await;

    println!("\n=== Ensemble Results ===");
    println!("Parsers: {}/{} succeeded", result.success_count, result.parsers_count);
    println!("Total time: {}ms", result.total_time_ms);

    // Display all successful results
    for parsed in &result.results {
        println!("\n--- {} ---", parsed.parser_id);
        println!("Action: {:?}", parsed.intent.action);
        println!("Topic: {:?}", parsed.intent.topic);
        println!("Expertise: {:?}", parsed.intent.expertise);
        println!("Constraints: {}", parsed.intent.constraints);
        println!("Confidence: {:.2}", parsed.confidence);
        println!("Trust: {:.2}", parsed.trust_level);
        println!("Parse time: {}ms", parsed.parsing_time_ms);
    }

    // Display errors
    if !result.errors.is_empty() {
        println!("\n=== Errors ===");
        for (parser_id, error) in &result.errors {
            println!("{}: {}", parser_id, error);
        }
    }

    // Use highest trust result (deterministic if available)
    if let Some(trusted) = result.get_highest_trust() {
        println!("\n=== Using Most Trusted Result ===");
        println!("Parser: {}", trusted.parser_id);
        println!("Intent: {:#?}", trusted.intent);
    }

    Ok(())
}
```

## Testing

Each parser includes comprehensive unit tests:

```bash
# Test all parsers
cargo test -p intent-parsers

# Test specific parser
cargo test -p intent-parsers deterministic
cargo test -p intent-parsers ollama
cargo test -p intent-parsers openai

# Test ensemble
cargo test -p intent-parsers ensemble
```

## Performance

Typical parsing times (on modest hardware):

| Parser | Time | Trust | Pros | Cons |
|--------|------|-------|------|------|
| Deterministic | <1ms | 1.0 | Instant, zero hallucination | Limited understanding |
| Ollama (llama2) | 500-2000ms | 0.75 | Local, private | Slower, requires setup |
| OpenAI (gpt-4o-mini) | 300-800ms | 0.8 | Fast, high quality | Requires API key, costs money |

**Ensemble:** All parsers run in parallel, so total time H slowest parser (~500-2000ms typically)

## Integration with Voting Module

The ensemble results can be passed to the voting module:

```rust
let ensemble_result = ensemble.parse_all(user_input).await;
let parser_results = ensemble_result.to_parser_results();

// Pass to voting module for consensus analysis
let voting_result = voting_module.vote(parser_results).await?;
```

## Error Handling

All parsers return `Result<ParsedIntent, ParserError>`:

```rust
pub enum ParserError {
    HttpError(reqwest::Error),
    JsonError(serde_json::Error),
    ParseError(String),
    ApiError(String),
    ConfigError(String),
    TimeoutError,
    InvalidInput(String),
}
```

The ensemble collects errors from failed parsers but continues with successful ones:

```rust
let result = ensemble.parse_all(input).await;

// Check for errors
for (parser_id, error) in &result.errors {
    match error {
        ParserError::TimeoutError => {
            println!("{} timed out", parser_id);
        }
        ParserError::ApiError(msg) => {
            println!("{} API error: {}", parser_id, msg);
        }
        _ => {
            println!("{} failed: {}", parser_id, error);
        }
    }
}

// Continue with successful results
if result.success_count > 0 {
    // Process results...
}
```

## Security Considerations

1. **DeterministicParser**: Highest security - no external calls, no hallucination
2. **OllamaParser**: Medium security - local inference, data stays private
3. **OpenAIParser**: Data sent to OpenAI - review privacy policy

Always use the ensemble approach to have deterministic fallback and cross-validation.

## Future Enhancements

- [ ] Add support for more LLM providers (Anthropic, Cohere, etc.)
- [ ] Implement caching for repeated queries
- [ ] Add parser health checks and circuit breakers
- [ ] Support for streaming responses
- [ ] Fine-tuned models for specific domains
