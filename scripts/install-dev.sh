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
echo "Du kannst jetzt das normale Install-Script ausführen:"
echo "curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo BRANCH=feat/docker-managment bash"
