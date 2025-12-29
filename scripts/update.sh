#!/bin/bash
# CSF-Core Update Script
# Lädt neue Version herunter und aktualisiert das System

set -e

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Konfiguration
INSTALL_DIR="/opt/csf-core"
SERVICE_NAME="csf-core"
GITHUB_REPO="CS-Foundry/CSF-Core"
BACKUP_DIR="/var/backups/csf-core"

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         CSF-Core Update Script                        ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}➜${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

check_root() {
    if [ "$EUID" -ne 0 ]; then 
        print_error "Bitte als root ausführen: sudo $0"
        exit 1
    fi
}

detect_architecture() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            print_error "Nicht unterstützte Architektur: $arch"
            exit 1
            ;;
    esac
}

get_current_version() {
    if [ -f "$INSTALL_DIR/VERSION" ]; then
        CURRENT_VERSION=$(cat "$INSTALL_DIR/VERSION")
        print_step "Aktuelle Version: $CURRENT_VERSION"
    else
        print_warning "Keine Versionsinformation gefunden"
        CURRENT_VERSION="unknown"
    fi
}

get_latest_version() {
    print_step "Ermittle neueste Version..."
    
    local release_info
    if release_info=$(curl -s "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"); then
        LATEST_VERSION=$(echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -n "$LATEST_VERSION" ]; then
            print_success "Neueste Version: $LATEST_VERSION"
            return 0
        fi
    fi
    
    print_error "Konnte neueste Version nicht ermitteln"
    return 1
}

check_update_available() {
    get_current_version
    
    if ! get_latest_version; then
        return 1
    fi
    
    if [ "$CURRENT_VERSION" = "$LATEST_VERSION" ]; then
        print_success "System ist bereits auf dem neuesten Stand!"
        return 1
    fi
    
    print_step "Update verfügbar: $CURRENT_VERSION → $LATEST_VERSION"
    return 0
}

create_backup() {
    print_step "Erstelle Backup..."
    
    mkdir -p "$BACKUP_DIR"
    
    local backup_name="csf-core-backup-$(date +%Y%m%d-%H%M%S)"
    local backup_path="$BACKUP_DIR/$backup_name"
    
    # Backup erstellen
    tar -czf "$backup_path.tar.gz" \
        -C "$INSTALL_DIR" \
        backend frontend agent VERSION COMMIT 2>/dev/null || true
    
    # Behalte nur die letzten 5 Backups
    cd "$BACKUP_DIR"
    ls -t | tail -n +6 | xargs rm -f 2>/dev/null || true
    
    print_success "Backup erstellt: $backup_path.tar.gz"
}

stop_services() {
    print_step "Stoppe Services..."
    
    systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    systemctl stop "${SERVICE_NAME}-agent" 2>/dev/null || true
    
    sleep 2
    
    print_success "Services gestoppt"
}

