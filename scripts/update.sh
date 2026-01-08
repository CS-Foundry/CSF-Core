#!/bin/bash

# Don't exit on errors immediately - we want to handle them gracefully
set +e

# CSF-Core Update Script
# This script downloads and installs updates for CSF-Core

VERSION="${1}"
REPO="CS-Foundry/CSF-Core"
INSTALL_DIR="/opt/csf-core"
STATUS_FILE="/tmp/csf-core-update-status.json"

# Determine if we need sudo early
if [ "$EUID" -eq 0 ]; then
    SUDO=""
else
    SUDO="sudo"
fi

# Function to find a writable backup directory
find_backup_dir() {
    local timestamp=$(date +%s)
    local dirs=(
        "/tmp/csf-core-backup-${timestamp}"
        "/var/tmp/csf-core-backup-${timestamp}"
        "${HOME}/.csf-core-backup-${timestamp}"
        "/opt/csf-core-backup-${timestamp}"
    )
    
    for dir in "${dirs[@]}"; do
        if mkdir -p "$dir" 2>/dev/null || $SUDO mkdir -p "$dir" 2>/dev/null; then
            echo "$dir"
            return 0
        fi
    done
    
    # If nothing works, return empty
    echo ""
    return 1
}

log() {
    local message="$1"
    local progress="${2:-0}"
    local timestamp="[$(date +'%Y-%m-%d %H:%M:%S')]"
    echo "$timestamp $message"
    update_status "in_progress" "$message" "$progress"
}

error() {
    local message="$1"
    local timestamp="[$(date +'%Y-%m-%d %H:%M:%S')]"
    echo "$timestamp ERROR: $message" >&2
    update_status "error" "$message" 0
    exit 1
}

update_status() {
    local status="$1"
    local message="$2"
    local progress="${3:-0}"
    
    # Ensure status file directory exists
    mkdir -p "$(dirname "$STATUS_FILE")" 2>/dev/null
    
    cat > "$STATUS_FILE" <<EOF
{
  "status": "$status",
  "message": "$message",
  "progress": $progress,
  "version": "$VERSION",
  "timestamp": "$(date -Iseconds 2>/dev/null || date +%Y-%m-%dT%H:%M:%S)"
}
EOF
}

# Check if version is provided
if [ -z "$VERSION" ]; then
    error "No version specified. Usage: $0 <version>"
fi

log "🚀 Starting CSF-Core update to version ${VERSION}..." 5
log "✓ Running with $([ "$EUID" -eq 0 ] && echo 'root privileges' || echo 'sudo')" 10

# Try to find/create a writable backup directory
log "📦 Trying to create backup directory..." 12
BACKUP_DIR=$(find_backup_dir)

if [ -z "$BACKUP_DIR" ]; then
    error "Failed to create backup directory. Tried: /tmp, /var/tmp, ${HOME}, /opt. Update aborted for safety."
fi

log "✓ Backup directory created: ${BACKUP_DIR}" 13

