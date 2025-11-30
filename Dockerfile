# Multi-stage build for FinanceVault - Single binary with frontend proxy

###################
# Frontend Build Stage
###################
FROM node:20-alpine AS frontend-builder

WORKDIR /frontend

# Copy frontend package files
COPY frontend/package*.json ./

# Install ALL dependencies first (dev + production)
RUN npm ci

# Copy frontend source
COPY frontend/ ./

# Set build-time environment variables for SvelteKit
ENV PUBLIC_API_BASE_URL=http://localhost:8000/api

# Build frontend for production
RUN npm run build

# Remove devDependencies, keep only production dependencies
RUN npm ci --omit=dev

###################
# Backend Build Stage
###################
FROM rustlang/rust:nightly AS backend-builder

WORKDIR /backend

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy all backend source (simplified - no dependency caching for now)
COPY backend/ ./

# Build the backend
RUN cargo build --release --verbose

###################
# Final Runtime Stage
###################
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies including Node.js for frontend
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    curl \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Create directories
RUN mkdir -p /data /app/frontend

# Copy backend binary from builder
COPY --from=backend-builder /backend/target/release/backend /app/backend

# Copy frontend build and node_modules from builder
COPY --from=frontend-builder /frontend/build /app/frontend/build
COPY --from=frontend-builder /frontend/node_modules /app/frontend/node_modules
COPY --from=frontend-builder /frontend/package.json /app/frontend/package.json

# Copy startup script
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Expose ports (backend proxies to frontend internally)
EXPOSE 8000 3000

# Set environment variables
ENV DATABASE_URL=sqlite:/data/finance.db
ENV RUST_LOG=info
ENV NODE_ENV=production
ENV PORT=3000
ENV FRONTEND_URL=http://localhost:3000
ENV ORIGIN=http://localhost:8000

# Health check on backend (which includes frontend proxy)
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

# Use entrypoint script to start both services
# Backend will proxy all non-API requests to frontend
ENTRYPOINT ["/app/docker-entrypoint.sh"]
