#!/bin/bash

# ==============================================================================
# Intent Segregation Cybersecurity Architecture - Setup Script
# ==============================================================================
# This script installs all dependencies required to run the system locally
# Supports: Ubuntu/Debian, macOS, and Fedora/RHEL
# ==============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/debian_version ]; then
            OS="debian"
            log_info "Detected Debian/Ubuntu Linux"
        elif [ -f /etc/redhat-release ]; then
            OS="redhat"
            log_info "Detected RedHat/Fedora/CentOS Linux"
        else
            OS="linux"
            log_info "Detected Linux (generic)"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        log_info "Detected macOS"
    else
        OS="unknown"
        log_warning "Unknown OS: $OSTYPE"
    fi
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust
install_rust() {
    log_info "Checking Rust installation..."

    if command_exists rustc; then
        RUST_VERSION=$(rustc --version)
        log_success "Rust already installed: $RUST_VERSION"
    else
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        log_success "Rust installed successfully"
    fi

    # Update rustup
    log_info "Updating Rust toolchain..."
    rustup update stable
    rustup default stable

    log_success "Rust setup complete"
}

# Install PostgreSQL
install_postgresql() {
    log_info "Checking PostgreSQL installation..."

    if command_exists psql; then
        PG_VERSION=$(psql --version)
        log_success "PostgreSQL already installed: $PG_VERSION"
        return
    fi

    log_info "Installing PostgreSQL..."

    case $OS in
        debian)
            sudo apt-get update
            sudo apt-get install -y postgresql postgresql-contrib libpq-dev
            sudo systemctl start postgresql
            sudo systemctl enable postgresql
            ;;
        redhat)
            sudo dnf install -y postgresql-server postgresql-contrib postgresql-devel
            sudo postgresql-setup --initdb
            sudo systemctl start postgresql
            sudo systemctl enable postgresql
            ;;
        macos)
            if command_exists brew; then
                brew install postgresql@15
                brew services start postgresql@15
            else
                log_error "Homebrew not found. Please install Homebrew first: https://brew.sh"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported OS for automatic PostgreSQL installation"
            log_info "Please install PostgreSQL manually and run this script again"
            exit 1
            ;;
    esac

    log_success "PostgreSQL installed successfully"
}

# Setup PostgreSQL database
setup_database() {
    log_info "Setting up database..."

    DB_NAME="intent_segregation"
    DB_USER="intent_user"
    DB_PASS="intent_pass"

    # Check if database exists
    if sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
        log_warning "Database '$DB_NAME' already exists"
    else
        log_info "Creating database '$DB_NAME'..."
        sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;"
        log_success "Database created"
    fi

    # Check if user exists
    if sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1; then
        log_warning "User '$DB_USER' already exists"
    else
        log_info "Creating database user '$DB_USER'..."
        sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
        log_success "User created"
    fi

    # Grant privileges
    log_info "Granting privileges..."
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"
    sudo -u postgres psql -c "ALTER DATABASE $DB_NAME OWNER TO $DB_USER;"

    log_success "Database setup complete"
    log_info "Connection string: postgresql://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
}

# Install Ollama
install_ollama() {
    log_info "Checking Ollama installation..."

    if command_exists ollama; then
        log_success "Ollama already installed"
    else
        log_info "Installing Ollama..."

        case $OS in
            linux|debian|redhat)
                curl -fsSL https://ollama.ai/install.sh | sh
                ;;
            macos)
                if command_exists brew; then
                    brew install ollama
                else
                    log_warning "Please download Ollama from https://ollama.ai/download"
                    return
                fi
                ;;
            *)
                log_warning "Please install Ollama manually from https://ollama.ai/download"
                return
                ;;
        esac

        log_success "Ollama installed successfully"
    fi

    # Pull default model
    log_info "Pulling Llama 2 model (this may take a while)..."
    if command_exists ollama; then
        ollama pull llama2 || log_warning "Failed to pull model. You can do this later with: ollama pull llama2"
        log_success "Ollama setup complete"
    fi
}

# Install Redis
install_redis() {
    log_info "Checking Redis installation..."

    if command_exists redis-server; then
        log_success "Redis already installed"
        return
    fi

    log_info "Installing Redis..."

    case $OS in
        debian)
            sudo apt-get update
            sudo apt-get install -y redis-server
            sudo systemctl start redis-server
            sudo systemctl enable redis-server
            ;;
        redhat)
            sudo dnf install -y redis
            sudo systemctl start redis
            sudo systemctl enable redis
            ;;
        macos)
            if command_exists brew; then
                brew install redis
                brew services start redis
            else
                log_error "Homebrew not found"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported OS for automatic Redis installation"
            exit 1
            ;;
    esac

    log_success "Redis installed successfully"
}

# Install system dependencies
install_system_deps() {
    log_info "Installing system dependencies..."

    case $OS in
        debian)
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                pkg-config \
                libssl-dev \
                curl \
                git \
                jq
            ;;
        redhat)
            sudo dnf groupinstall -y "Development Tools"
            sudo dnf install -y \
                pkg-config \
                openssl-devel \
                curl \
                git \
                jq
            ;;
        macos)
            if command_exists brew; then
                brew install pkg-config openssl curl git jq
            else
                log_error "Homebrew not found. Please install from https://brew.sh"
                exit 1
            fi
            ;;
    esac

    log_success "System dependencies installed"
}

# Setup environment file
setup_env_file() {
    log_info "Setting up environment file..."

    if [ -f .env ]; then
        log_warning ".env file already exists, skipping creation"
    else
        log_info "Creating .env from .env.example..."
        cp .env.example .env
        log_success ".env file created"
        log_warning "Please edit .env and add your API keys and secrets"
    fi
}

# Build the project
build_project() {
    log_info "Building the project..."

    cargo build --release

    log_success "Project built successfully"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."

    # Check if sqlx-cli is installed
    if ! command_exists sqlx; then
        log_info "Installing sqlx-cli..."
        cargo install sqlx-cli --no-default-features --features postgres
    fi

    # Run migrations if they exist
    if [ -d "migrations" ]; then
        sqlx migrate run
        log_success "Migrations completed"
    else
        log_warning "No migrations directory found, skipping"
    fi
}

# Main installation function
main() {
    echo "=================================================="
    echo "Intent Segregation Architecture - Setup"
    echo "=================================================="
    echo ""

    detect_os
    echo ""

    # Confirm installation
    read -p "This will install Rust, PostgreSQL, Ollama, Redis and dependencies. Continue? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Installation cancelled"
        exit 0
    fi

    echo ""
    log_info "Starting installation..."
    echo ""

    # Install components
    install_system_deps
    echo ""

    install_rust
    echo ""

    install_postgresql
    echo ""

    setup_database
    echo ""

    install_redis
    echo ""

    install_ollama
    echo ""

    setup_env_file
    echo ""

    build_project
    echo ""

    # Optional: Run migrations
    read -p "Run database migrations? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        run_migrations
        echo ""
    fi

    echo "=================================================="
    log_success "Installation complete!"
    echo "=================================================="
    echo ""
    log_info "Next steps:"
    echo "  1. Edit .env file with your API keys and configuration"
    echo "  2. Start services with: ./run_local.sh"
    echo "  3. Run tests with: cargo test"
    echo "  4. Run red-team tests with: cargo test --test redteam"
    echo ""
    log_info "For Docker deployment, use: docker-compose up -d"
    echo ""
}

# Run main function
main
