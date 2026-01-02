#!/bin/bash
# CSF Agent Installation Script
# Downloads and installs the latest pre-built agent binary from GitHub releases

set -e

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Konfiguration
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
GITHUB_REPO="CS-Foundry/CSF-Core"
VERSION="${VERSION:-latest}"

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         CSF Agent Installation Script                 ║${NC}"
    echo -e "${BLUE}║         Installing Pre-Built Binary from GitHub       ║${NC}"
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

detect_platform() {
    print_step "Erkenne Plattform..."
    
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case $os in
        linux)
            OS_NAME="linux"
            ;;
        darwin)
            OS_NAME="darwin"
            ;;
        *)
            print_error "Nicht unterstütztes Betriebssystem: $os"
            print_error "Unterstützte Systeme: Linux, macOS"
            exit 1
            ;;
    esac
    
    case $arch in
        x86_64)
            ARCH_NAME="amd64"
            ;;
        aarch64|arm64)
            ARCH_NAME="arm64"
            ;;
        *)
            print_error "Nicht unterstützte Architektur: $arch"
            print_error "Unterstützte Architekturen: x86_64, aarch64, arm64"
            exit 1
            ;;
    esac
    
    print_success "Plattform: ${OS_NAME}-${ARCH_NAME}"
}

get_latest_version() {
    print_step "Ermittle neueste Release-Version..."
    
    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local latest_version=$(curl -s "$api_url" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    
    if [ -z "$latest_version" ]; then
        print_error "Konnte neueste Version nicht ermitteln"
        print_error "Überprüfe deine Internetverbindung oder versuche es später erneut"
        exit 1
    fi
    
    print_success "Neueste Version: v${latest_version}"
    echo "$latest_version"
}

download_binary() {
    print_step "Download CSF Agent Binary..."
    
    # Get version
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
    fi
    
    # Construct download URL
    local binary_name="csf-agent-${OS_NAME}-${ARCH_NAME}"
    if [ "$OS_NAME" = "windows" ]; then
        binary_name="${binary_name}.exe"
    fi
    
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${binary_name}"
    
    print_step "Download von: $download_url"
    
    # Create temporary directory
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Download binary
    if curl -L -f "$download_url" -o csf-agent 2>/dev/null; then
        print_success "Binary erfolgreich heruntergeladen"
    else
        print_error "Download fehlgeschlagen!"
        print_error "URL: $download_url"
        print_error ""
        print_error "Mögliche Ursachen:"
        print_error "  - Release v${VERSION} existiert nicht"
        print_error "  - Binary für ${OS_NAME}-${ARCH_NAME} wurde nicht gebaut"
        print_error "  - Internetverbindung unterbrochen"
        cd - > /dev/null
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # Make executable
    chmod +x csf-agent
    
    # Install to system
    print_step "Installiere nach $INSTALL_DIR..."
    cp csf-agent "$INSTALL_DIR/csf-agent"
    
    # Cleanup
    cd - > /dev/null
    rm -rf "$temp_dir"
    
    print_success "CSF Agent installiert nach $INSTALL_DIR/csf-agent"
}

verify_installation() {
    print_step "Verifiziere Installation..."
    
    if [ -f "$INSTALL_DIR/csf-agent" ] && [ -x "$INSTALL_DIR/csf-agent" ]; then
        print_success "Installation erfolgreich verifiziert"
        
        # Try to get version
        if "$INSTALL_DIR/csf-agent" --version 2>/dev/null; then
            echo ""
        else
            print_warning "Binary installiert, aber --version schlägt fehl"
        fi
    else
        print_error "Installation fehlgeschlagen"
        exit 1
    fi
}

show_usage() {
    echo ""
    echo -e "${GREEN}Installation abgeschlossen!${NC}"
    echo ""
    echo "Der CSF Agent wurde nach $INSTALL_DIR/csf-agent installiert"
    echo ""
    echo "Nächste Schritte:"
    echo ""
    echo "1. Konfiguration erstellen:"
    echo "   sudo mkdir -p /etc/csf-agent"
    echo "   sudo nano /etc/csf-agent/config.toml"
    echo ""
    echo "2. Agent manuell starten:"
    echo "   csf-agent"
    echo ""
    echo "3. Als systemd Service einrichten (empfohlen):"
    echo "   Siehe INSTALLATION.md für Details"
    echo ""
    echo "4. Version überprüfen:"
    echo "   csf-agent --version"
    echo ""
}

# Main execution
main() {
    print_header
    check_root
    detect_platform
    download_binary
    verify_installation
    show_usage
}

main
