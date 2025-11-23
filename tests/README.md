# Integration and Regression Tests

This directory contains comprehensive integration and regression tests for the Intent Segregation Cybersecurity Architecture.

## Directory Structure

```
tests/
├── integration/              # Integration tests
│   ├── end_to_end_test.rs   # Full pipeline testing
│   ├── api_integration_test.rs  # API endpoint tests
│   ├── database_integration_test.rs  # Database operations
│   ├── llm_integration_test.rs  # LLM parser integration (with mocks)
│   └── mod.rs
├── regression/              # Regression tests
│   ├── known_issues_test.rs  # Previously fixed bugs
│   ├── performance_regression_test.rs  # Performance benchmarks
│   ├── security_regression_test.rs  # Security issue tests
│   └── mod.rs
├── fixtures/                # Test data and mock responses
│   ├── user_inputs.json
│   ├── provider_configs.json
│   └── mock_llm_responses.json
├── test_helpers.rs          # Shared test utilities
├── integration.rs           # Integration tests entry point
└── regression.rs            # Regression tests entry point
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Integration Tests Only
```bash
cargo test --test integration
```

### Run Regression Tests Only
```bash
cargo test --test regression
```

### Run Specific Test File
```bash
# Integration tests
cargo test --test integration end_to_end
cargo test --test integration api_integration
cargo test --test integration database_integration
cargo test --test integration llm_integration

# Regression tests
cargo test --test regression known_issues
cargo test --test regression performance_regression
cargo test --test regression security_regression
```

### Run Specific Test by Name
```bash
cargo test test_e2e_clean_input_approved_and_executed
cargo test test_api_process_endpoint_clean_input
cargo test test_perf_parser_response_time_under_100ms
```

### Run Tests with Output
```bash
cargo test -- --nocapture
cargo test --test integration -- --nocapture
```

### Run Tests in Parallel
```bash
cargo test -- --test-threads=4
```

## Test Categories

### Integration Tests

#### 1. End-to-End Tests (`end_to_end_test.rs`)
Tests the complete flow from user input to final result:
- ✅ Clean input → Approved → Executed
- ✅ Malicious input detection and blocking
- ✅ Parser disagreement → Voting → Escalation
- ✅ Policy violations (soft and hard mismatches)
- ✅ Approval workflow
- ✅ Ledger auditing
- ✅ Error handling and recovery

#### 2. API Integration Tests (`api_integration_test.rs`)
Tests HTTP API endpoints:
- `/api/process` - Process user input
- `/api/approval/:id` - Get approval status
- `/api/approval/:id/decision` - Submit approval decision
- `/api/ledger` - Query ledger entries
- `/api/health` - Health check

#### 3. Database Integration Tests (`database_integration_test.rs`)
Tests database operations:
- Ledger entry storage and retrieval
- Approval request management
- Query filtering and pagination
- Statistics and analytics
- Concurrent access
- Performance benchmarks

#### 4. LLM Integration Tests (`llm_integration_test.rs`)
Tests LLM parser integration with mocks:
- Parser ensemble consensus
- Individual parser behavior
- Error handling and retries
- Confidence scoring
- Security (injection prevention)

### Regression Tests

#### 1. Known Issues Tests (`known_issues_test.rs`)
Ensures previously fixed bugs stay fixed:
- Budget parsing with comma separators (Issue #001)
- Case-insensitive expertise matching (Issue #002)
- Empty array handling (Issue #003)
- Unicode character support (Issue #004)
- Voting with single parser (Issue #010)
- And more...

#### 2. Performance Regression Tests (`performance_regression_test.rs`)
Tracks performance over time:
- Parser response time < 100ms
- Voting completion < 50ms
- Policy comparison < 10ms
- E2E pipeline < 500ms
- Throughput ≥ 100 req/s
- Database operations benchmarks

#### 3. Security Regression Tests (`security_regression_test.rs`)
Validates security fixes remain in place:
- Prompt injection prevention
- SQL injection protection
- Command injection blocking
- XSS prevention
- Path traversal protection
- Authorization checks
- Rate limiting

## Test Fixtures

### User Inputs (`fixtures/user_inputs.json`)
Sample inputs covering:
- Clean, valid inputs
- Malicious injection attempts
- Edge cases (empty, very long, multilingual)
- Policy violations

### Provider Configs (`fixtures/provider_configs.json`)
Different policy configurations:
- Default config (balanced)
- Restrictive config (strict limits)
- Permissive config (minimal restrictions)

### Mock LLM Responses (`fixtures/mock_llm_responses.json`)
Simulated parser responses for:
- High consensus scenarios
- Parser disagreements
- Conflicts requiring escalation

## Test Helpers (`test_helpers.rs`)

Utility functions and builders:
- `IntentBuilder` - Build test Intent objects
- `ParsedIntentBuilder` - Build ParsedIntent objects
- `VotingResultBuilder` - Build VotingResult objects
- `MockMaliciousDetector` - Mock malicious content detection
- `MockParser` - Mock LLM parser
- Database setup/teardown helpers
- Assertion helpers
- Timing and performance measurement tools

## Best Practices

1. **Isolation**: Each test should be independent and not rely on other tests
2. **Cleanup**: Use setup/teardown for database tests
3. **Mocking**: Use mocks for external services (LLMs, databases in unit tests)
4. **Assertions**: Use descriptive assertion messages
5. **Documentation**: Each test should have a comment explaining what it tests
6. **Regression**: When fixing a bug, add a regression test

## Coverage

To generate test coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Open coverage report
open coverage/index.html
```

## CI/CD Integration

These tests are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Run Integration Tests
  run: cargo test --test integration

- name: Run Regression Tests
  run: cargo test --test regression

- name: Check Performance
  run: cargo test --test regression performance_regression
```

## Troubleshooting

### Tests Failing

1. **Database connection errors**: Ensure test database is running
   ```bash
   docker-compose up -d postgres-test
   ```

2. **Timeout errors**: Increase test timeout
   ```bash
   cargo test -- --test-threads=1 --nocapture
   ```

3. **Permission errors**: Check file permissions on test fixtures
   ```bash
   chmod +r tests/fixtures/*.json
   ```

### Writing New Tests

1. Add test to appropriate file in `integration/` or `regression/`
2. Use `test_helpers` for common patterns
3. Follow naming convention: `test_<category>_<description>`
4. Add regression test when fixing bugs
5. Update this README if adding new test categories

## Contributing

When adding new tests:
1. Place them in the appropriate category
2. Use descriptive names
3. Add comments explaining the test purpose
4. Update fixtures if needed
5. Ensure tests are deterministic
6. Run `cargo test` before committing

## License

Same as parent project (MIT)
