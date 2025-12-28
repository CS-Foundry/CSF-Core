# Docker Management Integration - Implementierungsplan

## Übersicht

Dieser Leitfaden beschreibt, wie Docker-Management-Funktionen zum CSF Core Backend hinzugefügt werden.

## Phase 1: Dependencies und Setup

### 1.1 Cargo.toml erweitern

```toml
# backend/Cargo.toml
[dependencies]
# ... bestehende dependencies ...
bollard = "0.17"  # Docker API Client für Rust
```

### 1.2 Docker Socket Zugriff

Das Backend kommuniziert über den Docker Socket:

- Linux: `/var/run/docker.sock`
- Windows: `npipe:////./pipe/docker_engine`
- macOS: `/var/run/docker.sock`

## Phase 2: Docker Service erstellen

### 2.1 Datei: `backend/src/docker_service.rs`

```rust
use bollard::Docker;
use bollard::container::{ListContainersOptions, Stats, StatsOptions};
use bollard::image::ListImagesOptions;
use bollard::volume::ListVolumesOptions;
use bollard::network::ListNetworksOptions;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DockerService {
    client: Docker,
}

impl DockerService {
    pub fn new() -> Result<Self, bollard::errors::Error> {
        let client = Docker::connect_with_socket_defaults()?;
        Ok(Self { client })
    }

    // Container Management
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>, DockerError> {
        // Implementation
    }

    pub async fn get_container(&self, id: &str) -> Result<ContainerDetails, DockerError> {
        // Implementation
    }

    pub async fn start_container(&self, id: &str) -> Result<(), DockerError> {
        // Implementation
    }

    pub async fn stop_container(&self, id: &str) -> Result<(), DockerError> {
        // Implementation
    }

    pub async fn restart_container(&self, id: &str) -> Result<(), DockerError> {
        // Implementation
    }

    pub async fn remove_container(&self, id: &str, force: bool) -> Result<(), DockerError> {
        // Implementation
    }

    pub async fn get_container_logs(&self, id: &str, tail: usize) -> Result<Vec<String>, DockerError> {
        // Implementation
    }

    pub async fn get_container_stats(&self, id: &str) -> Result<ContainerStats, DockerError> {
        // Implementation
    }

    // Image Management
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>, DockerError> {
        // Implementation
    }

    pub async fn pull_image(&self, name: &str, tag: &str) -> Result<(), DockerError> {
        // Implementation
    }

    pub async fn remove_image(&self, id: &str, force: bool) -> Result<(), DockerError> {
        // Implementation
    }

    // Volume Management
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>, DockerError> {
        // Implementation
    }

    pub async fn create_volume(&self, name: &str) -> Result<VolumeInfo, DockerError> {
        // Implementation
    }

    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<(), DockerError> {
        // Implementation
    }

    // Network Management
    pub async fn list_networks(&self) -> Result<Vec<NetworkInfo>, DockerError> {
        // Implementation
    }

    pub async fn create_network(&self, name: &str) -> Result<NetworkInfo, DockerError> {
        // Implementation
    }

    pub async fn remove_network(&self, id: &str) -> Result<(), DockerError> {
        // Implementation
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerDetails {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMount>,
    pub networks: Vec<String>,
    pub env: Vec<String>,
    pub created: i64,
    pub started: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerStats {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

// ... weitere Types ...
```

### 2.2 AppState erweitern

```rust
// backend/src/main.rs
#[derive(Clone)]
pub struct AppState {
    pub db_conn: DbConn,
    pub docker: Option<DockerService>,  // Optional, falls Docker nicht verfügbar
}
```

## Phase 3: Docker Routes erstellen

### 3.1 Datei: `backend/src/routes/docker.rs`

