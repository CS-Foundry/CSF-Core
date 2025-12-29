#!/bin/bash
# CSF-Core Binary Installation Script
# Lädt vorkompilierte Binaries herunter und installiert sie als systemd Service

set -e

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Konfiguration
INSTALL_DIR="/opt/csf-core"
SERVICE_USER="csf-core"
SERVICE_NAME="csf-core"
DATA_DIR="/var/lib/csf-core"
LOG_DIR="/var/log/csf-core"
GITHUB_REPO="CS-Foundry/CSF-Core"
BRANCH="${BRANCH:-main}"
VERSION="${VERSION:-latest}"
INSTALL_MODE="${INSTALL_MODE:-binary}"  # binary oder source

# Environment-Variablen
PUBLIC_API_BASE_URL="${PUBLIC_API_BASE_URL:-/api}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:3000}"
ORIGIN="${ORIGIN:-}"
ENV_FILE="${ENV_FILE:-}"
FRONTEND_ENV_FILE="${FRONTEND_ENV_FILE:-}"

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         CSF-Core Installation Script                  ║${NC}"
    echo -e "${BLUE}║         Binary Deployment (No Compilation Required)   ║${NC}"
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
    print_step "Erkenne System-Architektur..."
    
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
    
    print_success "Architektur: $ARCH"
}

check_dependencies() {
    print_step "Prüfe System-Abhängigkeiten..."
    
    local missing_deps=()
    
    for cmd in curl tar systemctl; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Fehlende Abhängigkeiten: ${missing_deps[*]}"
        print_step "Installiere fehlende Pakete..."
        if command -v apt-get &> /dev/null; then
            apt-get update -qq && apt-get install -y -qq curl tar systemd
        elif command -v dnf &> /dev/null; then
            dnf install -y -q curl tar systemd
        elif command -v yum &> /dev/null; then
            yum install -y -q curl tar systemd
        else
            print_error "Paketmanager nicht unterstützt"
            exit 1
        fi
    fi
    
    print_success "Alle Abhängigkeiten verfügbar"
}

install_postgresql() {
    print_step "Prüfe PostgreSQL Installation..."
    
    if command -v psql &> /dev/null; then
        print_success "PostgreSQL bereits installiert"
        if systemctl is-active --quiet postgresql; then
            print_success "PostgreSQL Service läuft"
        else
            systemctl start postgresql 2>/dev/null || true
            systemctl enable postgresql 2>/dev/null || true
        fi
        return
    fi
    
    print_step "Installiere PostgreSQL..."
    
    if command -v apt-get &> /dev/null; then
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq && apt-get install -y -qq postgresql postgresql-contrib
        systemctl enable postgresql && systemctl start postgresql
    elif command -v dnf &> /dev/null; then
        dnf install -y -q postgresql-server postgresql-contrib
        postgresql-setup --initdb 2>/dev/null || /usr/bin/postgresql-setup initdb
        systemctl enable postgresql && systemctl start postgresql
    elif command -v yum &> /dev/null; then
        yum install -y -q postgresql-server postgresql-contrib
        postgresql-setup --initdb 2>/dev/null || /usr/bin/postgresql-setup initdb
        systemctl enable postgresql && systemctl start postgresql
    else
        print_warning "PostgreSQL automatische Installation nicht möglich"
        print_warning "Bitte manuell installieren oder SQLite verwenden"
        USE_SQLITE=true
    fi
    
    sleep 2
    print_success "PostgreSQL installiert"
}

get_latest_release_tag() {
    print_step "Ermittle neueste Version..."
    
    # Versuche Release zu finden
    local release_info
    if release_info=$(curl -s "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"); then
        local tag_name=$(echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -n "$tag_name" ]; then
            echo "$tag_name"
            return
        fi
    fi
    
    # Fallback: Verwende Branch
    echo "$BRANCH"
}

