#!/bin/bash
set -e

# Get the new version from package.json
NEW_VERSION=$(node -p "require('./package.json').version")

echo "Updating versions to ${NEW_VERSION}..."

# Update frontend package.json
if [ -f "frontend/package.json" ]; then
    echo "Updating frontend/package.json..."
    sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"${NEW_VERSION}\"/" frontend/package.json
    rm -f frontend/package.json.bak
fi

# Update backend Cargo.toml
if [ -f "backend/Cargo.toml" ]; then
    echo "Updating backend/Cargo.toml..."
    sed -i.bak "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" backend/Cargo.toml
    rm -f backend/Cargo.toml.bak
fi

echo "Version update complete!"
