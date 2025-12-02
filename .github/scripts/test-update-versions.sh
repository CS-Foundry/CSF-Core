#!/bin/bash
# Test script for version updates

set -e

echo "🧪 Testing version update script..."
echo ""

# Backup current versions
echo "📦 Creating backups..."
cp package.json package.json.backup
cp frontend/package.json frontend/package.json.backup
cp backend/Cargo.toml backend/Cargo.toml.backup

# Set test version
TEST_VERSION="9.9.9"
echo "🔧 Setting test version to ${TEST_VERSION}..."

# Update root package.json
sed -i.tmp "s/\"version\": \"[^\"]*\"/\"version\": \"${TEST_VERSION}\"/" package.json
rm -f package.json.tmp

# Run update script
echo "▶️  Running update-versions.sh..."
chmod +x .github/scripts/update-versions.sh
.github/scripts/update-versions.sh

# Verify updates
echo ""
echo "✅ Verification:"
echo ""

ROOT_VERSION=$(node -p "require('./package.json').version")
FRONTEND_VERSION=$(node -p "require('./frontend/package.json').version")
BACKEND_VERSION=$(grep "^version = " backend/Cargo.toml | head -n1 | cut -d'"' -f2)

echo "Root version:     ${ROOT_VERSION}"
echo "Frontend version: ${FRONTEND_VERSION}"
echo "Backend version:  ${BACKEND_VERSION}"

# Restore backups
echo ""
echo "🔄 Restoring backups..."
mv package.json.backup package.json
mv frontend/package.json.backup frontend/package.json
mv backend/Cargo.toml.backup backend/Cargo.toml

# Check if all versions match
if [ "$ROOT_VERSION" = "$TEST_VERSION" ] && \
   [ "$FRONTEND_VERSION" = "$TEST_VERSION" ] && \
   [ "$BACKEND_VERSION" = "$TEST_VERSION" ]; then
    echo ""
    echo "✅ Test PASSED! All versions updated correctly."
    exit 0
else
    echo ""
    echo "❌ Test FAILED! Version mismatch detected."
    exit 1
fi
