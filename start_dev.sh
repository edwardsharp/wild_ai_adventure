#!/bin/bash

# Development startup script for WebAuthn Axum server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}WebAuthn Axum Development Setup${NC}"
echo "=================================="

# Check if .env file exists and source it
if [ -f ".env" ]; then
    echo -e "${GREEN}Found .env file, sourcing it...${NC}"
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo -e "${YELLOW}DATABASE_URL not set. Please set it or create a .env file.${NC}"
    echo "Example: export DATABASE_URL='postgresql://username:password@localhost:5432/webauthn_db'"
    echo ""

    if [ ! -f ".env" ]; then
        echo -e "${YELLOW}Creating .env file from example...${NC}"
        cp .env.example .env
        echo -e "${RED}Please edit .env file with your database credentials and run this script again.${NC}"
        exit 1
    else
        echo -e "${RED}DATABASE_URL not found in .env file. Please check your .env configuration.${NC}"
        exit 1
    fi
fi

# Verify DATABASE_URL is now set
if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}DATABASE_URL still not set. Please check your .env file.${NC}"
    exit 1
fi

echo -e "${GREEN}Using DATABASE_URL: ${DATABASE_URL}${NC}"
echo ""

# Check if PostgreSQL is running
echo -e "${YELLOW}Testing database connection...${NC}"
if ! cargo run --bin webauthn-admin -- stats > /dev/null 2>&1; then
    echo -e "${RED}Cannot connect to database. Please ensure:${NC}"
    echo "1. PostgreSQL is running"
    echo "2. Database exists"
    echo "3. User has proper permissions"
    echo "4. DATABASE_URL is correct"
    echo ""
    echo "Current DATABASE_URL: $DATABASE_URL"
    exit 1
fi

echo -e "${GREEN}Database connection successful!${NC}"
echo ""

# Generate some invite codes if none exist
echo -e "${YELLOW}Checking for existing invite codes...${NC}"
invite_count=$(cargo run --bin webauthn-admin -- stats 2>/dev/null | grep "Active codes:" | awk '{print $3}' || echo "0")

if [ "$invite_count" = "0" ]; then
    echo -e "${YELLOW}No active invite codes found. Generating 3 invite codes...${NC}"
    cargo run --bin webauthn-admin -- generate-invite --count 3
    echo ""
    echo -e "${GREEN}Generated invite codes. Use 'cargo run --bin webauthn-admin -- list-invites' to see them.${NC}"
    echo ""
else
    echo -e "${GREEN}Found $invite_count active invite codes.${NC}"
    echo ""
fi

# Show available invite codes
echo -e "${YELLOW}Available invite codes:${NC}"
cargo run --bin webauthn-admin -- list-invites --active-only
echo ""

# Set RUST_LOG if not set
if [ -z "$RUST_LOG" ]; then
    export RUST_LOG=info
fi

echo -e "${GREEN}Starting WebAuthn server...${NC}"
echo -e "${YELLOW}Server will be available at: http://localhost:8080${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
echo ""

# Start the server
cargo run
