#!/bin/bash
# CSF-Core Installation Script
# Installiert Backend + Frontend als systemd Service

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
VERSION="${VERSION:-latest}"

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
            systemctl start postgresql
            systemctl enable postgresql
        fi
        return
    fi
    
    print_step "PostgreSQL wird automatisch installiert..."
    
    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
        apt-get install -y -qq postgresql postgresql-contrib
        systemctl enable postgresql
        systemctl start postgresql
        
        # Warte bis PostgreSQL bereit ist
        sleep 3
        
        print_success "PostgreSQL installiert und gestartet"
    elif command -v yum &> /dev/null; then
        # RHEL/CentOS/Fedora
        yum install -y postgresql-server postgresql-contrib
        
        # Initialize DB if needed
        if [ ! -d "/var/lib/pgsql/data/base" ]; then
            postgresql-setup --initdb || /usr/bin/postgresql-setup initdb
        fi
        
        systemctl enable postgresql
        systemctl start postgresql
        
        # Warte bis PostgreSQL bereit ist
        sleep 3
        
        print_success "PostgreSQL installiert und gestartet"
    elif command -v dnf &> /dev/null; then
        # Fedora/RHEL 8+
        dnf install -y postgresql-server postgresql-contrib
        
        # Initialize DB if needed
        if [ ! -d "/var/lib/pgsql/data/base" ]; then
            postgresql-setup --initdb || /usr/bin/postgresql-setup initdb
        fi
        
        systemctl enable postgresql
        systemctl start postgresql
        
        # Warte bis PostgreSQL bereit ist
        sleep 3
        
        print_success "PostgreSQL installiert und gestartet"
    else
        print_warning "Paketmanager nicht unterstützt. Verwende SQLite als Fallback."
        USE_SQLITE=true
        return
    fi
    
    # Verifiziere Installation
    if ! systemctl is-active --quiet postgresql; then
        print_error "PostgreSQL konnte nicht gestartet werden"
        print_warning "Verwende SQLite als Fallback"
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
    print_step "Lade CSF-Core Release herunter..."
    
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Download latest release or specific version
    if [ "$VERSION" = "latest" ]; then
        local download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/csf-core-linux-${ARCH}.tar.gz"
    else
        local download_url="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/csf-core-linux-${ARCH}.tar.gz"
    fi
    
    print_step "Download von: $download_url"
    
    if ! curl -L -f "$download_url" -o csf-core.tar.gz; then
        print_error "Download fehlgeschlagen. Versuche Docker-basierte Installation..."
        install_via_docker
        return
    fi
    
    tar -xzf csf-core.tar.gz
    
    # Copy files to installation directory
    cp -r backend/* "$INSTALL_DIR/backend/"
    cp -r frontend/* "$INSTALL_DIR/frontend/"
    
    chmod +x "$INSTALL_DIR/backend/backend"
    
    cd - > /dev/null
    rm -rf "$temp_dir"
    
    print_success "Release heruntergeladen und extrahiert"
}

install_via_docker() {
    print_warning "Verwende Docker-basierte Installation als Fallback"
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker nicht gefunden. Bitte Docker installieren oder manuell bauen."
        exit 1
    fi
    
    # Pull image and copy binaries
    docker pull ghcr.io/cs-foundry/csf-core:latest
    
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

# Start script
ExecStartPre=/bin/bash -c 'cd ${INSTALL_DIR}/frontend && node build/index.js &'
ExecStart=${INSTALL_DIR}/backend/backend

# Process management
Restart=on-failure
RestartSec=10
KillMode=mixed
KillSignal=SIGTERM
TimeoutStopSec=30

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
    
    cat > ${INSTALL_DIR}/start.sh <<'EOF'
#!/bin/bash
# CSF-Core Unified Startup Script

# Start Frontend in background
cd /opt/csf-core/frontend
PORT=3000 node build/index.js > /var/log/csf-core/frontend.log 2>&1 &
FRONTEND_PID=$!

# Wait for frontend to be ready
echo "Waiting for frontend to start..."
for i in {1..30}; do
    if curl -s http://localhost:3000 > /dev/null; then
        echo "Frontend ready!"
        break
    fi
    sleep 1
done

# Start Backend (with frontend proxy)
cd /opt/csf-core
exec ./backend/backend
EOF

    chmod +x ${INSTALL_DIR}/start.sh
    chown ${SERVICE_USER}:${SERVICE_USER} ${INSTALL_DIR}/start.sh
    
    # Update service to use startup script
    sed -i "s|ExecStart=.*|ExecStart=${INSTALL_DIR}/start.sh|" /etc/systemd/system/${SERVICE_NAME}.service
    
    print_success "Startup-Script erstellt"
}

reload_systemd() {
    print_step "Lade systemd neu..."
    systemctl daemon-reload
    print_success "systemd neu geladen"
}

create_config_file() {
    print_step "Erstelle Konfigurationsdatei..."
    
    cat > ${INSTALL_DIR}/config.env <<EOF
# CSF-Core Configuration
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
RUST_LOG=info
NODE_ENV=production
PORT=3000
FRONTEND_URL=http://localhost:3000
ORIGIN=http://localhost:8000

# You can edit these values and restart the service:
# sudo systemctl restart csf-core
EOF

    chmod 600 ${INSTALL_DIR}/config.env
    chown ${SERVICE_USER}:${SERVICE_USER} ${INSTALL_DIR}/config.env
    
    print_success "Konfiguration gespeichert in: ${INSTALL_DIR}/config.env"
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
    echo "  • Web Interface: http://localhost:8000"
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
    check_root
    check_dependencies
    detect_architecture
    install_nodejs
    install_postgresql
    create_service_user
    create_directories
    download_release
    setup_database
    generate_jwt_secret
    install_systemd_service
    create_startup_script
    reload_systemd
    create_config_file
    print_next_steps
}

main "$@"
