# 🚀 CSF-Core Quick Deployment Guide

## Docker (Schnellste Methode)

```bash
docker pull ghcr.io/cs-foundry/csf-core:latest
docker run -d -p 8000:8000 -v csf_data:/data --name csf-core ghcr.io/cs-foundry/csf-core:latest
```

**Zugriff**: http://localhost:8000

## Native Linux Installation

```bash
# Stable (main branch)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash

# Development Branch
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash

# Service starten
sudo systemctl start csf-core
```

## Was ist CSF-Core?

CSF-Core ist ein **unified Backend + Frontend Service** für System-Monitoring und Container-Management:

- ✅ **Backend**: Rust (Axum) auf Port 8000
- ✅ **Frontend**: SvelteKit (Node) auf Port 3000 (intern)
- ✅ **Ein Entry Point**: Backend proxied automatisch zum Frontend
- ✅ **Systemd Service**: Läuft als Daemon auf Linux
- ✅ **Docker Management**: Container, Images, Networks, Volumes
- ✅ **System Monitoring**: CPU, Memory, Disk, Network

## Architektur

```
    ┌─────────────────────────────────┐
    │   http://localhost:8000         │
    │   (Public Entry Point)          │
    └────────────┬────────────────────┘
                 │
    ┌────────────▼────────────────────┐
    │   Backend (Rust/Axum)           │
    │   - /api/*  → API Endpoints     │
    │   - /*      → Proxy to Frontend │
    └────────────┬────────────────────┘
                 │ (intern)
    ┌────────────▼────────────────────┐
    │   Frontend (SvelteKit)          │
    │   Port 3000 (Internal Only)     │
    └─────────────────────────────────┘
```

## Features

### System Monitoring

- Real-time CPU, Memory, Disk Metriken
- Network RX/TX Statistics
- System Info (OS, Kernel, Uptime)

### Container Management (geplant)

- Docker Container Management
- Image Management
- Volume Management
- Network Management

### Security

- JWT Authentication
- RBAC (Role-Based Access Control)
- 2FA Support
- End-to-End Encryption ready

## Deployment Optionen

| Methode            | Empfohlen für              | Installation |
| ------------------ | -------------------------- | ------------ |
| **Docker**         | Quick Testing, Development | 5 Minuten    |
| **Native**         | Production, Performance    | 10 Minuten   |
| **Docker Compose** | Multi-Container Setup      | 5 Minuten    |

## Installation Details

➡️ Siehe [INSTALLATION.md](./INSTALLATION.md) für:

- Detaillierte Installations-Schritte
- Konfiguration
- Reverse Proxy Setup (Nginx/Apache)
- Backup & Restore
- Troubleshooting

## Development

```bash
# Backend
cd backend
cargo run

# Frontend
cd frontend
npm install
npm run dev

# Beide gleichzeitig
./start.sh  # oder docker-compose up
```

## Management

```bash
# Service Management (Native)
sudo systemctl {start|stop|restart|status} csf-core
sudo journalctl -u csf-core -f

# Docker
docker {start|stop|restart} csf-core
docker logs -f csf-core
```

## Verfügbare Images

### GitHub Container Registry (GHCR)

```bash
# Latest (empfohlen)
ghcr.io/cs-foundry/csf-core:latest

# Spezifische Version
ghcr.io/cs-foundry/csf-core:v1.2.3

# Multi-Arch Support
# Docker wählt automatisch: linux/amd64 oder linux/arm64
```

## Quick Commands

```bash
# Installation (main branch - stable)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash

# Installation (spezifischer Branch)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/BRANCH_NAME/scripts/install.sh | sudo bash

# Deinstallation
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/uninstall.sh | sudo bash

# Status
sudo systemctl status csf-core

# Logs
sudo journalctl -u csf-core -f

# Konfiguration
sudo nano /opt/csf-core/config.env
sudo systemctl restart csf-core
```

## Ports

| Port | Service                  | Zugriff       |
| ---- | ------------------------ | ------------- |
| 8000 | Backend + Frontend Proxy | Public        |
| 3000 | Frontend (SvelteKit)     | Internal Only |

## Environment Variables

```bash
DATABASE_URL=postgres://user:pass@localhost/csf_core
JWT_SECRET=your-secret-here
RUST_LOG=info
NODE_ENV=production
PORT=3000
FRONTEND_URL=http://localhost:3000
ORIGIN=http://localhost:8000
```

## Support

- 📖 [Vollständige Dokumentation](./INSTALLATION.md)
- 🐛 [Issues](https://github.com/CS-Foundry/CSF-Core/issues)
- 💬 [Discussions](https://github.com/CS-Foundry/CSF-Core/discussions)

## License

MIT - siehe [LICENSE](LICENSE)