# Backup current installation - THIS IS REQUIRED
if [ -d "${INSTALL_DIR}" ]; then
    log "💾 Starting backup of current installation..." 15
    log "⚠️  Backup is REQUIRED - update will abort if backup fails" 16
    
    # Check if we have enough space
    INSTALL_SIZE=$(du -sm "${INSTALL_DIR}" 2>/dev/null | awk '{print $1}')
    AVAILABLE_SPACE=$(df -m /tmp | tail -1 | awk '{print $4}')
    
    if [ -n "$INSTALL_SIZE" ] && [ -n "$AVAILABLE_SPACE" ]; then
        if [ "$INSTALL_SIZE" -gt "$AVAILABLE_SPACE" ]; then
            log "⚠️ Warning: May not have enough space for backup (need ${INSTALL_SIZE}MB, have ${AVAILABLE_SPACE}MB)" 20
        else
            log "ℹ️  Backing up ${INSTALL_SIZE}MB (${AVAILABLE_SPACE}MB available)" 20
        fi
    fi
    
    # Use rsync if available (better progress), otherwise cp
    if command -v rsync >/dev/null 2>&1; then
        log "Using rsync for backup..." 21
        if $SUDO rsync -a "${INSTALL_DIR}/" "${BACKUP_DIR}/csf-core/" 2>&1; then
            log "✓ Rsync backup completed" 23
        else
            log "❌ Rsync backup failed, trying with cp..." 22
            if $SUDO cp -rp "${INSTALL_DIR}" "${BACKUP_DIR}/" 2>&1; then
                log "✓ Backup completed with cp" 23
            else
                error "Failed to create backup. Update aborted for safety. Tried rsync and cp."
            fi
        fi
    else
        log "Using cp for backup..." 21
        if $SUDO cp -rp "${INSTALL_DIR}" "${BACKUP_DIR}/" 2>&1; then
            log "✓ Backup completed" 23
        else
            error "Failed to create backup with cp. Update aborted for safety."
        fi
    fi
    
    # Verify backup was created - REQUIRED CHECK
    if [ -d "${BACKUP_DIR}/csf-core" ] || [ -d "${BACKUP_DIR}/opt/csf-core" ]; then
        log "✓ Backup verified at ${BACKUP_DIR}" 25
    else
        error "Backup verification failed - backup directory structure is wrong. Update aborted."
    fi
else
    log "ℹ️  No existing installation found at ${INSTALL_DIR} - skipping backup" 25
fi

# Stop the service if running
if systemctl is-active --quiet csf-core.service 2>/dev/null; then
    log "⏸️  Stopping CSF-Core service..." 28
    if $SUDO systemctl stop csf-core.service 2>&1; then
        log "✓ Service stopped" 30
    else
        log "⚠️ Service stop had issues, continuing anyway..." 30
    fi
else
    log "ℹ️  Service not running, skipping stop" 30
fi

# Detect architecture
log "🔍 Detecting system architecture..." 35
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
log "✓ Detected architecture: $ARCH_NAME" 38

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
OS_NAME="$OS"
log "✓ Detected OS: $OS_NAME" 40

# Check if this is a production environment (Linux only)
if [ "$OS_NAME" != "linux" ]; then
    error "Updates are only supported on Linux production systems. Current OS: $OS_NAME. This appears to be a development environment."
fi

# Download the new binaries
log "📥 Downloading backend binary for version ${VERSION}..." 45
BACKEND_URL="https://github.com/${REPO}/releases/download/v${VERSION}/csf-backend-${OS_NAME}-${ARCH_NAME}"
TMP_DIR=$(mktemp -d)

log "ℹ️  Download URL: ${BACKEND_URL}" 47

if ! curl -L -f -o "${TMP_DIR}/backend" "${BACKEND_URL}" 2>&1; then
    rm -rf "${TMP_DIR}"
    error "Failed to download backend from ${BACKEND_URL}"
fi

# Verify download
if [ ! -f "${TMP_DIR}/backend" ]; then
    rm -rf "${TMP_DIR}"
    error "Backend binary was not downloaded"
fi

log "✓ Backend binary downloaded ($(du -h "${TMP_DIR}/backend" | cut -f1))" 55

# Make backend executable
chmod +x "${TMP_DIR}/backend"

# Download frontend package
log "📥 Downloading frontend package..." 60
FRONTEND_URL="https://github.com/${REPO}/releases/download/v${VERSION}/csf-frontend-${VERSION}.tar.gz"

if curl -L -f -o "${TMP_DIR}/frontend.tar.gz" "${FRONTEND_URL}" 2>&1; then
    if [ -f "${TMP_DIR}/frontend.tar.gz" ]; then
        log "✓ Frontend package downloaded ($(du -h "${TMP_DIR}/frontend.tar.gz" | cut -f1))" 65
    fi
