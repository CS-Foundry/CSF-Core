#!/bin/bash
set -e

echo "==================================="
echo "CSF Core Backend Installation"
echo "==================================="
echo ""

# Überprüfen ob als root ausgeführt
if [ "$EUID" -ne 0 ]; then 
    echo "Bitte als root ausführen (sudo ./install-daemon.sh)"
    exit 1
fi

# Variablen
INSTALL_DIR="/opt/csf-core"
SERVICE_USER="csf-core"
SERVICE_FILE="csf-core.service"
BINARY_NAME="backend"

echo "1. Erstelle Service-Benutzer..."
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd --system --no-create-home --shell /usr/sbin/nologin "$SERVICE_USER"
    echo "   ✓ Benutzer '$SERVICE_USER' erstellt"
else
    echo "   ✓ Benutzer '$SERVICE_USER' existiert bereits"
fi

# Füge Benutzer zur docker Gruppe hinzu (falls Docker installiert ist)
if getent group docker > /dev/null 2>&1; then
    usermod -aG docker "$SERVICE_USER"
    echo "   ✓ Benutzer zu 'docker' Gruppe hinzugefügt"
fi

echo ""
echo "2. Erstelle Installationsverzeichnis..."
mkdir -p "$INSTALL_DIR"
echo "   ✓ Verzeichnis '$INSTALL_DIR' erstellt"

echo ""
echo "3. Baue Backend..."
cd "$(dirname "$0")/backend"
cargo build --release
echo "   ✓ Backend erfolgreich gebaut"

echo ""
echo "4. Kopiere Binary..."
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo "   ✓ Binary nach '$INSTALL_DIR' kopiert"

echo ""
echo "5. Setze Berechtigungen..."
chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
echo "   ✓ Berechtigungen gesetzt"

echo ""
echo "6. Installiere systemd Service..."
cd "$(dirname "$0")"
cp "$SERVICE_FILE" "/etc/systemd/system/"
chmod 644 "/etc/systemd/system/$SERVICE_FILE"
echo "   ✓ Service-Datei nach /etc/systemd/system/ kopiert"

echo ""
echo "7. Konfiguriere Umgebungsvariablen..."
echo "   WICHTIG: Bitte passe die Umgebungsvariablen in"
echo "   /etc/systemd/system/$SERVICE_FILE an:"
echo ""
echo "   - DATABASE_URL: PostgreSQL Verbindung"
echo "   - JWT_SECRET: Sicherer Secret Key"
echo "   - FRONTEND_URL: URL des Frontends"
echo ""
read -p "   Möchtest du die Service-Datei jetzt bearbeiten? (j/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Jj]$ ]]; then
    ${EDITOR:-nano} "/etc/systemd/system/$SERVICE_FILE"
fi

echo ""
echo "8. Systemd neu laden..."
systemctl daemon-reload
echo "   ✓ Systemd neu geladen"

echo ""
echo "==================================="
echo "Installation abgeschlossen!"
echo "==================================="
echo ""
echo "Nächste Schritte:"
echo ""
echo "1. Service aktivieren (automatischer Start beim Boot):"
echo "   sudo systemctl enable csf-core"
echo ""
echo "2. Service starten:"
echo "   sudo systemctl start csf-core"
echo ""
echo "3. Status überprüfen:"
echo "   sudo systemctl status csf-core"
echo ""
echo "4. Logs ansehen:"
echo "   sudo journalctl -u csf-core -f"
echo ""
echo "5. Service neustarten:"
echo "   sudo systemctl restart csf-core"
echo ""
echo "6. Service stoppen:"
echo "   sudo systemctl stop csf-core"
echo ""
