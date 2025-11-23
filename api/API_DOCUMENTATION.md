# Intent Segregation API Documentation

## Overview

The Intent Segregation API is a REST API built with Axum that orchestrates all modules in the Intent Segregation Cybersecurity Architecture. It provides secure, auditable processing of user intents through a multi-stage pipeline.

## Architecture

The API orchestrates the following pipeline:

1. **Malicious Input Detection** - Fast regex-based detection of attack patterns
2. **Parser Ensemble** - Multiple independent parsers extract structured intents
3. **Voting Module** - Compares parser results and determines consensus
4. **Intent Comparator** - Validates intent against provider policy
5. **Human Approval** - Optional human-in-the-loop for high-risk intents
6. **Intent Generator** - Creates trusted, canonical intent
7. **Processing Engine** - Executes the validated intent
8. **Audit Ledger** - Immutable logging of all operations

## Base URL

```
http://localhost:3000
```

## Endpoints

### Health Check

#### GET /health

Returns the health status of the API and its services.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "services": {
    "database": true,
    "parsers": true,
    "ledger": true
  }
}
```

---

### Process Input

#### POST /api/process

Process user input through the complete intent segregation pipeline.

**Request Body:**
```json
{
  "user_input": "Find me security experts for a supply chain project with budget $20000",
  "user_id": "user_123",
  "session_id": "session_456"
}
```

**Success Response (Completed):**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "trusted_intent": {
    "action": "find_experts",
    "topic_id": "supply_chain",
    "expertise": ["security"],
    "constraints": {
      "max_budget": 20000
    },
    "content_refs": [],
    "metadata": {
      "id": "...",
      "timestamp": "2024-01-15T10:30:00Z",
      "user_id": "user_123",
      "session_id": "session_456"
    }
  },
  "result": {
    "experts": [
      {
        "id": "expert_1",
        "name": "Alice Smith",
        "expertise": ["security", "supply_chain"],
        "rate": 150
      }
    ]
  },
  "message": "Intent processed successfully",
  "pipeline_info": {
    "malicious_detection": {
      "blocked": false,
      "reason": null
    },
    "parser_results": [
      {
        "parser_id": "deterministic",
        "success": true,
        "confidence": 1.0
      },
      {
        "parser_id": "llm_parser_1",
        "success": true,
        "confidence": 0.95
      }
    ],
    "voting_result": {
      "confidence_level": "HighConfidence",
      "average_similarity": 0.98,
      "requires_human_review": false,
      "explanation": "All parsers are in strong agreement..."
    },
    "comparison_result": {
      "result": "approved",
      "message": "Intent approved - all checks passed",
      "reasons": []
    }
  }
}
```

**Response (Pending Approval):**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "pending_approval",
  "trusted_intent": null,
  "result": null,
  "message": "Request requires human approval. Use GET /api/approvals/550e8400-e29b-41d4-a716-446655440001 to check status.",
  "pipeline_info": { ... }
}
```

**Response (Blocked):**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440002",
  "status": "blocked",
  "trusted_intent": null,
  "result": null,
  "message": "Input blocked: Command injection detected",
  "pipeline_info": {
    "malicious_detection": {
      "blocked": true,
      "reason": "Command injection detected"
    }
  }
}
```

**Status Codes:**
- `200 OK` - Request processed (check status field for outcome)
- `400 Bad Request` - Invalid request format
- `500 Internal Server Error` - Server error during processing

---

### Approval Endpoints

#### GET /api/approvals/:id

Check the status of an approval request.

**Parameters:**
- `id` (path) - UUID of the approval request

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "intent": {
    "action": "find_experts",
    ...
  },
  "reason": "Parser conflict: parsers disagree on expertise area",
  "created_at": "2024-01-15T10:30:00Z",
  "decision": null
}
```

**Response (After Decision):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "approved",
  "intent": { ... },
  "reason": "Parser conflict: parsers disagree on expertise area",
  "created_at": "2024-01-15T10:30:00Z",
  "decision": {
    "approved": true,
    "approver_id": "admin_789",
    "reason": "Intent looks valid after manual review",
    "decided_at": "2024-01-15T10:35:00Z"
  }
}
```

