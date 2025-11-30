#!/bin/bash
set -e

echo "🚀 Starting FinanceVault..."

# Ensure environment variables are set
export RUST_LOG=${RUST_LOG:-info}
export DATABASE_URL=${DATABASE_URL:-sqlite:/data/finance.db}
export FRONTEND_URL=${FRONTEND_URL:-http://localhost:3000}

# Check if database directory exists
mkdir -p /data
echo "📁 Data directory: /data"
echo "💾 Database URL: $DATABASE_URL"
echo "🔧 Rust Log Level: $RUST_LOG"
echo "🌐 Frontend URL: $FRONTEND_URL"
echo ""

# Start frontend (SvelteKit with adapter-node) in background
echo "🎨 Starting Frontend on port 3000..."
cd /app/frontend

# Start frontend with logs going to stdout (with prefix)
(PORT=3000 HOST=0.0.0.0 ORIGIN=${ORIGIN:-http://localhost:8000} node build/index.js 2>&1 | sed 's/^/[FRONTEND] /') &
FRONTEND_PID=$!

echo "   Frontend PID: $FRONTEND_PID"
echo ""

# Wait for frontend to start
echo "⏳ Waiting for frontend to initialize..."
sleep 3

if ! kill -0 $FRONTEND_PID 2>/dev/null; then
    echo "❌ Frontend failed to start!"
    exit 1
fi

# Check if frontend is responding
MAX_RETRIES=10
RETRY=0
while [ $RETRY -lt $MAX_RETRIES ]; do
    if curl -f http://localhost:3000/ 2>/dev/null > /dev/null; then
        echo "✅ Frontend health check passed"
        break
    fi
    RETRY=$((RETRY+1))
    if [ $RETRY -lt $MAX_RETRIES ]; then
        echo "⏳ Frontend not ready yet, retrying ($RETRY/$MAX_RETRIES)..."
        sleep 2
    else
        echo "⚠️  Frontend health check failed after $MAX_RETRIES attempts (will proceed anyway)"
    fi
done

echo ""

# Start backend (which proxies to frontend)
echo "📦 Starting Backend on port 8000 (with frontend proxy)..."
cd /app

# Start backend in foreground with logging
RUST_LOG=$RUST_LOG DATABASE_URL=$DATABASE_URL FRONTEND_URL=$FRONTEND_URL exec ./backend

