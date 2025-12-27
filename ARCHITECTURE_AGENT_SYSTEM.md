# Agent-Based Infrastructure Management Architecture

## 🎯 Vision

Aufbau eines skalierbaren Management-Systems für physische Server, Container (Docker/Kubernetes) und Cloud-Ressourcen (Azure/AWS) ähnlich wie Kubernetes, Portainer oder Datadog.

## 🏗️ Architektur-Übersicht

```
┌─────────────────────────────────────────────────────────────┐
│                    Web Frontend (SvelteKit)                  │
│          - Dashboard mit Metriken                            │
│          - Server/Container Management                       │
│          - Echtzeit-Monitoring                               │
└──────────────────────┬──────────────────────────────────────┘
                       │ REST API / WebSocket
┌──────────────────────▼──────────────────────────────────────┐
│              Central Server (Axum Backend)                   │
│  ┌────────────┬─────────────┬──────────────┬──────────────┐ │
│  │ API Layer  │ WebSocket   │ Database     │ Scheduler    │ │
│  │ (REST)     │ (Real-time) │ (PostgreSQL) │ (Tasks)      │ │
│  └────────────┴─────────────┴──────────────┴──────────────┘ │
└───────┬──────────────┬──────────────┬──────────────────────┘
        │              │              │
        │ gRPC/        │ gRPC/        │ gRPC/
        │ WebSocket    │ WebSocket    │ WebSocket
        │              │              │
┌───────▼──────┐ ┌─────▼──────┐ ┌───▼──────────┐
│  Agent (Mac) │ │ Agent (VM) │ │ Agent (Cloud)│
│  ┌─────────┐ │ │ ┌─────────┐│ │ ┌──────────┐ │
│  │Collector││ │ │Collector││ │ │ Collector│ │
│  │ - CPU   ││ │ │ - CPU   ││ │ │ - CPU    │ │
│  │ - RAM   ││ │ │ - RAM   ││ │ │ - RAM    │ │
│  │ - Disk  ││ │ │ - Disk  ││ │ │ - Disk   │ │
│  │ - Net   ││ │ │ - Net   ││ │ │ - Net    │ │
│  └─────────┘ │ │ └─────────┘│ │ └──────────┘ │
│  ┌─────────┐ │ │ ┌─────────┐│ │ ┌──────────┐ │
│  │Executor ││ │ │Executor ││ │ │ Executor │ │
│  │ - Docker││ │ │ - Docker││ │ │ - K8s    │ │
│  │ - Tasks ││ │ │ - K8s   ││ │ │ - Tasks  │ │
│  └─────────┘ │ │ └─────────┘│ │ └──────────┘ │
└──────────────┘ └────────────┘ └──────────────┘
```

## 🔧 Komponenten

### 1. **Central Server** (Bestehend: Axum Backend)

- **Läuft in**: Docker Container
- **Aufgaben**:
  - REST API für Frontend
  - WebSocket-Server für Echtzeit-Updates
  - Datenbank-Management (PostgreSQL)
  - Agent-Verwaltung & Authentifizierung
  - Task-Scheduling & Orchestrierung
  - Aggregation von Metriken

### 2. **Agents** (Neu zu entwickeln)

- **Läuft auf**: Jedem zu verwaltenden Host (Mac, Linux, Windows, Cloud VM)
- **Programmiersprache**: Rust (klein, performant, sicher)
- **Kommunikation**: gRPC oder WebSocket zum Central Server
- **Komponenten**:

  #### a) **Metrics Collector**

  - System-Metriken sammeln (sysinfo crate)
  - CPU, RAM, Disk, Network
  - Container-Status (Docker API)
  - Kubernetes-Status (kubectl/API)

  #### b) **Command Executor**

  - Befehle vom Server empfangen
  - Container starten/stoppen
  - Deployments durchführen
  - Logs sammeln

  #### c) **Health Reporter**

  - Heartbeat zum Server
  - Status-Updates
  - Error-Reporting

### 3. **Frontend** (Bestehend: SvelteKit)

- **Läuft in**: Docker Container
- **Features**:
  - Dashboard mit Metriken-Übersicht
  - Server-Liste mit Echtzeit-Status
  - Container-Management
  - Log-Viewer
  - Deployment-Interface

## 📊 Datenbank-Schema

### Neue Tables

#### `agents`