**Status Codes:**
- `200 OK` - Approval found
- `404 Not Found` - Approval not found

#### POST /api/approvals/:id

Submit an approval decision for a pending request.

**Parameters:**
- `id` (path) - UUID of the approval request

**Request Body:**
```json
{
  "approved": true,
  "approver_id": "admin_789",
  "reason": "Intent looks valid after manual review"
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "approved": true,
  "message": "Intent approved by admin_789. Processing will continue."
}
```

**Status Codes:**
- `200 OK` - Decision recorded
- `404 Not Found` - Approval not found
- `409 Conflict` - Approval already decided

---

### Ledger Endpoints

#### GET /api/ledger/query

Query the audit ledger with various filters.

**Query Parameters:**
- `user_id` (optional) - Filter by user ID
- `session_id` (optional) - Filter by session ID
- `start_time` (optional) - Start of time range (ISO 8601)
- `end_time` (optional) - End of time range (ISO 8601)
- `elevation_only` (optional) - Only entries requiring elevation (true/false)
- `blocked_only` (optional) - Only blocked entries (true/false)
- `limit` (optional) - Maximum number of results (default: 100)

**Example Request:**
```
GET /api/ledger/query?user_id=user_123&limit=50
```

**Response:**
```json
{
  "entries": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user_id": "user_123",
      "session_id": "session_456",
      "timestamp": "2024-01-15T10:30:00Z",
      "user_input": "Find security experts...",
      "malicious_blocked": false,
      "voting_confidence": "HighConfidence",
      "comparison_decision": "Approved",
      "required_approval": false,
      "was_executed": true
    }
  ],
  "count": 1
}
```

**Status Codes:**
- `200 OK` - Query successful
- `400 Bad Request` - Invalid query parameters
- `500 Internal Server Error` - Database error

#### GET /api/ledger/:id

Get a specific ledger entry by ID.

