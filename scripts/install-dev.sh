#!/bin/bash
# CSF-Core Development Installation Script
# Kompiliert Backend + Frontend direkt auf dem Server (nicht in CI/CD Pipelines)
# Für Testing und Entwicklung auf Remote-Servern

# Fehlerbehandlung
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

# Environment-Variablen
DATABASE_URL="${DATABASE_URL:-}"
PUBLIC_API_BASE_URL="${PUBLIC_API_BASE_URL:-/api}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:3000}"

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║    CSF-Core Development Installation Script          ║${NC}"
    echo -e "${BLUE}║    Build from Source on Server (for Testing)         ║${NC}"
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
    print_success "Root-Rechte vorhanden"
}

install_dependencies() {
    print_step "Installiere System-Abhängigkeiten..."
    
    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        print_step "Verwende apt-get (Debian/Ubuntu)..."
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
        apt-get install -y -qq \
            build-essential \
            git \
            curl \
            wget \
            pkg-config \
            libssl-dev \
            postgresql \
            postgresql-contrib
        
        systemctl enable postgresql 2>/dev/null || true
        systemctl start postgresql 2>/dev/null || true
        
        print_success "Abhängigkeiten installiert (apt-get)"
        
    elif command -v dnf &> /dev/null; then
        # Fedora/RHEL 8+
        print_step "Verwende dnf (Fedora/RHEL 8+)..."
        dnf install -y \
            gcc \
            gcc-c++ \
            make \
            git \
            curl \
            wget \
            pkg-config \
            openssl-devel \
            postgresql-server \
            postgresql-contrib
        
        # Initialize PostgreSQL if needed
        if [ ! -d "/var/lib/pgsql/data/base" ]; then
            postgresql-setup --initdb 2>/dev/null || /usr/bin/postgresql-setup initdb 2>/dev/null || true
        fi
        
        systemctl enable postgresql 2>/dev/null || true
        systemctl start postgresql 2>/dev/null || true
        
        print_success "Abhängigkeiten installiert (dnf)"
        
    elif command -v yum &> /dev/null; then
        # RHEL/CentOS
        print_step "Verwende yum (RHEL/CentOS)..."
        yum install -y \
            gcc \
            gcc-c++ \
            make \
            git \
            curl \
            wget \
            pkg-config \
            openssl-devel \
            postgresql-server \
            postgresql-contrib
        
        # Initialize PostgreSQL if needed
        if [ ! -d "/var/lib/pgsql/data/base" ]; then
            postgresql-setup --initdb 2>/dev/null || /usr/bin/postgresql-setup initdb 2>/dev/null || true
        fi
        
        systemctl enable postgresql 2>/dev/null || true
        systemctl start postgresql 2>/dev/null || true
        
        print_success "Abhängigkeiten installiert (yum)"
        
    else
        print_error "Nicht unterstützter Paketmanager"
        exit 1
    fi
}

install_rust() {
    print_step "Installiere Rust..."
    
    if command -v cargo &> /dev/null; then
        print_success "Rust bereits installiert: $(cargo --version)"
        return 0
    fi
    
    # Install rustup as the service user (so they own the toolchain)
    print_step "Installiere Rust für System..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    
    # Add to PATH
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if command -v cargo &> /dev/null; then
        print_success "Rust installiert: $(cargo --version)"
    else
        print_error "Rust Installation fehlgeschlagen"
        exit 1
    fi
}

install_nodejs() {
    print_step "Installiere Node.js..."
    
    if command -v node &> /dev/null; then
        local node_version=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$node_version" -ge 18 ]; then
            print_success "Node.js bereits installiert: $(node --version)"
            return 0
        fi
    fi
    
    # Install Node.js 20.x via NodeSource
    print_step "Installiere Node.js 20.x via NodeSource..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - 2>&1 | tail -5 || {
        # If NodeSource fails, try alternative methods
        if command -v apt-get &> /dev/null; then
            apt-get install -y nodejs npm
        elif command -v dnf &> /dev/null; then
            dnf install -y nodejs npm
        elif command -v yum &> /dev/null; then
            yum install -y nodejs npm
        fi
    }
    
    if command -v node &> /dev/null; then
        print_success "Node.js installiert: $(node --version)"
    else
        print_error "Node.js Installation fehlgeschlagen"
        exit 1
    fi
}

