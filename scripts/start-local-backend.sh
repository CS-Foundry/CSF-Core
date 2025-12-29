#!/bin/bash

# Script to run backend locally (outside Docker) for development
# This will start postgres and frontend in Docker, backend locally

set -e

echo "🚀 Starting CSF-Core with local backend..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ .env file not found. Please create one first."
    exit 1
fi

# Load environment variables
source .env

# Start postgres and frontend in Docker
echo "📦 Starting postgres and frontend containers..."
docker-compose -f docker-compose.local-backend.yml up -d

# Wait for postgres to be ready
echo "⏳ Waiting for postgres to be ready..."
until docker exec $(docker ps -qf "name=postgres") pg_isready -U csf_user -d csf_core > /dev/null 2>&1; do
    sleep 1
done

echo "✅ Postgres is ready!"

# Set DATABASE_URL for local backend
export DATABASE_URL="postgres://csf_user:csf_password@localhost:5432/csf_core"
export RUST_LOG="${RUST_LOG:-debug}"

echo "🔧 Backend will connect to: $DATABASE_URL"
echo ""
echo "📝 To start the backend, run:"
echo "   cd backend && cargo run"
echo ""
echo "🌐 Frontend: http://localhost:3000"
echo "🔌 Backend: http://localhost:8000"
echo "🗄️  Postgres: localhost:5432"
echo ""
echo "To stop containers: docker-compose -f docker-compose.local-backend.yml down"