```rust
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::auth::middleware::AuthenticatedUser;
use crate::docker_service::{DockerService, ContainerInfo};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Container Routes
        .route("/docker/containers", get(list_containers))
        .route("/docker/containers/:id", get(get_container))
        .route("/docker/containers/:id/start", post(start_container))
        .route("/docker/containers/:id/stop", post(stop_container))
        .route("/docker/containers/:id/restart", post(restart_container))
        .route("/docker/containers/:id/remove", delete(remove_container))
        .route("/docker/containers/:id/logs", get(get_container_logs))
        .route("/docker/containers/:id/stats", get(get_container_stats))
        // Image Routes
        .route("/docker/images", get(list_images))
        .route("/docker/images/pull", post(pull_image))
        .route("/docker/images/:id", delete(remove_image))
        // Volume Routes
        .route("/docker/volumes", get(list_volumes))
        .route("/docker/volumes", post(create_volume))
        .route("/docker/volumes/:name", delete(remove_volume))
        // Network Routes
        .route("/docker/networks", get(list_networks))
        .route("/docker/networks", post(create_network))
        .route("/docker/networks/:id", delete(remove_network))
}

#[derive(Debug, Deserialize)]
struct ListContainersQuery {
    all: Option<bool>,
}

async fn list_containers(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Query(query): Query<ListContainersQuery>,
) -> Result<Json<Vec<ContainerInfo>>, DockerApiError> {
    let docker = state.docker
        .ok_or(DockerApiError::DockerNotAvailable)?;

    let containers = docker.list_containers(query.all.unwrap_or(false))
        .await
        .map_err(|e| DockerApiError::DockerError(e.to_string()))?;

    Ok(Json(containers))
}

// ... weitere Handler ...
```

### 3.2 Routes registrieren

```rust
// backend/src/routes/mod.rs
pub mod docker;

pub fn create_router() -> Router<AppState> {
    // ...
    let api_router = Router::new()
        .merge(agents::agents_routes())
        .merge(docker::routes())  // Neu
        .merge(expenses::expenses_routes())
        // ...
}
```

## Phase 4: Frontend Integration

### 4.1 Service: `frontend/src/lib/services/docker.ts`

```typescript
import { ApiClient } from "./api-client";

export interface Container {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  created: number;
}

export interface ContainerDetails extends Container {
  ports: PortMapping[];
  volumes: VolumeMount[];
  networks: string[];
  env: string[];
  started?: number;
}

export interface ContainerStats {
  cpu_usage_percent: number;
  memory_usage_bytes: number;
  memory_limit_bytes: number;
  memory_usage_percent: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
}

export async function listContainers(
  all: boolean = false
): Promise<Container[]> {
  const response = await ApiClient.get(`/docker/containers?all=${all}`);
  if (!response.ok) throw new Error("Failed to fetch containers");
  return response.json();
}

export async function getContainer(id: string): Promise<ContainerDetails> {
  const response = await ApiClient.get(`/docker/containers/${id}`);
  if (!response.ok) throw new Error("Failed to fetch container");
  return response.json();
}

export async function startContainer(id: string): Promise<void> {
  const response = await ApiClient.post(`/docker/containers/${id}/start`);
  if (!response.ok) throw new Error("Failed to start container");
}

export async function stopContainer(id: string): Promise<void> {
  const response = await ApiClient.post(`/docker/containers/${id}/stop`);
  if (!response.ok) throw new Error("Failed to stop container");
}

export async function restartContainer(id: string): Promise<void> {
  const response = await ApiClient.post(`/docker/containers/${id}/restart`);
  if (!response.ok) throw new Error("Failed to restart container");
}

export async function removeContainer(
  id: string,
  force: boolean = false
): Promise<void> {
  const response = await ApiClient.delete(
    `/docker/containers/${id}?force=${force}`
  );
  if (!response.ok) throw new Error("Failed to remove container");
}

export async function getContainerLogs(
  id: string,
  tail: number = 100
): Promise<string[]> {
  const response = await ApiClient.get(
    `/docker/containers/${id}/logs?tail=${tail}`
  );
  if (!response.ok) throw new Error("Failed to fetch logs");
  return response.json();
}

export async function getContainerStats(id: string): Promise<ContainerStats> {
  const response = await ApiClient.get(`/docker/containers/${id}/stats`);
  if (!response.ok) throw new Error("Failed to fetch stats");
  return response.json();
}
```

