# CSF-Core Troubleshooting Guide

## 🔍 Logs ansehen

### Backend & Service Logs

#### Live-Logs mit systemd (empfohlen)

```bash
# Alle Service-Logs live verfolgen
sudo journalctl -u csf-core -f

# Letzte 100 Zeilen anzeigen
sudo journalctl -u csf-core -n 100

# Logs seit einem bestimmten Zeitpunkt
sudo journalctl -u csf-core --since "2025-01-01 00:00:00"

# Logs mit höherer Detailtiefe (nur Fehler)
sudo journalctl -u csf-core -p err -f
```

#### Log-Dateien direkt lesen

```bash
# Backend Standard-Output (Info-Logs)
sudo tail -f /var/log/csf-core/csf-core.log

# Backend Fehler-Logs
sudo tail -f /var/log/csf-core/csf-core-error.log

# Frontend-Logs
sudo tail -f /var/log/csf-core/frontend.log

# Letzte 200 Zeilen aller Logs
sudo tail -n 200 /var/log/csf-core/*.log
```

### Service-Status prüfen

```bash
# Service-Status anzeigen
sudo systemctl status csf-core

# Prüfen ob Service läuft
sudo systemctl is-active csf-core

# Service neu starten
sudo systemctl restart csf-core

# Service stoppen
sudo systemctl stop csf-core

# Service starten
sudo systemctl start csf-core
```

## 🐛 Häufige Fehler

### 1. 500 Internal Server Error - "Cannot find package"

**Symptom:**

```
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'jsonwebtoken'
```

**Ursache:** Frontend-Build enthält nicht alle Runtime-Dependencies.

**Lösung:**

- ✅ **Bereits behoben** in neueren Releases (>= v1.0.1)
- Frontend-Package enthält nun `node_modules/` mit Production-Dependencies
- Einfach neu installieren:
  ```bash
  curl -fsSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
  ```

### 2. Backend startet nicht - Datenbankverbindung

**Symptom:**

```
Failed to connect to database
```

**Lösung:**

```bash
# PostgreSQL Status prüfen
sudo systemctl status postgresql

# PostgreSQL neu starten
sudo systemctl restart postgresql

# Datenbank-URL in Config prüfen
sudo cat /opt/csf-core/config.env | grep DATABASE_URL

# Bei SQLite-Problemen: Berechtigungen prüfen
sudo chown -R csf-core:csf-core /var/lib/csf-core
```

### 3. Frontend nicht erreichbar

**Symptom:** Browser zeigt "Connection refused" auf Port 3000

**Lösung:**

```bash
# Prüfen ob Frontend-Prozess läuft
ps aux | grep node

# Frontend-Logs prüfen
sudo tail -100 /var/log/csf-core/frontend.log

# Node.js Version prüfen (sollte >= 20 sein)
node -v

# Service neu starten
sudo systemctl restart csf-core
```

### 4. Backend nicht erreichbar (Port 8000)

**Symptom:** API-Requests auf Port 8000 schlagen fehl

**Lösung:**

```bash
# Firewall-Status prüfen
sudo firewall-cmd --list-all  # RHEL/CentOS
sudo ufw status              # Ubuntu/Debian

# Port 8000 öffnen
sudo firewall-cmd --permanent --add-port=8000/tcp
sudo firewall-cmd --reload
# oder
sudo ufw allow 8000/tcp

# Prüfen ob Backend auf Port 8000 lauscht
sudo netstat -tulpn | grep 8000
# oder
sudo ss -tulpn | grep 8000
```

## 🔧 Debug-Modus aktivieren

Für detailliertere Logs:

```bash
# Config bearbeiten
sudo nano /opt/csf-core/config.env

# Ändern von:
RUST_LOG=info
# zu:
RUST_LOG=debug

# Service neu starten
sudo systemctl restart csf-core

# Logs live verfolgen
sudo journalctl -u csf-core -f
```

## 📊 System-Informationen sammeln

Bei Support-Anfragen hilfreich:

```bash
# Alle relevanten Infos in eine Datei schreiben
{
  echo "=== CSF-Core System Info ==="
  echo "Date: $(date)"
  echo ""
  echo "=== Service Status ==="
  systemctl status csf-core --no-pager
  echo ""
  echo "=== Last 50 Log Lines ==="
  journalctl -u csf-core -n 50 --no-pager
  echo ""
  echo "=== Config ==="
  cat /opt/csf-core/config.env
  echo ""
  echo "=== Disk Space ==="
  df -h /opt/csf-core /var/lib/csf-core /var/log/csf-core
  echo ""
  echo "=== Processes ==="
  ps aux | grep -E '(backend|node|csf-core)'
} > csf-core-debug.txt

# Datei anzeigen
cat csf-core-debug.txt
```

## 🔄 Neuinstallation

Falls nichts hilft:

```bash
# Service stoppen
sudo systemctl stop csf-core

# Alte Installation sichern (optional)
sudo mv /opt/csf-core /opt/csf-core.backup
sudo mv /var/lib/csf-core /var/lib/csf-core.backup

# Neu installieren
curl -fsSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
```

## 📞 Support

- **GitHub Issues:** https://github.com/CS-Foundry/CSF-Core/issues
- **Logs bereitstellen:** Nutze den "System-Informationen sammeln" Befehl oben
- **Release-Seite:** https://github.com/CS-Foundry/CSF-Core/releases (Download-Statistiken)

## ⚡ Performance-Optimierung

### Backend-Optimierung

```bash
# In /opt/csf-core/config.env
RUST_LOG=info,sea_orm=warn  # Weniger DB-Logs
```

### Logs rotieren

```bash
# Logrotate einrichten
sudo nano /etc/logrotate.d/csf-core
```

Inhalt:

```
/var/log/csf-core/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 csf-core csf-core
    sharedscripts
    postrotate
        systemctl reload csf-core > /dev/null 2>&1 || true
    endscript
}
```

## 🎯 Nächste Schritte

Nach erfolgreicher Installation:

1. Öffne `http://your-server-ip:8000` im Browser
2. Erstelle einen Admin-Account
3. Prüfe Dashboard-Funktionen
4. Teste API-Endpoints: `http://your-server-ip:8000/api/health`

Bei weiteren Fragen: Siehe Dokumentation in `INSTALLATION.md` und `DEPLOYMENT.md`