```sql
CREATE TABLE agents (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45),
    agent_version VARCHAR(50),
    os_type VARCHAR(50),
    os_version VARCHAR(100),
    architecture VARCHAR(50),
    status VARCHAR(50), -- online, offline, error
    last_heartbeat TIMESTAMP,
    registered_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP,
    organization_id UUID REFERENCES organization(id),
    tags JSONB, -- Flexible metadata
    capabilities JSONB -- What the agent can do
);
```

#### `agent_metrics`

```sql
CREATE TABLE agent_metrics (
    id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    cpu_usage_percent FLOAT,
    memory_total_bytes BIGINT,
    memory_used_bytes BIGINT,
    disk_total_bytes BIGINT,
    disk_used_bytes BIGINT,
    network_rx_bytes BIGINT,
    network_tx_bytes BIGINT,
    custom_metrics JSONB
);

-- Index for time-series queries
CREATE INDEX idx_agent_metrics_agent_time
ON agent_metrics(agent_id, timestamp DESC);
```

#### `managed_containers`

```sql
CREATE TABLE managed_containers (
    id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    container_id VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    image VARCHAR(500),
    status VARCHAR(50),
    created_at TIMESTAMP,
    started_at TIMESTAMP,
    ports JSONB,
    environment JSONB,
    labels JSONB,
    last_seen TIMESTAMP
);
```

#### `agent_tasks`

```sql
CREATE TABLE agent_tasks (
    id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    task_type VARCHAR(100), -- deploy, restart, update, collect_logs
    payload JSONB,
    status VARCHAR(50), -- pending, running, completed, failed
    created_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    result JSONB,
    error TEXT
);
```

#### `agent_logs`

```sql
CREATE TABLE agent_logs (
    id UUID PRIMARY KEY,
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    level VARCHAR(20), -- debug, info, warn, error
    message TEXT,
    context JSONB,
    timestamp TIMESTAMP NOT NULL
);

CREATE INDEX idx_agent_logs_agent_time
ON agent_logs(agent_id, timestamp DESC);
```

## 🔐 Sicherheit

### Agent-Authentifizierung

```rust
// Agents nutzen JWT oder API-Keys
struct AgentCredentials {
    agent_id: Uuid,
    api_key: String, // SHA-256 hashed
    certificate: Option<String>, // TLS Client Certificate
}
```

### Kommunikation

- **TLS 1.3** für alle Verbindungen
- **mTLS** (mutual TLS) für Agent ↔ Server
- **Token-basiert** mit Rotation

## 📡 Agent-Kommunikation

### Protokoll-Optionen

#### Option 1: gRPC (Empfohlen)

**Vorteile:**

- Bidirektional (Server kann Push-Befehle senden)
- Binary Protocol (effizient)
- Code-Generierung (Proto-Files)
- HTTP/2 (Multiplexing)

```protobuf
// agent.proto
service AgentService {
  rpc StreamMetrics(stream Metrics) returns (stream Command);
  rpc RegisterAgent(AgentInfo) returns (RegistrationResponse);
  rpc ExecuteTask(Task) returns (TaskResult);
}

message Metrics {
  string agent_id = 1;
  int64 timestamp = 2;
  float cpu_usage = 3;
  int64 memory_used = 4;
  // ...
}

message Command {
  string task_id = 1;
  string command_type = 2;
  bytes payload = 3;
}
```

#### Option 2: WebSocket

**Vorteile:**

- Einfacher zu debuggen
- Firewall-freundlich
- JSON oder Binary

```rust
// Agent sendet Metriken
{
  "type": "metrics",
  "agent_id": "uuid",
  "timestamp": 1234567890,
  "data": {
    "cpu_usage": 45.2,
    "memory_used": 8589934592
  }
}

// Server sendet Befehl
{
  "type": "command",
  "task_id": "uuid",
  "action": "deploy_container",
  "payload": { ... }
}
```

## 🚀 Implementation Roadmap

### Phase 1: Basic Agent System (Current)

- [x] Central Server läuft in Docker
- [x] Frontend läuft in Docker
- [ ] Agent Binary (Rust)
  - [ ] System-Metriken sammeln
  - [ ] Verbindung zum Server herstellen
  - [ ] Heartbeat senden
- [ ] Server-Seite: Agent-Registration API
- [ ] Server-Seite: Metriken-Empfang & Speicherung
- [ ] Frontend: Agent-Übersicht

### Phase 2: Docker Management

