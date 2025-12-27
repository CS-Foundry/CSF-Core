#!/bin/bash
# CSF-Core Installation Script
# Installiert Backend + Frontend als systemd Service

# Nicht bei Fehlern abbrechen - wir handhaben Fehler manuell
# set -e wurde entfernt für bessere Fehlerbehandlung

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
VERSION="${VERSION:-latest}"

# Environment-Variablen für URLs
# Standard: /api (relative path für Production)
PUBLIC_API_BASE_URL="${PUBLIC_API_BASE_URL:-/api}"

# FRONTEND_URL für Backend (wohin Backend das Frontend proxied)
# Standard: http://localhost:3000
FRONTEND_URL="${FRONTEND_URL:-http://localhost:3000}"

# ORIGIN für CORS (erlaubte Origins)
# Standard: wird automatisch gesetzt basierend auf externer IP
ORIGIN="${ORIGIN:-}"

# Optional: Pfad zu einer existierenden .env Datei
# Diese wird dann für Frontend-Build verwendet
# Beispiel: ENV_FILE=/path/to/.env bash install.sh
ENV_FILE="${ENV_FILE:-}"
FRONTEND_ENV_FILE="${FRONTEND_ENV_FILE:-}"

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         CSF-Core Installation Script                  ║${NC}"
    echo -e "${BLUE}║         Backend + Frontend Unified Deployment         ║${NC}"
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

check_dependencies() {
    print_step "Prüfe System-Abhängigkeiten..."
    
    local missing_deps=()
    
    for cmd in curl systemctl useradd; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Fehlende Abhängigkeiten: ${missing_deps[*]}"
        exit 1
    fi
    
    print_success "Alle Abhängigkeiten verfügbar"
}

