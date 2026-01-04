#!/bin/bash

set -e

# CSF-Core Update Script
# This script downloads and installs updates for CSF-Core

VERSION="${1}"
REPO="CS-Foundry/CSF-Core"
INSTALL_DIR="/opt/csf-core"
BACKUP_DIR="/tmp/csf-core-backup-$(date +%s)"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1" >&2
    exit 1
}

# Check if version is provided
if [ -z "$VERSION" ]; then
    error "No version specified. Usage: $0 <version>"
fi

log "Starting CSF-Core update to version ${VERSION}..."

# Check if running as root or with sudo
if [ "$EUID" -eq 0 ]; then
    SUDO=""
else
    SUDO="sudo"
fi

# Create backup directory
log "Creating backup at ${BACKUP_DIR}..."
$SUDO mkdir -p "${BACKUP_DIR}"

# Backup current installation
if [ -d "${INSTALL_DIR}" ]; then
    log "Backing up current installation..."
    $SUDO cp -r "${INSTALL_DIR}" "${BACKUP_DIR}/"
fi

# Stop the service if running
if systemctl is-active --quiet csf-core.service 2>/dev/null; then
    log "Stopping CSF-Core service..."
    $SUDO systemctl stop csf-core.service || log "Service stop failed or not installed"
fi

# Download the new version
DOWNLOAD_URL="https://github.com/${REPO}/archive/refs/tags/v${VERSION}.tar.gz"
TMP_DIR=$(mktemp -d)
log "Downloading version ${VERSION} from ${DOWNLOAD_URL}..."

if ! curl -L -o "${TMP_DIR}/csf-core.tar.gz" "${DOWNLOAD_URL}"; then
    error "Failed to download update from ${DOWNLOAD_URL}"
fi

# Extract the archive
log "Extracting archive..."
cd "${TMP_DIR}"
tar -xzf csf-core.tar.gz || error "Failed to extract archive"

# Find the extracted directory (should be CSF-Core-<version>)
EXTRACTED_DIR=$(find "${TMP_DIR}" -maxdepth 1 -type d -name "CSF-Core-*" | head -1)
if [ -z "$EXTRACTED_DIR" ]; then
    error "Could not find extracted directory"
fi

log "Extracted to ${EXTRACTED_DIR}"

# Install the update
log "Installing update..."
cd "${EXTRACTED_DIR}"

# Run installation script if it exists
if [ -f "scripts/install.sh" ]; then
    log "Running installation script..."
    $SUDO bash scripts/install.sh || error "Installation script failed"
else
    # Manual installation fallback
    log "No installation script found, performing manual update..."
    
    # Copy files to installation directory
    $SUDO mkdir -p "${INSTALL_DIR}"
    $SUDO cp -r ./* "${INSTALL_DIR}/"
    
    # Set permissions
    $SUDO chown -R root:root "${INSTALL_DIR}"
    $SUDO chmod +x "${INSTALL_DIR}/scripts/"*.sh 2>/dev/null || true
    
    # Restart service if systemd service exists
    if [ -f "${INSTALL_DIR}/csf-core.service" ]; then
        log "Reloading systemd and starting service..."
        $SUDO systemctl daemon-reload
        $SUDO systemctl start csf-core.service
        $SUDO systemctl enable csf-core.service
    fi
fi

# Cleanup
log "Cleaning up temporary files..."
rm -rf "${TMP_DIR}"

# Verify installation
if systemctl is-active --quiet csf-core.service 2>/dev/null; then
    log "✓ CSF-Core service is running"
else
    log "⚠ CSF-Core service is not running. Manual start may be required."
fi

log "Update to version ${VERSION} completed successfully!"
log "Backup saved at: ${BACKUP_DIR}"
log ""
log "If you encounter any issues, you can restore from the backup:"
log "  sudo rm -rf ${INSTALL_DIR}"
log "  sudo mv ${BACKUP_DIR}/csf-core ${INSTALL_DIR}"
log "  sudo systemctl restart csf-core.service"

exit 0
