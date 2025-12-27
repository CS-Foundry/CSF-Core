#!/bin/bash
# CSF-Core Unified Startup Script
# Startet Backend mit Frontend Proxy

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="${SCRIPT_DIR}/frontend"
BACKEND_BIN="${SCRIPT_DIR}/backend/backend"
LOG_DIR="${LOG_DIR:-/var/log/csf-core}"

# Erstelle Log-Verzeichnis falls nicht vorhanden
mkdir -p "$LOG_DIR" 2>/dev/null || true

echo "🚀 Starting CSF-Core..."
echo "   Frontend Dir: $FRONTEND_DIR"
echo "   Backend Binary: $BACKEND_BIN"
echo "   Logs: $LOG_DIR"
echo ""

# Start Frontend in background
if [ -d "$FRONTEND_DIR" ] && [ -f "$FRONTEND_DIR/package.json" ]; then
    echo "▶️  Starting Frontend (Port ${PORT:-3000})..."
    cd "$FRONTEND_DIR"
    
    # Load config.env for environment variables
    if [ -f "$SCRIPT_DIR/config.env" ]; then
        set -a
        source "$SCRIPT_DIR/config.env"
        set +a
    fi
    
    # Set PUBLIC_API_BASE_URL for runtime (Frontend erreichbar über Backend Port 8000)
    export PUBLIC_API_BASE_URL="/api"
    export ORIGIN="${ORIGIN:-http://localhost:8000}"
    
    PORT=${PORT:-3000} node build/index.js > "$LOG_DIR/frontend.log" 2>&1 &
    FRONTEND_PID=$!
    echo "   Frontend PID: $FRONTEND_PID"
    echo "   PUBLIC_API_BASE_URL: $PUBLIC_API_BASE_URL"
    echo "   Frontend läuft intern auf Port ${PORT:-3000}, extern über Backend auf Port 8000"
    
    # Wait for frontend to be ready
    echo "⏳ Waiting for frontend to start..."
    for i in {1..30}; do
        if curl -s http://localhost:${PORT:-3000} > /dev/null 2>&1; then
            echo "✅ Frontend ready!"
            break
        fi
        if [ $i -eq 30 ]; then
            echo "❌ Frontend failed to start in time"
            kill $FRONTEND_PID 2>/dev/null || true
            exit 1
        fi
        sleep 1
    done
else
    echo "⚠️  Frontend directory not found, skipping..."
fi

# Start Backend (will proxy to frontend)
echo "▶️  Starting Backend (Port 8000)..."
cd "$SCRIPT_DIR"

if [ -f "$BACKEND_BIN" ]; then
    exec "$BACKEND_BIN"
else
    echo "❌ Backend binary not found: $BACKEND_BIN"
    exit 1
fi