install_docker() {
    print_step "Prüfe Docker Installation..."
    
    if command -v docker &> /dev/null; then
        print_success "Docker bereits installiert: $(docker --version)"
    else
        print_warning "Docker nicht installiert"
        print_step "Installiere Docker..."
        
        if curl -fsSL https://get.docker.com | sh; then
            systemctl enable docker
            systemctl start docker
            print_success "Docker installiert"
        else
            print_warning "Docker Installation fehlgeschlagen. Container-Management nicht verfügbar."
        fi
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
    
    # Add user to docker group if Docker is installed
    if command -v docker &> /dev/null; then
        if getent group docker > /dev/null 2>&1; then
            print_step "Füge $SERVICE_USER zur docker-Gruppe hinzu..."
            usermod -aG docker "$SERVICE_USER" 2>/dev/null || true
            print_success "$SERVICE_USER zur docker-Gruppe hinzugefügt"
            
            # Set docker socket permissions
            if [ -e /var/run/docker.sock ]; then
                chgrp docker /var/run/docker.sock 2>/dev/null || true
                chmod 660 /var/run/docker.sock 2>/dev/null || true
                print_success "Docker-Socket Berechtigungen gesetzt"
            fi
        fi
    fi
}

create_directories() {
    print_step "Erstelle Verzeichnisse..."
    
    mkdir -p "$INSTALL_DIR"/{backend,frontend}
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    
    print_success "Verzeichnisse erstellt"
}

clone_repository() {
    print_step "Clone Repository (Branch: $BRANCH)..."
    
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    if git clone --depth 1 --branch "$BRANCH" "https://github.com/${GITHUB_REPO}.git" csf-core; then
        print_success "Repository geclont"
    else
        print_error "Repository konnte nicht geclont werden"
        exit 1
    fi
    
    cd csf-core
    echo "$temp_dir/csf-core"
}

build_backend() {
    local repo_dir=$1
    print_step "Baue Backend (Rust)..."
    
    cd "$repo_dir/backend"
    
    # Export PATH for cargo
    export PATH="$HOME/.cargo/bin:$PATH"
    
    print_step "Kompiliere Backend im Release-Modus..."
    if cargo build --release; then
        cp target/release/backend "$INSTALL_DIR/backend/"
        chmod +x "$INSTALL_DIR/backend/backend"
        print_success "Backend kompiliert und installiert"
    else
        print_error "Backend Build fehlgeschlagen"
        exit 1
    fi
}

build_frontend() {
    local repo_dir=$1
    print_step "Baue Frontend (SvelteKit)..."
    
    cd "$repo_dir/frontend"
    
    # Create .env file for build
    cat > .env <<EOF
PUBLIC_API_BASE_URL=${PUBLIC_API_BASE_URL}
EOF
    
    print_step "Installiere Frontend Dependencies..."
    if npm ci --production=false; then
        print_success "Frontend Dependencies installiert"
    else
        print_warning "npm ci fehlgeschlagen, versuche npm install..."
        npm install
    fi
    
    print_step "Baue Frontend..."
    if npm run build; then
        print_success "Frontend gebaut"
    else
        print_error "Frontend Build fehlgeschlagen"
        exit 1
    fi
    
    # Copy build and production dependencies
    mkdir -p "$INSTALL_DIR/frontend"
    cp -r build "$INSTALL_DIR/frontend/"
    cp -r node_modules "$INSTALL_DIR/frontend/"
    cp package.json "$INSTALL_DIR/frontend/"
    
    print_success "Frontend installiert"
}

setup_database() {
    print_step "Konfiguriere Datenbank..."
    
    # Generate random password
    local db_password=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
    
    if command -v psql &> /dev/null && systemctl is-active --quiet postgresql; then
        print_step "Erstelle PostgreSQL Datenbank..."
        
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
        else
            print_warning "PostgreSQL Datenbank konnte nicht erstellt werden, verwende SQLite"
            DATABASE_URL="sqlite:$DATA_DIR/csf-core.db"
        fi
    else
        print_warning "PostgreSQL nicht verfügbar, verwende SQLite"
        DATABASE_URL="sqlite:$DATA_DIR/csf-core.db"
    fi
    
    print_success "Datenbank konfiguriert"
}

generate_jwt_secret() {
    print_step "Generiere JWT Secret..."
    JWT_SECRET=$(openssl rand -base64 64 | tr -d '\n')
    print_success "JWT Secret generiert"
}

create_systemd_service() {
    print_step "Erstelle systemd Service..."
    
    cat > /etc/systemd/system/${SERVICE_NAME}.service <<EOF
[Unit]
Description=CSF-Core Service (Backend + Frontend)
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${INSTALL_DIR}
EnvironmentFile=${INSTALL_DIR}/config.env
ExecStart=${INSTALL_DIR}/start.sh
Restart=always
RestartSec=10
StandardOutput=append:${LOG_DIR}/service.log
StandardError=append:${LOG_DIR}/service-error.log

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR} ${LOG_DIR}

[Install]
WantedBy=multi-user.target
EOF
    
    print_success "systemd Service erstellt"
}

