# Testing Guide

This guide covers the comprehensive testing strategy for the WebAuthn tutorial project, including integration tests, end-to-end tests, TypeScript client testing, and code coverage.

## Table of Contents

- [Overview](#overview)
- [Test Types](#test-types)
- [Quick Start](#quick-start)
- [Rust Integration Tests](#rust-integration-tests)
- [TypeScript Client Testing](#typescript-client-testing)
- [Code Coverage](#code-coverage)
- [Performance Testing](#performance-testing)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

## Overview

Our testing approach focuses on integration and end-to-end testing rather than unit tests, as we're more interested in testing the input/output behavior of the complete system than internal implementation details.

### Test Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   TypeScript    │    │      Rust       │    │   PostgreSQL    │
│     Client      │◄──►│   Integration   │◄──►│   Test DB       │
│     Tests       │    │     Tests       │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Coverage      │
                    │   Reporting     │
                    └─────────────────┘
```

## Test Types

### 1. Integration Tests (Rust)
- **Purpose**: Test API endpoints with real database interactions
- **Scope**: HTTP request/response, database operations, business logic
- **Tools**: `testcontainers`, `reqwest`, `sqlx`

### 2. End-to-End Tests (TypeScript)
- **Purpose**: Test the API from a client perspective
- **Scope**: Full request/response cycle, error handling, data validation
- **Tools**: `zod`, `jest`, generated TypeScript client

### 3. Performance Tests
- **Purpose**: Ensure the API can handle concurrent requests
- **Scope**: Load testing, stress testing, latency measurement
- **Tools**: `wrk`, custom Rust benchmarks

### 4. Database Tests
- **Purpose**: Test data persistence and migrations
- **Scope**: Schema validation, data integrity, migration compatibility
- **Tools**: `testcontainers`, `sqlx-migrate`

## Quick Start

### Prerequisites

```bash
# Install required tools
cargo install cargo-llvm-cov sqlx-cli --no-default-features --features postgres
npm install -g jest typescript

# Install Docker for test containers
# macOS: brew install docker
# Ubuntu: apt-get install docker.io
```

### Run All Tests

```bash
# Complete test suite with coverage
./scripts/build_and_test.sh full

# Or step by step:
./scripts/build_and_test.sh setup    # Initial setup
./scripts/build_and_test.sh build    # Build project
./scripts/build_and_test.sh generate # Generate TypeScript client
./scripts/build_and_test.sh test     # Run all tests
./scripts/build_and_test.sh coverage # Generate coverage report
```

### Quick Test Commands

```bash
# Rust integration tests only
cargo test --test integration_tests -- --test-threads=1

# TypeScript client tests only (requires running server)
cd generated/ts-client && npm test

# Coverage report
cargo llvm-cov --html --output-dir target/coverage/html
```

## Rust Integration Tests

### Test Structure

Integration tests are located in `server/tests/` and use the following structure:

```rust
// server/tests/integration_tests.rs
mod common;

use common::{TestApp, assert_json_response};

#[tokio::test]
async fn test_endpoint() {
    let app = TestApp::spawn().await;
    let client = app.client();

    let response = client
        .post(&format!("{}/api/endpoint", app.address))
        .json(&request_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

### Test Database

Each test gets a fresh PostgreSQL container:

- **Isolation**: Tests run in parallel with separate databases
- **Speed**: Containers start quickly (~2-3 seconds)
- **Real Environment**: Uses actual PostgreSQL, not mocks

### Running Integration Tests

```bash
# Run all integration tests
cargo test --test integration_tests

# Run specific test
cargo test --test integration_tests -- test_register_user

# Run with debug output
RUST_LOG=debug cargo test --test integration_tests -- --nocapture

# Run with test containers cleanup
docker system prune -f # Clean up after tests
```

### Test Scenarios Covered

- ✅ User registration flow
- ✅ Authentication flow
- ✅ Error handling (4xx, 5xx responses)
- ✅ Concurrent request handling
- ✅ Database persistence
- ✅ Input validation
- ✅ Session management

## TypeScript Client Testing

### Generated Client

The TypeScript client is automatically generated from the API specification:

```typescript
// Usage example
import { ApiClient } from './generated/ts-client';

const client = new ApiClient('http://localhost:3000');

// Type-safe API calls with validation
const response = await client.registerStart({
  username: 'testuser',
  display_name: 'Test User'
});
```

### Test Structure

```typescript
// generated/ts-client/tests/integration.test.ts
describe('WebAuthn API Integration Tests', () => {
  let client: TestApiClient;

  beforeAll(() => {
    client = new TestApiClient(process.env.API_BASE_URL);
  });

  it('should handle registration flow', async () => {
    const user = client.generateTestUser();
    const challenge = await client.registerStart(user);

    expect(challenge.challenge).toBeDefined();
    expect(challenge.user.name).toBe(user.username);
  });
});
```

### Running TypeScript Tests

```bash
cd generated/ts-client

# Install dependencies
npm install

# Run all tests
npm test

# Run integration tests only
npm run test:integration

# Run with coverage
npm run test:coverage
```

### TypeScript Test Features

- **Schema Validation**: Uses Zod to validate API responses
- **Error Handling**: Tests both success and error scenarios
- **Concurrent Testing**: Verifies thread safety
- **Performance Testing**: Basic load testing capabilities

## Code Coverage

### Coverage Tools

We use `cargo-llvm-cov` for Rust code coverage:

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html --output-dir target/coverage/html
```

### Coverage Reports

Coverage reports are generated in multiple formats:

- **HTML**: `target/coverage/html/index.html` - Interactive browser report
- **LCOV**: `target/coverage/lcov.info` - For CI/CD integration
- **JSON**: `target/coverage/coverage.json` - Programmatic access

### Coverage from TypeScript Tests

The TypeScript integration tests contribute to Rust code coverage:

```bash
# Start Rust server with coverage instrumentation
LLVM_PROFILE_FILE="coverage-%p-%m.profraw" cargo run

# Run TypeScript tests against instrumented server
cd generated/ts-client && npm run test:integration

# Generate coverage report including TypeScript-driven coverage
cargo llvm-cov report --html
```

### Coverage Goals

- **Minimum Threshold**: 70% line coverage
- **Focus Areas**: API endpoints, database operations, error handling
- **Exclusions**: Test code, generated code, build scripts

## Performance Testing

### Load Testing with wrk

```bash
# Basic load test
wrk -t4 -c100 -d30s http://localhost:3000/health

# Registration endpoint test
wrk -t2 -c10 -d10s -s scripts/perf/register.lua http://localhost:3000/
```

### Custom Performance Tests

```rust
#[tokio::test]
async fn concurrent_requests_performance() {
    let app = TestApp::spawn().await;
    let start = Instant::now();

    let tasks: Vec<_> = (0..100)
        .map(|_| test_register_flow(&app))
        .collect();

    for task in tasks {
        task.await.unwrap();
    }

    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(10));
}
```

### Performance Metrics

- **Throughput**: Requests per second
- **Latency**: Response time percentiles (p50, p95, p99)
- **Concurrency**: Maximum concurrent connections
- **Memory Usage**: Peak memory during load

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install coverage tools
        run: cargo install cargo-llvm-cov

      - name: Run tests with coverage
        run: ./scripts/build_and_test.sh coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: target/coverage/lcov.info
```

### Docker-based CI

```bash
# Run tests in containers
docker-compose -f docker-compose.test.yml up --build test-runner

# Run end-to-end tests
docker-compose -f docker-compose.test.yml up --build typescript-test
```

## Troubleshooting

### Common Issues

#### Database Connection Failures

```bash
# Check if PostgreSQL container is running
docker ps | grep postgres

# Check container logs
docker logs webauthn-test-db

# Manually test connection
psql postgres://postgres:postgres@localhost:5433/test_db -c "SELECT 1;"
```

#### Port Conflicts

```bash
# Kill processes using test ports
lsof -ti:3000 | xargs kill -9
lsof -ti:5433 | xargs kill -9

# Use different ports
API_PORT=3001 TEST_DB_PORT=5434 ./scripts/build_and_test.sh test
```

#### TypeScript Client Issues

```bash
# Regenerate client
./scripts/build_and_test.sh generate

# Clean and rebuild
cd generated/ts-client
rm -rf node_modules dist
npm install
npm run build
```

#### Coverage Issues

```bash
# Clean coverage data
cargo llvm-cov clean

# Rebuild with coverage
cargo build --workspace

# Generate fresh report
cargo llvm-cov --html
```

### Debug Mode

Enable debug logging for more detailed output:

```bash
RUST_LOG=debug ./scripts/build_and_test.sh test
```

### Container Debugging

```bash
# Enter test container
docker-compose -f docker-compose.test.yml run test-runner bash

# Check container health
docker-compose -f docker-compose.test.yml ps
docker-compose -f docker-compose.test.yml logs test-postgres
```

## Best Practices

### Test Design

1. **Independent Tests**: Each test should be able to run in isolation
2. **Real Dependencies**: Use actual databases and HTTP clients
3. **Fast Feedback**: Tests should complete quickly (< 30 seconds total)
4. **Deterministic**: Tests should produce consistent results

### Data Management

1. **Fresh Data**: Each test gets a clean database
2. **Realistic Data**: Use data that matches production scenarios
3. **Edge Cases**: Test boundary conditions and error cases

### Maintenance

1. **Update Tests**: Keep tests in sync with API changes
2. **Monitor Coverage**: Ensure coverage doesn't decrease
3. **Performance Baseline**: Track performance regressions
4. **Documentation**: Keep this guide updated

## Useful Commands

```bash
# Quick test run
cargo test --test integration_tests -- --test-threads=1

# Full test suite with coverage
./scripts/build_and_test.sh full

# Generate TypeScript client only
./scripts/build_and_test.sh generate

# Clean everything
./scripts/build_and_test.sh clean

# Start development environment
./scripts/build_and_test.sh dev

# Check health of services
./scripts/health-check.sh check

# Wait for services to be ready
./scripts/health-check.sh wait
```

For more information, see the individual test files and scripts in the `scripts/` directory.