download_update() {
    print_step "Lade Update herunter..."
    
    local temp_dir=$(mktemp -d)
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/csf-core-linux-${ARCH}.tar.gz"
    
    if curl -L -f -o "$temp_dir/csf-core.tar.gz" "$download_url"; then
        print_success "Download erfolgreich"
    else
        print_error "Download fehlgeschlagen"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Verifiziere Checksum
    if curl -L -f -o "$temp_dir/csf-core.tar.gz.sha256" "${download_url}.sha256" 2>/dev/null; then
        print_step "Verifiziere Checksum..."
        cd "$temp_dir"
        if sha256sum -c csf-core.tar.gz.sha256; then
            print_success "Checksum verifiziert"
        else
            print_error "Checksum Verifikation fehlgeschlagen"
            rm -rf "$temp_dir"
            return 1
        fi
        cd - > /dev/null
    fi
    
    # Extrahiere Update
    print_step "Extrahiere Update..."
    tar -xzf "$temp_dir/csf-core.tar.gz" -C "$temp_dir"
    
    # Installiere neue Binaries
    print_step "Installiere Backend..."
    cp "$temp_dir/backend/backend" "$INSTALL_DIR/backend/"
    chmod +x "$INSTALL_DIR/backend/backend"
    
    print_step "Installiere Agent..."
    cp "$temp_dir/agent/csf-agent" "$INSTALL_DIR/agent/"
    chmod +x "$INSTALL_DIR/agent/csf-agent"
    # Behalte existierende config.toml
    if [ ! -f "$INSTALL_DIR/agent/config.toml" ]; then
        cp "$temp_dir/agent/config.toml" "$INSTALL_DIR/agent/"
    fi
    
    print_step "Installiere Frontend..."
    rm -rf "$INSTALL_DIR/frontend/build" 2>/dev/null || true
    rm -rf "$INSTALL_DIR/frontend/node_modules" 2>/dev/null || true
    cp -r "$temp_dir/frontend/"* "$INSTALL_DIR/frontend/"
    
    # Update Version-Info
    if [ -f "$temp_dir/VERSION" ]; then
        cp "$temp_dir/VERSION" "$INSTALL_DIR/VERSION"
    fi
    if [ -f "$temp_dir/COMMIT" ]; then
        cp "$temp_dir/COMMIT" "$INSTALL_DIR/COMMIT"
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    print_success "Update installiert"
    return 0
}

start_services() {
    print_step "Starte Services..."
    
    systemctl start "$SERVICE_NAME"
    systemctl start "${SERVICE_NAME}-agent"
    
    sleep 3
    
    if systemctl is-active --quiet "$SERVICE_NAME" && systemctl is-active --quiet "${SERVICE_NAME}-agent"; then
        print_success "Services erfolgreich gestartet"
        return 0
    else
        print_error "Services konnten nicht gestartet werden"
        print_warning "Prüfe Logs mit: journalctl -u ${SERVICE_NAME} -n 50"
        return 1
    fi
}

restore_backup() {
    print_step "Stelle Backup wieder her..."
    
    local latest_backup=$(ls -t "$BACKUP_DIR"/*.tar.gz 2>/dev/null | head -n1)
    
    if [ -z "$latest_backup" ]; then
        print_error "Kein Backup gefunden"
        return 1
    fi
    
    stop_services
    
    tar -xzf "$latest_backup" -C "$INSTALL_DIR"
    
    print_success "Backup wiederhergestellt: $(basename $latest_backup)"
    
    start_services
}

show_info() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              Update erfolgreich!                       ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Version:${NC}"
    echo -e "  Vorher: $CURRENT_VERSION"
    echo -e "  Jetzt:  $LATEST_VERSION"
    echo ""
    echo -e "${BLUE}Services:${NC}"
    echo -e "  Backend: systemctl status ${SERVICE_NAME}"
    echo -e "  Agent:   systemctl status ${SERVICE_NAME}-agent"
    echo ""
    echo -e "${BLUE}Logs:${NC}"
    echo -e "  journalctl -u ${SERVICE_NAME} -f"
    echo ""
}

# Hauptprogramm
main() {
    print_header
    
    check_root
    detect_architecture
    
    if ! check_update_available; then
        exit 0
    fi
    
    # Bestätigung einholen
    echo ""
    read -p "Möchten Sie das Update installieren? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_step "Update abgebrochen"
        exit 0
    fi
    
    create_backup
    stop_services
    
    if download_update; then
        if start_services; then
            show_info
        else
            print_error "Services konnten nicht gestartet werden"
            print_step "Stelle Backup wieder her? (y/N)"
            read -p "" -n 1 -r
            echo ""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                restore_backup
            fi
            exit 1
        fi
    else
        print_error "Update fehlgeschlagen"
        print_step "Stelle Backup wieder her..."
        restore_backup
        exit 1
    fi
}

# Kommandozeilen-Optionen
case "${1:-}" in
    --check)
        detect_architecture
        check_update_available
        ;;
    --force)
        main
        ;;
    --restore)
        check_root
        restore_backup
        ;;
    *)
        main
        ;;
esac