download_binary_release() {
    print_step "Lade vorkompilierte Binaries herunter..."
    
    local download_url
    local temp_dir=$(mktemp -d)
    
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_release_tag)
    fi
    
    # Prüfe ob es ein Release-Tag ist
    if [[ "$VERSION" == v* ]]; then
        download_url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/csf-core-linux-${ARCH}.tar.gz"
        print_step "Download von Release: $VERSION"
    else
        # Branch-basierter Download - nutze GitHub Actions Artifacts
        print_warning "Branch-basierter Download: $VERSION"
        print_warning "Verwende den branch-spezifischen Artifact-Download"
        
        # Für branch builds müssen wir den workflow run finden
        download_url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/csf-core-linux-${ARCH}.tar.gz"
    fi
    
    print_step "Download URL: $download_url"
    
    if curl -L -f -o "$temp_dir/csf-core.tar.gz" "$download_url"; then
        print_success "Download erfolgreich"
    else
        print_error "Download fehlgeschlagen"
        print_warning "Versuche alternativen Download..."
        
        # Alternative: Latest von Branch
        download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/csf-core-linux-${ARCH}.tar.gz"
        if curl -L -f -o "$temp_dir/csf-core.tar.gz" "$download_url"; then
            print_success "Alternative Download erfolgreich"
        else
            print_error "Kein Binary verfügbar für $VERSION ($ARCH)"
            print_error "Bitte verwende INSTALL_MODE=source für Kompilierung aus Quellcode"
            rm -rf "$temp_dir"
            exit 1
        fi
    fi
    
    # Verifiziere Checksum wenn verfügbar
    if curl -L -f -o "$temp_dir/csf-core.tar.gz.sha256" "${download_url}.sha256" 2>/dev/null; then
        print_step "Verifiziere Checksum..."
        cd "$temp_dir"
        if sha256sum -c csf-core.tar.gz.sha256; then
            print_success "Checksum verifiziert"
        else
            print_error "Checksum Verifikation fehlgeschlagen"
            rm -rf "$temp_dir"
            exit 1
        fi
        cd - > /dev/null
    fi
    
    # Extrahiere Archiv
    print_step "Extrahiere Binaries..."
    tar -xzf "$temp_dir/csf-core.tar.gz" -C "$temp_dir"
    
    # Installiere Binaries
    print_step "Installiere Backend..."
    mkdir -p "$INSTALL_DIR/backend"
    cp "$temp_dir/backend/backend" "$INSTALL_DIR/backend/"
    chmod +x "$INSTALL_DIR/backend/backend"
    
    print_step "Installiere Agent..."
    mkdir -p "$INSTALL_DIR/agent"
    cp "$temp_dir/agent/csf-agent" "$INSTALL_DIR/agent/"
    cp "$temp_dir/agent/config.toml" "$INSTALL_DIR/agent/"
    chmod +x "$INSTALL_DIR/agent/csf-agent"
    
    print_step "Installiere Frontend..."
    mkdir -p "$INSTALL_DIR/frontend"
    cp -r "$temp_dir/frontend/"* "$INSTALL_DIR/frontend/"
    
    # Speichere Version
    if [ -f "$temp_dir/VERSION" ]; then
        cp "$temp_dir/VERSION" "$INSTALL_DIR/VERSION"
        VERSION_INSTALLED=$(cat "$INSTALL_DIR/VERSION")
        print_success "Version installiert: $VERSION_INSTALLED"
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    print_success "Binaries erfolgreich installiert"
}

create_service_user() {
    print_step "Erstelle Service-Benutzer..."
    
    if id "$SERVICE_USER" &>/dev/null; then
        print_success "Benutzer '$SERVICE_USER' existiert bereits"
    else
        useradd --system --no-create-home --shell /usr/sbin/nologin "$SERVICE_USER"
        print_success "Benutzer '$SERVICE_USER' erstellt"
    fi
}

create_directories() {
    print_step "Erstelle Verzeichnisse..."
    
    mkdir -p "$INSTALL_DIR"/{backend,frontend,agent}
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    
    print_success "Verzeichnisse erstellt"
}

setup_database() {
    print_step "Konfiguriere Datenbank..."
    
    if [ "${USE_SQLITE:-false}" = "true" ]; then
        print_step "Verwende SQLite..."
        DATABASE_URL="sqlite://${DATA_DIR}/csf-core.db"
        touch "${DATA_DIR}/csf-core.db"
    else
        print_step "Erstelle PostgreSQL Datenbank..."
        
        DB_NAME="csf_core"
        DB_USER="csf_core"
        DB_PASSWORD=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
        
        sudo -u postgres psql -c "CREATE USER ${DB_USER} WITH PASSWORD '${DB_PASSWORD}';" 2>/dev/null || true
        sudo -u postgres psql -c "CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};" 2>/dev/null || true
        sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO ${DB_USER};" 2>/dev/null || true
        
        DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@localhost/${DB_NAME}"
        
        print_success "PostgreSQL Datenbank erstellt"
    fi
}

generate_secrets() {
    print_step "Generiere Sicherheitsschlüssel..."
    
    JWT_SECRET=$(openssl rand -base64 64 | tr -d "=+/" | cut -c1-64)
    RSA_PRIVATE_KEY_PATH="$DATA_DIR/rsa_private_key.pem"
    RSA_PUBLIC_KEY_PATH="$DATA_DIR/rsa_public_key.pem"
    
    openssl genpkey -algorithm RSA -out "$RSA_PRIVATE_KEY_PATH" -pkeyopt rsa_keygen_bits:4096 2>/dev/null
    openssl rsa -pubout -in "$RSA_PRIVATE_KEY_PATH" -out "$RSA_PUBLIC_KEY_PATH" 2>/dev/null
    
    chmod 600 "$RSA_PRIVATE_KEY_PATH"
    chmod 644 "$RSA_PUBLIC_KEY_PATH"
    
    print_success "Sicherheitsschlüssel generiert"
}

