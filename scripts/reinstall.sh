#!/bin/bash
# CSF-Core - Vollstaendige Neuinstallation
# Dieses Script deinstalliert alles und installiert neu

set +e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     CSF-Core - Vollstaendige Neuinstallation          ${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Bitte als root ausfuehren: sudo $0${NC}"
    exit 1
fi

echo -e "${YELLOW}WARNUNG: Dies wird CSF-Core vollstaendig deinstallieren und neu installieren!${NC}"
echo ""
read -p "Fortfahren? (j/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Jj]$ ]]; then
    echo "Abgebrochen."
    exit 0
fi

# Step 1: Deinstallation
echo ""
echo -e "${BLUE}[1/3] Deinstalliere CSF-Core...${NC}"
echo ""

# Stop service
systemctl stop csf-core 2>/dev/null || true
systemctl disable csf-core 2>/dev/null || true

# Remove systemd service
rm -f /etc/systemd/system/csf-core.service
systemctl daemon-reload

# Remove directories
rm -rf /opt/csf-core
rm -rf /var/lib/csf-core
rm -rf /var/log/csf-core

# Remove PostgreSQL database
if command -v psql &> /dev/null && systemctl is-active --quiet postgresql 2>/dev/null; then
    echo "Entferne PostgreSQL Datenbank..."
    sudo -u postgres psql -c "DROP DATABASE IF EXISTS csf_core;" 2>/dev/null || true
    sudo -u postgres psql -c "DROP USER IF EXISTS csf_core;" 2>/dev/null || true
fi

# Remove user
userdel csf-core 2>/dev/null || true

echo -e "${GREEN}Deinstallation abgeschlossen${NC}"

# Step 2: Optional - PostgreSQL neu starten
echo ""
echo -e "${BLUE}[2/3] PostgreSQL pruefen...${NC}"
echo ""

if systemctl is-active --quiet postgresql 2>/dev/null; then
    echo -e "${GREEN}PostgreSQL laeuft${NC}"
else
    echo -e "${YELLOW}PostgreSQL laeuft nicht, versuche zu starten...${NC}"
    systemctl start postgresql 2>/dev/null || true
    sleep 2
    if systemctl is-active --quiet postgresql 2>/dev/null; then
        echo -e "${GREEN}PostgreSQL gestartet${NC}"
    else
        echo -e "${YELLOW}PostgreSQL konnte nicht gestartet werden${NC}"
        echo "   Installation wird SQLite verwenden"
    fi
fi

# Step 3: Neuinstallation
echo ""
echo -e "${BLUE}[3/3] Installiere CSF-Core neu...${NC}"
echo ""
sleep 2

# Download und execute install script
BRANCH="${BRANCH:-feat/docker-managment}"
echo "Lade Install-Script von Branch: $BRANCH"
echo ""

curl -sSL "https://raw.githubusercontent.com/CS-Foundry/CSF-Core/${BRANCH}/scripts/install.sh" | bash

echo ""
echo -e "${GREEN}========================================================${NC}"
echo -e "${GREEN}     Neuinstallation abgeschlossen!                    ${NC}"
echo -e "${GREEN}========================================================${NC}"
echo ""
echo "Naechste Schritte:"
echo ""
echo "  sudo systemctl start csf-core"
echo "  sudo systemctl status csf-core"
echo ""