install_build_tools() {
    print_step "Installiere Build-Tools (gcc, make, git)..."
    
    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        export DEBIAN_FRONTEND=noninteractive
        if apt-get update -qq && apt-get install -y -qq build-essential git 2>&1 | tail -5; then
            print_success "Build-Tools installiert"
        else
            print_error "Build-Tools Installation fehlgeschlagen"
            return 1
        fi
    elif command -v dnf &> /dev/null; then
        # Fedora/RHEL 8+
        if dnf install -y -q gcc gcc-c++ make git 2>&1 | tail -5; then
            print_success "Build-Tools installiert"
        else
            print_error "Build-Tools Installation fehlgeschlagen"
            return 1
        fi
    elif command -v yum &> /dev/null; then
        # RHEL/CentOS
        if yum install -y -q gcc gcc-c++ make git 2>&1 | tail -5; then
            print_success "Build-Tools installiert"
        else
            print_error "Build-Tools Installation fehlgeschlagen"
            return 1
        fi
    else
        print_warning "Konnte Build-Tools nicht automatisch installieren"
        print_warning "Bitte installiere manuell: gcc, g++, make, git"
        return 1
    fi
    
    return 0
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

install_nodejs() {
    print_step "Prüfe Node.js Installation..."
    
    if command -v node &> /dev/null; then
        local node_version=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$node_version" -ge 18 ]; then
            print_success "Node.js $(node -v) bereits installiert"
            return
        fi
    fi
    
    print_step "Installiere Node.js 20 LTS..."
    
    if command -v apt-get &> /dev/null; then
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
        apt-get install -y nodejs
    elif command -v yum &> /dev/null; then
        curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -
        yum install -y nodejs
    else
        print_error "Paketmanager nicht unterstützt. Bitte Node.js 18+ manuell installieren."
        exit 1
    fi
    
    print_success "Node.js installiert: $(node -v)"
}

install_postgresql() {
    print_step "Prüfe PostgreSQL Installation..."
    
    if command -v psql &> /dev/null; then
        print_success "PostgreSQL bereits installiert"
        # Prüfe ob Service läuft
        if systemctl is-active --quiet postgresql; then
            print_success "PostgreSQL Service läuft"
        else
            print_step "Starte PostgreSQL Service..."
            systemctl start postgresql 2>/dev/null || true
            systemctl enable postgresql 2>/dev/null || true
        fi
        return
    fi
    
    print_step "PostgreSQL wird automatisch installiert..."
    
    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        print_step "Installiere PostgreSQL via apt-get..."
        export DEBIAN_FRONTEND=noninteractive
        
        if ! apt-get update -qq 2>/dev/null; then
            print_warning "apt-get update fehlgeschlagen, versuche trotzdem Installation..."
        fi
        
        if apt-get install -y -qq postgresql postgresql-contrib 2>/dev/null; then
            systemctl enable postgresql 2>/dev/null || true
            systemctl start postgresql 2>/dev/null || true
            sleep 3
            print_success "PostgreSQL installiert und gestartet"
        else
            print_error "PostgreSQL Installation fehlgeschlagen"
            print_warning "Verwende SQLite als Fallback"
            USE_SQLITE=true
            return
        fi
        
    elif command -v dnf &> /dev/null; then
        # Fedora/RHEL 8+
        print_step "Installiere PostgreSQL via dnf..."
        
        if dnf install -y postgresql-server postgresql-contrib 2>/dev/null; then
            # Initialize DB if needed
            if [ ! -d "/var/lib/pgsql/data/base" ]; then
                postgresql-setup --initdb 2>/dev/null || /usr/bin/postgresql-setup initdb 2>/dev/null || true
            fi
            
            systemctl enable postgresql 2>/dev/null || true
            systemctl start postgresql 2>/dev/null || true
            sleep 3
            print_success "PostgreSQL installiert und gestartet"
        else
            print_error "PostgreSQL Installation fehlgeschlagen"
            print_warning "Verwende SQLite als Fallback"
            USE_SQLITE=true
            return
        fi
        
    elif command -v yum &> /dev/null; then
        # RHEL/CentOS
        print_step "Installiere PostgreSQL via yum..."
        
        if yum install -y postgresql-server postgresql-contrib 2>/dev/null; then
            # Initialize DB if needed
            if [ ! -d "/var/lib/pgsql/data/base" ]; then
                postgresql-setup --initdb 2>/dev/null || /usr/bin/postgresql-setup initdb 2>/dev/null || true
            fi
            
            systemctl enable postgresql 2>/dev/null || true
            systemctl start postgresql 2>/dev/null || true
            sleep 3
            print_success "PostgreSQL installiert und gestartet"
        else
            print_error "PostgreSQL Installation fehlgeschlagen"
            print_warning "Verwende SQLite als Fallback"
            USE_SQLITE=true
            return
        fi
        
    else
        print_warning "Paketmanager nicht unterstützt. Verwende SQLite als Fallback."
        USE_SQLITE=true
        return
    fi
    
    # Verifiziere Installation
    if ! systemctl is-active --quiet postgresql 2>/dev/null; then
        print_warning "PostgreSQL Service läuft nicht, verwende SQLite als Fallback"
        USE_SQLITE=true
    fi
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
    
    mkdir -p "$INSTALL_DIR"/{backend,frontend}
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    
    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$LOG_DIR"
    
    print_success "Verzeichnisse erstellt"
}

download_release() {
    print_step "Installiere CSF-Core..."
    
    # Detect branch from script URL or use environment variable
    BRANCH="${BRANCH:-main}"
    
    # For main branch (production): ONLY use releases, never build from source
    if [ "$BRANCH" = "main" ] && [ -z "$BUILD_FROM_SOURCE" ]; then
        print_step "Production Installation - verwende nur Pre-Built Releases"
        
        local temp_dir=$(mktemp -d)
        cd "$temp_dir"
        
        # Download latest release or specific version
        if [ "$VERSION" = "latest" ]; then
            local download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/csf-core-linux-${ARCH}.tar.gz"
        else
            local download_url="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/csf-core-linux-${ARCH}.tar.gz"
        fi
        
        print_step "Download Release von: $download_url"
        
        if curl -L -f "$download_url" -o csf-core.tar.gz 2>/dev/null; then
            tar -xzf csf-core.tar.gz
            
            # Copy files to installation directory
            cp -r backend/* "$INSTALL_DIR/backend/"
            cp -r frontend/* "$INSTALL_DIR/frontend/"
            
            chmod +x "$INSTALL_DIR/backend/backend"
            
            cd - > /dev/null
            rm -rf "$temp_dir"
            
            print_success "Release heruntergeladen und installiert"
        else
            cd - > /dev/null
            rm -rf "$temp_dir"
            
            print_error "Kein Release gefunden für main Branch"
            print_error "Bitte warte bis GitHub Actions ein Release gebaut hat"
            print_error "Oder verwende: BUILD_FROM_SOURCE=1 bash install.sh"
            exit 1
        fi
    else
        # For development branches: Try release first, then build from source
        print_step "Development Installation - versuche Release, baue sonst aus Quellcode"
        
        local temp_dir=$(mktemp -d)
        cd "$temp_dir"
        
        # Download latest release or specific version
        if [ "$VERSION" = "latest" ]; then
            local download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/csf-core-linux-${ARCH}.tar.gz"
        else
            local download_url="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/csf-core-linux-${ARCH}.tar.gz"
        fi
        
        print_step "Versuche Release Download von: $download_url"
        
        if curl -L -f "$download_url" -o csf-core.tar.gz 2>/dev/null; then
            tar -xzf csf-core.tar.gz
            
            # Copy files to installation directory
            cp -r backend/* "$INSTALL_DIR/backend/"
            cp -r frontend/* "$INSTALL_DIR/frontend/"
            
            chmod +x "$INSTALL_DIR/backend/backend"
            
            cd - > /dev/null
            rm -rf "$temp_dir"
            
            print_success "Release heruntergeladen und installiert"
        else
            cd - > /dev/null
            rm -rf "$temp_dir"
            
            print_warning "Kein Release gefunden, baue aus Quellcode..."
            build_from_source
        fi
    fi
}

install_rust() {
    print_step "Installiere Rust/Cargo..."
    
    # Install rustup (official Rust installer)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path 2>&1 | tail -10
    
    # Add cargo to PATH for current session
    export PATH="$HOME/.cargo/bin:$PATH"
    
    # Verify installation
    if command -v cargo &> /dev/null; then
        print_success "Rust/Cargo installiert ($(cargo --version))"
        return 0
    else
        print_error "Rust Installation fehlgeschlagen"
        return 1
    fi
}

build_from_source() {
    print_step "Baue CSF-Core aus Quellcode..."
    
    # Install build tools (gcc, make, git) if missing
    if ! command -v gcc &> /dev/null || ! command -v make &> /dev/null; then
        print_warning "Build-Tools fehlen, installiere..."
        if ! install_build_tools; then
            print_error "Kann Build-Tools nicht installieren"
            install_via_docker
            return
        fi
    fi
    
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Clone repository
    BRANCH="${BRANCH:-feat/docker-managment}"
    print_step "Clone Repository (Branch: $BRANCH)..."
    
    if ! git clone -b "$BRANCH" --depth 1 "https://github.com/${GITHUB_REPO}.git" csf-core 2>/dev/null; then
        print_error "Git clone fehlgeschlagen"
        cd - > /dev/null
        rm -rf "$temp_dir"
        install_via_docker
        return
    fi
    
    cd csf-core
    
    # Check for Cargo, install if missing
    if ! command -v cargo &> /dev/null; then
        print_warning "Rust/Cargo nicht gefunden, installiere..."
        if ! install_rust; then
            cd "$temp_dir"
            rm -rf "$temp_dir"
            install_via_docker
            return
        fi
    fi
    
    # Build Backend
    print_step "Baue Backend (kann mehrere Minuten dauern)..."
    cd backend
    
    # Build with cargo
    if cargo build --release 2>&1; then
        if [ -f "target/release/backend" ]; then
            mkdir -p "$INSTALL_DIR/backend"
            cp target/release/backend "$INSTALL_DIR/backend/"
            chmod +x "$INSTALL_DIR/backend/backend"
            print_success "Backend gebaut"
        else
            print_error "Backend Binary nicht gefunden"
            cd "$temp_dir"
            rm -rf "$temp_dir"
            install_via_docker
            return
        fi
    else
        print_error "Backend build fehlgeschlagen"
        cd "$temp_dir"
        rm -rf "$temp_dir"
        install_via_docker
        return
    fi
    cd ..
    
    # Build Frontend
    print_step "Baue Frontend (kann mehrere Minuten dauern)..."
    cd frontend
    
    # Check Node.js version
    local node_version=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$node_version" -lt 20 ]; then
        print_error "Node.js Version zu alt: $(node -v). Benötigt: >= 20"
        cd - > /dev/null
        rm -rf "$temp_dir"
        install_via_docker
        return
    fi
    
    print_step "Node.js $(node -v) gefunden"
    
    # Create or copy .env file for build (required by SvelteKit)
    if [ -n "$ENV_FILE" ] && [ -f "$ENV_FILE" ]; then
        print_step "Kopiere .env Datei von: $ENV_FILE"
        cp "$ENV_FILE" .env
        print_success ".env kopiert von $ENV_FILE"
    else
        print_step "Erstelle Frontend .env Datei..."
        cat > .env << EOF
PUBLIC_API_BASE_URL=${PUBLIC_API_BASE_URL}
EOF
        print_success ".env erstellt mit PUBLIC_API_BASE_URL=${PUBLIC_API_BASE_URL}"
    fi
    
    # Clean install with retries
    print_step "Installiere Frontend Dependencies..."
    rm -rf node_modules package-lock.json 2>/dev/null || true
    
    if ! npm install --legacy-peer-deps 2>&1 | tail -15; then
        print_error "npm install fehlgeschlagen"
        cd - > /dev/null
        rm -rf "$temp_dir"
        install_via_docker
        return
    fi
    
    # Build frontend - show more output for debugging
    print_step "Baue Frontend (npm run build)..."
    if ! npm run build 2>&1 | tail -30; then
        print_error "Frontend build fehlgeschlagen"
        print_error "Siehe Fehler oben für Details"
        cd - > /dev/null
        rm -rf "$temp_dir"
        install_via_docker
        return
    fi
    
    # Copy built files
    if [ -d "build" ]; then
        mkdir -p "$INSTALL_DIR/frontend"
        cp -r build "$INSTALL_DIR/frontend/"
        cp -r node_modules "$INSTALL_DIR/frontend/"
        cp package.json "$INSTALL_DIR/frontend/"
        print_success "Frontend gebaut"
    else
        print_error "Frontend build Verzeichnis nicht gefunden"
        cd - > /dev/null
        rm -rf "$temp_dir"
        install_via_docker
        return
    fi
    
    cd - > /dev/null
    rm -rf "$temp_dir"
    
    print_success "CSF-Core aus Quellcode gebaut und installiert"
}

install_via_docker() {
    print_warning "Verwende Docker-basierte Installation als Fallback"
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker nicht gefunden."
        print_error ""
        print_error "Bitte installiere eine der folgenden Abhängigkeiten:"
        print_error "  1. Docker: curl -fsSL https://get.docker.com | sh"
        print_error "  2. Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        print_error "  3. Oder verwende ein vorhandenes Release"
        exit 1
    fi
    
    print_step "Lade Docker Image..."
    
    # Pull image and copy binaries
    if ! docker pull ghcr.io/cs-foundry/csf-core:latest 2>/dev/null; then
        print_error "Docker Image konnte nicht geladen werden"
        print_error "Bitte baue manuell oder warte auf ein Release"
        exit 1
    fi
    
    # Create temporary container and copy files
    local container_id=$(docker create ghcr.io/cs-foundry/csf-core:latest)
    docker cp "$container_id:/app/backend" "$INSTALL_DIR/"
    docker cp "$container_id:/app/frontend" "$INSTALL_DIR/"
    docker rm "$container_id"
    
    chmod +x "$INSTALL_DIR/backend/backend"
    
    print_success "Installation via Docker abgeschlossen"
}

setup_database() {
    print_step "Konfiguriere Datenbank..."
    
    # Check if we should use SQLite (set by install_postgresql if it failed)
    if [ "$USE_SQLITE" = "true" ]; then
        print_step "Verwende SQLite als Datenbank..."
        DATABASE_URL="sqlite:$DATA_DIR/csf-core.db"
        print_success "SQLite Datenbank konfiguriert: $DATA_DIR/csf-core.db"
        return
    fi
    
    # Generate random password
    local db_password=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
    
    # Try to create database and user with PostgreSQL
    if command -v psql &> /dev/null && systemctl is-active --quiet postgresql; then
        print_step "Erstelle PostgreSQL Datenbank und Benutzer..."
        
        # Drop existing database/user if exists (for re-installation)
        sudo -u postgres psql -c "DROP DATABASE IF EXISTS csf_core;" 2>/dev/null || true
        sudo -u postgres psql -c "DROP USER IF EXISTS csf_core;" 2>/dev/null || true
        
        # Create new database and user
        if sudo -u postgres psql <<EOF
CREATE USER csf_core WITH PASSWORD '$db_password';
CREATE DATABASE csf_core OWNER csf_core;
GRANT ALL PRIVILEGES ON DATABASE csf_core TO csf_core;
EOF
        then
            print_success "PostgreSQL Datenbank 'csf_core' erstellt"
            DATABASE_URL="postgres://csf_core:$db_password@localhost/csf_core"
            
            # Test connection
            if PGPASSWORD="$db_password" psql -U csf_core -d csf_core -h localhost -c "SELECT 1;" &>/dev/null; then
                print_success "Datenbankverbindung erfolgreich getestet"
            else
                print_warning "Datenbankverbindung konnte nicht getestet werden, aber Datenbank wurde erstellt"
            fi
        else
            print_error "PostgreSQL Datenbank konnte nicht erstellt werden"
            print_warning "Verwende SQLite als Fallback"
            DATABASE_URL="sqlite:$DATA_DIR/csf-core.db"
        fi
    else
        print_warning "PostgreSQL nicht verfügbar, verwende SQLite"
        DATABASE_URL="sqlite:$DATA_DIR/csf-core.db"
    fi
    
    print_success "Datenbank konfiguriert"
}

generate_jwt_secret() {
    JWT_SECRET=$(openssl rand -hex 32)
}

install_systemd_service() {
    print_step "Installiere systemd Service..."
    
    cat > /etc/systemd/system/${SERVICE_NAME}.service <<EOF
[Unit]
Description=CSF-Core Backend and Frontend Service
Documentation=https://github.com/${GITHUB_REPO}
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${INSTALL_DIR}

# Load environment from config file (if exists)
EnvironmentFile=-${INSTALL_DIR}/config.env

# Environment variables
Environment="DATABASE_URL=${DATABASE_URL}"
Environment="JWT_SECRET=${JWT_SECRET}"
Environment="RUST_LOG=info"
Environment="NODE_ENV=production"
Environment="PORT=3000"
Environment="FRONTEND_URL=http://localhost:3000"
Environment="ORIGIN=http://localhost:8000"

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${INSTALL_DIR}
ReadWritePaths=${DATA_DIR}
ReadWritePaths=${LOG_DIR}

# Use startup script
ExecStart=${INSTALL_DIR}/start.sh

# Process management
Restart=on-failure
RestartSec=10
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=30
TimeoutStartSec=60

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Logging
StandardOutput=append:${LOG_DIR}/csf-core.log
StandardError=append:${LOG_DIR}/csf-core-error.log

[Install]
WantedBy=multi-user.target
EOF

    chmod 644 /etc/systemd/system/${SERVICE_NAME}.service
    
    print_success "systemd Service installiert"
}

create_startup_script() {
    print_step "Erstelle Startup-Script..."
    
    # Ensure directory exists
    mkdir -p ${INSTALL_DIR}
    
    cat > ${INSTALL_DIR}/start.sh <<'EOFSCRIPT'
#!/bin/bash
# CSF-Core Unified Startup Script

set -e

SCRIPT_DIR="/opt/csf-core"
FRONTEND_DIR="${SCRIPT_DIR}/frontend"
BACKEND_BIN="${SCRIPT_DIR}/backend/backend"
LOG_DIR="/var/log/csf-core"

# Erstelle Log-Verzeichnis falls nicht vorhanden
mkdir -p "$LOG_DIR" 2>/dev/null || true

echo "🚀 Starting CSF-Core..."

# Start Frontend in background if it exists
if [ -d "$FRONTEND_DIR" ] && [ -f "$FRONTEND_DIR/package.json" ]; then
    echo "▶️  Starting Frontend (Port ${PORT:-3000})..."
    cd "$FRONTEND_DIR"
    PORT=${PORT:-3000} node build/index.js > "$LOG_DIR/frontend.log" 2>&1 &
    FRONTEND_PID=$!
    echo "   Frontend PID: $FRONTEND_PID"
    
    # Wait for frontend to be ready
    echo "⏳ Waiting for frontend to start..."
    for i in {1..30}; do
        if curl -s http://localhost:${PORT:-3000} > /dev/null 2>&1; then
            echo "✅ Frontend ready!"
            break
        fi
        if [ $i -eq 30 ]; then
            echo "⚠️  Frontend did not start in time, continuing anyway..."
        fi
        sleep 1
    done
else
    echo "⚠️  Frontend directory not found, skipping..."
fi

# Start Backend
echo "▶️  Starting Backend (Port 8000)..."
cd "$SCRIPT_DIR"

if [ -f "$BACKEND_BIN" ]; then
    exec "$BACKEND_BIN"
else
    echo "❌ Backend binary not found: $BACKEND_BIN"
    exit 1
fi
EOFSCRIPT

    chmod +x ${INSTALL_DIR}/start.sh
    chown ${SERVICE_USER}:${SERVICE_USER} ${INSTALL_DIR}/start.sh
    
    print_success "Startup-Script erstellt"
}

reload_systemd() {
    print_step "Lade systemd neu..."
    systemctl daemon-reload
    print_success "systemd neu geladen"
}

create_config_file() {
    print_step "Erstelle Konfigurationsdatei..."
    
    # Auto-detect externe IP für ORIGIN falls nicht gesetzt
    if [ -z "$ORIGIN" ]; then
        local external_ip=$(hostname -I 2>/dev/null | awk '{print $1}')
        if [ -n "$external_ip" ]; then
            ORIGIN="http://${external_ip}:8000,http://localhost:8000"
        else
            ORIGIN="http://localhost:8000"
        fi
    fi
    
    cat > ${INSTALL_DIR}/config.env <<EOF
# CSF-Core Configuration
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
RUST_LOG=info
NODE_ENV=production

# Frontend läuft auf Port 3000 (intern)
PORT=3000
FRONTEND_URL=${FRONTEND_URL}

# CORS Origins - erlaubte URLs für API-Zugriff
# Mehrere Origins mit Komma trennen
ORIGIN=${ORIGIN}

# Backend Port (extern erreichbar)
BACKEND_PORT=8000

# You can edit these values and restart the service:
# sudo systemctl restart csf-core
EOF

    chmod 600 ${INSTALL_DIR}/config.env
    chown ${SERVICE_USER}:${SERVICE_USER} ${INSTALL_DIR}/config.env
    
    print_success "Konfiguration gespeichert in: ${INSTALL_DIR}/config.env"
}

open_firewall_ports() {
    print_step "Öffne Firewall-Ports für externen Zugriff..."
    
    # Port 8000 für Backend/Frontend
    if command -v ufw &> /dev/null; then
        print_step "Verwende ufw (Ubuntu/Debian)..."
        ufw allow 8000/tcp 2>/dev/null || true
        ufw reload 2>/dev/null || true
        print_success "Port 8000 in ufw geöffnet"
    elif command -v firewall-cmd &> /dev/null; then
        print_step "Verwende firewalld (RHEL/CentOS)..."
        firewall-cmd --permanent --add-port=8000/tcp 2>/dev/null || true
        firewall-cmd --reload 2>/dev/null || true
        print_success "Port 8000 in firewalld geöffnet"
    else
        print_warning "Keine Firewall (ufw/firewalld) gefunden"
        print_warning "Falls iptables verwendet wird:"
        print_warning "  sudo iptables -A INPUT -p tcp --dport 8000 -j ACCEPT"
    fi
    
    return 0
}

print_next_steps() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         Installation erfolgreich abgeschlossen!       ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Show database info
    echo -e "${GREEN}Installierte Komponenten:${NC}"
    echo "  • Backend: Rust/Axum"
    echo "  • Frontend: SvelteKit/Node.js $(node -v)"
    if [[ $DATABASE_URL == postgres* ]]; then
        echo "  • Datenbank: PostgreSQL"
    else
        echo "  • Datenbank: SQLite"
    fi
    echo ""
    
    echo -e "${GREEN}Nächste Schritte:${NC}"
    echo ""
    echo "1. Service aktivieren (Auto-Start beim Boot):"
    echo -e "   ${YELLOW}sudo systemctl enable ${SERVICE_NAME}${NC}"
    echo ""
    echo "2. Service starten:"
    echo -e "   ${YELLOW}sudo systemctl start ${SERVICE_NAME}${NC}"
    echo ""
    echo "3. Status überprüfen:"
    echo -e "   ${YELLOW}sudo systemctl status ${SERVICE_NAME}${NC}"
    echo ""
    echo "4. Logs ansehen:"
    echo -e "   ${YELLOW}sudo journalctl -u ${SERVICE_NAME} -f${NC}"
    echo ""
    echo "5. Konfiguration anpassen (optional):"
    echo -e "   ${YELLOW}sudo nano ${INSTALL_DIR}/config.env${NC}"
    echo -e "   ${YELLOW}sudo systemctl restart ${SERVICE_NAME}${NC}"
    echo ""
    echo -e "${GREEN}Zugriff:${NC}"
    echo "  • Lokal: http://localhost:8000"
    
    # Externe IP ermitteln
    local external_ip=$(hostname -I 2>/dev/null | awk '{print $1}')
    if [ -n "$external_ip" ]; then
        echo "  • Extern: http://$external_ip:8000"
        echo ""
        echo "  ${YELLOW}Wichtig:${NC} Port 8000 muss in der Firewall geöffnet sein!"
        echo "  Falls nicht erreichbar, prüfe: sudo ufw status"
    fi
    
    echo ""
    echo "  • API Docs: http://localhost:8000/swagger-ui"
    echo "  • API: http://localhost:8000/api"
    echo ""
    echo -e "${GREEN}Deinstallation:${NC}"
    echo -e "   ${YELLOW}curl -sSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/scripts/uninstall.sh | sudo bash${NC}"
    echo ""
}

# Main Installation
main() {
    print_header
    
    echo -e "${BLUE}[DEBUG] Starting installation...${NC}"
    
    check_root || { print_error "Root check failed"; exit 1; }
    check_dependencies || { print_error "Dependency check failed"; exit 1; }
    detect_architecture || { print_error "Architecture detection failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Installing Node.js...${NC}"
    install_nodejs || { print_warning "Node.js installation had issues, continuing..."; }
    
    echo -e "${BLUE}[DEBUG] Installing PostgreSQL...${NC}"
    install_postgresql || { print_warning "PostgreSQL installation had issues, will use SQLite..."; USE_SQLITE=true; }
    
    echo -e "${BLUE}[DEBUG] Creating service user...${NC}"
    create_service_user || { print_error "Service user creation failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Creating directories...${NC}"
    create_directories || { print_error "Directory creation failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Downloading release...${NC}"
    download_release || { print_error "Download failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Setting up database...${NC}"
    setup_database || { print_warning "Database setup had issues, continuing..."; }
    
    echo -e "${BLUE}[DEBUG] Generating JWT secret...${NC}"
    generate_jwt_secret || { print_error "JWT generation failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Installing systemd service...${NC}"
    install_systemd_service || { print_error "Systemd service installation failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Creating startup script...${NC}"
    create_startup_script || { print_error "Startup script creation failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Reloading systemd...${NC}"
    reload_systemd || { print_error "Systemd reload failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Creating config file...${NC}"
    create_config_file || { print_error "Config file creation failed"; exit 1; }
    
    echo -e "${BLUE}[DEBUG] Opening firewall ports...${NC}"
    open_firewall_ports || { print_warning "Firewall configuration had issues, continuing..."; }
    
    echo -e "${BLUE}[DEBUG] Installation complete!${NC}"
    print_next_steps
}

main "$@"
