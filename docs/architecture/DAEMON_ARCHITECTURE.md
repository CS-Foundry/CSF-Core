# CSF Core - Local System Monitoring & Orchestration

## Übersicht

Das CSF Core Backend läuft als Daemon-Service auf einem Linux-System und sammelt direkt System-Metriken sowie orchestriert Container und Services. Es benötigt **keinen separaten Agent**, sondern läuft direkt auf dem Host-System.

## Architektur

```
┌─────────────────────────────────────────────────────────┐
│                    Linux Host System                     │
│                                                           │
│  ┌───────────────────────────────────────────────────┐  │
│  │         CSF Core Backend (Daemon)                  │  │
│  │                                                     │  │
│  │  ┌─────────────────┐  ┌─────────────────────────┐ │  │
│  │  │ System Collector│  │  Future: Docker Manager │ │  │
│  │  │                 │  │                         │ │  │
│  │  │ - CPU Metriken  │  │ - Container Management  │ │  │
│  │  │ - Memory Usage  │  │ - Image Management      │ │  │
│  │  │ - Disk Usage    │  │ - Volume Management     │ │  │
│  │  │ - Network Stats │  │ - Network Management    │ │  │
│  │  └─────────────────┘  └─────────────────────────┘ │  │
│  │                                                     │  │
│  │  REST API: /api/system/*  (geplant: /api/docker/*) │  │
│  └───────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐│
│  │     Docker Daemon (für Container-Orchestrierung)    ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

## Implementierte Features

### 1. Local System Monitoring

Das Backend sammelt direkt System-Metriken vom Host-System:

#### Backend-Komponenten:

- **`system_collector.rs`**: Sammelt System-Metriken mit `sysinfo` crate

  - CPU: Modell, Kerne, Threads, Auslastung
  - Memory: Gesamt, verwendet, Prozent
  - Disk: Gesamt, verwendet, Prozent
  - Network: RX/TX Bytes
  - System: OS, Version, Kernel, Hostname, Uptime

- **`routes/system.rs`**: REST API Endpunkte
  - `GET /api/system/info` - Statische System-Informationen
  - `GET /api/system/metrics` - Aktuelle Metriken in Echtzeit

#### Frontend-Komponenten:

- **`services/system.ts`**: Service für System-API-Calls
- **`routes/local-system/+page.svelte`**: Dashboard für lokales System
  - Echtzeit-Metriken mit Auto-Refresh (5s)
  - Radial Charts für CPU/Memory/Disk
  - System-Informationen
  - Network-Statistiken

## Verwendung

### Backend als Service ausführen

```bash
# Development
cd backend
cargo run

# Production Build
cargo build --release
./target/release/backend

# Als systemd Service (Linux)
sudo cp csf-core.service /etc/systemd/system/
sudo systemctl enable csf-core
sudo systemctl start csf-core
```

### System-Metriken abrufen

```bash
# System Info
curl -H "Authorization: Bearer <token>" http://localhost:8000/api/system/info

# Aktuelle Metriken
curl -H "Authorization: Bearer <token>" http://localhost:8000/api/system/metrics
```

### Frontend

Navigiere zu `/local-system` im Frontend, um das Dashboard zu sehen.

## Geplante Features

### Docker/Container Management

Als nächstes wird das Backend um Docker-Management-Funktionen erweitert:

#### Geplante Endpunkte:

```
/api/docker/containers
  GET     - Liste aller Container
  POST    - Neuen Container starten

/api/docker/containers/:id
  GET     - Container-Details
  POST    - Container starten/stoppen
  DELETE  - Container entfernen

/api/docker/images
  GET     - Liste aller Images
  POST    - Image pullen
  DELETE  - Image löschen

/api/docker/volumes
  GET     - Liste aller Volumes
  POST    - Volume erstellen
  DELETE  - Volume löschen

/api/docker/networks
  GET     - Liste aller Networks
  POST    - Network erstellen
  DELETE  - Network löschen
```

#### Benötigte Dependencies:

```toml
# Cargo.toml
bollard = "0.17"  # Docker API Client für Rust
```

#### Implementation Plan:

1. **Docker Client Service** (`docker_service.rs`)

   - Verbindung zum Docker Socket (`/var/run/docker.sock`)
   - Container-Lifecycle-Management
   - Image-Management
   - Volume & Network Management

2. **Docker Routes** (`routes/docker.rs`)

   - REST API Endpunkte
   - Request/Response Models
   - Error Handling

3. **Frontend Integration**
   - Container-Liste und Details
   - Container-Logs anzeigen
   - Container starten/stoppen/neustarten
   - Image-Management UI
   - Volume & Network Management

## System-Anforderungen

- Linux-System (Ubuntu, Debian, CentOS, etc.)
- Rust 1.70+
- Docker (optional, für Container-Orchestrierung)
- PostgreSQL (für Metadaten-Speicherung)

## Zugriffsrechte

Damit das Backend auf System-Ressourcen und Docker zugreifen kann:

```bash
# Docker Socket Zugriff
sudo usermod -aG docker csf-core-user

# System Metriken (kein root nötig mit sysinfo crate)
# Die sysinfo crate funktioniert ohne root-Rechte
```

## Service-Datei Beispiel

```ini
[Unit]
Description=CSF Core Backend Daemon
After=network.target docker.service
Requires=postgresql.service

[Service]
Type=simple
User=csf-core
Group=csf-core
WorkingDirectory=/opt/csf-core
Environment="DATABASE_URL=postgres://user:pass@localhost/csf_core"
Environment="JWT_SECRET=your-secret-key"
ExecStart=/opt/csf-core/backend
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Entwicklung

### Testing der System-Metriken

```bash
# Backend starten
cd backend
cargo run

# In anderem Terminal: API testen
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"password"}'

# Token speichern und verwenden
TOKEN="<your-token>"

curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/system/metrics | jq
```

### Nächste Schritte

1. ✅ System-Metriken-Sammlung implementiert
2. ✅ REST API Endpunkte erstellt
3. ✅ Frontend Dashboard erstellt
4. 🔄 Docker Client Service entwickeln
5. ⏳ Container-Management Endpunkte
6. ⏳ Frontend für Container-Management
7. ⏳ Volume & Network Management
8. ⏳ Container-Logs Streaming

## Lizenz

Siehe LICENSE Datei im Root-Verzeichnis.
