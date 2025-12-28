#!/bin/bash
# CSF-Core Quick Fix Script
# Behebt den fehlenden Service-User und andere Probleme

set +e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════${NC}"
echo -e "${BLUE}  CSF-Core Quick Fix${NC}"
echo -e "${BLUE}═══════════════════════════════════════${NC}"
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Bitte als root ausführen: sudo $0${NC}"
    exit 1
fi

# 1. Service User erstellen
echo -e "${GREEN}[1/5] Erstelle Service-User...${NC}"
if id "csf-core" &>/dev/null; then
    echo "✓ User csf-core existiert bereits"
else
    useradd --system --no-create-home --shell /usr/sbin/nologin csf-core
    echo "✓ User csf-core erstellt"
fi

# 2. Verzeichnisse erstellen und Rechte setzen
echo -e "${GREEN}[2/5] Erstelle Verzeichnisse...${NC}"
mkdir -p /opt/csf-core/{backend,frontend}
mkdir -p /var/lib/csf-core
mkdir -p /var/log/csf-core

chown -R csf-core:csf-core /opt/csf-core
chown -R csf-core:csf-core /var/lib/csf-core
chown -R csf-core:csf-core /var/log/csf-core

echo "✓ Verzeichnisse erstellt und Rechte gesetzt"

# 3. Prüfe ob Backend existiert
echo -e "${GREEN}[3/5] Prüfe Installation...${NC}"
if [ ! -f "/opt/csf-core/backend/backend" ]; then
    echo -e "${YELLOW}⚠ Backend binary nicht gefunden${NC}"
    echo "Bitte führe vollständige Installation aus:"
    echo "curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/reinstall.sh | sudo bash"
    exit 1
fi

if [ ! -d "/opt/csf-core/frontend/build" ]; then
    echo -e "${YELLOW}⚠ Frontend build nicht gefunden${NC}"
fi

echo "✓ Installation gefunden"

# 4. Service neu laden
echo -e "${GREEN}[4/5] Lade systemd neu...${NC}"
systemctl daemon-reload
echo "✓ systemd neu geladen"

# 5. Versuche Service zu starten
echo -e "${GREEN}[5/5] Starte Service...${NC}"
systemctl stop csf-core 2>/dev/null || true
sleep 2
systemctl start csf-core

sleep 3

if systemctl is-active --quiet csf-core; then
    echo -e "${GREEN}✓ Service läuft!${NC}"
    systemctl status csf-core --no-pager -l
else
    echo -e "${RED}✗ Service konnte nicht gestartet werden${NC}"
    echo ""
    echo "Logs:"
    journalctl -u csf-core -n 20 --no-pager
    echo ""
    echo "Bitte führe vollständige Neuinstallation aus:"
    echo "curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/reinstall.sh | sudo bash"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════${NC}"
