#!/bin/bash

# Direkt auf dem Server ausführen um Build-Tools zu installieren

echo "=== CSF-Core Development Installation Fix ==="
echo ""
echo "Installiere Build-Tools..."

if command -v apt-get &> /dev/null; then
    echo "Verwende apt-get (Debian/Ubuntu)..."
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y build-essential git curl
    echo "✓ Build-Tools installiert"
elif command -v dnf &> /dev/null; then
    echo "Verwende dnf (Fedora/RHEL 8+)..."
    dnf install -y gcc gcc-c++ make git curl
    echo "✓ Build-Tools installiert"
elif command -v yum &> /dev/null; then
    echo "Verwende yum (RHEL/CentOS)..."
    yum install -y gcc gcc-c++ make git curl
    echo "✓ Build-Tools installiert"
else
    echo "✗ Konnte Paketmanager nicht erkennen"
    exit 1
fi

echo ""
echo "=== Prüfe Installation ==="
echo "gcc: $(which gcc 2>/dev/null || echo 'NICHT GEFUNDEN')"
echo "g++: $(which g++ 2>/dev/null || echo 'NICHT GEFUNDEN')"
echo "make: $(which make 2>/dev/null || echo 'NICHT GEFUNDEN')"
echo "git: $(which git 2>/dev/null || echo 'NICHT GEFUNDEN')"
echo ""

echo "=== Öffne Firewall-Ports ==="
# Port 8000 für Backend/Frontend (Backend proxyt zum Frontend)
if command -v ufw &> /dev/null; then
    echo "Verwende ufw (Ubuntu/Debian)..."
    ufw allow 8000/tcp
    ufw reload 2>/dev/null || true
    echo "✓ Port 8000 geöffnet"
elif command -v firewall-cmd &> /dev/null; then
    echo "Verwende firewalld (RHEL/CentOS)..."
    firewall-cmd --permanent --add-port=8000/tcp
    firewall-cmd --reload
    echo "✓ Port 8000 geöffnet"
else
    echo "⚠ Keine Firewall gefunden oder iptables wird verwendet"
    echo "  Falls iptables: sudo iptables -A INPUT -p tcp --dport 8000 -j ACCEPT"
fi

echo ""
echo "=== Externe Zugriff ==="
SERVER_IP=$(hostname -I | awk '{print $1}')
echo "Server IP: $SERVER_IP"
echo ""
echo "Nach der Installation erreichbar unter:"
echo "  → http://$SERVER_IP:8000"
echo ""
echo "Du kannst jetzt das normale Install-Script ausführen:"
echo "curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo BRANCH=feat/docker-managment bash"
