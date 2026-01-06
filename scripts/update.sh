#!/bin/bash

set -e

# CSF-Core Update Script
# This script downloads and installs updates for CSF-Core

VERSION="${1}"
REPO="CS-Foundry/CSF-Core"
INSTALL_DIR="/opt/csf-core"
BACKUP_DIR="/tmp/csf-core-backup-$(date +%s)"
STATUS_FILE="/tmp/csf-core-update-status.json"

log() {
    local message="[$(date +'%Y-%m-%d %H:%M:%S')] $1"
    echo "$message"
    update_status "in_progress" "$1"
}

error() {
    local message="[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1"
    echo "$message" >&2
    update_status "error" "$1"
    exit 1
}

update_status() {
    local status="$1"
    local message="$2"
    local progress="${3:-0}"
    
    cat > "$STATUS_FILE" <<EOF
{
  "status": "$status",
  "message": "$message",
  "progress": $progress,
  "version": "$VERSION",
  "timestamp": "$(date -Iseconds)"
}
EOF
}

# Check if version is provided
if [ -z "$VERSION" ]; then
    error "No version specified. Usage: $0 <version>"
fi

log "🚀 Starting CSF-Core update to version ${VERSION}..." 5

# Check if running as root or with sudo
if [ "$EUID" -eq 0 ]; then
    SUDO=""
    log "✓ Running with root privileges" 10
else
    SUDO="sudo"
    log "✓ Running with sudo" 10
fi

# Create backup directory
log "📦 Creating backup directory at ${BACKUP_DIR}..." 15
$SUDO mkdir -p "${BACKUP_DIR}"

# Backup current installation
if [ -d "${INSTALL_DIR}" ]; then
    log "💾 Backing up current installation (this may take a moment)..." 20
    $SUDO cp -r "${INSTALL_DIR}" "${BACKUP_DIR}/"
    log "✓ Backup completed successfully" 25
else
    log "⚠ No existing installation found to backup" 25
fi

# Stop the service if running
if systemctl is-active --quiet csf-core.service 2>/dev/null; then
    log "⏸️  Stopping CSF-Core service..." 30
    $SUDO systemctl stop csf-core.service || log "⚠ Service stop failed or not installed"
    log "✓ Service stopped" 35
else
    log "ℹ️  Service not running" 35
fi

# Detect architecture
log "🔍 Detecting system architecture..." 40
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH_NAME="amd64"
        ;;
    aarch64|arm64)
        ARCH_NAME="arm64"
        ;;
    *)
        error "Unsupported architecture: $ARCH"
        ;;
esac
log "✓ Detected architecture: $ARCH_NAME" 42

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
log "✓ Detected OS: $OS" 45

# Download the new binaries
log "📥 Downloading backend binary for version ${VERSION}..." 50
BACKEND_URL="https://github.com/${REPO}/releases/download/v${VERSION}/csf-backend-${OS}-${ARCH_NAME}"
TMP_DIR=$(mktemp -d)

if ! curl -L -f -o "${TMP_DIR}/backend" "${BACKEND_URL}" 2>&1 | grep -v "^[0-9 ]*$" || true; then
    error "Failed to download backend from ${BACKEND_URL}"
fi
log "✓ Backend binary downloaded" 60

# Download frontend package
log "📥 Downloading frontend package..." 65
FRONTEND_URL="https://github.com/${REPO}/releases/download/v${VERSION}/csf-frontend-${VERSION}.tar.gz"

if ! curl -L -f -o "${TMP_DIR}/frontend.tar.gz" "${FRONTEND_URL}" 2>&1 | grep -v "^[0-9 ]*$" || true; then
    log "⚠ Frontend package not found, will try to keep existing frontend"
fi

if [ -f "${TMP_DIR}/frontend.tar.gz" ]; then
    log "✓ Frontend package downloaded" 70
fi

# Install the binaries
log "📦 Installing backend binary..." 75
$SUDO mkdir -p "${INSTALL_DIR}/backend"
$SUDO cp "${TMP_DIR}/backend" "${INSTALL_DIR}/backend/"
$SUDO chmod +x "${INSTALL_DIR}/backend/backend"
log "✓ Backend installed" 80

# Install frontend if downloaded
if [ -f "${TMP_DIR}/frontend.tar.gz" ]; then
    log "📦 Installing frontend..." 85
    $SUDO mkdir -p "${INSTALL_DIR}/frontend"
    $SUDO tar -xzf "${TMP_DIR}/frontend.tar.gz" -C "${INSTALL_DIR}/frontend/"
    log "✓ Frontend installed" 90
fi

# Set ownership
log "🔐 Setting correct permissions..." 92
$SUDO chown -R csf-core:csf-core "${INSTALL_DIR}" 2>/dev/null || $SUDO chown -R root:root "${INSTALL_DIR}"
log "✓ Permissions set" 95

# Restart service
log "🔄 Restarting CSF-Core service..." 97
if systemctl list-unit-files | grep -q csf-core.service; then
    $SUDO systemctl daemon-reload
    $SUDO systemctl start csf-core.service
    sleep 2
    
    if systemctl is-active --quiet csf-core.service; then
        log "✅ CSF-Core service is running" 100
        update_status "completed" "Update completed successfully!" 100
    else
        log "❌ Service failed to start" 100
        update_status "error" "Service failed to start after update" 100
        error "Service failed to start"
    fi
else
    log "⚠ No systemd service found" 100
    update_status "completed" "Binaries updated, but no service configured" 100
fi

# Cleanup
log "🧹 Cleaning up temporary files..."
rm -rf "${TMP_DIR}"

log "✅ Update to version ${VERSION} completed successfully!"
log "📦 Backup saved at: ${BACKUP_DIR}"
log ""
log "ℹ️  If you encounter any issues, you can restore from the backup:"
log "   sudo systemctl stop csf-core.service"
log "   sudo rm -rf ${INSTALL_DIR}"
log "   sudo mv ${BACKUP_DIR}/csf-core ${INSTALL_DIR}"
log "   sudo systemctl start csf-core.service"

exit 0
