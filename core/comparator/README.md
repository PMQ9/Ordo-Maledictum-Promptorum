# Intent Comparator

The Intent Comparator module validates user intents against provider-defined policies and constraints. It ensures that requested actions, expertise areas, and resource constraints fall within allowed boundaries.

## Features

- **Action Validation**: Checks if requested actions are in the allowed actions list
- **Expertise Validation**: Validates that requested expertise areas are permitted
- **Budget Constraints**: Ensures budget requests don't exceed maximum allowed values
- **Flexible Results**: Returns Approved, SoftMismatch, or HardMismatch with detailed reasons
- **Strict Mode**: Optional strict validation mode for enhanced security
- **Async Support**: Built with async/await for integration with modern Rust applications

## Usage

### Basic Example

```rust
use intent_comparator::{IntentComparator, ComparisonResult};
use intent_schema::{Intent, ProviderConfig, IntentMetadata};
use std::collections::HashMap;
use serde_json::json;

#[tokio::main]
async fn main() {
    // Create a provider configuration
    let config = ProviderConfig {
        allowed_actions: vec!["find_experts".to_string(), "summarize".to_string()],
        allowed_expertise: vec!["security".to_string(), "ml".to_string()],
        max_budget: Some(50000),
        allowed_domains: vec![],
    };

    // Create an intent
    let mut constraints = HashMap::new();
    constraints.insert("max_budget".to_string(), json!(20000));

    let intent = Intent {
        action: "find_experts".to_string(),
        topic_id: "supply_chain_risk".to_string(),
        expertise: vec!["security".to_string()],
        constraints,
        content_refs: vec![],
        metadata: IntentMetadata {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_id: "user_123".to_string(),
            session_id: "session_456".to_string(),
        },
    };

    // Compare intent against config
    let comparator = IntentComparator::new();
    let result = comparator.compare(&intent, &config).await.unwrap();

    match result {
        ComparisonResult::Approved { message } => {
            println!("✓ {}", message);
        }
        ComparisonResult::SoftMismatch { reasons, message } => {
            println!("⚠ {}", message);
            for reason in reasons {
                println!("  - {}", reason.description);
            }
        }
        ComparisonResult::HardMismatch { reasons, message } => {
            println!("✗ {}", message);
            for reason in reasons {
                println!("  - {}", reason.description);
            }
        }
    }
}
```

## Comparison Examples

### Example 1: Approved Intent

**Provider Config:**
```json
{
  "allowed_actions": ["find_experts", "summarize"],
  "allowed_expertise": ["security", "ml"],
  "max_budget": 50000
}
```

**User Intent:**
```json
{
  "action": "find_experts",
  "topic_id": "supply_chain_risk",
  "expertise": ["security"],
  "constraints": {
    "max_budget": 20000
  }
}
```

**Result:**
```
✓ Approved - Intent approved - all checks passed
```

---

### Example 2: Action Not Allowed (Hard Mismatch)

**Provider Config:**
```json
{
  "allowed_actions": ["find_experts", "summarize"],
  "allowed_expertise": ["security", "ml"],
  "max_budget": 50000
}
```

**User Intent:**
```json
{
  "action": "draft_proposal",
  "topic_id": "new_project"
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 1 violation(s) found
  - Action 'draft_proposal' is not in the allowed actions list. Allowed actions: ["find_experts", "summarize"]
```

**Reason:** The action "draft_proposal" is not in the allowed actions list.

---

### Example 3: Expertise Not Allowed (Hard Mismatch)

**Provider Config:**
```json
{
  "allowed_actions": ["find_experts"],
  "allowed_expertise": ["security", "ml"],
  "max_budget": 50000
}
```

**User Intent:**
```json
{
  "action": "find_experts",
  "expertise": ["security", "frontend"]
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 1 violation(s) found
  - Requested expertise areas not allowed: ["frontend"]. Allowed expertise: ["security", "ml"]
```

**Reason:** The expertise "frontend" is not in the allowed expertise list.

---

### Example 4: Budget Exceeded (Hard Mismatch)

**Provider Config:**
```json
{
  "allowed_actions": ["find_experts"],
  "allowed_expertise": ["security"],
  "max_budget": 50000
}
```

**User Intent:**
```json
{
  "action": "find_experts",
  "constraints": {
    "max_budget": 100000
  }
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 1 violation(s) found
  - Requested budget $100000 exceeds maximum allowed budget $50000
```

**Reason:** The requested budget of $100,000 exceeds the maximum allowed budget of $50,000.

---

### Example 5: Multiple Violations (Hard Mismatch)

**Provider Config:**
```json
{
  "allowed_actions": ["find_experts", "summarize"],
  "allowed_expertise": ["security", "ml"],
  "max_budget": 50000
}
```

**User Intent:**
```json
{
  "action": "draft_proposal",
  "expertise": ["frontend"],
  "constraints": {
    "max_budget": 200000
  }
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 3 violation(s) found
  - Action 'draft_proposal' is not in the allowed actions list. Allowed actions: ["find_experts", "summarize"]
  - Requested expertise areas not allowed: ["frontend"]. Allowed expertise: ["security", "ml"]
  - Requested budget $200000 exceeds maximum allowed budget $50000
```

**Reason:** Multiple violations:
1. Action not allowed
2. Expertise not allowed  
3. Budget exceeded

---

### Example 6: Empty Expertise Allowed (Approved)

**Provider Config:**
```json
{
  "allowed_actions": ["find_experts"],
  "allowed_expertise": [],
  "max_budget": 50000
}
```

**User Intent:**
```json
{
  "action": "find_experts",
  "expertise": ["frontend", "backend", "anything"]
}
```

**Result:**
```
✓ Approved - Intent approved - all checks passed
```

**Reason:** An empty `allowed_expertise` list means no restrictions on expertise areas.

---

## Strict Mode

Enable strict mode for enhanced security:

```rust
let comparator = IntentComparator::new_strict();
```

In strict mode, Medium and High severity violations are treated as hard mismatches instead of soft mismatches.

## ComparisonResult

The comparator returns one of three result types:

- **Approved**: Intent fully complies with all constraints
- **SoftMismatch**: Minor issues that may require user confirmation
- **HardMismatch**: Critical violations that should block the intent

Each result includes:
- `message`: Human-readable summary
- `reasons`: Detailed list of mismatches (for SoftMismatch and HardMismatch)

## Mismatch Categories

- **ActionNotAllowed**: Requested action is not in allowed actions list
- **ExpertiseNotAllowed**: Requested expertise areas are not permitted
- **BudgetExceeded**: Budget constraint exceeds maximum allowed
- **CustomConstraintViolation**: Custom constraint validation failed

## Severity Levels

- **Critical**: Immediate security concerns (e.g., disallowed actions)
- **High**: Significant policy violations
- **Medium**: Moderate concerns requiring review
- **Low**: Minor issues or warnings

## Integration

The comparator integrates seamlessly with the Intent Segregation Architecture:

```
User Input → Parser Ensemble → Voting Module → Intent Comparator → Trusted Intent Generator
                                                         ↓
                                                  Provider Config
```

The comparator sits between the voting module and the trusted intent generator, ensuring only validated intents proceed to execution.

## Testing

Run the test suite:

```bash
cargo test -p intent-comparator
```

All tests validate different scenarios including approved intents, action violations, expertise violations, budget violations, and combinations thereof.
