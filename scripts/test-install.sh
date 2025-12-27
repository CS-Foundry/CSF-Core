#!/bin/bash
# CSF-Core Installation Script - Quick Test Version
# Dieses Script zeigt genau wo es hängt

set -x  # Debug mode - zeigt jeden Befehl

echo "=== CSF-Core Installation Test ==="
echo "1. Checking for PostgreSQL..."

if command -v psql &> /dev/null; then
    echo "✓ PostgreSQL command found"
else
    echo "⚠ PostgreSQL not found - will install"
    
    if command -v apt-get &> /dev/null; then
        echo "→ Using apt-get..."
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq || echo "apt-get update failed"
        apt-get install -y postgresql postgresql-contrib || echo "apt-get install failed"
        systemctl start postgresql || echo "systemctl start failed"
        systemctl enable postgresql || echo "systemctl enable failed"
    fi
fi

echo ""
echo "2. Testing PostgreSQL..."
if systemctl is-active --quiet postgresql; then
    echo "✓ PostgreSQL is running"
else
    echo "✗ PostgreSQL is NOT running"
fi

echo ""
echo "3. Creating test database..."
sudo -u postgres psql -c "SELECT version();" || echo "PostgreSQL connection failed"

echo ""
echo "=== Test Complete ==="
echo "If you see this message, the script ran to completion"