### 4.2 Container Liste: `frontend/src/routes/containers/+page.svelte`

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { listContainers, startContainer, stopContainer } from "$lib/services/docker";
  import type { Container } from "$lib/services/docker";
  import * as Table from "$lib/components/ui/table";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";

  let containers = $state<Container[]>([]);
  let loading = $state(true);

  onMount(async () => {
    await loadContainers();
  });

  async function loadContainers() {
    loading = true;
    containers = await listContainers(true);
    loading = false;
  }

  function getStateBadge(state: string) {
    switch (state) {
      case 'running': return 'bg-green-500';
      case 'exited': return 'bg-gray-500';
      case 'paused': return 'bg-yellow-500';
      default: return 'bg-gray-400';
    }
  }
</script>

<div class="space-y-6">
  <div class="flex justify-between">
    <h1 class="text-3xl font-bold">Docker Containers</h1>
    <Button onclick={loadContainers}>Refresh</Button>
  </div>

  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head>Name</Table.Head>
        <Table.Head>Image</Table.Head>
        <Table.Head>State</Table.Head>
        <Table.Head>Status</Table.Head>
        <Table.Head>Actions</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each containers as container}
        <Table.Row>
          <Table.Cell>{container.name}</Table.Cell>
          <Table.Cell>{container.image}</Table.Cell>
          <Table.Cell>
            <Badge class={getStateBadge(container.state)}>
              {container.state}
            </Badge>
          </Table.Cell>
          <Table.Cell>{container.status}</Table.Cell>
          <Table.Cell>
            {#if container.state === 'running'}
              <Button size="sm" onclick={() => stopContainer(container.id)}>Stop</Button>
            {:else}
              <Button size="sm" onclick={() => startContainer(container.id)}>Start</Button>
            {/if}
          </Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>
```

## Phase 5: Testing

### 5.1 Backend Testing

```bash
# Docker Service testen
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/docker/containers

# Container starten
curl -X POST -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/docker/containers/my-container/start
```

### 5.2 Integration Testing

```bash
# Erstelle Test-Container
docker run -d --name test-nginx nginx:alpine

# Teste Backend
curl http://localhost:8000/api/docker/containers

# Cleanup
docker rm -f test-nginx
```

## Phase 6: Weitere Features

### 6.1 Container Logs Streaming

WebSocket-Verbindung für Live-Logs:

```rust
// backend/src/routes/docker.rs
use axum::extract::ws::{WebSocket, WebSocketUpgrade};

async fn container_logs_ws(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_log_stream(socket, id))
}

async fn handle_log_stream(mut socket: WebSocket, container_id: String) {
    // Stream logs to websocket
}
```

### 6.2 Container Metrics Historie

In Datenbank speichern für Langzeit-Monitoring:

```sql
CREATE TABLE container_metrics (
    id UUID PRIMARY KEY,
    container_id VARCHAR(255) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    cpu_usage_percent REAL,
    memory_usage_bytes BIGINT,
    network_rx_bytes BIGINT,
    network_tx_bytes BIGINT
);
```

### 6.3 Docker Compose Support

YAML-Dateien parsen und Multi-Container-Apps deployen.

## Checkliste

- [ ] bollard dependency hinzufügen
- [ ] DockerService implementieren
- [ ] Docker routes erstellen
- [ ] Frontend Service implementieren
- [ ] Container-Liste UI erstellen
- [ ] Container-Details Seite erstellen
- [ ] Start/Stop/Restart Buttons
- [ ] Image Management
- [ ] Volume Management
- [ ] Network Management
- [ ] Logs Viewer
- [ ] Stats Monitoring
- [ ] WebSocket für Live-Logs
- [ ] Container Metrics Historie
- [ ] Docker Compose Support

## Ressourcen

- [Bollard Documentation](https://docs.rs/bollard/latest/bollard/)
- [Docker Engine API](https://docs.docker.com/engine/api/)
- [Docker Socket Permission](https://docs.docker.com/engine/install/linux-postinstall/)
