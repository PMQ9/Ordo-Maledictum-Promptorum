# The Oathbound Engine (Processing Engine Module)

The Oathbound Engine is the core execution layer of Ordo Maledictum Promptorum. It ensures that all AI operations are performed through type-safe function calls rather than raw prompts, preventing prompt injection attacks and ensuring auditability.

## Overview

### Security Guarantees

The processing engine provides critical security guarantees:

1. **No Raw Prompts**: The engine only accepts structured `Intent` types, making it impossible to execute arbitrary text as commands
2. **Type Safety**: All operations are Rust-type-checked at compile time
3. **Explicit Function Calls**: Each action maps to a specific, well-defined function
4. **Auditability**: Every execution is logged with metadata, warnings, and timing information
5. **Structured Results**: All results are returned in a standardized `ProcessingResult` format

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Processing Engine                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Input: Trusted Intent (validated, type-safe)               │
│                        ↓                                      │
│  ┌──────────────────────────────────────────────┐           │
│  │  Action Dispatcher                            │           │
│  │  • MathQuestion     → solve_math_question()  │           │
│  └──────────────────────────────────────────────┘           │
│                        ↓                                      │
│  Output: ProcessingResult (structured, auditable)            │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

### Main Components

#### 1. ProcessingEngine Struct

```rust
pub struct ProcessingEngine {
    config: EngineConfig,
}
```

The main engine responsible for executing intents.

**Methods:**
- `new()`: Create engine with default configuration
- `with_config(config)`: Create engine with custom configuration
- `async fn execute(&self, intent: &Intent) -> Result<ProcessingResult, ProcessingError>`

#### 2. Intent Execution

The `execute` method:
1. Records start time
2. Dispatches to appropriate typed function based on `intent.action`
3. Executes the function with validated parameters
4. Captures execution metadata (timing, function name, warnings)
5. Returns structured `ProcessingResult`

**Example:**
```rust
let engine = ProcessingEngine::new();

let intent = Intent {
    action: Action::MathQuestion,
    topic: Some("What is 2 + 2?".to_string()),
    expertise: vec![],
    constraints: Constraints::default(),
    content_refs: None,
    metadata: None,
};

let result = engine.execute(&intent).await?;
// result.success = true
// result.data = {"answer": "4", "explanation": "2 + 2 = 4", "steps": [...]}
// result.metadata.function_called = "solve_math_question"
```

### Type-Safe Function Implementations

All functions are **mock implementations** demonstrating the pattern. In production, these would integrate with:
- LLM APIs with structured outputs for step-by-step math solutions
- Math computation engines
- Educational content databases

#### solve_math_question()

```rust
fn solve_math_question(
    question: &str,
) -> MathResult
```

Returns a `MathResult` with the answer, explanation, and step-by-step solution.

**Example Output:**
```json
{
  "result": {
    "question": "What is 2 + 2?",
    "answer": "4",
    "explanation": "Adding 2 and 2 together gives us 4.",
    "steps": [
      "Step 1: Start with the first number: 2",
      "Step 2: Add the second number: 2",
      "Step 3: 2 + 2 = 4"
    ],
    "confidence": 1.0,
    "solved_at": "2025-11-23T00:45:00Z"
  }
}
```

## Security Features

### 1. No Raw Prompts Accepted

The type system prevents raw prompts from being executed:

```rust
// ✅ This works - structured intent
let intent = Intent {
    action: Action::MathQuestion,
    topic: Some("What is 15 + 27?".to_string()),
    ...
};
engine.execute(&intent).await;

// ❌ This does NOT compile - type error
let raw_prompt = "What is 15 + 27?";
engine.execute(raw_prompt).await;  // Compile error!
```

### 2. Explicit Action Mapping

Every action is explicitly mapped to a function:

```rust
match intent.action {
    Action::MathQuestion => self.execute_math_question(intent).await,
    // ... only predefined actions can execute
}
```

### 3. Complete Audit Trail

Every execution produces detailed metadata:

```rust
pub struct ProcessingMetadata {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub function_called: String,
    pub warnings: Vec<String>,
}
```

## Usage Example

See `examples/basic_usage.rs` for a complete working example:

```bash
cargo run --example basic_usage
```

**Expected Output:**
```
=== Intent Segregation Processing Engine Demo ===

Example 1: Solving a math question
-----------------------------------
Success: true
Function called: solve_math_question
Duration: 2ms
Result data: {
  "result": {
    "question": "What is 2 + 2?",
    "answer": "4",
    "explanation": "Adding 2 and 2 together gives us 4.",
    "steps": [
      "Step 1: Start with the first number: 2",
      "Step 2: Add the second number: 2",
      "Step 3: 2 + 2 = 4"
    ]
  }
}

Example 2: Type safety demonstration
------------------------------------
The following would NOT compile:
  // let raw_prompt = "What is 2 + 2?";
  // engine.execute(raw_prompt).await;  // ❌ Type error!

Only structured Intent types are accepted, ensuring:
  ✓ No raw prompts can execute privileged actions
  ✓ All inputs are validated and type-checked
  ✓ All operations are traceable and auditable
  ✓ Function calls are explicit and well-defined

=== Demo Complete ===
```

## Testing

The module includes comprehensive unit tests:

```bash
cargo test -p processing-engine
```

**Test Coverage:**
- ✓ Execute solve_math_question with various problems
- ✓ Type safety (no raw prompts)
- ✓ Function result structures
- ✓ Error handling
- ✓ Metadata generation
- ✓ Step-by-step solution validation

## Integration

### With Intent Generator

The processing engine receives `Intent` objects from the Trusted Intent Generator:

```rust
// Intent Generator creates trusted intent
let trusted_intent = intent_generator.generate(voted_intent, config)?;

// Processing Engine executes it
let result = processing_engine.execute(&trusted_intent).await?;
```

### With Intent Ledger

All execution results can be logged to the ledger:

```rust
let result = processing_engine.execute(&intent).await?;
ledger.record_processing_result(&result).await?;
```

## Dependencies

```toml
[dependencies]
intent-schema = { path = "../schema" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
```

## Future Enhancements

### Production Integration

In a production system, replace mock functions with:

1. **LLM Integration with Structured Outputs**
   ```rust
   async fn solve_math_question(question: &str) -> MathResult {
       // Call LLM with JSON schema constraint for step-by-step solutions
       llm.generate_structured::<MathResult>(question, schema).await
   }
   ```

2. **Math Computation Engine Integration**
   ```rust
   async fn solve_math_question(question: &str) -> MathResult {
       // Use symbolic math solver or computational engine
       math_engine.solve(question).await
   }
   ```

3. **Rate Limiting & Quotas**
   ```rust
   pub struct EngineConfig {
       pub max_executions_per_minute: u32,
       pub max_concurrent_executions: u32,
   }
   ```

4. **Advanced Error Recovery**
   ```rust
   pub enum ProcessingError {
       RateLimitExceeded,
       QuotaExceeded,
       UpstreamServiceError,
       ValidationFailed,
       InvalidMathQuestion,
   }
   ```

## License

MIT