**Parameters:**
- `id` (path) - UUID of the ledger entry

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "session_456",
  "user_id": "user_123",
  "timestamp": "2024-01-15T10:30:00Z",
  "user_input": "Find security experts for supply chain project",
  "user_input_hash": "a1b2c3...",
  "malicious_score": null,
  "malicious_blocked": false,
  "voting_result": { ... },
  "comparison_result": { ... },
  "elevation_event": null,
  "trusted_intent": { ... },
  "processing_output": { ... },
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0..."
}
```

**Status Codes:**
- `200 OK` - Entry found
- `404 Not Found` - Entry not found

#### GET /api/ledger/stats

Get aggregate statistics about the ledger.

**Response:**
```json
{
  "total_entries": 1523,
  "total_users": 47,
  "total_sessions": 892,
  "blocked_entries": 23,
  "elevation_events": 15,
  "oldest_entry": "2024-01-01T00:00:00Z",
  "newest_entry": "2024-01-15T12:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Stats retrieved successfully

---

## Error Responses

All endpoints may return error responses in the following format:

```json
{
  "error": {
    "type": "validation_error",
    "message": "Invalid user_id format"
  }
}
```

**Error Types:**
- `malicious_input` - Input blocked by malicious detector
- `parser_error` - Parser ensemble failed
- `voting_error` - Voting module failed
- `comparison_error` - Intent comparator failed
- `generation_error` - Intent generator failed
- `processing_error` - Processing engine failed
- `ledger_error` - Ledger operation failed
- `database_error` - Database operation failed
- `approval_not_found` - Approval request not found
- `approval_already_decided` - Approval already has a decision
- `approval_pending` - Approval is still pending
- `validation_error` - Request validation failed
- `not_found` - Resource not found
- `internal_error` - Unexpected server error

---

## Configuration

The API is configured via `config/default.toml`. Key settings:

```toml
[server]
port = 3000
frontend_path = "../frontend/dist"

[database]
url = "postgresql://user:pass@localhost:5432/db"
max_connections = 10

[parsers]
enable_deterministic = true
enable_openai = false
enable_ollama = false

[provider]
allowed_actions = ["find_experts", "summarize", "draft_proposal"]
allowed_expertise = ["ml", "security", "embedded", "cloud"]
max_budget = 50000
max_results = 20
require_human_approval = false

[notifications]
enable_email = false
admin_emails = []
```

Settings can be overridden with environment variables:
```bash
export APP__SERVER__PORT=8080
export APP__DATABASE__URL="postgresql://..."
export APP__PARSERS__OPENAI_API_KEY="sk-..."
```

---

## Running the Server

### Development

```bash
# Set up database
createdb intent_db
sqlx migrate run

# Run server
cargo run --bin intent-api

# Server will start on http://localhost:3000
```

### Production

```bash
# Build release binary
cargo build --release

# Run with production config
export APP__DATABASE__URL="postgresql://..."
./target/release/intent-api
```

### Docker

```bash
# Build image
docker build -t intent-api .

# Run container
docker run -p 3000:3000 \
  -e APP__DATABASE__URL="postgresql://..." \
  intent-api
```

---

## Examples

### Process a Simple Request

```bash
curl -X POST http://localhost:3000/api/process \
  -H "Content-Type: application/json" \
  -d '{
    "user_input": "Find ML experts for my project",
    "user_id": "user_123",
    "session_id": "sess_456"
  }'
```

### Check Approval Status

```bash
curl http://localhost:3000/api/approvals/550e8400-e29b-41d4-a716-446655440000
```

### Submit Approval Decision

```bash
curl -X POST http://localhost:3000/api/approvals/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "approved": true,
    "approver_id": "admin_789",
    "reason": "Looks good"
  }'
```

### Query Ledger

```bash
# Get recent entries for a user
curl "http://localhost:3000/api/ledger/query?user_id=user_123&limit=10"

# Get entries requiring elevation
curl "http://localhost:3000/api/ledger/query?elevation_only=true"

# Get entries in time range
curl "http://localhost:3000/api/ledger/query?start_time=2024-01-01T00:00:00Z&end_time=2024-01-15T23:59:59Z"
```

### Get Ledger Statistics

```bash
curl http://localhost:3000/api/ledger/stats
```

---

## Security Considerations

1. **Input Validation** - All user input is validated and sanitized
2. **Malicious Detection** - Automatic blocking of known attack patterns
3. **Multi-Parser Validation** - Multiple independent parsers reduce risk
4. **Policy Enforcement** - Strict comparison against provider policies
5. **Human Approval** - Optional human-in-the-loop for high-risk operations
6. **Audit Logging** - Immutable ledger of all operations
7. **Rate Limiting** - Consider implementing rate limiting in production
8. **Authentication** - Add authentication/authorization as needed
9. **HTTPS** - Use HTTPS in production with proper certificates
10. **Database Security** - Use strong credentials and encrypted connections

---

## Monitoring and Observability

The API includes:

- **Structured Logging** - All operations logged with tracing
- **Request IDs** - Every request gets a unique ID for tracking
- **Health Checks** - `/health` endpoint for load balancers
- **Ledger Statistics** - Aggregate metrics via `/api/ledger/stats`

For production, consider adding:

- Prometheus metrics endpoint
- Distributed tracing (OpenTelemetry)
- Error tracking (Sentry)
- Log aggregation (ELK stack)

---

## Database Schema

The ledger uses PostgreSQL with the following main table:

```sql
CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY,
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    user_input TEXT NOT NULL,
    user_input_hash TEXT NOT NULL,
    malicious_score FLOAT,
    malicious_blocked BOOLEAN NOT NULL,
    voting_result JSONB NOT NULL,
    comparison_result JSONB NOT NULL,
    elevation_event JSONB,
    trusted_intent JSONB,
    processing_output JSONB,
    ip_address TEXT,
    user_agent TEXT,

    INDEX idx_user_id (user_id),
    INDEX idx_session_id (session_id),
    INDEX idx_timestamp (timestamp)
);
```

---

## Development

### Running Tests

```bash
cargo test --all
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Security audit
cargo audit
```

### Adding New Endpoints

1. Add handler function in `src/handlers/`
2. Add route in `src/main.rs::build_router()`
3. Add request/response types in `src/types.rs`
4. Update this documentation

---

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-org/intent-segregation
- Documentation: See README.md in project root
- Architecture: See system architecture diagram in README.md

---

## License

MIT License - See LICENSE file for details
