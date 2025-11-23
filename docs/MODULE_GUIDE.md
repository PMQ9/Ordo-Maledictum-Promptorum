# Module Guide

Detailed documentation for each module in the Intent Segregation Cybersecurity Architecture.

## Table of Contents

1. [Schema Module](#schema-module)
2. [Malicious Detector Module](#malicious-detector-module)
3. [Parser Module](#parser-module)
4. [Voting Module](#voting-module)
5. [Comparator Module](#comparator-module)
6. [Intent Generator Module](#intent-generator-module)
7. [Processing Engine Module](#processing-engine-module)
8. [Supervision Module](#supervision-module)
9. [Notifications Module](#notifications-module)
10. [Ledger Module](#ledger-module)

---

## Schema Module

**Location**: `core/schema/`

### Purpose

Defines shared types, structures, and schemas used across all modules. This ensures type consistency and provides a single source of truth for intent representation.

### Responsibilities

- Define core intent types (`Intent`, `Action`, `Expertise`)
- Provide JSON schema validation
- Type conversions and serialization
- Shared error types

### Public API

#### Core Types

```rust
use intent_schema::{Intent, Action, Expertise, Constraint};

// Intent structure
pub struct Intent {
    pub action: Action,
    pub topic: Option<String>,
    pub expertise: Vec<Expertise>,
    pub constraints: serde_json::Value,
    pub content_refs: Vec<String>,
}

// Actions (enum)
pub enum Action {
    FindExperts,
    Summarize,
    DraftProposal,
    AnalyzeDocument,
    GenerateReport,
    SearchKnowledge,
}

// Expertise areas (enum)
pub enum Expertise {
    MachineLearning,
    Embedded,
    Security,
    Cloud,
    Backend,
    Frontend,
    DataScience,
    DevOps,
}

// Constraints
pub struct Constraint {
    pub max_budget: Option<i64>,
    pub max_results: Option<u32>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}
```

#### Metadata Types

```rust
pub struct IntentMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub session_id: String,
    pub request_id: Option<Uuid>,
}

pub struct TrustedIntent {
    pub intent: Intent,
    pub metadata: IntentMetadata,
    pub signature: String,  // Cryptographic signature
}
```

### Configuration Options

None (pure types module)

### Examples

```rust
use intent_schema::{Intent, Action, Expertise};
use serde_json::json;

// Create an intent
let intent = Intent {
    action: Action::FindExperts,
    topic: Some("supply chain security".to_string()),
    expertise: vec![Expertise::Security, Expertise::Cloud],
    constraints: json!({
        "max_budget": 50000,
        "max_results": 10
    }),
    content_refs: vec![],
};

// Serialize to JSON
let json = serde_json::to_string(&intent)?;

// Deserialize from JSON
let parsed: Intent = serde_json::from_str(&json)?;
```

---

## Malicious Detector Module

**Location**: `core/malicious_detector/`

### Purpose

Provides fast, lightweight detection of obviously malicious or attack-oriented inputs before deeper processing. This is the first line of defense.

### Responsibilities

- Regex-based pattern matching for known attack vectors
- Command injection detection
- SQL injection detection
- Path traversal detection
- XSS pattern detection
- Optional ML-based classification

### Public API

```rust
use malicious_detector::{MaliciousDetector, DetectionResult};

// Create detector
let detector = MaliciousDetector::new();

// Check input
let result: DetectionResult = detector.check(user_input).await?;

// Result structure
pub struct DetectionResult {
    pub blocked: bool,
    pub score: f64,  // 0.0 = safe, 1.0 = definitely malicious
    pub reasons: Vec<String>,
    pub matched_patterns: Vec<String>,
}
```

### Integration Points

- **Input**: Raw user input string
- **Output**: `DetectionResult` indicating if input should be blocked
- **Used by**: API gateway (before parser ensemble)

### Configuration Options

```rust
pub struct MaliciousDetectorConfig {
    // Enable/disable detector
    pub enabled: bool,

    // Block on detection (vs. just log)
    pub block_on_detection: bool,

    // Minimum score to block (0.0 - 1.0)
    pub block_threshold: f64,

    // Enable ML classifier
    pub enable_ml_classifier: bool,

    // Strict mode (more aggressive)
    pub strict_mode: bool,
}
```

Environment variables:
- `MALICIOUS_DETECTOR_ENABLED=true`
- `MALICIOUS_DETECTOR_BLOCK_ON_DETECTION=true`
- `MALICIOUS_DETECTOR_STRICT_MODE=false`

### Examples

#### Basic Usage

```rust
use malicious_detector::MaliciousDetector;

let detector = MaliciousDetector::new();

// Safe input
let result = detector.check("Find ML experts for my project").await?;
assert!(!result.blocked);

// Malicious input
let result = detector.check("'; DROP TABLE users; --").await?;
assert!(result.blocked);
assert!(result.reasons.contains(&"SQL injection pattern detected".to_string()));
```

#### Custom Configuration

```rust
use malicious_detector::{MaliciousDetector, MaliciousDetectorConfig};

let config = MaliciousDetectorConfig {
    enabled: true,
    block_on_detection: true,
    block_threshold: 0.7,
    enable_ml_classifier: false,
    strict_mode: true,
};

let detector = MaliciousDetector::with_config(config);
```

---

## Parser Module

**Location**: `core/parsers/`

### Purpose

Extracts structured intent from unstructured user input using multiple independent parsers (deterministic and LLM-based).

### Responsibilities

- Parse user input into structured `Intent` objects
- Provide multiple independent parsing strategies
- Return confidence scores for each parse
- Handle parser failures gracefully

### Public API

#### Parser Trait

```rust
#[async_trait]
pub trait IntentParser: Send + Sync {
    async fn parse(&self, user_input: &str) -> Result<ParsedIntent, ParserError>;
    fn parser_type(&self) -> ParserType;
    fn parser_id(&self) -> String;
    fn trust_level(&self) -> f64;
}

pub struct ParsedIntent {
    pub intent: Intent,
    pub confidence: f64,
    pub parser_id: String,
    pub trust_level: f64,
    pub parsing_time_ms: u64,
}
```

#### Parser Ensemble

```rust
use intent_parsers::{ParserEnsemble, ParserConfig};

// Create ensemble
let config = ParserConfig::from_env()?;
let ensemble = ParserEnsemble::new(config);

// Parse with all parsers in parallel
let result = ensemble.parse_all(user_input).await;

// Access results
println!("Success: {}/{}", result.success_count, result.parsers_count);

// Get highest trust result
if let Some(trusted) = result.get_highest_trust() {
    println!("Using: {}", trusted.parser_id);
}
```

### Integration Points

- **Input**: User input string
- **Output**: `EnsembleResult` with multiple parsed intents
- **Used by**: Voting module
- **Dependencies**: Ollama API, OpenAI API

### Configuration Options

```rust
pub struct ParserConfig {
    pub enable_deterministic: bool,
    pub enable_ollama: bool,
    pub enable_openai: bool,

    pub ollama: OllamaConfig,
    pub openai: OpenAIConfig,
}

pub struct OllamaConfig {
    pub endpoint: String,      // http://localhost:11434
    pub model: String,          // llama2, mistral, etc.
    pub temperature: f64,       // 0.0 for deterministic
    pub timeout_secs: u64,      // 30
}

pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,       // https://api.openai.com/v1
    pub model: String,          // gpt-4o-mini
    pub temperature: f64,       // 0.0
    pub timeout_secs: u64,      // 30
}
```

Environment variables:
- `ENABLE_DETERMINISTIC=true`
- `ENABLE_OLLAMA=true`
- `ENABLE_OPENAI=false`
- `OLLAMA_ENDPOINT=http://localhost:11434`
- `OLLAMA_MODEL=llama2`
- `OPENAI_API_KEY=sk-...`
- `OPENAI_MODEL=gpt-4o-mini`

### Examples

#### Using Individual Parsers

```rust
use intent_parsers::{DeterministicParser, OllamaParser, OpenAIParser};

// Deterministic parser (rule-based, no LLM)
let det_parser = DeterministicParser::new();
let result = det_parser.parse("Find ML experts with $50k budget").await?;

// Ollama parser (local LLM)
let ollama_config = OllamaConfig {
    endpoint: "http://localhost:11434".to_string(),
    model: "llama2".to_string(),
    temperature: 0.0,
    timeout_secs: 30,
};
let ollama_parser = OllamaParser::new(ollama_config);
let result = ollama_parser.parse("Find ML experts with $50k budget").await?;

// OpenAI parser (cloud LLM)
let openai_config = OpenAIConfig {
    api_key: "sk-...".to_string(),
    model: "gpt-4o-mini".to_string(),
    temperature: 0.0,
    timeout_secs: 30,
    base_url: "https://api.openai.com/v1".to_string(),
};
let openai_parser = OpenAIParser::new(openai_config);
let result = openai_parser.parse("Find ML experts with $50k budget").await?;
```

#### Using Ensemble

```rust
use intent_parsers::{ParserEnsemble, ParserConfig};

// Load from environment
let config = ParserConfig::from_env()?;
let ensemble = ParserEnsemble::new(config);

// Parse with all enabled parsers
let result = ensemble.parse_all(user_input).await;

// Handle results
for parsed in &result.results {
    println!("{}: {:?}", parsed.parser_id, parsed.intent.action);
}

// Handle errors
for (parser_id, error) in &result.errors {
    eprintln!("{} failed: {}", parser_id, error);
}

// Get best result (highest trust)
if let Some(best) = result.get_highest_trust() {
    println!("Using {} (trust: {})", best.parser_id, best.trust_level);
}
```

---

## Voting Module

**Location**: `core/voting/`

### Purpose

Compares outputs from multiple parsers, determines consensus, and selects the canonical intent with a confidence level.

### Responsibilities

- Compare parser outputs pairwise
- Calculate similarity scores
- Determine confidence level (High/Low/Conflict)
- Select canonical intent (prefer deterministic)
- Flag conflicts for human review
- Generate human-readable explanations

### Public API

```rust
use intent_voting::{VotingModule, VotingResult, ConfidenceLevel};

// Create voting module
let voting = VotingModule::new();

// Vote on parser results
let parser_results = vec![/* ParsedIntent from parsers */];
let result: VotingResult = voting.vote(parser_results).await?;

// Voting result structure
pub struct VotingResult {
    pub confidence: ConfidenceLevel,
    pub canonical_intent: Intent,
    pub average_similarity: f64,
    pub requires_human_review: bool,
    pub explanation: String,
    pub field_comparisons: Vec<FieldComparison>,
}

pub enum ConfidenceLevel {
    HighConfidence,    // ≥95% similarity
    LowConfidence,     // 75-95% similarity
    Conflict,          // <75% similarity
}
```

### Integration Points

- **Input**: Vector of `ParsedIntent` from parser ensemble
- **Output**: `VotingResult` with canonical intent and confidence
- **Used by**: Intent comparator
- **Dependencies**: Parser module

### Configuration Options

```rust
pub struct VotingConfig {
    // Similarity threshold for high confidence (default: 0.95)
    pub high_confidence_threshold: f64,

    // Similarity threshold for low confidence (default: 0.75)
    pub low_confidence_threshold: f64,

    // Field weights for similarity calculation
    pub action_weight: f64,      // default: 3.0
    pub topic_weight: f64,       // default: 2.0
    pub expertise_weight: f64,   // default: 2.0
    pub constraints_weight: f64, // default: 1.5
}
```

### Examples

#### Basic Usage

```rust
use intent_voting::VotingModule;

let voting = VotingModule::new();

let parser_results = vec![
    ParsedIntent { /* from deterministic */ },
    ParsedIntent { /* from ollama */ },
    ParsedIntent { /* from openai */ },
];

let result = voting.vote(parser_results).await?;

match result.confidence {
    ConfidenceLevel::HighConfidence => {
        println!("All parsers agree! Confidence: {}", result.average_similarity);
        // Auto-approve
    }
    ConfidenceLevel::LowConfidence => {
        println!("Minor discrepancies detected");
        // Use deterministic fallback
    }
    ConfidenceLevel::Conflict => {
        println!("Major conflict! Requires human review");
        // Escalate to supervisor
    }
}
```

#### Custom Thresholds

```rust
use intent_voting::VotingModule;

let voting = VotingModule::with_thresholds(
    0.98,  // High confidence threshold
    0.80,  // Low confidence threshold
);
```

---

## Comparator Module

**Location**: `core/comparator/`

### Purpose

Validates the canonical intent against provider-defined policies and constraints.

### Responsibilities

- Load provider configuration
- Validate action is allowed
- Validate expertise areas
- Check budget constraints
- Semantic topic matching
- Determine approval/denial/escalation

### Public API

```rust
use intent_comparator::{IntentComparator, ComparisonResult, ProviderConfig};

// Create comparator
let config = ProviderConfig::load("b2b_consulting")?;
let comparator = IntentComparator::new(config);

// Compare intent against policy
let result: ComparisonResult = comparator.compare(&intent).await?;

// Comparison result
pub struct ComparisonResult {
    pub result: ComparisonDecision,
    pub message: String,
    pub violations: Vec<PolicyViolation>,
    pub needs_approval: bool,
}

pub enum ComparisonDecision {
    Approved,       // All checks passed
    SoftMismatch,   // Minor issues, needs confirmation
    HardMismatch,   // Policy violation, block/escalate
}
```

### Integration Points

- **Input**: Canonical intent from voting module
- **Output**: `ComparisonResult` with approval decision
- **Used by**: Supervision module (if needs approval)
- **Dependencies**: Provider configuration

### Configuration Options

Provider configuration (`config/provider_config.json`):

```json
{
  "allowed_actions": ["find_experts", "summarize", "draft_proposal"],
  "allowed_expertise": ["ml", "security", "cloud"],
  "max_budget": 100000,
  "max_results": 50,
  "require_human_approval": true,
  "custom_constraints": {
    "allowed_topics": ["supply_chain", "cybersecurity"],
    "forbidden_actions": ["delete", "modify_records"]
  }
}
```

### Examples

```rust
use intent_comparator::{IntentComparator, ProviderConfig};

// Load provider config
let provider_config = ProviderConfig::load_from_file(
    "config/provider_config.json",
    "b2b_consulting"
)?;

let comparator = IntentComparator::new(provider_config);

// Compare intent
let result = comparator.compare(&intent).await?;

match result.result {
    ComparisonDecision::Approved => {
        println!("✓ Intent approved");
    }
    ComparisonDecision::SoftMismatch => {
        println!("⚠ Soft mismatch: {}", result.message);
        println!("Violations: {:?}", result.violations);
    }
    ComparisonDecision::HardMismatch => {
        println!("✗ Hard mismatch - blocked");
        println!("Violations: {:?}", result.violations);
    }
}
```

---

## Intent Generator Module

**Location**: `core/intent_generator/`

### Purpose

Creates a trusted, canonical, and cryptographically signed intent object that can be safely executed.

### Responsibilities

- Sanitize intent fields
- Add metadata (timestamp, user, session)
- Sign intent cryptographically
- Validate schema compliance
- Ensure immutability

### Public API

```rust
use intent_generator::{IntentGenerator, TrustedIntent};

// Create generator
let generator = IntentGenerator::new(signing_key);

// Generate trusted intent
let trusted: TrustedIntent = generator.generate(&intent, &metadata).await?;

// Trusted intent structure
pub struct TrustedIntent {
    pub intent: Intent,
    pub metadata: IntentMetadata,
    pub signature: String,
    pub created_at: DateTime<Utc>,
}
```

### Integration Points

- **Input**: Canonical intent + metadata
- **Output**: `TrustedIntent` with signature
- **Used by**: Processing engine
- **Dependencies**: Crypto library (for signing)

### Configuration Options

```rust
pub struct GeneratorConfig {
    // Signing key (hex-encoded)
    pub signing_key: String,

    // Encryption key for sensitive data
    pub encryption_key: Option<String>,
}
```

Environment variables:
- `SIGNING_KEY=fedcba9876543210...`
- `ENCRYPTION_KEY=0123456789abcdef...`

### Examples

```rust
use intent_generator::IntentGenerator;

let generator = IntentGenerator::new(signing_key);

let metadata = IntentMetadata {
    id: Uuid::new_v4(),
    timestamp: Utc::now(),
    user_id: "user_123".to_string(),
    session_id: "session_456".to_string(),
    request_id: Some(request_id),
};

let trusted = generator.generate(&canonical_intent, &metadata).await?;

// Verify signature
assert!(generator.verify(&trusted)?);
```

---

## Processing Engine Module

**Location**: `core/processing_engine/`

### Purpose

Executes validated, trusted intents via typed function calls (not free-form LLM prompts).

### Responsibilities

- Route intents to appropriate handlers
- Execute typed functions (find_experts, summarize, etc.)
- Return structured results
- Log execution to ledger
- Handle execution errors

### Public API

```rust
use processing_engine::{ProcessingEngine, ProcessingResult};

// Create engine
let engine = ProcessingEngine::new(db_pool);

// Execute trusted intent
let result: ProcessingResult = engine.execute(&trusted_intent).await?;

// Processing result
pub struct ProcessingResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}
```

### Integration Points

- **Input**: `TrustedIntent` from generator
- **Output**: `ProcessingResult` with execution outcome
- **Used by**: API response handler
- **Dependencies**: Database, external APIs (for handlers)

### Configuration Options

None (uses database connection from app state)

### Examples

```rust
use processing_engine::ProcessingEngine;

let engine = ProcessingEngine::new(db_pool);

// Execute intent
let result = engine.execute(&trusted_intent).await?;

if result.success {
    println!("Result: {}", serde_json::to_string_pretty(&result.data)?);
} else {
    eprintln!("Execution error: {:?}", result.error);
}
```

#### Custom Handler

```rust
use processing_engine::{ProcessingEngine, Handler};
use async_trait::async_trait;

struct CustomHandler;

#[async_trait]
impl Handler for CustomHandler {
    async fn handle(&self, intent: &TrustedIntent) -> Result<serde_json::Value> {
        // Custom logic
        Ok(json!({ "custom": "result" }))
    }

    fn action(&self) -> Action {
        Action::CustomAction
    }
}

// Register custom handler
let mut engine = ProcessingEngine::new(db_pool);
engine.register_handler(Box::new(CustomHandler));
```

---

## Supervision Module

**Location**: `core/supervision/`

### Purpose

Manages human-in-the-loop approval workflow for elevated-risk or conflicting intents.

### Responsibilities

- Create approval requests
- Store pending approvals
- Notify supervisors (email/Slack)
- Track approval decisions
- Handle timeouts and escalations

### Public API

```rust
use supervision::{SupervisionModule, ApprovalRequest, ApprovalDecision};

// Create module
let supervision = SupervisionModule::new(db_pool, notifications);

// Create approval request
let approval = supervision.create_approval(
    &intent,
    "Parser conflict detected",
    &parser_diff
).await?;

// Check approval status
let status = supervision.get_approval(approval.id).await?;

// Submit decision
let decision = ApprovalDecision {
    approved: true,
    approver_id: "admin_1".to_string(),
    reason: "Intent is valid after review".to_string(),
};
supervision.submit_decision(approval.id, decision).await?;
```

### Integration Points

- **Input**: Intent + reason for escalation
- **Output**: `ApprovalRequest` with status
- **Used by**: API comparator (when needs_approval)
- **Dependencies**: Database, notifications module

### Configuration Options

```rust
pub struct SupervisionConfig {
    // Enable human approval
    pub enabled: bool,

    // Timeout for approvals (minutes)
    pub approval_timeout_minutes: u32,

    // Auto-deny after timeout
    pub auto_deny_on_timeout: bool,
}
```

Environment variables:
- `ENABLE_HUMAN_APPROVAL=true`
- `APPROVAL_TIMEOUT_MINUTES=60`

### Examples

```rust
use supervision::SupervisionModule;

let supervision = SupervisionModule::new(db_pool, notifications);

// Create approval
let approval = supervision.create_approval(
    &intent,
    "Parser conflict: deterministic vs LLM disagree on action",
    &parser_diff
).await?;

println!("Approval created: {}", approval.id);
println!("Status: {}", approval.status);

// Later: submit decision
let decision = ApprovalDecision {
    approved: true,
    approver_id: "admin_123".to_string(),
    reason: "Reviewed manually, intent is safe".to_string(),
};

supervision.submit_decision(approval.id, decision).await?;
```

---

## Notifications Module

**Location**: `core/notifications/`

### Purpose

Sends notifications to administrators for security events, approvals, and alerts.

### Responsibilities

- Send email notifications (SMTP)
- Send Slack notifications (webhooks)
- Format notification messages
- Handle delivery failures
- Track notification history

### Public API

```rust
use notifications::{NotificationService, NotificationType, Notification};

// Create service
let notifications = NotificationService::new(config);

// Send notification
notifications.send(
    NotificationType::ApprovalRequired,
    "New approval request pending",
    json!({
        "approval_id": approval.id,
        "user_id": "user_123",
        "reason": "Parser conflict"
    })
).await?;
```

### Integration Points

- **Input**: Notification type + message + data
- **Output**: Delivery confirmation
- **Used by**: Supervision module, malicious detector
- **Dependencies**: SMTP server, Slack API

### Configuration Options

```rust
pub struct NotificationConfig {
    // Email configuration
    pub email_enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub admin_emails: Vec<String>,

    // Slack configuration
    pub slack_enabled: bool,
    pub slack_webhook_url: String,
    pub slack_channel: String,
}
```

Environment variables:
- `SMTP_HOST=smtp.gmail.com`
- `SMTP_PORT=587`
- `SMTP_USERNAME=...`
- `SMTP_PASSWORD=...`
- `SMTP_FROM=noreply@example.com`
- `ALERT_EMAIL_RECIPIENTS=admin@example.com`
- `SLACK_WEBHOOK_URL=https://hooks.slack.com/...`
- `SLACK_CHANNEL=#security-alerts`

### Examples

```rust
use notifications::{NotificationService, NotificationType};

let config = NotificationConfig {
    email_enabled: true,
    smtp_host: "smtp.gmail.com".to_string(),
    smtp_port: 587,
    smtp_username: "noreply@example.com".to_string(),
    smtp_password: "app_password".to_string(),
    smtp_from: "noreply@example.com".to_string(),
    admin_emails: vec!["admin@example.com".to_string()],
    slack_enabled: true,
    slack_webhook_url: "https://hooks.slack.com/...".to_string(),
    slack_channel: "#security".to_string(),
};

let notifications = NotificationService::new(config);

// Send approval notification
notifications.send(
    NotificationType::ApprovalRequired,
    "Approval Required",
    json!({
        "approval_id": "123e4567-e89b-12d3-a456-426614174000",
        "reason": "Parser conflict detected"
    })
).await?;

// Send security alert
notifications.send(
    NotificationType::SecurityAlert,
    "Malicious Input Blocked",
    json!({
        "user_id": "user_123",
        "input": "'; DROP TABLE users; --",
        "reason": "SQL injection detected"
    })
).await?;
```

---

## Ledger Module

**Location**: `core/ledger/`

### Purpose

Provides an immutable, append-only audit log of all operations for compliance, forensics, and debugging.

### Responsibilities

- Write ledger entries (no updates/deletes)
- Query ledger with filters
- Provide statistics and analytics
- Ensure data integrity
- Enforce retention policies

### Public API

```rust
use ledger::{Ledger, LedgerEntry, LedgerQuery};

// Create ledger
let ledger = Ledger::new(db_pool);

// Write entry
let entry = LedgerEntry {
    session_id: "session_123".to_string(),
    user_id: "user_456".to_string(),
    user_input: "Find ML experts".to_string(),
    malicious_blocked: false,
    voting_result: serde_json::to_value(&voting_result)?,
    comparison_result: serde_json::to_value(&comparison_result)?,
    trusted_intent: Some(serde_json::to_value(&trusted_intent)?),
    processing_output: Some(serde_json::to_value(&result)?),
    // ...
};

let entry_id = ledger.write(entry).await?;

// Query ledger
let query = LedgerQuery {
    user_id: Some("user_456".to_string()),
    start_time: Some(start),
    end_time: Some(end),
    blocked_only: false,
    elevation_only: false,
    limit: 100,
};

let entries = ledger.query(query).await?;

// Get statistics
let stats = ledger.get_stats().await?;
```

### Integration Points

- **Input**: `LedgerEntry` with all pipeline data
- **Output**: Entry ID (UUID)
- **Used by**: API handlers (all requests)
- **Dependencies**: PostgreSQL database

### Configuration Options

```rust
pub struct LedgerConfig {
    // Enable ledger (should always be true in production)
    pub enabled: bool,

    // Storage backend (postgres, s3, file)
    pub storage: String,

    // Retention period (days)
    pub retention_days: u32,

    // Encrypt entries at rest
    pub encrypt_at_rest: bool,
}
```

Environment variables:
- `LEDGER_ENABLED=true`
- `LEDGER_STORAGE=postgres`
- `LEDGER_RETENTION_DAYS=365`
- `LEDGER_ENCRYPT_AT_REST=true`

### Examples

#### Write Entry

```rust
use ledger::{Ledger, LedgerEntry};

let ledger = Ledger::new(db_pool);

let entry = LedgerEntry {
    session_id: session_id.clone(),
    user_id: user_id.clone(),
    timestamp: Utc::now(),
    user_input: user_input.clone(),
    user_input_hash: sha256(&user_input),
    malicious_blocked: false,
    voting_result: serde_json::to_value(&voting_result)?,
    comparison_result: serde_json::to_value(&comparison_result)?,
    trusted_intent: Some(serde_json::to_value(&trusted_intent)?),
    processing_output: Some(serde_json::to_value(&result)?),
    was_executed: true,
    ip_address: Some("192.168.1.1".to_string()),
    user_agent: Some("Mozilla/5.0...".to_string()),
    request_id: Some(request_id),
};

let entry_id = ledger.write(entry).await?;
println!("Ledger entry: {}", entry_id);
```

#### Query Entries

```rust
use ledger::{Ledger, LedgerQuery};
use chrono::{Utc, Duration};

let ledger = Ledger::new(db_pool);

// Query last 24 hours for specific user
let query = LedgerQuery {
    user_id: Some("user_123".to_string()),
    start_time: Some(Utc::now() - Duration::days(1)),
    end_time: Some(Utc::now()),
    blocked_only: false,
    elevation_only: false,
    limit: 50,
};

let entries = ledger.query(query).await?;

for entry in entries {
    println!("{}: {} - {}",
        entry.timestamp,
        entry.user_input,
        if entry.was_executed { "executed" } else { "blocked" }
    );
}
```

#### Get Statistics

```rust
let stats = ledger.get_stats().await?;

println!("Total entries: {}", stats.total_entries);
println!("Blocked: {}", stats.blocked_entries);
println!("Required approval: {}", stats.elevation_events);
println!("Unique users: {}", stats.total_users);
```

---

## Module Interaction Flow

```
User Input
    │
    ▼
[Schema] ← Types used by all modules
    │
    ▼
[Malicious Detector] ──► [Ledger] (if blocked)
    │
    ▼
[Parsers] ─┬─ Deterministic
           ├─ Ollama
           └─ OpenAI
    │
    ▼
[Voting Module] ──► [Ledger] (voting result)
    │
    ▼
[Comparator] ──► [Ledger] (comparison result)
    │
    ├─► [Supervision] ──► [Notifications] ──► [Ledger] (if needs approval)
    │
    ▼
[Intent Generator]
    │
    ▼
[Processing Engine] ──► [Ledger] (execution result)
    │
    ▼
Response
```

---

For more information, see:
- [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture
- [DEVELOPMENT.md](../DEVELOPMENT.md) - Development guide
- [SECURITY.md](SECURITY.md) - Security documentation
