# CSF-Core Installation & Deployment Guide

## 🚀 Schnellstart

CSF-Core ist ein unified Backend + Frontend Service, der auf Linux als systemd Service läuft.

### 📊 Download-Statistiken

[![GitHub Downloads (all releases)](https://img.shields.io/github/downloads/CS-Foundry/CSF-Core/total?style=for-the-badge&logo=github&label=Total%20Downloads)](https://github.com/CS-Foundry/CSF-Core/releases)
[![GitHub Release](https://img.shields.io/github/v/release/CS-Foundry/CSF-Core?style=for-the-badge&logo=github)](https://github.com/CS-Foundry/CSF-Core/releases/latest)

**📈 [Aktuelle Download-Statistiken anzeigen →](https://github.com/CS-Foundry/CSF-Core/releases)**

Nach jedem GitHub Actions Build werden die Download-Zahlen für jedes Binary im Workflow-Summary angezeigt.

### Voraussetzungen

- Linux (Ubuntu 20.04+, Debian 11+, RHEL 8+)
- PostgreSQL 12+ (wird automatisch installiert falls nicht vorhanden)
- Node.js 18+ (wird automatisch installiert)
- 2GB RAM minimum
- 10GB Festplattenspeicher

## Installation

### Option 1: One-Line Installation (Empfohlen)

```bash
# Von main Branch (PRODUCTION) - verwendet nur Pre-Built Releases von GitHub
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash

# Von Development Branch - baut aus Quellcode (installiert gcc, Rust automatisch)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo BRANCH=feat/docker-managment bash

# Mit custom API URL (z.B. für externes Backend)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo BRANCH=feat/docker-managment PUBLIC_API_BASE_URL=http://your-backend.com/api bash

# Von einem bestimmten Tag/Release
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/v1.2.3/scripts/install.sh | sudo bash

# Explizit aus Quellcode bauen (auch für main Branch)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo BUILD_FROM_SOURCE=1 bash
```

**Environment-Variablen:**

- `BRANCH` - Git Branch zum Klonen (Standard: main bzw. feat/docker-managment)
- `PUBLIC_API_BASE_URL` - Frontend API URL (Standard: `/api` für relative Pfade)
- `BUILD_FROM_SOURCE` - Erzwingt Build aus Quellcode (Standard: nur für Development)
- `VERSION` - Spezifische Version/Tag (Standard: `latest`)

Das Script:

- ✅ Installiert alle Abhängigkeiten (Node.js 20, PostgreSQL)
- ✅ **Production (main)**: Verwendet NUR Pre-Built Releases von GitHub Actions
- ✅ **Development**: Installiert gcc, Rust automatisch und baut aus Quellcode
- ✅ Erstellt PostgreSQL-Datenbank automatisch
- ✅ Richtet systemd Service ein
- ✅ Konfiguriert Backend + Frontend
- ✅ Generiert sichere Secrets (JWT, DB-Passwort)
- ✅ SQLite Fallback wenn PostgreSQL fehlschlägt

**Installation Strategie:**

**Production (main Branch):**

1. ✅ Lädt Pre-Built Release von GitHub (gebaut via GitHub Actions)
2. ❌ Baut NICHT aus Quellcode (außer BUILD_FROM_SOURCE=1 gesetzt)
3. ⚠️ Schlägt fehl wenn kein Release verfügbar → warte auf GitHub Actions Build

**Development (andere Branches):**

1. Versucht Release Download (wenn verfügbar)
2. Falls nicht verfügbar: Installiert Build-Tools (gcc, make, git)
3. Installiert Rust/Cargo automatisch
4. Baut Backend + Frontend aus Quellcode (~10-15 Min)

**Was wird automatisch installiert:**

- Node.js 20 LTS (wenn nicht vorhanden)
- PostgreSQL (automatisch, keine Benutzerinteraktion nötig)
- **Build-Tools** (gcc, make, git) - nur für Development
- **Rust/Cargo** - nur für Development
- Systemd Service (Backend + Frontend)
- Datenbank wird automatisch initialisiert

**Voraussetzungen:**

**Production:**

- Nur curl, systemd (normalerweise schon vorhanden)
- Keine Build-Tools nötig!

**Development:**

- Wird automatisch installiert: gcc, make, git, Rust

### Option 2: Docker Installation

```bash
# Pull Image
docker pull ghcr.io/cs-foundry/csf-core:latest

# Run Container
docker run -d \
  --name csf-core \
  -p 8000:8000 \
  -v csf_data:/data \
  -e JWT_SECRET=$(openssl rand -hex 32) \
  ghcr.io/cs-foundry/csf-core:latest
```

### Option 3: Docker Compose

```yaml
version: "3.8"

services:
  csf-core:
    image: ghcr.io/cs-foundry/csf-core:latest
    container_name: csf-core
    ports:
      - "8000:8000"
    volumes:
      - csf_data:/data
    environment:
      - DATABASE_URL=sqlite:/data/csf-core.db
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    restart: unless-stopped

volumes:
  csf_data:
```

```bash
JWT_SECRET=$(openssl rand -hex 32) docker-compose up -d
```

## Verwendung

### Service Management (Native Installation)

```bash
# Service starten
sudo systemctl start csf-core

# Service stoppen
sudo systemctl stop csf-core

# Status prüfen
sudo systemctl status csf-core

# Logs ansehen
sudo journalctl -u csf-core -f

# Auto-Start aktivieren
sudo systemctl enable csf-core
```

## 🐛 Troubleshooting

Bei Problemen oder Fehlern (z.B. 500 Internal Server Error):

**📖 [Komplette Troubleshooting-Anleitung →](../troubleshooting/TROUBLESHOOTING.md)**

Häufige Befehle:

```bash
# Alle Logs live ansehen
sudo journalctl -u csf-core -f

# Backend-Fehler-Logs
sudo tail -f /var/log/csf-core/csf-core-error.log

# Frontend-Logs
sudo tail -f /var/log/csf-core/frontend.log

# Service neu starten
sudo systemctl restart csf-core

# Debug-Modus aktivieren
sudo nano /opt/csf-core/config.env  # RUST_LOG=debug
sudo systemctl restart csf-core
```

### Konfiguration anpassen

Die Konfiguration befindet sich in `/opt/csf-core/config.env`:

```bash
sudo nano /opt/csf-core/config.env
sudo systemctl restart csf-core
```

Wichtige Einstellungen:

```bash
# Datenbank
DATABASE_URL=postgres://csf_core:password@localhost/csf_core

# Security
JWT_SECRET=your-secret-here

# Logging
RUST_LOG=info  # debug, info, warn, error

# Network
ORIGIN=http://localhost:8000
```

## Zugriff

Nach der Installation ist CSF-Core verfügbar unter:

- **Web Interface**: http://localhost:8000
- **API Docs**: http://localhost:8000/swagger-ui
- **API Endpoint**: http://localhost:8000/api

## Architektur

CSF-Core verwendet eine unified Architecture:

```
┌─────────────────────────────────────┐
│  Port 8000 (Public)                 │
│  ┌───────────────────────────────┐  │
│  │  Backend (Rust/Axum)          │  │
│  │  - API Routes: /api/*         │  │
│  │  - Frontend Proxy: /*         │  │
│  └───────────┬───────────────────┘  │
│              │                       │
│              ↓ (internal)            │
│  ┌───────────────────────────────┐  │
│  │  Frontend (SvelteKit/Node)    │  │
│  │  Port 3000 (Internal only)    │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**Vorteile:**

- ✅ Ein Port (8000) für alles
- ✅ Keine CORS-Probleme
- ✅ Einfaches Deployment
- ✅ Automatisches HTTPS Proxy möglich

## Systemanforderungen

### Minimale Anforderungen

| Komponente | Minimum           |
| ---------- | ----------------- |
| CPU        | 1 Core            |
| RAM        | 2GB               |
| Disk       | 10GB              |
| OS         | Linux Kernel 4.x+ |

### Empfohlene Anforderungen

| Komponente | Empfohlen        |
| ---------- | ---------------- |
| CPU        | 2+ Cores         |
| RAM        | 4GB+             |
| Disk       | 20GB SSD         |
| OS         | Ubuntu 22.04 LTS |

## Reverse Proxy Setup (Production)

### Nginx

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

### Apache

```apache
<VirtualHost *:80>
    ServerName your-domain.com

    ProxyPreserveHost On
    ProxyPass / http://localhost:8000/
    ProxyPassReverse / http://localhost:8000/

    <Proxy *>
        Order allow,deny
        Allow from all
    </Proxy>
</VirtualHost>
```

## Backup & Restore

### Backup

```bash
# Database Backup (PostgreSQL)
sudo -u postgres pg_dump csf_core > csf-core-backup-$(date +%Y%m%d).sql

# Oder bei SQLite
sudo cp /var/lib/csf-core/csf-core.db ~/csf-core-backup-$(date +%Y%m%d).db

# Konfiguration
sudo cp /opt/csf-core/config.env ~/csf-core-config-backup.env
```

### Restore

```bash
# Database Restore (PostgreSQL)
sudo -u postgres psql csf_core < csf-core-backup-20231218.sql

# Oder bei SQLite
sudo systemctl stop csf-core
sudo cp ~/csf-core-backup-20231218.db /var/lib/csf-core/csf-core.db
sudo chown csf-core:csf-core /var/lib/csf-core/csf-core.db
sudo systemctl start csf-core
```

## Updates

### Native Installation

```bash
# Neue Version herunterladen und installieren
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo VERSION=1.2.3 bash

# Service neu starten
sudo systemctl restart csf-core
```

### Docker Installation

```bash
# Neue Version pullen
docker pull ghcr.io/cs-foundry/csf-core:latest

# Container neu starten
docker stop csf-core
docker rm csf-core
docker run -d \
  --name csf-core \
  -p 8000:8000 \
  -v csf_data:/data \
  -e JWT_SECRET=$(cat /path/to/secret) \
  ghcr.io/cs-foundry/csf-core:latest
```

## Deinstallation

```bash
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/uninstall.sh | sudo bash
```

## Troubleshooting

### Service startet nicht

```bash
# Logs prüfen
sudo journalctl -u csf-core -n 50

# Status prüfen
sudo systemctl status csf-core

# Konfiguration prüfen
sudo cat /opt/csf-core/config.env
```

### Datenbank-Verbindung fehlgeschlagen

```bash
# PostgreSQL Status prüfen
sudo systemctl status postgresql

# Verbindung testen
psql -h localhost -U csf_core -d csf_core
```

### Port bereits belegt

```bash
# Prüfen welcher Prozess Port 8000 verwendet
sudo lsof -i :8000

# Oder
sudo netstat -tulpn | grep 8000
```

### Frontend lädt nicht

```bash
# Prüfen ob Frontend läuft
curl http://localhost:3000

# Node.js Version prüfen
node -v  # Sollte >= 18.x sein
```

## Support & Dokumentation

- 📖 [Vollständige Dokumentation](https://github.com/CS-Foundry/CSF-Core)
- 🐛 [Bug Reports](https://github.com/CS-Foundry/CSF-Core/issues)
- 💬 [Discussions](https://github.com/CS-Foundry/CSF-Core/discussions)

## License

MIT License - siehe [LICENSE](LICENSE)
