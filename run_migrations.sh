#!/bin/bash

# WebAuthn Server Migration Runner
# This script helps run database migrations for the WebAuthn authentication server

set -e  # Exit on any error

# Configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-webauthn_db}"
DB_USER="${DB_USER:-webauthn_user}"
DB_PASSWORD="${DB_PASSWORD:-webauthn_password}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if psql is available
check_psql() {
    if ! command -v psql &> /dev/null; then
        log_error "psql command not found. Please install PostgreSQL client tools."
        exit 1
    fi
}

# Check if sqlx-cli is available
check_sqlx() {
    if ! command -v sqlx &> /dev/null; then
        log_warning "sqlx-cli not found. You can install it with: cargo install sqlx-cli --features postgres"
        return 1
    fi
    return 0
}

# Test database connection
test_connection() {
    log_info "Testing database connection..."
    if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c '\q' &> /dev/null; then
        log_success "Database connection successful"
        return 0
    else
        log_error "Failed to connect to database"
        log_info "Connection details:"
        log_info "  Host: $DB_HOST"
        log_info "  Port: $DB_PORT"
        log_info "  Database: $DB_NAME"
        log_info "  User: $DB_USER"
        return 1
    fi
}

# Run migrations using sqlx-cli
run_sqlx_migrations() {
    log_info "Running migrations with sqlx-cli..."

    export DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

    if sqlx migrate run --source migrations; then
        log_success "All migrations completed successfully with sqlx-cli"
        return 0
    else
        log_error "Migration failed with sqlx-cli"
        return 1
    fi
}

# Run migrations manually using psql
run_manual_migrations() {
    log_info "Running migrations manually with psql..."

    local migrations=(
        "001_initial.sql"
        "002_session_storage.sql"
        "003_analytics.sql"
    )

    for migration in "${migrations[@]}"; do
        local migration_file="migrations/$migration"

        if [[ ! -f "$migration_file" ]]; then
            log_warning "Migration file $migration_file not found, skipping..."
            continue
        fi

        log_info "Running migration: $migration"

        if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$migration_file" &> /dev/null; then
            log_success "Migration $migration completed"
        else
            log_error "Migration $migration failed"
            return 1
        fi
    done

    log_success "All manual migrations completed successfully"
}

# Check migration status
check_status() {
    log_info "Checking migration status..."

    # Check if base tables exist
    local tables_query="SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('invite_codes', 'users', 'webauthn_credentials', 'tower_sessions', 'request_analytics');"

    log_info "Existing tables:"
    PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "$tables_query" 2>/dev/null || {
        log_error "Failed to check table status"
        return 1
    }
}

# Reset database (DANGEROUS - for development only)
reset_database() {
    log_warning "⚠️  DANGER: This will DROP ALL TABLES and data!"
    read -p "Are you sure you want to reset the database? Type 'yes' to confirm: " confirm

    if [[ "$confirm" != "yes" ]]; then
        log_info "Database reset cancelled"
        return 0
    fi

    log_info "Dropping all tables..."

    local drop_query="
    DROP TABLE IF EXISTS request_analytics CASCADE;
    DROP TABLE IF EXISTS webauthn_credentials CASCADE;
    DROP TABLE IF EXISTS tower_sessions CASCADE;
    DROP TABLE IF EXISTS users CASCADE;
    DROP TABLE IF EXISTS invite_codes CASCADE;
    DROP FUNCTION IF EXISTS cleanup_expired_sessions();
    "

    if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "$drop_query" &> /dev/null; then
        log_success "Database reset completed"
    else
        log_error "Database reset failed"
        return 1
    fi
}

# Show usage information
show_usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  run         Run database migrations (default)"
    echo "  status      Check migration status"
    echo "  reset       Reset database (DANGEROUS - development only)"
    echo "  help        Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  DB_HOST     Database host (default: localhost)"
    echo "  DB_PORT     Database port (default: 5432)"
    echo "  DB_NAME     Database name (default: webauthn_db)"
    echo "  DB_USER     Database user (default: webauthn_user)"
    echo "  DB_PASSWORD Database password (default: webauthn_password)"
    echo ""
    echo "Examples:"
    echo "  $0                              # Run migrations"
    echo "  $0 status                       # Check status"
    echo "  DB_HOST=remote.db $0 run        # Run on remote database"
}

# Main function
main() {
    local command="${1:-run}"

    case "$command" in
        "run")
            check_psql
            if ! test_connection; then
                exit 1
            fi

            # Try sqlx-cli first, fall back to manual
            if check_sqlx && run_sqlx_migrations; then
                exit 0
            else
                log_info "Falling back to manual migration..."
                run_manual_migrations
            fi
            ;;
        "status")
            check_psql
            if test_connection; then
                check_status
            fi
            ;;
        "reset")
            check_psql
            if test_connection; then
                reset_database
            fi
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
