# CSF-Core - Deinstallation & Neuinstallation

## 🗑️ Vollständige Deinstallation

```bash
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/uninstall.sh | sudo bash
```

**Was wird entfernt:**

- systemd Service
- Installation in `/opt/csf-core`
- Logs in `/var/log/csf-core`
- Service-Benutzer `csf-core`
- Optional: Datenbank und Daten (wenn bestätigt)

---

## 🔄 Vollständige Neuinstallation (Clean Install)

```bash
# Alles deinstallieren und neu installieren
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/reinstall.sh | sudo bash
```

**Was passiert:**

1. ✅ Stoppt alle Services
2. ✅ Entfernt alte Installation
3. ✅ Löscht alte Datenbank
4. ✅ Prüft PostgreSQL
5. ✅ Installiert neu

---

## 🔧 Manuelle Schritte

### 1. Service stoppen

```bash
sudo systemctl stop csf-core
sudo systemctl disable csf-core
```

### 2. Alte Installation entfernen

```bash
sudo rm -rf /opt/csf-core
sudo rm -rf /var/lib/csf-core
sudo rm -rf /var/log/csf-core
sudo rm -f /etc/systemd/system/csf-core.service
sudo systemctl daemon-reload
```

### 3. Datenbank entfernen (PostgreSQL)

```bash
sudo -u postgres psql -c "DROP DATABASE IF EXISTS csf_core;"
sudo -u postgres psql -c "DROP USER IF EXISTS csf_core;"
```

### 4. Service-Benutzer entfernen

```bash
sudo userdel csf-core
```

### 5. Neu installieren

```bash
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash
```

---

## 🐛 Troubleshooting

### PostgreSQL läuft nicht

```bash
# Status prüfen
sudo systemctl status postgresql

# Starten
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Logs ansehen
sudo journalctl -u postgresql -n 50
```

### Installation schlägt fehl

```bash
# 1. Komplett deinstallieren
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/uninstall.sh | sudo bash

# 2. PostgreSQL prüfen
sudo systemctl status postgresql

# 3. Neu installieren mit Debug
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash -x
```

### Service startet nicht

```bash
# Logs prüfen
sudo journalctl -u csf-core -n 100 --no-pager

# Manuell testen
cd /opt/csf-core
sudo -u csf-core ./backend/backend

# Config prüfen
sudo cat /opt/csf-core/config.env
```

---

## 📋 Prüf-Checkliste nach Installation

```bash
# 1. PostgreSQL läuft
sudo systemctl status postgresql
# Erwartung: active (running)

# 2. Service existiert
sudo systemctl status csf-core
# Erwartung: Service file exists

# 3. Installation vorhanden
ls -la /opt/csf-core/
# Erwartung: backend/, frontend/, start.sh, config.env

# 4. Datenbank erreichbar
sudo -u postgres psql -c "\l" | grep csf_core
# Erwartung: csf_core database listed

# 5. Config korrekt
sudo cat /opt/csf-core/config.env
# Erwartung: DATABASE_URL, JWT_SECRET, etc.
```

---

## 💡 Tipps

### Nur Backend neu installieren

```bash
cd /path/to/CSF-Core/backend
cargo build --release
sudo systemctl stop csf-core
sudo cp target/release/backend /opt/csf-core/backend/
sudo systemctl start csf-core
```

### Nur Frontend neu installieren

```bash
cd /path/to/CSF-Core/frontend
npm ci
npm run build
sudo systemctl stop csf-core
sudo cp -r build /opt/csf-core/frontend/
sudo systemctl start csf-core
```

### Datenbank zurücksetzen (ohne Neuinstallation)

```bash
sudo -u postgres psql -c "DROP DATABASE csf_core;"
sudo -u postgres psql -c "CREATE DATABASE csf_core OWNER csf_core;"
sudo systemctl restart csf-core
```

---

## 🚨 Notfall-Deinstallation

Wenn nichts mehr geht:

```bash
sudo pkill -f csf-core
sudo systemctl stop csf-core
sudo rm -rf /opt/csf-core /var/lib/csf-core /var/log/csf-core
sudo rm -f /etc/systemd/system/csf-core.service
sudo systemctl daemon-reload
sudo userdel csf-core
```

Dann neu installieren:

```bash
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash
```