create_startup_script() {
    print_step "Erstelle Startup-Script..."
    
    cat > ${INSTALL_DIR}/start.sh <<'EOFSCRIPT'
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_BIN="${SCRIPT_DIR}/backend/backend"
FRONTEND_DIR="${SCRIPT_DIR}/frontend"

# Load environment
if [ -f "${SCRIPT_DIR}/config.env" ]; then
    set -a
    source "${SCRIPT_DIR}/config.env"
    set +a
fi

# Start Frontend in background
if [ -d "$FRONTEND_DIR" ]; then
    echo "▶️  Starting Frontend (Port ${PORT:-3000})..."
    cd "$FRONTEND_DIR"
    export PUBLIC_API_BASE_URL="${PUBLIC_API_BASE_URL:-/api}"
    PORT=${PORT:-3000} HOST=0.0.0.0 node build/index.js &
    FRONTEND_PID=$!
    echo "   Frontend PID: $FRONTEND_PID"
    
    sleep 3
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
    print_success "Startup-Script erstellt"
}

create_config_file() {
    print_step "Erstelle Konfigurationsdatei..."
    
    cat > ${INSTALL_DIR}/config.env <<EOF
# CSF-Core Configuration (Development Build)
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
RUST_LOG=debug
NODE_ENV=production

# Frontend läuft auf Port 3000 (intern)
PORT=3000
FRONTEND_URL=${FRONTEND_URL}

# Backend Port (extern erreichbar)
BACKEND_PORT=8000

# API Base URL für Frontend
PUBLIC_API_BASE_URL=${PUBLIC_API_BASE_URL}

# Docker Socket
# Falls Docker-Probleme: sudo usermod -aG docker csf-core && sudo systemctl restart csf-core
EOF
    
    chmod 600 ${INSTALL_DIR}/config.env
    print_success "Konfiguration erstellt"
}

set_permissions() {
    print_step "Setze Berechtigungen..."
    
    chown -R ${SERVICE_USER}:${SERVICE_USER} "$INSTALL_DIR"
    chown -R ${SERVICE_USER}:${SERVICE_USER} "$DATA_DIR"
    chown -R ${SERVICE_USER}:${SERVICE_USER} "$LOG_DIR"
    
    print_success "Berechtigungen gesetzt"
}

open_firewall() {
    print_step "Öffne Firewall-Ports..."
    
    if command -v ufw &> /dev/null; then
        ufw allow 8000/tcp 2>/dev/null || true
        ufw reload 2>/dev/null || true
        print_success "Port 8000 in ufw geöffnet"
    elif command -v firewall-cmd &> /dev/null; then
        firewall-cmd --permanent --add-port=8000/tcp 2>/dev/null || true
        firewall-cmd --reload 2>/dev/null || true
        print_success "Port 8000 in firewalld geöffnet"
    else
        print_warning "Keine Firewall gefunden"
    fi
}

start_service() {
    print_step "Starte Service..."
    
    systemctl daemon-reload
    systemctl enable ${SERVICE_NAME}
    systemctl restart ${SERVICE_NAME}
    
    # Wait a moment for service to start
    sleep 2
    
    if systemctl is-active --quiet ${SERVICE_NAME}; then
        print_success "Service gestartet"
    else
        print_error "Service konnte nicht gestartet werden"
        print_step "Zeige letzte Logs:"
        journalctl -u ${SERVICE_NAME} -n 20 --no-pager
        exit 1
    fi
}

print_success_info() {
    local server_ip=$(hostname -I | awk '{print $1}')
    
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                Installation erfolgreich!              ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Service Management:${NC}"
    echo "  sudo systemctl status ${SERVICE_NAME}"
    echo "  sudo systemctl restart ${SERVICE_NAME}"
    echo "  sudo systemctl stop ${SERVICE_NAME}"
    echo ""
    echo -e "${BLUE}Logs:${NC}"
    echo "  sudo journalctl -u ${SERVICE_NAME} -f"
    echo "  sudo tail -f ${LOG_DIR}/service.log"
    echo ""
    echo -e "${BLUE}Zugriff:${NC}"
    echo "  → http://localhost:8000"
    echo "  → http://${server_ip}:8000"
    echo ""
    echo -e "${BLUE}Docker-Probleme beheben:${NC}"
    echo "  sudo usermod -aG docker ${SERVICE_USER}"
    echo "  sudo chmod 666 /var/run/docker.sock"
    echo "  sudo systemctl restart ${SERVICE_NAME}"
    echo ""
}

cleanup() {
    print_step "Räume auf..."
    cd /
    rm -rf "$1"
    print_success "Temporäre Dateien gelöscht"
}

# Main Installation
main() {
    print_header
    
    check_root
    install_dependencies
    install_rust
    install_nodejs
    install_docker
    
    create_service_user
    create_directories
    
    local repo_dir=$(clone_repository)
    
    build_backend "$repo_dir"
    build_frontend "$repo_dir"
    
    setup_database
    generate_jwt_secret
    
    create_systemd_service
    create_startup_script
    create_config_file
    set_permissions
    
    open_firewall
    start_service
    
    cleanup "$repo_dir"
    
    print_success_info
}

main "$@"
