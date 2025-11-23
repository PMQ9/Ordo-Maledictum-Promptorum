# ==============================================================================
# Intent Segregation Cybersecurity Architecture - Dockerfile
# ==============================================================================
# Multi-stage build for efficient container images
# ==============================================================================

# Stage 1: Builder
FROM rust:1.75-slim-bullseye AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY core/ ./core/
COPY api/ ./api/

# Build dependencies (cached layer)
RUN cargo build --release --bin api

# Stage 2: Runtime
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash appuser

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api /app/api

# Copy configuration files
COPY config/ /app/config/

# Create logs directory
RUN mkdir -p /app/logs && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["/app/api"]

# ==============================================================================
# Build Instructions:
# ==============================================================================
# Build:
#   docker build -t intent-segregation-api:latest .
#
# Run:
#   docker run -p 3000:3000 \
#     -e DATABASE_URL=postgresql://user:pass@host:5432/db \
#     intent-segregation-api:latest
#
# Or use docker-compose:
#   docker-compose up -d
# ==============================================================================
