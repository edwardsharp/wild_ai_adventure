#!/bin/bash

# Rust Integration Testing with Code Coverage
# This script runs integration tests and generates coverage reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Running Rust Integration Tests with Coverage${NC}"

# Install cargo-llvm-cov if not present
if ! cargo llvm-cov --version > /dev/null 2>&1; then
    echo -e "${YELLOW}Installing cargo-llvm-cov...${NC}"
    cargo install cargo-llvm-cov
fi

# Install grcov if not present (alternative coverage tool)
if ! grcov --version > /dev/null 2>&1; then
    echo -e "${YELLOW}Installing grcov...${NC}"
    cargo install grcov
fi

# Create coverage output directory
mkdir -p target/coverage

# Clean previous coverage data
cargo llvm-cov clean --workspace

echo -e "${BLUE}Starting test database containers...${NC}"

# Start test containers (if using docker-compose for test deps)
if [ -f "docker-compose.test.yml" ]; then
    docker-compose -f docker-compose.test.yml up -d
    sleep 5
fi

echo -e "${BLUE}Running integration tests...${NC}"

# Run tests with coverage using llvm-cov
cargo llvm-cov \
    --workspace \
    --all-features \
    --lcov \
    --output-path target/coverage/lcov.info \
    --ignore-filename-regex="target/.*" \
    --ignore-filename-regex="tests/.*" \
    -- --test-threads=1 --nocapture

# Generate HTML report
cargo llvm-cov report \
    --html \
    --output-dir target/coverage/html \
    --ignore-filename-regex="target/.*" \
    --ignore-filename-regex="tests/.*"

# Generate JSON report for programmatic access
cargo llvm-cov report \
    --json \
    --output-path target/coverage/coverage.json \
    --ignore-filename-regex="target/.*" \
    --ignore-filename-regex="tests/.*"

echo -e "${GREEN}✅ Tests completed!${NC}"

# Display coverage summary
echo -e "${BLUE}Coverage Summary:${NC}"
cargo llvm-cov report \
    --ignore-filename-regex="target/.*" \
    --ignore-filename-regex="tests/.*"

# Check if TypeScript client exists and run its tests
if [ -d "generated/ts-client" ]; then
    echo -e "${BLUE}🔧 Running TypeScript integration tests...${NC}"
    cd generated/ts-client

    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}Installing TypeScript client dependencies...${NC}"
        npm install
    fi

    # Set the API base URL to match the test server
    export API_BASE_URL="http://localhost:3000"

    # Build the TypeScript client
    npm run build

    # Run TypeScript integration tests
    npm run test:integration

    cd ../..
fi

# Clean up test containers
if [ -f "docker-compose.test.yml" ]; then
    echo -e "${BLUE}Cleaning up test containers...${NC}"
    docker-compose -f docker-compose.test.yml down
fi

echo -e "${GREEN}🎉 All tests completed successfully!${NC}"
echo -e "${BLUE}Coverage reports available at:${NC}"
echo -e "  📊 HTML: ${YELLOW}target/coverage/html/index.html${NC}"
echo -e "  📄 LCOV: ${YELLOW}target/coverage/lcov.info${NC}"
echo -e "  📋 JSON: ${YELLOW}target/coverage/coverage.json${NC}"

# Optional: Open HTML report in browser (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    read -p "Open HTML coverage report in browser? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open target/coverage/html/index.html
    fi
fi

# Check coverage threshold (adjust as needed)
COVERAGE_THRESHOLD=70
COVERAGE_PERCENT=$(cargo llvm-cov report --ignore-filename-regex="target/.*" --ignore-filename-regex="tests/.*" | grep -E "TOTAL.*%" | grep -oE "[0-9]+\.[0-9]+%" | grep -oE "[0-9]+\.[0-9]+" || echo "0")

if (( $(echo "$COVERAGE_PERCENT >= $COVERAGE_THRESHOLD" | bc -l) )); then
    echo -e "${GREEN}✅ Coverage ($COVERAGE_PERCENT%) meets threshold ($COVERAGE_THRESHOLD%)${NC}"
    exit 0
else
    echo -e "${RED}❌ Coverage ($COVERAGE_PERCENT%) below threshold ($COVERAGE_THRESHOLD%)${NC}"
    exit 1
fi
