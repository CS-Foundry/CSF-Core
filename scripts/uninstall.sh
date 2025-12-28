#!/bin/bash
# CSF-Core Deinstallation Script

set +e  # Nicht bei Fehlern abbrechen

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SERVICE_NAME="csf-core"
INSTALL_DIR="/opt/csf-core"
DATA_DIR="/var/lib/csf-core"
LOG_DIR="/var/log/csf-core"
SERVICE_USER="csf-core"

print_header() {
    echo -e "${YELLOW}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║         CSF-Core Deinstallation                       ║${NC}"
    echo -e "${YELLOW}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Bitte als root ausführen: sudo $0${NC}"
    exit 1
fi

print_header

echo -e "${YELLOW}WARNUNG: Dies wird CSF-Core vollständig entfernen!${NC}"
echo ""
read -p "Möchtest du auch die Datenbank-Daten löschen? (j/N) " -n 1 -r
echo
REMOVE_DATA=$REPLY

# Stop service
echo -e "${GREEN}Stoppe Service...${NC}"
systemctl stop ${SERVICE_NAME} 2>/dev/null || true
systemctl disable ${SERVICE_NAME} 2>/dev/null || true

# Remove systemd service
echo -e "${GREEN}Entferne systemd Service...${NC}"
rm -f /etc/systemd/system/${SERVICE_NAME}.service
systemctl daemon-reload

# Remove installation directory
echo -e "${GREEN}Entferne Installation...${NC}"
rm -rf ${INSTALL_DIR}
rm -rf ${LOG_DIR}

# Remove data if requested
if [[ $REMOVE_DATA =~ ^[Jj]$ ]]; then
    echo -e "${GREEN}Entferne Daten...${NC}"
    rm -rf ${DATA_DIR}
    
    # Drop PostgreSQL database
    if command -v psql &> /dev/null && systemctl is-active --quiet postgresql 2>/dev/null; then
        echo -e "${GREEN}Entferne PostgreSQL Datenbank...${NC}"
        sudo -u postgres psql -c "DROP DATABASE IF EXISTS csf_core;" 2>/dev/null || true
        sudo -u postgres psql -c "DROP USER IF EXISTS csf_core;" 2>/dev/null || true
    fi
fi

# Remove user
echo -e "${GREEN}Entferne Service-Benutzer...${NC}"
userdel ${SERVICE_USER} 2>/dev/null || true

echo ""
echo -e "${GREEN}✓ CSF-Core wurde erfolgreich deinstalliert${NC}"
echo ""

if [[ ! $REMOVE_DATA =~ ^[Jj]$ ]]; then
    echo -e "${YELLOW}Hinweis: Daten wurden nicht gelöscht und befinden sich in:${NC}"
    echo "  - ${DATA_DIR}"
    echo ""
fi
