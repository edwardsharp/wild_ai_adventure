#!/bin/bash

# Health check script for containers
# This script checks if the application is healthy and ready to serve requests

set -e

# Configuration
API_HOST=${API_HOST:-"localhost"}
API_PORT=${API_PORT:-"3000"}
DATABASE_URL=${DATABASE_URL:-"postgres://postgres:postgres@localhost:5432/test_db"}
TIMEOUT=${TIMEOUT:-"10"}
MAX_RETRIES=${MAX_RETRIES:-"30"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[HEALTH] $1${NC}"
}

error() {
    echo -e "${RED}[HEALTH ERROR] $1${NC}" >&2
}

success() {
    echo -e "${GREEN}[HEALTH OK] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[HEALTH WARN] $1${NC}"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check database connectivity
check_database() {
    log "Checking database connectivity..."

    if command_exists psql; then
        if psql "$DATABASE_URL" -c "SELECT 1;" >/dev/null 2>&1; then
            success "Database is accessible"
            return 0
        else
            error "Database is not accessible"
            return 1
        fi
    elif command_exists pg_isready; then
        # Extract connection details from DATABASE_URL
        local db_host=$(echo "$DATABASE_URL" | sed -n 's/.*@\([^:]*\):.*/\1/p')
        local db_port=$(echo "$DATABASE_URL" | sed -n 's/.*:\([0-9]*\)\/.*/\1/p')

        if pg_isready -h "$db_host" -p "$db_port" >/dev/null 2>&1; then
            success "Database is ready"
            return 0
        else
            error "Database is not ready"
            return 1
        fi
    else
        warn "No PostgreSQL client available, skipping database check"
        return 0
    fi
}

# Check HTTP endpoint
check_http_endpoint() {
    local endpoint="$1"
    local expected_status="${2:-200}"

    log "Checking HTTP endpoint: $endpoint"

    if command_exists curl; then
        local response_code
        response_code=$(curl -s -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "$endpoint" 2>/dev/null || echo "000")

        if [ "$response_code" = "$expected_status" ]; then
            success "HTTP endpoint $endpoint returned $response_code"
            return 0
        else
            error "HTTP endpoint $endpoint returned $response_code, expected $expected_status"
            return 1
        fi
    elif command_exists wget; then
        if wget --timeout="$TIMEOUT" --tries=1 -q --spider "$endpoint" 2>/dev/null; then
            success "HTTP endpoint $endpoint is accessible"
            return 0
        else
            error "HTTP endpoint $endpoint is not accessible"
            return 1
        fi
    else
        error "No HTTP client (curl/wget) available"
        return 1
    fi
}

# Check if port is listening
check_port() {
    local host="$1"
    local port="$2"

    log "Checking if port $port is listening on $host..."

    if command_exists nc; then
        if nc -z "$host" "$port" 2>/dev/null; then
            success "Port $port is listening on $host"
            return 0
        else
            error "Port $port is not listening on $host"
            return 1
        fi
    elif command_exists telnet; then
        if timeout "$TIMEOUT" telnet "$host" "$port" </dev/null >/dev/null 2>&1; then
            success "Port $port is listening on $host"
            return 0
        else
            error "Port $port is not listening on $host"
            return 1
        fi
    else
        warn "No network client (nc/telnet) available, skipping port check"
        return 0
    fi
}

# Check application health endpoint
check_app_health() {
    local base_url="http://${API_HOST}:${API_PORT}"

    log "Checking application health..."

    # Check if the main port is listening
    if ! check_port "$API_HOST" "$API_PORT"; then
        return 1
    fi

    # Check health endpoint
    if ! check_http_endpoint "${base_url}/health"; then
        return 1
    fi

    # Additional endpoint checks
    local endpoints=(
        "/health:200"
    )

    for endpoint_check in "${endpoints[@]}"; do
        local endpoint=$(echo "$endpoint_check" | cut -d: -f1)
        local expected_status=$(echo "$endpoint_check" | cut -d: -f2)

        if ! check_http_endpoint "${base_url}${endpoint}" "$expected_status"; then
            return 1
        fi
    done

    success "Application health checks passed"
    return 0
}

# Check system resources
check_system_resources() {
    log "Checking system resources..."

    # Check memory usage
    if command_exists free; then
        local mem_usage
        mem_usage=$(free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}')
        log "Memory usage: ${mem_usage}%"

        if (( $(echo "$mem_usage > 90.0" | bc -l 2>/dev/null || echo 0) )); then
            warn "High memory usage: ${mem_usage}%"
        fi
    fi

    # Check disk usage
    if command_exists df; then
        local disk_usage
        disk_usage=$(df / | tail -1 | awk '{print $5}' | sed 's/%//')
        log "Disk usage: ${disk_usage}%"

        if [ "$disk_usage" -gt 80 ]; then
            warn "High disk usage: ${disk_usage}%"
        fi
    fi

    # Check load average
    if [ -f /proc/loadavg ]; then
        local load_avg
        load_avg=$(cat /proc/loadavg | cut -d' ' -f1)
        log "Load average: $load_avg"
    fi

    return 0
}

# Wait for service to be ready
wait_for_service() {
    local retries=0

    log "Waiting for service to be ready (max $MAX_RETRIES attempts)..."

    while [ $retries -lt $MAX_RETRIES ]; do
        if check_app_health >/dev/null 2>&1; then
            success "Service is ready after $retries attempts"
            return 0
        fi

        retries=$((retries + 1))
        log "Attempt $retries/$MAX_RETRIES failed, retrying in 2 seconds..."
        sleep 2
    done

    error "Service failed to become ready after $MAX_RETRIES attempts"
    return 1
}

# Main health check function
main_health_check() {
    log "Starting comprehensive health check..."

    local exit_code=0

    # Check database connectivity
    if ! check_database; then
        exit_code=1
    fi

    # Check application health
    if ! check_app_health; then
        exit_code=1
    fi

    # Check system resources (non-fatal)
    check_system_resources

    if [ $exit_code -eq 0 ]; then
        success "All health checks passed"
    else
        error "Some health checks failed"
    fi

    return $exit_code
}

# Show usage information
show_usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  check     - Run comprehensive health check (default)"
    echo "  wait      - Wait for service to be ready"
    echo "  db        - Check database connectivity only"
    echo "  app       - Check application health only"
    echo "  port      - Check if port is listening"
    echo "  help      - Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  API_HOST       - API hostname (default: localhost)"
    echo "  API_PORT       - API port (default: 3000)"
    echo "  DATABASE_URL   - Database connection URL"
    echo "  TIMEOUT        - Request timeout in seconds (default: 10)"
    echo "  MAX_RETRIES    - Maximum retry attempts (default: 30)"
}

# Main execution
case "${1:-check}" in
    "check")
        main_health_check
        ;;
    "wait")
        wait_for_service
        ;;
    "db")
        check_database
        ;;
    "app")
        check_app_health
        ;;
    "port")
        check_port "$API_HOST" "$API_PORT"
        ;;
    "help"|"-h"|"--help")
        show_usage
        exit 0
        ;;
    *)
        error "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac

exit $?
