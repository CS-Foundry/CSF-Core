# Multi-stage build for FinanceVault Backend (with embedded frontend proxy)

###################
# Backend Build Stage
###################
FROM rust:latest AS backend-builder

WORKDIR /usr/src/app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace Cargo files
COPY Cargo.toml ./
COPY migration/Cargo.toml ./migration/
COPY entity/Cargo.toml ./entity/

# Create dummy source files for dependency caching
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p migration/src && \
    echo "pub use sea_orm_migration::prelude::*; pub struct Migrator; impl MigratorTrait for Migrator { fn migrations() -> Vec<Box<dyn MigrationTrait>> { vec![] } }" > migration/src/lib.rs && \
    mkdir -p entity/src && \
    echo "pub mod entities; pub mod prelude; pub use entities::*;" > entity/src/lib.rs && \
    mkdir -p entity/src/entities && \
    echo "pub mod prelude { pub use super::*; }" > entity/src/entities/mod.rs

# Build dependencies (this will be cached)
RUN cargo build --release || true

# Copy all source code
COPY migration/src ./migration/src
COPY entity/src ./entity/src
COPY src ./src

# Build the actual application
RUN cargo build --release

###################
# Final Runtime Stage
###################
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create data directory for SQLite
RUN mkdir -p /data

# Copy backend binary from builder
COPY --from=backend-builder /usr/src/app/target/release/backend /app/backend

# Expose backend port (frontend proxy is handled by backend)
EXPOSE 8000

# Set environment variables
ENV DATABASE_URL=sqlite:/data/finance.db
ENV RUST_LOG=info
ENV FRONTEND_URL=http://localhost:3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD ["/app/backend", "--health-check"] || exit 1

# Start backend (it will proxy to frontend)
CMD ["./backend"]