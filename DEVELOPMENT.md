# Development Guide

Complete guide for local development, debugging, and common development tasks for the Intent Segregation Cybersecurity Architecture project.

## Table of Contents

1. [Local Development Setup](#local-development-setup)
2. [Running in Development Mode](#running-in-development-mode)
3. [Debugging](#debugging)
4. [Database Management](#database-management)
5. [Working with LLM Providers](#working-with-llm-providers)
6. [Common Development Tasks](#common-development-tasks)
7. [Testing](#testing)
8. [Troubleshooting](#troubleshooting)

## Local Development Setup

### Prerequisites Installation

#### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install postgresql@15 redis ollama
brew install rust node@18

# Start services
brew services start postgresql@15
brew services start redis
```

#### Ubuntu/Debian

```bash
# Update package list
sudo apt update

# Install build essentials
sudo apt install -y build-essential pkg-config libssl-dev curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib libpq-dev
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Install Redis
sudo apt install -y redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
```

#### Fedora/RHEL/CentOS

```bash
# Install development tools
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkg-config openssl-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL
sudo dnf install -y postgresql-server postgresql-contrib postgresql-devel
sudo postgresql-setup --initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Install Redis
sudo dnf install -y redis
sudo systemctl start redis
sudo systemctl enable redis

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Install Node.js
sudo dnf install -y nodejs npm
```

### Project Setup

1. **Clone the repository**

```bash
git clone https://github.com/your-org/Intent-Segregation-Cybersecurity-Architecture-for-AI.git
cd Intent-Segregation-Cybersecurity-Architecture-for-AI
```

2. **Create and configure environment file**

```bash
cp .env.example .env
```

Edit `.env` with your settings:

```bash
# Essential settings for local development
DATABASE_URL=postgresql://intent_user:your_password@localhost:5432/intent_segregation
REDIS_HOST=localhost
REDIS_PORT=6379

# Enable/disable parsers
ENABLE_DETERMINISTIC=true
ENABLE_OLLAMA=true
ENABLE_OPENAI=false  # Set to true if you have an API key

# Ollama configuration
OLLAMA_ENDPOINT=http://localhost:11434
OLLAMA_MODEL=llama2

# OpenAI (optional)
OPENAI_API_KEY=sk-your-api-key-here
OPENAI_MODEL=gpt-4o-mini

# Development settings
ENVIRONMENT=development
LOG_LEVEL=debug
RUST_LOG=intent_segregation=debug,tower_http=debug
```

3. **Set up the database**

```bash
# Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE intent_segregation;
CREATE USER intent_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE intent_segregation TO intent_user;
ALTER DATABASE intent_segregation OWNER TO intent_user;
EOF
```

4. **Install SQLx CLI for migrations**

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

5. **Run database migrations**

```bash
sqlx migrate run
```

6. **Pull Ollama models**

```bash
ollama pull llama2
# Optional: pull other models
ollama pull mistral
```

7. **Build the project**

```bash
cargo build
```

8. **Install frontend dependencies**

```bash
cd frontend
npm install
cd ..
```

## Running in Development Mode

### Option 1: Quick Start (Script)

```bash
./run_local.sh
```

This script:
- Checks that all services are running
- Runs database migrations
- Starts the API server
- Starts the frontend dev server

### Option 2: Manual Start

#### Terminal 1: API Server

```bash
# Run with hot-reload using cargo-watch
cargo install cargo-watch
cargo watch -x run
```

Or without hot-reload:

```bash
cargo run --bin intent-api
# API will be available at http://localhost:3000
```

#### Terminal 2: Frontend Dev Server

```bash
cd frontend
npm run dev
# Frontend will be available at http://localhost:5173
```

#### Terminal 3: Ollama (if not running as service)

```bash
ollama serve
```

### Option 3: Docker Compose

```bash
docker-compose up
```

This starts all services in containers:
- PostgreSQL on port 5432
- Redis on port 6379
- Ollama on port 11434
- API on port 3000
- Frontend on port 5173

## Debugging

### Rust Backend Debugging

#### Using VS Code

1. Install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension

2. Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug API Server",
      "cargo": {
        "args": [
          "build",
          "--bin=intent-api",
          "--package=intent-api"
        ],
        "filter": {
          "name": "intent-api",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug",
        "DATABASE_URL": "postgresql://intent_user:password@localhost:5432/intent_segregation"
      }
    }
  ]
}
```

3. Set breakpoints in your code
4. Press F5 to start debugging

#### Using rust-gdb

```bash
# Build with debug symbols
cargo build --bin intent-api

# Run with gdb
rust-gdb target/debug/intent-api

# In gdb
(gdb) break main
(gdb) run
(gdb) step
(gdb) print variable_name
```

#### Logging

Enable detailed logging:

```bash
# Set environment variable
export RUST_LOG=debug

# Or for specific modules
export RUST_LOG=intent_segregation=trace,intent_parsers=debug,sqlx=info

# Run
cargo run
```

View structured logs with formatting:

```bash
cargo run 2>&1 | jq -R 'fromjson? | .'
```

### Frontend Debugging

#### Browser DevTools

1. Open Chrome/Firefox DevTools (F12)
2. Check Console for errors
3. Use Network tab to inspect API calls
4. Use React DevTools extension

#### VS Code Debugging

Install [Debugger for Chrome](https://marketplace.visualstudio.com/items?itemName=msjsdiag.debugger-for-chrome)

Add to `.vscode/launch.json`:

```json
{
  "type": "chrome",
  "request": "launch",
  "name": "Debug Frontend",
  "url": "http://localhost:5173",
  "webRoot": "${workspaceFolder}/frontend/src",
  "sourceMapPathOverrides": {
    "webpack:///./src/*": "${webRoot}/*"
  }
}
```

### Database Debugging

#### Inspect Database

```bash
# Connect to PostgreSQL
psql postgresql://intent_user:password@localhost:5432/intent_segregation

# List tables
\dt

# Describe table
\d ledger_entries

# Query recent entries
SELECT * FROM ledger_entries ORDER BY timestamp DESC LIMIT 10;

# Check approval requests
SELECT * FROM approval_requests WHERE status = 'pending';
```

#### Query Logs

Enable PostgreSQL query logging:

```bash
# Edit postgresql.conf
sudo nano /etc/postgresql/15/main/postgresql.conf

# Add these lines:
log_statement = 'all'
log_duration = on

# Restart PostgreSQL
sudo systemctl restart postgresql

# View logs
sudo tail -f /var/log/postgresql/postgresql-15-main.log
```

#### SQLx Query Logs

```bash
# Enable SQLx query logging
export RUST_LOG=sqlx=debug

cargo run
```

### Performance Profiling

#### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile the application
cargo flamegraph --bin intent-api

# Open flamegraph.svg in browser
```

#### Memory Profiling

```bash
# Install heaptrack
sudo apt install heaptrack  # Ubuntu
brew install heaptrack      # macOS

# Run with heaptrack
heaptrack target/debug/intent-api

# Analyze results
heaptrack_gui heaptrack.intent-api.*
```

## Database Management

### Migrations

#### Create a New Migration

```bash
sqlx migrate add create_new_table

# Edit the generated file in migrations/
# Example: migrations/20240115000000_create_new_table.sql
```

```sql
-- Migration: create_new_table
-- Description: Creates a new table for storing X

CREATE TABLE new_table (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_new_table_name ON new_table(name);
```

#### Run Migrations

```bash
# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

#### Check Migration Status

```bash
sqlx migrate info
```

### Database Backups

#### Backup

```bash
# Backup entire database
pg_dump -U intent_user intent_segregation > backup_$(date +%Y%m%d).sql

# Backup with compression
pg_dump -U intent_user intent_segregation | gzip > backup_$(date +%Y%m%d).sql.gz

# Backup specific table
pg_dump -U intent_user -t ledger_entries intent_segregation > ledger_backup.sql
```

#### Restore

```bash
# Restore from backup
psql -U intent_user intent_segregation < backup_20240115.sql

# Restore from compressed backup
gunzip -c backup_20240115.sql.gz | psql -U intent_user intent_segregation
```

### Database Seeding

Create a seed script `scripts/seed_database.sql`:

```sql
-- Seed provider policies
INSERT INTO provider_policies (provider_name, config, active)
VALUES (
    'b2b_consulting',
    '{"allowed_actions": ["find_experts", "summarize"], "max_budget": 100000}'::jsonb,
    true
);

-- Add test users, etc.
```

Run the seed:

```bash
psql -U intent_user intent_segregation < scripts/seed_database.sql
```

### Reset Database

```bash
# Drop and recreate database
sudo -u postgres psql << EOF
DROP DATABASE IF EXISTS intent_segregation;
CREATE DATABASE intent_segregation;
GRANT ALL PRIVILEGES ON DATABASE intent_segregation TO intent_user;
ALTER DATABASE intent_segregation OWNER TO intent_user;
EOF

# Run migrations
sqlx migrate run

# Seed database
psql -U intent_user intent_segregation < scripts/seed_database.sql
```

## Working with LLM Providers

### Ollama (Local)

#### Starting Ollama

```bash
# As a service (Linux)
sudo systemctl start ollama

# Manual start
ollama serve
```

#### Managing Models

```bash
# List available models
ollama list

# Pull a model
ollama pull llama2
ollama pull mistral

# Remove a model
ollama rm llama2

# Show model info
ollama show llama2
```

#### Testing Ollama Directly

```bash
# Interactive chat
ollama run llama2

# Single query
curl http://localhost:11434/api/generate -d '{
  "model": "llama2",
  "prompt": "Why is the sky blue?",
  "stream": false
}'
```

#### Troubleshooting Ollama

```bash
# Check if Ollama is running
curl http://localhost:11434/

# Check logs
journalctl -u ollama -f

# Restart Ollama
sudo systemctl restart ollama
```

### OpenAI API

#### Set API Key

```bash
# In .env file
OPENAI_API_KEY=sk-your-api-key-here

# Or export temporarily
export OPENAI_API_KEY=sk-your-api-key-here
```

#### Test OpenAI Connection

```bash
curl https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

#### Monitor API Usage

Check usage at: https://platform.openai.com/usage

### Adding a New LLM Provider

1. **Create parser file**: `core/parsers/src/anthropic.rs`

```rust
use async_trait::async_trait;
use crate::{IntentParser, ParsedIntent, ParserError, ParserType};

pub struct AnthropicParser {
    api_key: String,
    model: String,
}

#[async_trait]
impl IntentParser for AnthropicParser {
    async fn parse(&self, user_input: &str) -> Result<ParsedIntent, ParserError> {
        // Implementation
        todo!()
    }

    fn parser_type(&self) -> ParserType {
        ParserType::LLM
    }

    fn parser_id(&self) -> String {
        "anthropic".to_string()
    }

    fn trust_level(&self) -> f64 {
        0.85
    }
}
```

2. **Add to ensemble**: Update `core/parsers/src/ensemble.rs`

3. **Add configuration**: Update `.env.example` and `config.rs`

4. **Write tests**: Create `core/parsers/tests/anthropic_tests.rs`

## Common Development Tasks

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p intent-parsers

# Run specific test
cargo test test_deterministic_parser

# Run with output
cargo test -- --nocapture

# Run ignored tests (e.g., requires API keys)
cargo test -- --ignored

# Run in parallel
cargo test -- --test-threads=4
```

### Code Formatting and Linting

```bash
# Format code
cargo fmt

# Check formatting without changing
cargo fmt -- --check

# Lint with clippy
cargo clippy

# Clippy with warnings as errors
cargo clippy -- -D warnings

# Fix clippy suggestions automatically
cargo clippy --fix
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build specific package
cargo build -p intent-api

# Clean build artifacts
cargo clean
```

### Generating Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate without dependencies
cargo doc --no-deps

# Generate for specific package
cargo doc -p intent-parsers --open
```

### Checking Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Security audit
cargo audit

# Install cargo-deny for dependency policies
cargo install cargo-deny
cargo deny check
```

### Database Tasks

```bash
# Create migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Frontend Tasks

```bash
cd frontend

# Install dependencies
npm install

# Start dev server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Lint
npm run lint

# Format
npm run format

# Type check
npm run type-check
```

## Testing

### Unit Tests

Run unit tests for a specific module:

```bash
# Test parsers
cargo test -p intent-parsers

# Test voting
cargo test -p intent-voting

# Test with coverage
cargo tarpaulin -p intent-parsers
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run specific integration test
cargo test --test integration test_full_pipeline
```

### Red-Team Tests

```bash
# Run red-team security tests
cargo test --test redteam

# Run with detailed output
cargo test --test redteam -- --nocapture

# Run specific attack scenario
cargo test --test redteam test_prompt_injection
```

### End-to-End Tests

```bash
# Start all services
./run_local.sh

# In another terminal, run E2E tests
cd frontend
npm run test:e2e
```

### Load Testing

```bash
# Install k6
brew install k6  # macOS
# OR
wget https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz

# Create load test script: load_test.js
cat > load_test.js << 'EOF'
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 10,
  duration: '30s',
};

export default function() {
  let payload = JSON.stringify({
    user_input: 'Find ML experts with $50k budget',
    user_id: 'test_user',
    session_id: 'test_session'
  });

  let params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  let res = http.post('http://localhost:3000/api/process', payload, params);

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 3000ms': (r) => r.timings.duration < 3000,
  });
}
EOF

# Run load test
k6 run load_test.js
```

## Troubleshooting

### Common Issues

#### 1. Database Connection Fails

**Symptom**: `Error: connection refused` or `could not connect to server`

**Solutions**:

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Start PostgreSQL
sudo systemctl start postgresql

# Check connection string in .env
echo $DATABASE_URL

# Test connection
psql $DATABASE_URL

# Check PostgreSQL logs
sudo tail -f /var/log/postgresql/postgresql-15-main.log
```

#### 2. Ollama Parser Fails

**Symptom**: `Error: Failed to connect to Ollama` or timeouts

**Solutions**:

```bash
# Check if Ollama is running
curl http://localhost:11434/

# Start Ollama
ollama serve

# Check if model is available
ollama list

# Pull model if missing
ollama pull llama2

# Check Ollama logs
journalctl -u ollama -f

# Increase timeout in .env
OLLAMA_TIMEOUT=60
```

#### 3. Build Fails

**Symptom**: Compilation errors

**Solutions**:

```bash
# Update Rust toolchain
rustup update stable

# Clean and rebuild
cargo clean
cargo build

# Check for missing system dependencies
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
```

#### 4. Tests Fail

**Symptom**: Test failures

**Solutions**:

```bash
# Ensure test database exists
createdb intent_segregation_test

# Set test database URL
export TEST_DATABASE_URL=postgresql://intent_user:password@localhost:5432/intent_segregation_test

# Run migrations on test database
sqlx migrate run --database-url $TEST_DATABASE_URL

# Run tests with output to see details
cargo test -- --nocapture
```

#### 5. Frontend Build Fails

**Symptom**: npm errors or missing dependencies

**Solutions**:

```bash
cd frontend

# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Check Node version (should be 18+)
node --version

# Update npm
npm install -g npm@latest
```

#### 6. Redis Connection Issues

**Symptom**: `Error: Connection refused` to Redis

**Solutions**:

```bash
# Check if Redis is running
redis-cli ping

# Start Redis
sudo systemctl start redis

# Check Redis configuration
redis-cli
> CONFIG GET bind
> CONFIG GET protected-mode
```

#### 7. CORS Errors in Frontend

**Symptom**: CORS policy errors in browser console

**Solutions**:

Check API CORS configuration in `api/src/main.rs`:

```rust
let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE]);
```

Update frontend API URL in `frontend/.env`:

```bash
VITE_API_URL=http://localhost:3000
```

### Performance Issues

#### Slow Database Queries

```sql
-- Enable query logging
ALTER DATABASE intent_segregation SET log_statement = 'all';
ALTER DATABASE intent_segregation SET log_duration = on;

-- Check for missing indexes
SELECT schemaname, tablename, indexname
FROM pg_indexes
WHERE schemaname = 'public';

-- Analyze query performance
EXPLAIN ANALYZE SELECT * FROM ledger_entries WHERE user_id = 'user_123';
```

#### High Memory Usage

```bash
# Check memory usage
htop

# Profile with heaptrack
heaptrack target/release/intent-api

# Reduce parser concurrency in code
# or limit Ollama memory usage in Docker
```

#### Slow Parser Response

```bash
# Check individual parser performance
curl -X POST http://localhost:3000/api/process \
  -H "Content-Type: application/json" \
  -d '{"user_input": "test", "user_id": "test"}' \
  -w "\nTime: %{time_total}s\n"

# Monitor parser health
psql -U intent_user intent_segregation \
  -c "SELECT * FROM parser_health ORDER BY checked_at DESC;"
```

### Getting Help

If you're still stuck:

1. **Check logs**: Look at application logs, database logs, and system logs
2. **Search issues**: Check GitHub issues for similar problems
3. **Ask for help**: Open a new GitHub issue with:
   - Error messages
   - Steps to reproduce
   - Environment details (OS, versions, etc.)
   - Relevant logs

### Useful Commands Reference

```bash
# Check all services
sudo systemctl status postgresql redis ollama

# View all logs
tail -f logs/*.log

# Monitor resources
htop

# Check ports
sudo netstat -tulpn | grep LISTEN

# Test API health
curl http://localhost:3000/health | jq

# Interactive database session
psql $DATABASE_URL

# Interactive Redis session
redis-cli

# Check Rust version
rustc --version

# Check Node version
node --version

# List running Docker containers
docker ps
```

---

Happy coding! For more information, see:
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [docs/MODULE_GUIDE.md](docs/MODULE_GUIDE.md) - Module documentation
- [docs/SECURITY.md](docs/SECURITY.md) - Security documentation