- [ ] Agent: Docker API Integration
- [ ] Agent: Container-Status sammeln
- [ ] Server: Container-Management API
- [ ] Frontend: Container-Liste & Controls

### Phase 3: Task Execution

- [ ] Agent: Command Executor
- [ ] Server: Task Queue System
- [ ] Server: Task-Status Tracking
- [ ] Frontend: Task Management

### Phase 4: Kubernetes Support

- [ ] Agent: kubectl Integration
- [ ] Agent: K8s API Client
- [ ] Server: K8s Resources API
- [ ] Frontend: K8s Dashboard

### Phase 5: Cloud Integration

- [ ] Agent: Azure SDK
- [ ] Agent: AWS SDK
- [ ] Server: Cloud Resource Management
- [ ] Frontend: Cloud Resources View

## 🛠️ Technologie-Stack

### Agent (Rust Binary)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tonic = "0.11"  # gRPC
sysinfo = "0.32"  # System metrics
bollard = "0.17"  # Docker API
kube = "0.95"  # Kubernetes
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Server (Bestehend + Erweiterungen)

```toml
[dependencies]
# Bestehend: axum, sea-orm, ...
tonic = "0.11"  # gRPC Server
tokio-tungstenite = "0.23"  # WebSocket
```

## 📁 Projekt-Struktur

```
CSF-Core/
├── backend/          # Axum Server (in Docker)
│   ├── src/
│   │   ├── routes/
│   │   │   ├── agents.rs      # NEW: Agent Management
│   │   │   ├── metrics.rs     # NEW: Metrics API
│   │   │   └── tasks.rs       # NEW: Task Management
│   │   ├── services/
│   │   │   ├── agent_service.rs   # NEW
│   │   │   └── metrics_service.rs # NEW
│   │   └── grpc/              # NEW: gRPC Services
│   └── proto/
│       └── agent.proto        # NEW: Protocol Definition
│
├── agent/            # NEW: Rust Agent Binary
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── collector/
│   │   │   ├── system.rs      # sysinfo
│   │   │   ├── docker.rs      # bollard
│   │   │   └── kubernetes.rs  # kube
│   │   ├── executor/
│   │   │   └── commands.rs
│   │   ├── transport/
│   │   │   ├── grpc.rs
│   │   │   └── websocket.rs
│   │   └── config.rs
│   ├── install.sh             # Agent Installer Script
│   └── agent.service          # systemd Service File
│
├── frontend/         # SvelteKit (in Docker)
│   └── src/routes/
│       ├── agents/            # NEW: Agent Management UI
│       ├── containers/        # NEW: Container Management
│       └── metrics/           # NEW: Metrics Dashboard
│
└── docker-compose.yml  # Nur Server + Frontend + DB
```

## 🔄 Deployment-Workflow

### Server (Central)

```bash
# Wie bisher: Docker Compose
docker compose up -d
```

### Agent (auf jedem Host)

```bash
# Download Agent Binary
curl -L https://your-server.com/agent/install.sh | bash

# Oder manuell
./csf-agent install \
  --server-url https://your-server.com \
  --api-key your-api-key \
  --name "MacBook-Pro" \
  --tags "environment=dev,location=home"

# Agent läuft als systemd service (Linux) oder launchd (macOS)
systemctl status csf-agent
```

## 💡 Vorteile dieser Architektur

1. **Skalierbar**: Unbegrenzt viele Agents
2. **Flexibel**: Agents auf beliebigen Hosts
3. **Sicher**: TLS + Authentifizierung
4. **Effizient**: Rust = geringe Ressourcen
5. **Erweiterbar**: Einfach neue Capabilities hinzufügen
6. **Standard**: Ähnlich wie Kubernetes, Prometheus Node Exporter

## 🎯 Nächste Schritte

1. **Agent-Projekt erstellen**

   ```bash
   cargo new --bin agent
   ```

2. **Proto-Files definieren** (gRPC Schema)

3. **Basic Agent implementieren**

   - System-Metriken sammeln
   - Verbindung zum Server
   - Heartbeat

4. **Server-API erweitern**

   - Agent-Registration Endpoint
   - Metriken-Empfang
   - Agent-Status Tracking

5. **Frontend erweitern**
   - Agents-Liste
   - Metriken-Dashboard

Soll ich mit der Implementierung beginnen? Welche Phase möchtest du zuerst umsetzen?
