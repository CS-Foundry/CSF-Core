#!/bin/bash
set -e

# Get the new version from parameter or package.json
if [ -n "$1" ]; then
    NEW_VERSION="$1"
    echo "Using version from parameter: ${NEW_VERSION}"
else
    NEW_VERSION=$(node -p "require('./package.json').version")
    echo "Using version from package.json: ${NEW_VERSION}"
fi

echo "Updating versions to ${NEW_VERSION}..."

# Update frontend package.json
if [ -f "frontend/package.json" ]; then
    echo "Updating frontend/package.json..."
    # Use a more portable approach for macOS and Linux
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"${NEW_VERSION}\"/" frontend/package.json
    else
        sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"${NEW_VERSION}\"/" frontend/package.json
    fi
    echo "✓ Updated frontend/package.json to ${NEW_VERSION}"
fi

# Update backend Cargo.toml
if [ -f "backend/Cargo.toml" ]; then
    echo "Updating backend/Cargo.toml..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" backend/Cargo.toml
    else
        sed -i "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" backend/Cargo.toml
    fi
    echo "✓ Updated backend/Cargo.toml to ${NEW_VERSION}"
fi

echo "Version update complete! All files updated to ${NEW_VERSION}"
