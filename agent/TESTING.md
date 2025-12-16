# CSF Agent - Testing Guide

## ✅ Agent erfolgreich implementiert und getestet!

### Was wurde implementiert:

1. **Metrics Collector** (`collector.rs`)
   - Sammelt CPU, RAM, Disk, Network Metriken
   - Nutzt `sysinfo` crate für plattformübergreifende Metriken
   - Erfasst OS-Informationen und Uptime

2. **Server Client** (`client.rs`)
   - HTTP Client für Kommunikation mit Central Server
   - Endpoints: Registration, Heartbeat, Metrics Upload
   - Authentifizierung via API-Key

3. **Configuration** (`config.rs`)
   - TOML-basierte Konfiguration
   - Automatisches Laden aus lokalem oder System-Pfad
   - Generiert UUID für Agent-ID

4. **Main Loop** (`main.rs`)
   - Async Runtime mit Tokio
   - Automatische Registration beim Start
   - Heartbeat Task (alle 20s im Test)
   - Metrics Collection Loop (alle 10s im Test)

### Test-Ergebnisse:

```
✅ Agent kompiliert erfolgreich
✅ Configuration wird geladen
✅ Metriken werden gesammelt:
   - CPU: 15.1%
   - RAM: 80.4%
   - Disk: 13.3%
✅ Registration-Request wird gesendet
✅ Heartbeat-Task läuft
✅ Metrics-Upload versucht

❌ Server nicht erreichbar (erwartet, da Backend noch nicht gestartet)
```

### Binary Size (Release):

```bash
$ ls -lh target/release/csf-agent
-rwxr-xr-x  1 user  staff   7.2M Dec 16 19:11 csf-agent
```

Mit `strip` und optimierten Compiler-Flags ~7MB.

## 🚀 Nächste Schritte:

### 1. Backend-Routes testen

Das Backend muss die neuen Agent-Endpoints haben:
- `POST /api/agents/register` - Agent Registration
- `POST /api/agents/heartbeat` - Heartbeat empfangen
- `POST /api/agents/metrics` - Metriken speichern
- `GET /api/agents` - Liste aller Agents
- `GET /api/agents/:id` - Agent Details
- `GET /api/agents/:id/metrics` - Metriken eines Agents

### 2. Integration Test

```bash
# Terminal 1: Backend starten
cd backend
docker compose -f docker-compose.dev.yml up -d postgres
cargo run

# Terminal 2: Agent starten
cd agent
RUST_LOG=info ./target/release/csf-agent
```

### 3. Frontend entwickeln

Route: `/agents`
- Liste aller registrierten Agents
- Status (online/offline)
- Letzte Metriken
- Echtzeit-Updates

## 📝 Agent Konfiguration

### Lokale Test-Config (`config.toml`):

```toml
agent_id = "550e8400-e29b-41d4-a716-446655440000"
name = "Test-MacBook"
server_url = "http://localhost:8000"
api_key = "test-api-key-123"
collection_interval = 10  # Sekunden
heartbeat_interval = 20   # Sekunden
tags = ["test", "development", "macos"]
```

### Produktions-Config:

```toml
agent_id = "auto-generated-uuid"
name = "Production-Server-01"
server_url = "https://your-server.com"
api_key = "secure-api-key-here"
collection_interval = 30
heartbeat_interval = 60
tags = ["production", "kubernetes", "aws-eu-west-1"]
```

## 🔧 Agent Befehle:

```bash
# Build (Development)
cargo build

# Build (Release - optimiert)
cargo build --release

# Run mit Logging
RUST_LOG=info ./target/release/csf-agent

# Run mit Debug-Logging
RUST_LOG=debug ./target/release/csf-agent

# Install als System Service (später)
sudo cp target/release/csf-agent /usr/local/bin/
sudo cp csf-agent.service /etc/systemd/system/
sudo systemctl enable csf-agent
sudo systemctl start csf-agent
```

## 📊 Gesammelte Metriken:

### System Information:
- Hostname
- OS Type & Version
- Kernel Version
- Architecture (x86_64, arm64, etc.)
- Uptime

### CPU:
- Model/Brand
- Physical Cores
- Logical Threads
- Usage Percentage (average)

### Memory:
- Total Bytes
- Used Bytes
- Usage Percentage

### Disk:
- Total Bytes (all disks)
- Used Bytes (all disks)
- Usage Percentage

### Network:
- Total RX Bytes
- Total TX Bytes

## 🎯 Was noch fehlt:

1. ✅ Agent Binary - **FERTIG**
2. ⏳ Backend API-Routes - **Code erstellt, muss getestet werden**
3. ⏳ Migration ausführen - **Erstellt, muss angewendet werden**
4. ❌ Frontend UI - **TODO**
5. ❌ Docker Container Management - **TODO (Phase 2)**
6. ❌ Kubernetes Support - **TODO (Phase 4)**

## 🐛 Bekannte Limitationen:

1. **Keine TLS-Verschlüsselung** - HTTP only (TODO: HTTPS)
2. **Keine Authentifizierung** - API-Key nur in Header (TODO: JWT)
3. **Kein Reconnect** - Bei Server-Ausfall stirbt der Agent (TODO: Retry-Logic)
4. **Keine Metriken-Aggregation** - Sendet jedes Mal alle Daten (TODO: Deltas)

## 💡 Verbesserungsideen:

1. **Graceful Shutdown** - SIGTERM/SIGINT Handler
2. **Config Reload** - SIGHUP für Live-Reload
3. **Health Check Endpoint** - HTTP Server für Monitoring
4. **Metrics Buffering** - Queue für offline Operation
5. **Compression** - gzip für Metriken-Upload
6. **Batch Upload** - Mehrere Metriken in einem Request
