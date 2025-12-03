# Intent Segregation API Server

A high-performance REST API server built with Axum that orchestrates the complete Intent Segregation Cybersecurity Architecture.

## Features

- **Full Pipeline Orchestration** - Integrates all modules (detector → parsers → voting → comparator → generator → engine → ledger)
- **RESTful Endpoints** - Clean, well-documented API for processing intents and managing approvals
- **Human-in-the-Loop** - Approval workflow for high-risk or uncertain operations
- **Audit Logging** - Complete immutable ledger of all operations
- **Configuration Management** - TOML-based configuration with environment variable overrides
- **Database Pooling** - Efficient PostgreSQL connection management
- **Middleware** - CORS, logging, request tracing, error handling
- **Graceful Shutdown** - Proper cleanup on SIGTERM/SIGINT
- **Static File Serving** - Optional frontend integration

## Architecture

```
User Request
    ↓
[Malicious Detector]
    ↓
[Parser Ensemble] → [Voting Module]
    ↓
[Intent Comparator]
    ↓
[Human Approval?] (if needed)
    ↓
[Intent Generator]
    ↓
[Processing Engine]
    ↓
[Ledger] → Response
```

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- (Optional) OpenAI API key for LLM parser
- (Optional) Ollama for local LLM parser

### Setup

1. **Clone and build:**
```bash
cd api
cargo build
```

2. **Set up database:**
```bash
# Create database
createdb intent_db

# Run migrations
sqlx migrate run
```

3. **Configure:**
Edit `config/default.toml` or use environment variables:
```bash
export APP__SERVER__PORT=3000
export APP__DATABASE__URL="postgresql://user:pass@localhost:5432/intent_db"
export APP__PARSERS__OPENAI_API_KEY="sk-..."  # Optional
```

4. **Run:**
```bash
cargo run
```

Server starts at `http://localhost:3000`

## API Endpoints

### Core Endpoints

- `POST /api/process` - Process user input through full pipeline
- `GET /api/approvals/:id` - Check approval status
- `POST /api/approvals/:id` - Submit approval decision
- `GET /api/ledger/query` - Query audit log
- `GET /api/ledger/:id` - Get specific ledger entry
- `GET /api/ledger/stats` - Get ledger statistics
- `GET /health` - Health check

### Example: Process Input

```bash
curl -X POST http://localhost:3000/api/process \
  -H "Content-Type: application/json" \
  -d '{
    "user_input": "What is 2 + 2?",
    "user_id": "user_123",
    "session_id": "session_456"
  }'
```

See [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) for complete API reference.

## Configuration

Configuration is loaded from:
1. `config/default.toml` (defaults)
2. `config/local.toml` (local overrides, optional)
3. Environment variables (highest priority)

### Key Settings

```toml
[server]
port = 3000
frontend_path = "../frontend/dist"  # Optional

[database]
url = "postgresql://..."
max_connections = 10

[parsers]
enable_deterministic = true
enable_openai = false    # Requires API key
enable_ollama = false    # Requires local Ollama

[provider]
allowed_actions = ["math_question"]
allowed_expertise = []
max_budget = 50000
require_human_approval = false
```

### Environment Variables

Override any setting with `APP__SECTION__KEY`:

```bash
export APP__SERVER__PORT=8080
export APP__DATABASE__URL="postgresql://..."
export APP__PARSERS__OPENAI_API_KEY="sk-..."
export APP__PROVIDER__MAX_BUDGET=100000
```

## Project Structure

```
api/
├── src/
│   ├── main.rs              # Server entry point
│   ├── config.rs            # Configuration loading
│   ├── state.rs             # Application state
│   ├── error.rs             # Error handling
│   ├── types.rs             # Request/response types
│   ├── middleware.rs        # Custom middleware
│   └── handlers/            # Route handlers
│       ├── mod.rs
│       ├── process.rs       # Main pipeline handler
│       ├── approval.rs      # Approval workflow
│       ├── ledger.rs        # Ledger queries
│       └── health.rs        # Health checks
├── config/
│   └── default.toml         # Default configuration
├── Cargo.toml
├── README.md                # This file
└── API_DOCUMENTATION.md     # Complete API docs
```

## Development

### Run in development mode:
```bash
cargo run
```

### Run with debug logging:
```bash
RUST_LOG=debug cargo run
```

### Run tests:
```bash
cargo test
```

### Format code:
```bash
cargo fmt
```

### Lint:
```bash
cargo clippy
```

## Production Deployment

### Build release binary:
```bash
cargo build --release
```

### Run with production config:
```bash
export APP__DATABASE__URL="postgresql://..."
export APP__SERVER__PORT=3000
./target/release/intent-api
```

### Docker:
```bash
docker build -t intent-api .
docker run -p 3000:3000 \
  -e APP__DATABASE__URL="postgresql://..." \
  intent-api
```

### Systemd Service:
```ini
[Unit]
Description=Intent Segregation API
After=network.target postgresql.service

[Service]
Type=simple
User=intent-api
WorkingDirectory=/opt/intent-api
Environment=APP__DATABASE__URL=postgresql://...
ExecStart=/opt/intent-api/intent-api
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## Security

- **Input Validation** - All inputs validated and sanitized
- **Malicious Detection** - Automatic blocking of attack patterns
- **Multi-Parser Validation** - Reduces single point of failure
- **Policy Enforcement** - Strict comparison against allowed operations
- **Audit Logging** - Immutable ledger of all operations
- **Human Approval** - Optional approval for high-risk operations

### Production Recommendations:

1. Enable HTTPS with proper certificates
2. Add authentication/authorization middleware
3. Enable rate limiting
4. Use strong database credentials
5. Keep API keys in secure secret management
6. Monitor and alert on suspicious patterns
7. Regular security audits

## Monitoring

The API provides:

- **Structured Logging** - JSON logs with tracing IDs
- **Health Endpoint** - `/health` for load balancers
- **Ledger Statistics** - Metrics via `/api/ledger/stats`
- **Request IDs** - Unique ID per request for tracking

Consider adding:
- Prometheus metrics
- Distributed tracing (OpenTelemetry)
- Error tracking (Sentry)
- Log aggregation (ELK/Loki)

## Troubleshooting

### Database Connection Errors
```bash
# Check PostgreSQL is running
pg_isready

# Verify connection string
psql "postgresql://user:pass@localhost:5432/intent_db"

# Run migrations
sqlx migrate run
```

### Parser Errors
```bash
# For OpenAI parser:
export APP__PARSERS__OPENAI_API_KEY="sk-..."
export APP__PARSERS__ENABLE_OPENAI=true

# For Ollama parser:
# Ensure Ollama is running
curl http://localhost:11434/api/tags
export APP__PARSERS__ENABLE_OLLAMA=true
```

### Port Already in Use
```bash
# Use different port
export APP__SERVER__PORT=8080
```

## Dependencies

Key dependencies:
- **axum** - Web framework
- **tokio** - Async runtime
- **sqlx** - Database access
- **tower-http** - HTTP middleware
- **serde** - Serialization
- **config** - Configuration management
- **tracing** - Structured logging

All core modules from the workspace:
- intent-schema
- malicious-detector
- intent-parsers
- intent-voting
- intent-comparator
- intent-generator
- processing-engine
- intent-ledger
- intent-supervision

## Contributing

1. Format code: `cargo fmt`
2. Run lints: `cargo clippy`
3. Run tests: `cargo test`
4. Update documentation
5. Submit PR

## License

MIT - See LICENSE file in repository root
