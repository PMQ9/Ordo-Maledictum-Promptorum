#!/bin/bash

# ==============================================================================
# Intent Segregation Cybersecurity Architecture - Local Service Runner
# ==============================================================================
# This script starts all required services locally for development
# ==============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# PID file locations
PIDS_DIR="./.pids"
POSTGRES_PID="$PIDS_DIR/postgres.pid"
REDIS_PID="$PIDS_DIR/redis.pid"
OLLAMA_PID="$PIDS_DIR/ollama.pid"
API_PID="$PIDS_DIR/api.pid"

# Log functions
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

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if a service is running
is_running() {
    if [ -f "$1" ]; then
        pid=$(cat "$1")
        if ps -p "$pid" > /dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

# Create PID directory
mkdir -p "$PIDS_DIR"

# Load environment variables
if [ -f .env ]; then
    log_info "Loading environment variables from .env"
    export $(cat .env | grep -v '^#' | xargs)
else
    log_warning ".env file not found. Using defaults."
    log_warning "Run './setup.sh' first or copy .env.example to .env"
fi

# Start PostgreSQL
start_postgres() {
    log_info "Checking PostgreSQL..."

    if ! command_exists psql; then
        log_error "PostgreSQL not installed. Run ./setup.sh first"
        exit 1
    fi

    # Check if already running
    if pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
        log_success "PostgreSQL already running"
    else
        log_info "Starting PostgreSQL..."
        case "$(uname -s)" in
            Linux*)
                sudo systemctl start postgresql
                ;;
            Darwin*)
                brew services start postgresql@15 || brew services start postgresql
                ;;
        esac
        sleep 2
        log_success "PostgreSQL started"
    fi
}

# Start Redis
start_redis() {
    log_info "Checking Redis..."

    if ! command_exists redis-server; then
        log_warning "Redis not installed. Skipping..."
        return
    fi

    # Check if already running
    if redis-cli ping >/dev/null 2>&1; then
        log_success "Redis already running"
    else
        log_info "Starting Redis..."
        case "$(uname -s)" in
            Linux*)
                sudo systemctl start redis-server || sudo systemctl start redis
                ;;
            Darwin*)
                brew services start redis
                ;;
        esac
        sleep 1
        log_success "Redis started"
    fi
}

# Start Ollama
start_ollama() {
    log_info "Checking Ollama..."

    if ! command_exists ollama; then
        log_warning "Ollama not installed. LLM parsers will not work."
        return
    fi

    # Check if already running
    if curl -s http://localhost:11434 >/dev/null 2>&1; then
        log_success "Ollama already running"
    else
        log_info "Starting Ollama server..."
        ollama serve > /dev/null 2>&1 &
        echo $! > "$OLLAMA_PID"
        sleep 3

        if curl -s http://localhost:11434 >/dev/null 2>&1; then
            log_success "Ollama started (PID: $(cat $OLLAMA_PID))"
        else
            log_error "Failed to start Ollama"
            rm -f "$OLLAMA_PID"
        fi
    fi
}

# Build the project
build_project() {
    log_info "Building project..."
    cargo build --release
    log_success "Build complete"
}

# Start the API server
start_api() {
    log_info "Starting API server..."

    if is_running "$API_PID"; then
        log_warning "API server already running (PID: $(cat $API_PID))"
        return
    fi

    # Run the API server
    cargo run --release --bin api > ./logs/api.log 2>&1 &
    echo $! > "$API_PID"

    sleep 2

    if is_running "$API_PID"; then
        log_success "API server started (PID: $(cat $API_PID))"
        log_info "API available at http://localhost:${SERVER_PORT:-3000}"
    else
        log_error "Failed to start API server. Check ./logs/api.log"
        rm -f "$API_PID"
    fi
}

# Stop all services
stop_services() {
    log_info "Stopping services..."

    # Stop API
    if is_running "$API_PID"; then
        kill $(cat "$API_PID") 2>/dev/null || true
        rm -f "$API_PID"
        log_success "API server stopped"
    fi

    # Stop Ollama
    if is_running "$OLLAMA_PID"; then
        kill $(cat "$OLLAMA_PID") 2>/dev/null || true
        rm -f "$OLLAMA_PID"
        log_success "Ollama stopped"
    fi

    log_success "All services stopped"
}

# Check service status
check_status() {
    echo "=================================================="
    echo "Service Status"
    echo "=================================================="

    # PostgreSQL
    if pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
        echo -e "PostgreSQL: ${GREEN}●${NC} Running"
    else
        echo -e "PostgreSQL: ${RED}●${NC} Stopped"
    fi

    # Redis
    if redis-cli ping >/dev/null 2>&1; then
        echo -e "Redis:      ${GREEN}●${NC} Running"
    else
        echo -e "Redis:      ${RED}●${NC} Stopped"
    fi

    # Ollama
    if curl -s http://localhost:11434 >/dev/null 2>&1; then
        echo -e "Ollama:     ${GREEN}●${NC} Running (http://localhost:11434)"
    else
        echo -e "Ollama:     ${RED}●${NC} Stopped"
    fi

    # API Server
    if is_running "$API_PID"; then
        port=${SERVER_PORT:-3000}
        echo -e "API Server: ${GREEN}●${NC} Running (PID: $(cat $API_PID), http://localhost:$port)"
    else
        echo -e "API Server: ${RED}●${NC} Stopped"
    fi

    echo "=================================================="
}

# Show logs
show_logs() {
    log_info "Showing API logs (Ctrl+C to exit)..."
    tail -f ./logs/api.log
}

# Run tests
run_tests() {
    log_info "Running unit tests..."
    cargo test --lib

    echo ""
    log_info "Running integration tests..."
    cargo test --test integration

    echo ""
    log_info "Running red-team security tests..."
    cargo test --test redteam -- --nocapture

    log_success "All tests complete!"
}

# Main function
main() {
    case "${1:-start}" in
        start)
            echo "=================================================="
            echo "Starting Intent Segregation Services"
            echo "=================================================="
            echo ""

            # Create logs directory
            mkdir -p ./logs

            start_postgres
            echo ""

            start_redis
            echo ""

            start_ollama
            echo ""

            build_project
            echo ""

            start_api
            echo ""

            check_status
            echo ""

            log_success "All services started!"
            echo ""
            log_info "Commands:"
            echo "  ./run_local.sh status   - Check service status"
            echo "  ./run_local.sh logs     - View API logs"
            echo "  ./run_local.sh stop     - Stop all services"
            echo "  ./run_local.sh test     - Run tests"
            echo "  ./run_local.sh restart  - Restart services"
            echo ""
            ;;

        stop)
            stop_services
            ;;

        restart)
            stop_services
            echo ""
            sleep 1
            main start
            ;;

        status)
            check_status
            ;;

        logs)
            show_logs
            ;;

        test)
            run_tests
            ;;

        *)
            log_error "Unknown command: $1"
            echo ""
            echo "Usage: $0 {start|stop|restart|status|logs|test}"
            echo ""
            echo "Commands:"
            echo "  start   - Start all services (default)"
            echo "  stop    - Stop all services"
            echo "  restart - Restart all services"
            echo "  status  - Check service status"
            echo "  logs    - View API logs"
            echo "  test    - Run all tests including red-team"
            exit 1
            ;;
    esac
}

# Trap Ctrl+C
trap 'echo ""; log_info "Use \"./run_local.sh stop\" to stop services"; exit 0' INT

# Run main function
main "$@"
