# Intent Segregation Cybersecurity Architecture - Technical Documentation

## Table of Contents

1. [System Overview](#system-overview)
2. [High-Level Architecture](#high-level-architecture)
3. [Module Dependency Graph](#module-dependency-graph)
4. [Data Flow Pipeline](#data-flow-pipeline)
5. [Security Architecture Layers](#security-architecture-layers)
6. [Database Schema](#database-schema)
7. [API Architecture](#api-architecture)
8. [Frontend Architecture](#frontend-architecture)
9. [Deployment Architecture](#deployment-architecture)
10. [Technology Stack](#technology-stack)

## System Overview

The Intent Segregation Cybersecurity Architecture is a defense-in-depth system designed to prevent prompt injection attacks and unauthorized LLM actions by separating user intent from user content.

### Core Principle

**Intent Segregation**: Never allow unvalidated user content to directly influence system behavior. Instead:

1. **Parse** user input into structured intent using multiple independent parsers
2. **Validate** intent through consensus voting
3. **Compare** against strict provider policies
4. **Approve** via human review when necessary
5. **Generate** a trusted, canonical intent object
6. **Execute** through typed function calls (not free-form LLM prompts)
7. **Audit** every step immutably

### Design Goals

- **Security**: Prevent prompt injection, command injection, and privilege escalation
- **Reliability**: Multiple parser redundancy with deterministic fallback
- **Auditability**: Immutable ledger of all operations
- **Transparency**: Human-in-the-loop for elevated-risk actions
- **Performance**: Sub-second response times for most operations
- **Scalability**: Handle 1000s of requests per second

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          USER INTERFACE                              │
│                     (Web UI / API Clients)                          │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         API GATEWAY                                  │
│                      (Axum REST API)                                │
│  • Authentication       • Rate Limiting       • Request Validation  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    MALICIOUS INPUT DETECTOR                          │
│  • Regex Pattern Matching    • ML Classification (Optional)         │
│  • Command Injection Check   • SQL Injection Check                  │
│  • XSS Detection             • Path Traversal Check                 │
└────────────┬────────────────────────────────────────────────────────┘
             │
             ├─────► [BLOCKED] ──► Ledger ──► Alert ──► Response
             │
             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      PARSER ENSEMBLE                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ Deterministic│  │    Ollama    │  │    OpenAI    │             │
│  │    Parser    │  │   Parser     │  │    Parser    │             │
│  │ (Trust: 1.0) │  │ (Trust: 0.75)│  │ (Trust: 0.8) │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                 │                 │                       │
│         └─────────────────┴─────────────────┘                       │
│                           │                                         │
│                  (Parsed Intents)                                   │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       VOTING MODULE                                  │
│  • Compare Parser Outputs    • Calculate Similarity                 │
│  • Detect Conflicts          • Determine Confidence                 │
│  • Select Canonical Intent   • Generate Explanation                 │
│                                                                      │
│  Confidence Levels:                                                 │
│  • High (≥95%):   Auto-approve                                      │
│  • Low (75-95%):  Use deterministic, may confirm                    │
│  • Conflict (<75%): Escalate to human                               │
└────────────┬────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     INTENT COMPARATOR                                │
│  • Load Provider Policy      • Validate Action                      │
│  • Check Expertise           • Verify Constraints                   │
│  • Semantic Topic Matching   • Budget Limits                        │
│                                                                      │
│  Decision:                                                          │
│  • Approved: Continue        • Soft Mismatch: Confirm              │
│  • Hard Mismatch: Block/Escalate                                   │
└────────────┬────────────────────────────────────────────────────────┘
             │
             ├─────► [If Mismatch or Conflict]
             │           │
             │           ▼
             │      ┌──────────────────────────────────────┐
             │      │    HUMAN APPROVAL WORKFLOW           │
             │      │  • Create Approval Request           │
             │      │  • Notify Supervisors (Email/Slack)  │
             │      │  • Present Intent Diff UI            │
             │      │  • Wait for Decision                 │
             │      │  • Log Decision                      │
             │      └────────────┬─────────────────────────┘
             │                   │
             │      ┌────────────┴────────────┐
             │      │                         │
             │      ▼                         ▼
             │  [Approved]                [Denied] ──► Ledger ──► Response
             │      │
             ◄──────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   TRUSTED INTENT GENERATOR                           │
│  • Sanitize Intent           • Add Metadata                         │
│  • Sign Intent               • Reference Content                    │
│  • Validate Schema           • Ensure Immutability                  │
└────────────┬────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    PROCESSING ENGINE                                 │
│  • Route to Handler          • Execute Typed Functions              │
│  • find_experts()           • summarize()                           │
│  • draft_proposal()         • analyze_document()                    │
│  • No Free-form LLM calls   • All Actions Logged                    │
└────────────┬────────────────────────────────────────────────────────┘
             │
             ├──────────────► INTENT LEDGER (Immutable Audit Log)
             │
             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           RESPONSE                                   │
│  • Processing Results        • Intent Details                       │
│  • Pipeline Info            • Audit Reference                       │
└─────────────────────────────────────────────────────────────────────┘
```

## Module Dependency Graph

```
┌──────────────────┐
│   intent-schema  │  ← Core types used by all modules
└────────┬─────────┘
         │
         ├──────────────────────────────────────┐
         │                                      │
         ▼                                      ▼
┌──────────────────┐                 ┌──────────────────┐
│ malicious-       │                 │  intent-parsers  │
│ detector         │                 │  - deterministic │
└────────┬─────────┘                 │  - ollama        │
         │                           │  - openai        │
         │                           └────────┬─────────┘
         │                                    │
         │                                    ▼
         │                           ┌──────────────────┐
         │                           │  intent-voting   │
         │                           └────────┬─────────┘
         │                                    │
         ├────────────────────────────────────┤
         │                                    │
         ▼                                    ▼
┌──────────────────┐                 ┌──────────────────┐
│ intent-comparator│                 │ intent-generator │
└────────┬─────────┘                 └────────┬─────────┘
         │                                    │
         ▼                                    ▼
┌──────────────────┐                 ┌──────────────────┐
│ supervision      │◄────────────────┤ processing-engine│
│ (human approval) │                 │                  │
└────────┬─────────┘                 └────────┬─────────┘
         │                                    │
         │     ┌──────────────────┐           │
         ├────►│  notifications   │◄──────────┤
         │     │  - email         │           │
         │     │  - slack         │           │
         │     └──────────────────┘           │
         │                                    │
         │     ┌──────────────────┐           │
         └────►│  intent-ledger   │◄──────────┘
               │  (audit log)     │
               └──────────────────┘
                        │
                        ▼
               ┌──────────────────┐
               │  PostgreSQL DB   │
               └──────────────────┘
```

### External Dependencies

```
┌──────────────────┐
│   LLM Providers  │
│  - Ollama (local)│
│  - OpenAI API    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐       ┌──────────────────┐
│   PostgreSQL     │       │      Redis       │
│  (Data Storage)  │       │     (Cache)      │
└──────────────────┘       └──────────────────┘
         │                          │
         └──────────┬───────────────┘
                    │
                    ▼
         ┌──────────────────┐
         │   API Server     │
         │   (Axum/Rust)    │
         └──────────────────┘
```

## Data Flow Pipeline

### 1. Request Flow

```
HTTP Request
    │
    ├─ Method: POST
    ├─ Path: /api/process
    ├─ Headers: { Content-Type: application/json, X-Request-ID: ... }
    │
    └─ Body: {
         "user_input": "Find security experts for $20k",
         "user_id": "user_123",
         "session_id": "session_456"
       }
    │
    ▼
┌────────────────────────────────┐
│  API Layer (Axum)              │
│  1. Parse JSON                 │
│  2. Validate schema            │
│  3. Generate request_id        │
│  4. Extract user context       │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│  Malicious Detector            │
│  Input: "Find security..."     │
│  Output: { blocked: false }    │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│  Parser Ensemble (Parallel)    │
│                                │
│  ┌─────────────────────────┐  │
│  │ Deterministic: 2ms      │  │
│  │ {                       │  │
│  │   action: "find_experts"│  │
│  │   expertise: ["security"]│ │
│  │   budget: 20000         │  │
│  │ }                       │  │
│  └─────────────────────────┘  │
│                                │
│  ┌─────────────────────────┐  │
│  │ Ollama: 850ms           │  │
│  │ {                       │  │
│  │   action: "find_experts"│  │
│  │   expertise: ["security"]│ │
│  │   budget: 20000         │  │
│  │ }                       │  │
│  └─────────────────────────┘  │
│                                │
│  ┌─────────────────────────┐  │
│  │ OpenAI: 420ms           │  │
│  │ {                       │  │
│  │   action: "find_experts"│  │
│  │   expertise: ["security","cybersecurity"]│
│  │   budget: 20000         │  │
│  │ }                       │  │
│  └─────────────────────────┘  │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│  Voting Module                 │
│  • Compare 3 results           │
│  • Similarity: 97%             │
│  • Confidence: High            │
│  • Canonical: Deterministic    │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│  Intent Comparator             │
│  • Load provider config        │
│  • Check: find_experts allowed │
│  • Check: security in allowed  │
│  • Check: 20000 < 100000       │
│  • Decision: Approved          │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│  Intent Generator              │
│  • Create trusted intent       │
│  • Add metadata & timestamp    │
│  • Sign intent                 │
│  • Reference sanitized content │
└───────────┬────────────────────┘
            │
            ▼
┌────────────────────────────────┐
│  Processing Engine             │
│  • Route to find_experts()     │
│  • Execute database query      │
│  • Return results              │
└───────────┬────────────────────┘
            │
            ├──────► Ledger (write entry)
            │
            ▼
┌────────────────────────────────┐
│  Response                      │
│  {                             │
│    "request_id": "...",        │
│    "status": "completed",      │
│    "trusted_intent": {...},    │
│    "result": {                 │
│      "experts": [...]          │
│    }                           │
│  }                             │
└────────────────────────────────┘
```

### 2. Human Approval Flow

```
User Input (Malicious or Ambiguous)
    │
    ▼
[Parser Conflict Detected]
    │
    ▼
┌─────────────────────────────────┐
│  Supervision Module             │
│  1. Create ApprovalRequest      │
│  2. Store in database           │
│  3. Generate approval_id        │
└────────┬────────────────────────┘
         │
         ├──────► Notifications
         │        ├─ Email to admins
         │        └─ Slack webhook
         │
         ▼
┌─────────────────────────────────┐
│  Return to User:                │
│  {                              │
│    status: "pending_approval",  │
│    approval_id: "...",          │
│    message: "Requires review"   │
│  }                              │
└─────────────────────────────────┘
         │
         │ [Wait for human decision]
         │
         ▼
┌─────────────────────────────────┐
│  Admin UI / POST /approvals/:id │
│  {                              │
│    approved: true,              │
│    approver_id: "admin_1",      │
│    reason: "Intent is valid"    │
│  }                              │
└────────┬────────────────────────┘
         │
         ▼
    [If Approved]
         │
         ▼
Continue to Intent Generator → Processing Engine → Response
```

### 3. Ledger Write Flow

Every request writes to the immutable ledger:

```sql
INSERT INTO ledger_entries (
    id,
    session_id,
    user_id,
    timestamp,
    user_input,
    user_input_hash,
    malicious_blocked,
    voting_result,
    comparison_result,
    elevation_event,
    trusted_intent,
    processing_output
) VALUES (
    uuid_generate_v4(),
    'session_456',
    'user_123',
    NOW(),
    'Find security experts for $20k',
    sha256('...'),
    false,
    '{"confidence": "High", ...}'::jsonb,
    '{"result": "Approved", ...}'::jsonb,
    NULL,
    '{"action": "find_experts", ...}'::jsonb,
    '{"experts": [...]}'::jsonb
);
```

## Security Architecture Layers

### Layer 1: Network & Transport Security

```
┌─────────────────────────────────────┐
│  TLS 1.3 Encryption                 │
│  • Certificate pinning              │
│  • Strong cipher suites             │
│  • HSTS enabled                     │
└─────────────────────────────────────┘
```

### Layer 2: API Gateway Security

```
┌─────────────────────────────────────┐
│  API Gateway                        │
│  • Rate limiting (60 req/min)       │
│  • API key authentication           │
│  • JWT token validation             │
│  • Request size limits (1MB)        │
│  • CORS policy enforcement          │
│  • Request ID tracking              │
└─────────────────────────────────────┘
```

### Layer 3: Input Validation

```
┌─────────────────────────────────────┐
│  Schema Validation                  │
│  • JSON schema enforcement          │
│  • Type checking                    │
│  • Length limits                    │
│  • Character whitelist              │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Malicious Input Detection          │
│  • Command injection patterns       │
│  • SQL injection patterns           │
│  • Path traversal attempts          │
│  • XSS patterns                     │
│  • Known attack signatures          │
└─────────────────────────────────────┘
```

### Layer 4: Intent Validation

```
┌─────────────────────────────────────┐
│  Multi-Parser Validation            │
│  • Deterministic parser (trust: 1.0)│
│  • LLM parser 1 (trust: 0.75)       │
│  • LLM parser 2 (trust: 0.8)        │
│  • Consensus voting                 │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Policy Enforcement                 │
│  • Action whitelist check           │
│  • Expertise validation             │
│  • Budget limits                    │
│  • Topic semantic matching          │
└─────────────────────────────────────┘
```

### Layer 5: Execution Isolation

```
┌─────────────────────────────────────┐
│  Typed Function Calls Only          │
│  • No raw LLM execution             │
│  • Parameterized queries            │
│  • Sandboxed operations             │
│  • Least privilege principle        │
└─────────────────────────────────────┘
```

### Layer 6: Audit & Monitoring

```
┌─────────────────────────────────────┐
│  Immutable Audit Ledger             │
│  • All operations logged            │
│  • Tamper-evident storage           │
│  • Cryptographic hashing            │
│  • Retention: 365 days              │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Security Monitoring                │
│  • Real-time alerts                 │
│  • Anomaly detection                │
│  • Failed attempt tracking          │
│  • Incident response triggers       │
└─────────────────────────────────────┘
```

## Database Schema

### Core Tables

#### ledger_entries (Immutable Audit Log)

```sql
CREATE TABLE ledger_entries (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Request context
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- User input
    user_input TEXT NOT NULL,
    user_input_hash TEXT NOT NULL,  -- SHA-256 hash

    -- Malicious detection
    malicious_score FLOAT,
    malicious_blocked BOOLEAN NOT NULL DEFAULT false,
    malicious_reason TEXT,

    -- Parsing results
    parser_results JSONB NOT NULL,  -- Array of parser outputs

    -- Voting result
    voting_result JSONB NOT NULL,
    -- {
    --   "confidence": "High" | "Low" | "Conflict",
    --   "average_similarity": 0.98,
    --   "requires_human_review": false,
    --   "canonical_intent": {...},
    --   "explanation": "..."
    -- }

    -- Comparison result
    comparison_result JSONB NOT NULL,
    -- {
    --   "result": "Approved" | "Denied" | "NeedsApproval",
    --   "message": "...",
    --   "policy_violations": [...]
    -- }

    -- Elevation/approval event
    elevation_event JSONB,
    -- {
    --   "approval_id": "...",
    --   "approver_id": "...",
    --   "decision": "approved" | "denied",
    --   "reason": "...",
    --   "decided_at": "..."
    -- }

    -- Trusted intent (after all validation)
    trusted_intent JSONB,
    -- {
    --   "action": "find_experts",
    --   "topic_id": "security",
    --   "expertise": ["security"],
    --   "constraints": {...},
    --   "metadata": {...}
    -- }

    -- Processing output
    processing_output JSONB,
    was_executed BOOLEAN DEFAULT false,
    execution_error TEXT,

    -- Metadata
    ip_address INET,
    user_agent TEXT,
    request_id UUID,

    -- Indices for common queries
    INDEX idx_user_id (user_id),
    INDEX idx_session_id (session_id),
    INDEX idx_timestamp (timestamp DESC),
    INDEX idx_blocked (malicious_blocked) WHERE malicious_blocked = true,
    INDEX idx_elevation (elevation_event) WHERE elevation_event IS NOT NULL,
    INDEX idx_request_id (request_id)
);

-- Enforce immutability (append-only)
CREATE RULE ledger_no_update AS ON UPDATE TO ledger_entries DO INSTEAD NOTHING;
CREATE RULE ledger_no_delete AS ON DELETE TO ledger_entries DO INSTEAD NOTHING;
```

#### approval_requests

```sql
CREATE TABLE approval_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Request details
    ledger_entry_id UUID REFERENCES ledger_entries(id),
    status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'denied')),

    -- Intent to be reviewed
    parsed_intent JSONB NOT NULL,
    original_input TEXT NOT NULL,

    -- Reason for escalation
    escalation_reason TEXT NOT NULL,
    -- "Parser conflict"
    -- "Policy violation: action not allowed"
    -- "High-risk operation"
    -- "Manual review requested"

    -- Parser comparison data
    parser_diff JSONB,

    -- Decision
    decision JSONB,
    -- {
    --   "approved": true,
    --   "approver_id": "admin_1",
    --   "reason": "...",
    --   "decided_at": "2024-01-15T10:35:00Z"
    -- }

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_at TIMESTAMPTZ,

    -- Notification tracking
    notifications_sent JSONB,

    INDEX idx_status (status),
    INDEX idx_created_at (created_at DESC),
    INDEX idx_pending (status, created_at) WHERE status = 'pending'
);
```

#### provider_policies (Runtime Policy Storage)

```sql
CREATE TABLE provider_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    provider_name TEXT UNIQUE NOT NULL,
    config JSONB NOT NULL,
    -- Full provider configuration from provider_config.json

    active BOOLEAN DEFAULT true,
    version INTEGER DEFAULT 1,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    INDEX idx_active (active) WHERE active = true
);
```

#### parser_health (Monitoring)

```sql
CREATE TABLE parser_health (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parser_id TEXT NOT NULL,

    -- Health metrics
    status TEXT CHECK (status IN ('healthy', 'degraded', 'down')),
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    avg_response_time_ms FLOAT,
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    last_error TEXT,

    -- Circuit breaker state
    circuit_breaker_state TEXT CHECK (circuit_breaker_state IN ('closed', 'open', 'half_open')),

    checked_at TIMESTAMPTZ DEFAULT NOW(),

    INDEX idx_parser_id (parser_id),
    INDEX idx_checked_at (checked_at DESC)
);
```

### Entity Relationships

```
┌──────────────────┐
│ ledger_entries   │
│ (audit log)      │
└────────┬─────────┘
         │ 1
         │
         │ 0..1
         ▼
┌──────────────────┐
│ approval_requests│
│ (human review)   │
└──────────────────┘

┌──────────────────┐
│ provider_policies│
│ (access control) │
└──────────────────┘

┌──────────────────┐
│ parser_health    │
│ (monitoring)     │
└──────────────────┘
```

## API Architecture

### Request/Response Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Axum Router                            │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Middleware Stack                                  │    │
│  │  1. TraceLayer (request logging)                   │    │
│  │  2. CorsLayer (CORS headers)                       │    │
│  │  3. RateLimitLayer (60 req/min)                    │    │
│  │  4. AuthLayer (API key / JWT)                      │    │
│  │  5. RequestIdLayer (X-Request-ID)                  │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│                          ▼                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Routes                                            │    │
│  │  • POST   /api/process                             │    │
│  │  • GET    /api/approvals/:id                       │    │
│  │  • POST   /api/approvals/:id                       │    │
│  │  • GET    /api/ledger/query                        │    │
│  │  • GET    /api/ledger/:id                          │    │
│  │  • GET    /api/ledger/stats                        │    │
│  │  • GET    /health                                  │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│                          ▼                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Handlers (async functions)                        │    │
│  │  • process_input_handler()                         │    │
│  │  • get_approval_handler()                          │    │
│  │  • submit_approval_handler()                       │    │
│  │  • query_ledger_handler()                          │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│                          ▼                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │  State (Arc<AppState>)                             │    │
│  │  • db_pool: PgPool                                 │    │
│  │  • redis_client: RedisClient                       │    │
│  │  • parser_ensemble: ParserEnsemble                 │    │
│  │  • config: Config                                  │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Handler Example

```rust
async fn process_input_handler(
    State(app): State<Arc<AppState>>,
    Json(request): Json<ProcessRequest>,
) -> Result<Json<ProcessResponse>, ApiError> {
    let request_id = Uuid::new_v4();

    // 1. Malicious detection
    let malicious_result = app.malicious_detector
        .check(&request.user_input)
        .await?;

    if malicious_result.blocked {
        // Log to ledger and return blocked response
        return Ok(blocked_response(request_id, malicious_result.reason));
    }

    // 2. Parse with ensemble
    let parse_result = app.parser_ensemble
        .parse_all(&request.user_input)
        .await;

    // 3. Vote
    let voting_result = app.voting_module
        .vote(parse_result.results)
        .await?;

    // 4. Compare against policy
    let comparison_result = app.comparator
        .compare(&voting_result.canonical_intent, &app.config.provider)
        .await?;

    // 5. Check if approval needed
    if voting_result.requires_human_review || comparison_result.needs_approval {
        let approval = create_approval_request(...).await?;
        return Ok(pending_approval_response(request_id, approval.id));
    }

    // 6. Generate trusted intent
    let trusted_intent = app.intent_generator
        .generate(&voting_result.canonical_intent)
        .await?;

    // 7. Execute
    let result = app.processing_engine
        .execute(&trusted_intent)
        .await?;

    // 8. Log to ledger
    app.ledger.write_entry(...).await?;

    // 9. Return response
    Ok(Json(ProcessResponse {
        request_id,
        status: "completed",
        trusted_intent,
        result,
        pipeline_info: ...,
    }))
}
```

## Frontend Architecture

### Component Hierarchy

```
App (Route Container)
├── Layout (Header, Footer, Navigation)
│   ├── Header
│   │   ├── Logo
│   │   ├── Navigation
│   │   └── UserMenu
│   └── Footer
│
├── QueryInterface (Main Input Page)
│   ├── InputForm
│   │   ├── TextArea (user input)
│   │   └── SubmitButton
│   ├── ResultDisplay
│   │   ├── IntentVisualization
│   │   │   ├── ParsedIntentCard
│   │   │   ├── VotingResultCard
│   │   │   └── ComparisonResultCard
│   │   └── ProcessingResult
│   └── PipelineVisualization
│       └── StepIndicator (shows progress)
│
├── ApprovalReview (Admin Dashboard)
│   ├── ApprovalList
│   │   └── ApprovalCard (for each pending)
│   │       ├── IntentDiff
│   │       ├── ParserComparison
│   │       └── ApprovalActions
│   │           ├── ApproveButton
│   │           └── DenyButton
│   └── ApprovalDetails
│       ├── RawInput
│       ├── ParserOutputs
│       ├── VotingExplanation
│       └── PolicyViolations
│
└── AuditLogs (Ledger Viewer)
    ├── FilterPanel
    │   ├── UserFilter
    │   ├── DateRangeFilter
    │   └── StatusFilter
    ├── LogTable
    │   └── LogRow (for each entry)
    │       ├── Timestamp
    │       ├── UserID
    │       ├── Status
    │       └── ViewDetailsButton
    └── LogDetails (Modal)
        ├── InputDisplay
        ├── PipelineSteps
        ├── FinalIntent
        └── Result
```

### State Management

```typescript
// API Client Layer
class APIClient {
  async processInput(request: ProcessRequest): Promise<ProcessResponse>
  async getApproval(id: string): Promise<ApprovalRequest>
  async submitDecision(id: string, decision: Decision): Promise<void>
  async queryLedger(filters: LedgerFilters): Promise<LedgerEntry[]>
}

// React Query for data fetching
const { data, isLoading } = useQuery({
  queryKey: ['approval', approvalId],
  queryFn: () => apiClient.getApproval(approvalId),
})

// Local state with useState
const [userInput, setUserInput] = useState('')
const [result, setResult] = useState<ProcessResponse | null>(null)
```

## Deployment Architecture

### Docker Compose Setup

```
┌──────────────────────────────────────────────────────────┐
│                    Docker Network                        │
│                 (intent-network)                         │
│                                                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ PostgreSQL │  │   Redis    │  │   Ollama   │       │
│  │  :5432     │  │   :6379    │  │  :11434    │       │
│  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘       │
│         │                │                │             │
│         └────────────────┼────────────────┘             │
│                          │                              │
│                    ┌─────▼─────┐                        │
│                    │ API Server│                        │
│                    │   :3000   │                        │
│                    └─────┬─────┘                        │
│                          │                              │
│                    ┌─────▼─────┐                        │
│                    │  Frontend │                        │
│                    │   :5173   │                        │
│                    └───────────┘                        │
│                                                          │
│  [Optional: Monitoring]                                 │
│  ┌────────────┐  ┌────────────┐                        │
│  │ Prometheus │  │  Grafana   │                        │
│  │  :9090     │  │   :3001    │                        │
│  └────────────┘  └────────────┘                        │
└──────────────────────────────────────────────────────────┘
```

### Production Deployment (Kubernetes)

```
┌─────────────────────────────────────────────────┐
│              Kubernetes Cluster                 │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │         Ingress Controller               │  │
│  │         (nginx/traefik)                  │  │
│  │  TLS termination, rate limiting          │  │
│  └────────────┬─────────────────────────────┘  │
│               │                                 │
│  ┌────────────▼─────────────────────────────┐  │
│  │         API Service                      │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐ │  │
│  │  │ Pod 1   │  │ Pod 2   │  │ Pod 3   │ │  │
│  │  │ API     │  │ API     │  │ API     │ │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘ │  │
│  └───────┼────────────┼─────────────┼──────┘  │
│          │            │             │          │
│  ┌───────▼────────────▼─────────────▼──────┐  │
│  │       Shared Services Layer             │  │
│  │                                          │  │
│  │  ┌──────────────┐  ┌──────────────┐    │  │
│  │  │ PostgreSQL   │  │ Redis Cluster│    │  │
│  │  │ (StatefulSet)│  │              │    │  │
│  │  └──────────────┘  └──────────────┘    │  │
│  │                                          │  │
│  │  ┌──────────────┐                       │  │
│  │  │ Ollama       │                       │  │
│  │  │ (GPU nodes)  │                       │  │
│  │  └──────────────┘                       │  │
│  └──────────────────────────────────────────┘  │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │         Frontend Service                 │  │
│  │  ┌─────────┐  ┌─────────┐               │  │
│  │  │ Pod 1   │  │ Pod 2   │               │  │
│  │  └─────────┘  └─────────┘               │  │
│  └──────────────────────────────────────────┘  │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │         Monitoring Stack                 │  │
│  │  • Prometheus                            │  │
│  │  • Grafana                               │  │
│  │  • Alertmanager                          │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Technology Stack

### Backend

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust 1.75+ | Performance, safety, concurrency |
| Web Framework | Axum 0.7 | Async HTTP server |
| Database | PostgreSQL 15 | ACID transactions, JSONB support |
| Cache | Redis 7 | Session storage, rate limiting |
| ORM | SQLx 0.7 | Compile-time SQL verification |
| Async Runtime | Tokio 1.35 | Async task execution |
| Serialization | Serde 1.0 | JSON/struct conversion |
| Error Handling | thiserror, anyhow | Structured error types |
| Validation | validator 0.18 | Input validation |
| Logging | tracing 0.1 | Structured logging |
| HTTP Client | reqwest 0.11 | LLM API calls |

### Frontend

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | TypeScript 5.0+ | Type safety |
| Framework | React 18 | UI components |
| Build Tool | Vite 5.0 | Fast builds, HMR |
| Routing | React Router 6 | Client-side routing |
| HTTP Client | Axios | API communication |
| State Management | React Query | Server state caching |
| Styling | Tailwind CSS | Utility-first CSS |
| Charts | Recharts | Data visualization |

### Infrastructure

| Component | Technology | Purpose |
|-----------|------------|---------|
| Containerization | Docker | Application packaging |
| Orchestration | Docker Compose / K8s | Multi-service deployment |
| LLM (Local) | Ollama | Privacy-preserving inference |
| LLM (Cloud) | OpenAI API | High-quality parsing |
| Reverse Proxy | Nginx / Traefik | Load balancing, TLS |
| Monitoring | Prometheus + Grafana | Metrics and dashboards |
| Secrets | HashiCorp Vault (optional) | Secrets management |

### Development Tools

| Tool | Purpose |
|------|---------|
| `rustfmt` | Code formatting |
| `clippy` | Linting |
| `cargo-audit` | Security vulnerability scanning |
| `sqlx-cli` | Database migrations |
| `cargo-tarpaulin` | Code coverage |
| `prettier` | Frontend code formatting |
| `eslint` | Frontend linting |

---

## Performance Characteristics

### Latency Targets

| Operation | Target | Typical |
|-----------|--------|---------|
| Malicious Detection | <5ms | 2ms |
| Deterministic Parser | <10ms | <1ms |
| Ollama Parser | <2000ms | 500-1500ms |
| OpenAI Parser | <1000ms | 300-800ms |
| Voting Module | <50ms | 10ms |
| Intent Comparator | <100ms | 20ms |
| Processing Engine | <500ms | 100-300ms |
| Ledger Write | <100ms | 30ms |
| **Total (end-to-end)** | <3000ms | 800-2000ms |

### Throughput

- **Single Instance**: 100-200 requests/second
- **With Horizontal Scaling**: 1000+ requests/second
- **Database**: 10,000+ writes/second (PostgreSQL)
- **Cache**: 100,000+ ops/second (Redis)

---

This architecture provides defense-in-depth security while maintaining performance and auditability. Every layer is designed to fail safely, with deterministic fallbacks and human oversight for edge cases.