create_env_files() {
    print_step "Erstelle Konfigurationsdateien..."
    
    # Backend .env
    cat > "$INSTALL_DIR/backend/.env" << EOF
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
RSA_PRIVATE_KEY_PATH=${RSA_PRIVATE_KEY_PATH}
RSA_PUBLIC_KEY_PATH=${RSA_PUBLIC_KEY_PATH}
FRONTEND_URL=${FRONTEND_URL}
ORIGIN=${ORIGIN}
RUST_LOG=info
EOF

    # Frontend .env
    cat > "$INSTALL_DIR/frontend/.env" << EOF
PUBLIC_API_BASE_URL=${PUBLIC_API_BASE_URL}
ORIGIN=http://localhost:3000
NODE_ENV=production
EOF

    # Agent config
    cat > "$INSTALL_DIR/agent/config.toml" << EOF
[agent]
name = "$(hostname)"
server_url = "http://localhost:8080"
collection_interval_secs = 30
api_key = "your-api-key-here"

[server]
enabled = true
host = "0.0.0.0"
port = 8081
EOF

    print_success "Konfigurationsdateien erstellt"
}

create_systemd_service() {
    print_step "Erstelle systemd Service..."
    
    cat > "/etc/systemd/system/${SERVICE_NAME}.service" << EOF
[Unit]
Description=CSF-Core Backend and Frontend Service
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${INSTALL_DIR}
EnvironmentFile=${INSTALL_DIR}/backend/.env

# Start backend
ExecStart=${INSTALL_DIR}/backend/backend

# Restart policy
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR} ${LOG_DIR} ${INSTALL_DIR}

# Logging
StandardOutput=append:${LOG_DIR}/backend.log
StandardError=append:${LOG_DIR}/backend-error.log
SyslogIdentifier=csf-core

[Install]
WantedBy=multi-user.target
EOF

    # Agent Service
    cat > "/etc/systemd/system/${SERVICE_NAME}-agent.service" << EOF
[Unit]
Description=CSF-Core Monitoring Agent
After=network.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${INSTALL_DIR}/agent
ExecStart=${INSTALL_DIR}/agent/csf-agent

Restart=always
RestartSec=10

StandardOutput=append:${LOG_DIR}/agent.log
StandardError=append:${LOG_DIR}/agent-error.log

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    
    print_success "systemd Services erstellt"
}

set_permissions() {
    print_step "Setze Berechtigungen..."
    
    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$LOG_DIR"
    
    print_success "Berechtigungen gesetzt"
}

run_migrations() {
    print_step "Führe Datenbank-Migrationen aus..."
    
    if systemctl start "$SERVICE_NAME"; then
        sleep 5
        if systemctl is-active --quiet "$SERVICE_NAME"; then
            print_success "Migrationen erfolgreich"
        else
            print_warning "Service konnte nicht gestartet werden"
            journalctl -u "$SERVICE_NAME" -n 20 --no-pager
        fi
    fi
}

start_services() {
    print_step "Starte Services..."
    
    systemctl enable "$SERVICE_NAME"
    systemctl enable "${SERVICE_NAME}-agent"
    systemctl restart "$SERVICE_NAME"
    systemctl restart "${SERVICE_NAME}-agent"
    
    sleep 3
    
    if systemctl is-active --quiet "$SERVICE_NAME" && systemctl is-active --quiet "${SERVICE_NAME}-agent"; then
        print_success "Services erfolgreich gestartet"
    else
        print_warning "Services konnten nicht gestartet werden"
        print_step "Backend Status:"
        systemctl status "$SERVICE_NAME" --no-pager || true
        print_step "Agent Status:"
        systemctl status "${SERVICE_NAME}-agent" --no-pager || true
    fi
}

show_info() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              Installation abgeschlossen!               ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Dienste:${NC}"
    echo -e "  Backend:  systemctl status ${SERVICE_NAME}"
    echo -e "  Agent:    systemctl status ${SERVICE_NAME}-agent"
    echo ""
    echo -e "${BLUE}Logs:${NC}"
    echo -e "  Backend:  journalctl -u ${SERVICE_NAME} -f"
    echo -e "  Agent:    journalctl -u ${SERVICE_NAME}-agent -f"
    echo ""
    echo -e "${BLUE}Installationsverzeichnis:${NC}"
    echo -e "  ${INSTALL_DIR}"
    echo ""
    echo -e "${BLUE}Daten:${NC}"
    echo -e "  ${DATA_DIR}"
    echo ""
    
    if [ -f "$INSTALL_DIR/VERSION" ]; then
        echo -e "${BLUE}Version:${NC}"
        echo -e "  $(cat $INSTALL_DIR/VERSION)"
        echo ""
    fi
}

# Hauptprogramm
main() {
    print_header
    
    check_root
    detect_architecture
    check_dependencies
    install_postgresql
    create_service_user
    create_directories
    download_binary_release
    setup_database
    generate_secrets
    create_env_files
    create_systemd_service
    set_permissions
    run_migrations
    start_services
    show_info
}

main "$@"
