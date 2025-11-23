# Intent Voting Module

The voting module compares outputs from multiple intent parsers (deterministic and LLM-based) to determine the canonical intent with a confidence level.

## Purpose

In a prompt injection defense system, we can't trust a single parser (especially LLM-based ones) to always correctly extract user intent. The voting module:

1. **Compares multiple parser outputs** to detect discrepancies
2. **Calculates similarity scores** using smart diffing
3. **Determines confidence level** based on agreement
4. **Selects canonical intent** (preferring deterministic parser when available)
5. **Flags conflicts** for human review

## Confidence Levels

### High Confidence (≥95% similarity)
- All parsers strongly agree
- Intent can be processed automatically
- No human review needed

### Low Confidence (75-95% similarity)
- Minor discrepancies detected
- Falls back to deterministic parser
- May request user confirmation
- No immediate human review unless policy requires it

### Conflict (<75% similarity)
- Major discrepancies detected
- Escalates to human review
- Still returns deterministic parser result as fallback
- Execution blocked until approved

## Architecture

```
┌─────────────────┐
│ Parser Results  │
│ - Deterministic │
│ - LLM Parser 1  │
│ - LLM Parser 2  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Pairwise Diffs  │
│ - Calculate     │
│   similarity    │
│ - Identify      │
│   differences   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Confidence      │
│ Determination   │
│ - High/Low/     │
│   Conflict      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Canonical       │
│ Intent Selection│
│ - Prefer        │
│   deterministic │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Voting Result   │
│ + Explanation   │
└─────────────────┘
```

## Smart Diffing

The module uses weighted similarity scoring:

- **Action** (weight: 3.0) - Most important field
- **Topic** (weight: 2.0) - Word overlap analysis
- **Expertise** (weight: 2.0) - Set similarity with semantic matching
- **Constraints** (weight: 1.5) - Numeric tolerance

## Usage

```rust
use intent_voting::{VotingModule, ParserResult};
use intent_schema::{Intent, Action, Expertise};

#[tokio::main]
async fn main() {
    let voting = VotingModule::new();
    
    let results = vec![
        ParserResult {
            parser_name: "deterministic".to_string(),
            is_deterministic: true,
            intent: /* ... */,
            parser_confidence: Some(1.0),
        },
        ParserResult {
            parser_name: "llm1".to_string(),
            is_deterministic: false,
            intent: /* ... */,
            parser_confidence: Some(0.9),
        },
    ];
    
    match voting.vote(results).await {
        Ok(result) => {
            println!("Confidence: {:?}", result.confidence);
            println!("Canonical: {:?}", result.canonical_intent);
            
            if result.requires_human_review {
                // Escalate to supervisor
            }
        }
        Err(e) => eprintln!("Voting error: {}", e),
    }
}
```

## Example Scenarios

Run the example scenarios to see the voting module in action:

```bash
cargo run --example voting_scenarios
```

This demonstrates:
1. **High Confidence** - All parsers agree
2. **Low Confidence** - Minor discrepancies
3. **Conflict** - Major differences requiring review
4. **Prompt Injection** - How voting neutralizes attacks

## Security Properties

1. **Deterministic Fallback**: Always prefers the rule-based parser when available
2. **Multi-Parser Validation**: Requires agreement among independent parsers
3. **Human-in-the-Loop**: Escalates suspicious discrepancies
4. **Audit Trail**: Returns detailed comparison data for logging

## Testing

```bash
# Run unit tests
cargo test -p intent-voting

# Run with output
cargo test -p intent-voting -- --nocapture
```

## Configuration

Custom thresholds can be set:

```rust
let voting = VotingModule::with_thresholds(
    0.98,  // High confidence threshold
    0.80,  // Low confidence threshold
);
```

## Integration

The voting module integrates with:
- **Parser Ensemble** - Receives parser results
- **Intent Ledger** - Logs voting decisions
- **Supervision Module** - Escalates conflicts
- **Processing Engine** - Provides validated intents