else
    log "⚠️ Frontend package not found, will keep existing frontend" 65
fi

# Install the binaries
log "📦 Installing backend binary..." 70
if ! $SUDO mkdir -p "${INSTALL_DIR}/backend"; then
    rm -rf "${TMP_DIR}"
    error "Failed to create backend directory"
fi

if ! $SUDO cp "${TMP_DIR}/backend" "${INSTALL_DIR}/backend/backend"; then
    rm -rf "${TMP_DIR}"
    error "Failed to copy backend binary"
fi

if ! $SUDO chmod +x "${INSTALL_DIR}/backend/backend"; then
    rm -rf "${TMP_DIR}"
    error "Failed to make backend executable"
fi

log "✓ Backend installed" 75

# Install frontend if downloaded
if [ -f "${TMP_DIR}/frontend.tar.gz" ]; then
    log "📦 Installing frontend..." 80
    if ! $SUDO mkdir -p "${INSTALL_DIR}/frontend"; then
        rm -rf "${TMP_DIR}"
        error "Failed to create frontend directory"
    fi
    
    if ! $SUDO tar -xzf "${TMP_DIR}/frontend.tar.gz" -C "${INSTALL_DIR}/frontend/"; then
        rm -rf "${TMP_DIR}"
        error "Failed to extract frontend package"
    fi
    log "✓ Frontend installed" 85
else
    log "ℹ️  Keeping existing frontend" 85
fi

# Set ownership
log "🔐 Setting correct permissions..." 88
if $SUDO chown -R csf-core:csf-core "${INSTALL_DIR}" 2>/dev/null; then
    log "✓ Ownership set to csf-core user" 90
elif $SUDO chown -R root:root "${INSTALL_DIR}" 2>/dev/null; then
    log "✓ Ownership set to root" 90
else
    log "⚠️ Could not set ownership, continuing..." 90
fi

# Restart service
log "🔄 Restarting CSF-Core service..." 92
if systemctl list-unit-files 2>/dev/null | grep -q csf-core.service; then
    log "Reloading systemd daemon..." 93
    $SUDO systemctl daemon-reload
    
    log "Starting service..." 95
    if $SUDO systemctl start csf-core.service 2>&1; then
        log "Service start command completed" 96
    else
        log "⚠️ Service start command had issues" 96
    fi
    
    # Wait a moment for service to start
    sleep 3
    
    # Check if service is running
    if systemctl is-active --quiet csf-core.service 2>/dev/null; then
        log "✅ CSF-Core service is running" 100
        update_status "completed" "Update completed successfully! Reloading..." 100
    else
        # Get service status for debugging
        SERVICE_STATUS=$(systemctl status csf-core.service 2>&1 | head -n 10 || echo "Could not get status")
        log "❌ Service failed to start. Status: ${SERVICE_STATUS}" 100
        update_status "error" "Service failed to start after update. Check logs with: journalctl -u csf-core.service -n 50" 100
        
        # Try to restore backup - we ALWAYS have a backup at this point
        log "🔄 Attempting to restore from backup..."
        if [ -d "${BACKUP_DIR}/csf-core" ]; then
            $SUDO systemctl stop csf-core.service 2>/dev/null || true
            $SUDO rm -rf "${INSTALL_DIR}"
            $SUDO mv "${BACKUP_DIR}/csf-core" "${INSTALL_DIR}"
            $SUDO systemctl start csf-core.service 2>/dev/null || true
            error "Update failed. System has been restored from backup at ${BACKUP_DIR}"
        else
            error "Service failed to start and backup structure unexpected at ${BACKUP_DIR}"
        fi
    fi
else
    log "⚠️ No systemd service found, skipping service restart" 100
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
log "   sudo cp -r ${BACKUP_DIR}/csf-core ${INSTALL_DIR}"
log "   sudo systemctl start csf-core.service"

exit 0
