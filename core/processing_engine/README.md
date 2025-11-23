# Processing Engine Module

The Processing Engine is the core execution layer of the Intent Segregation Architecture. It ensures that all AI operations are performed through type-safe function calls rather than raw prompts, preventing prompt injection attacks and ensuring auditability.

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
│  │  • FindExperts      → find_experts()         │           │
│  │  • Summarize        → summarize_document()   │           │
│  │  • DraftProposal    → draft_proposal()       │           │
│  │  • AnalyzeDocument  → analyze_document()     │           │
│  │  • GenerateReport   → generate_report()      │           │
│  │  • SearchKnowledge  → search_knowledge()     │           │
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

let result = engine.execute(&intent).await?;
// result.success = true
// result.data = {"experts": [...], "count": 2}
// result.metadata.function_called = "find_experts"
```

### Type-Safe Function Implementations

All functions are **mock implementations** demonstrating the pattern. In production, these would integrate with:
- Expert databases
- Document processing pipelines
- LLM APIs with structured outputs
- Knowledge bases

#### find_experts()

```rust
fn find_experts(
    topic: Option<String>,
    expertise: Vec<Expertise>,
    max_results: u32,
    max_budget: Option<u64>,
) -> Vec<Expert>
```

Returns a list of `Expert` structs matching the criteria.

**Example Output:**
```json
{
  "experts": [
    {
      "id": "exp_001",
      "name": "Dr. Sarah Chen",
      "expertise": ["security", "cloud"],
      "availability": true,
      "hourly_rate": 250,
      "confidence_score": 0.95,
      "bio": "Expert in cloud security with 15 years of experience",
      "years_experience": 15
    }
  ],
  "count": 1
}
```

#### summarize_document()

```rust
fn summarize_document(
    document_id: &str,
    topic: Option<String>
) -> DocumentSummary
```

Returns a `DocumentSummary` with key points and analysis.

**Example Output:**
```json
{
  "summary": {
    "document_id": "doc_123",
    "title": "Document: doc_123",
    "summary": "This document covers cybersecurity trends. It provides comprehensive analysis...",
    "key_points": [
      "Key finding 1: Market analysis shows strong growth potential",
      "Key finding 2: Risk mitigation strategies are essential",
      "Key finding 3: Timeline estimates are optimistic but achievable"
    ],
    "word_count": 2500,
    "confidence": 0.89,
    "generated_at": "2025-11-23T00:45:00Z"
  }
}
```

#### draft_proposal()

```rust
fn draft_proposal(
    topic: Option<String>,
    expertise: Vec<Expertise>,
    budget: Option<u64>,
) -> Proposal
```

Returns a `Proposal` with structured sections.

**Example Output:**
```json
{
  "proposal": {
    "id": "prop_a1b2c3d4",
    "title": "Proposal: AI integration project",
    "sections": [
      {
        "heading": "Executive Summary",
        "content": "This proposal outlines a comprehensive approach to AI integration project...",
        "order": 1
      },
      {
        "heading": "Scope of Work",
        "content": "Detailed breakdown of deliverables, milestones, and timeline.",
        "order": 2
      },
      {
        "heading": "Team and Expertise",
        "content": "Our team comprises industry-leading experts with proven track records.",
        "order": 3
      },
      {
        "heading": "Budget and Timeline",
        "content": "Estimated budget: $75000. Timeline: 12-16 weeks.",
        "order": 4
      }
    ],
    "created_at": "2025-11-23T00:45:00Z",
    "estimated_budget": 75000,
    "timeline_weeks": 14
  }
}
```

## Security Features

### 1. No Raw Prompts Accepted

The type system prevents raw prompts from being executed:

```rust
// ✅ This works - structured intent
let intent = Intent {
    action: Action::FindExperts,
    topic: Some("blockchain".to_string()),
    ...
};
engine.execute(&intent).await;

// ❌ This does NOT compile - type error
let raw_prompt = "Find me some experts in blockchain";
engine.execute(raw_prompt).await;  // Compile error!
```

### 2. Explicit Action Mapping

Every action is explicitly mapped to a function:

```rust
match intent.action {
    Action::FindExperts => self.execute_find_experts(intent).await,
    Action::Summarize => self.execute_summarize(intent).await,
    Action::DraftProposal => self.execute_draft_proposal(intent).await,
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

Example 1: Finding security experts
-----------------------------------
Success: true
Function called: find_experts
Duration: 2ms
Result data: {
  "experts": [...],
  "count": 2
}

Example 2: Summarizing a document
----------------------------------
Success: true
Function called: summarize_document
Duration: 1ms
Result data: {
  "summary": {...}
}

Example 3: Drafting a proposal
-------------------------------
Success: true
Function called: draft_proposal
Duration: 1ms
Result data: {
  "proposal": {...}
}

Example 4: Type safety demonstration
------------------------------------
The following would NOT compile:
  // let raw_prompt = "Find me experts in security";
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
- ✓ Execute find_experts with budget filtering
- ✓ Execute summarize with document validation
- ✓ Execute draft_proposal with warnings
- ✓ Type safety (no raw prompts)
- ✓ Function result structures
- ✓ Error handling
- ✓ Metadata generation

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

1. **Real Expert Database Integration**
   ```rust
   async fn find_experts(...) -> Vec<Expert> {
       // Query PostgreSQL or similar
       database.query_experts(topic, expertise).await
   }
   ```

2. **LLM Integration with Structured Outputs**
   ```rust
   async fn summarize_document(...) -> DocumentSummary {
       // Call LLM with JSON schema constraint
       llm.generate_structured::<DocumentSummary>(prompt, schema).await
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
   }
   ```

## License

MIT
