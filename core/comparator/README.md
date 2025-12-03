# The Judicator of Concordance (Intent Comparator)

The Judicator of Concordance module validates user intents against provider-defined policies (The Edict of the High Magister) and constraints. It ensures that requested actions, expertise areas, and resource constraints fall within allowed boundaries.

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
        allowed_actions: vec!["math_question".to_string()],
        allowed_expertise: vec![],
        max_budget: None,
        allowed_domains: vec![],
    };

    // Create an intent
    let constraints = HashMap::new();

    let intent = Intent {
        action: "math_question".to_string(),
        topic_id: "algebra".to_string(),
        expertise: vec![],
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
  "allowed_actions": ["math_question"],
  "allowed_expertise": []
}
```

**User Intent:**
```json
{
  "action": "math_question",
  "topic_id": "algebra",
  "expertise": []
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
  "allowed_actions": ["math_question"],
  "allowed_expertise": []
}
```

**User Intent:**
```json
{
  "action": "find_experts",
  "topic_id": "security"
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 1 violation(s) found
  - Action 'find_experts' is not in the allowed actions list. Allowed actions: ["math_question"]
```

**Reason:** The action "find_experts" is not in the allowed actions list.

---

### Example 3: Expertise Not Allowed (Hard Mismatch)

**Provider Config:**
```json
{
  "allowed_actions": ["math_question"],
  "allowed_expertise": ["algebra"]
}
```

**User Intent:**
```json
{
  "action": "math_question",
  "expertise": ["algebra", "calculus"]
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 1 violation(s) found
  - Requested expertise areas not allowed: ["calculus"]. Allowed expertise: ["algebra"]
```

**Reason:** The expertise "calculus" is not in the allowed expertise list.

---

### Example 4: Multiple Math Topics (Approved)

**Provider Config:**
```json
{
  "allowed_actions": ["math_question"],
  "allowed_expertise": []
}
```

**User Intent:**
```json
{
  "action": "math_question",
  "topic_id": "arithmetic"
}
```

**Result:**
```
✓ Approved - Intent approved - all checks passed
```

**Reason:** Math question with arithmetic topic is allowed.

---

### Example 5: Multiple Violations (Hard Mismatch)

**Provider Config:**
```json
{
  "allowed_actions": ["math_question"],
  "allowed_expertise": ["algebra"]
}
```

**User Intent:**
```json
{
  "action": "find_experts",
  "expertise": ["calculus"]
}
```

**Result:**
```
✗ HardMismatch - Intent denied - 2 violation(s) found
  - Action 'find_experts' is not in the allowed actions list. Allowed actions: ["math_question"]
  - Requested expertise areas not allowed: ["calculus"]. Allowed expertise: ["algebra"]
```

**Reason:** Multiple violations:
1. Action not allowed
2. Expertise not allowed

---

### Example 6: Empty Expertise Allowed (Approved)

**Provider Config:**
```json
{
  "allowed_actions": ["math_question"],
  "allowed_expertise": []
}
```

**User Intent:**
```json
{
  "action": "math_question",
  "expertise": []
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
- **CustomConstraintViolation**: Custom constraint validation failed

## Severity Levels

- **Critical**: Immediate security concerns (e.g., disallowed actions)
- **High**: Significant policy violations
- **Medium**: Moderate concerns requiring review
- **Low**: Minor issues or warnings

## Integration

The Judicator of Concordance integrates seamlessly with Ordo Maledictum Promptorum architecture:

```
User Input → Sacrificial Testing → Council of the Oracular Cogitors → Voting Module → The Judicator of Concordance → Trusted Intent Generator
                                                                                                  ↓
                                                                              The Edict of the High Magister
```

The Judicator sits between the voting module and the trusted intent generator, ensuring only validated intents proceed to execution. When policies are violated, approval from The Overseer-Prime is required.

## Testing

Run the test suite:

```bash
cargo test -p intent-comparator
```

All tests validate different scenarios including approved intents, action violations, expertise violations, budget violations, and combinations thereof.
