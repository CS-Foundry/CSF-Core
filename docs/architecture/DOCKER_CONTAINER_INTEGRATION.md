# Docker Container Integration

## Übersicht

Die Docker-Integration ermöglicht es, Docker-Container direkt aus der CSF-Core-Oberfläche zu steuern. Das Backend kommuniziert über den Docker-Socket mit der Docker-Engine.

## Features

✅ **Implementiert:**

- Start, Stop, Restart von Containern
- Automatische Status-Synchronisation mit Docker
- Container-Status in Echtzeit abrufen
- Marketplace-Filter für Docker-Ressourcen
- Bearbeiten von Container-Konfigurationen

## Voraussetzungen

### 1. Docker installieren

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install docker.io

# macOS
brew install --cask docker

# Oder Docker Desktop installieren
```

### 2. Docker-Socket Zugriff

Das Backend benötigt Zugriff auf den Docker-Socket:

```bash
# Linux: Benutzer zur docker-Gruppe hinzufügen
sudo usermod -aG docker $USER
newgrp docker

# Socket-Berechtigung prüfen
ls -la /var/run/docker.sock
```

## Container-Verwaltung

### Container-ID setzen

Wenn Sie eine Ressource vom Typ `docker-container` erstellen, muss die `container_id` gesetzt werden:

```json
{
  "name": "My Nginx Container",
  "resource_type": "docker-container",
  "resource_group_id": "uuid-here",
  "container_id": "container_name_or_id",
  "configuration": {
    "image": "nginx:latest",
    "ports": [{ "container": 80, "host": 8080 }],
    "environment": {
      "ENV_VAR": "value"
    }
  }
}
```

### Bestehende Container verknüpfen

Um einen bereits laufenden Docker-Container zu verknüpfen:

1. **Container-ID ermitteln:**

```bash
docker ps -a
# Oder
docker inspect <container-name> | grep Id
```

2. **Ressource erstellen** mit der Container-ID im Feld `container_id`

### Container-Aktionen

Die folgenden Aktionen sind über die UI oder API verfügbar:

```bash
# Über API
curl -X POST http://localhost:8000/api/resources/{id}/action \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action": "start"}'

# Verfügbare Aktionen: "start", "stop", "restart"
```

## Status-Synchronisation

Der Container-Status wird automatisch synchronisiert:

- **Beim Abrufen:** `GET /api/resources/{id}` prüft den aktuellen Docker-Status
- **Nach Aktionen:** Start/Stop/Restart aktualisieren den Status in Echtzeit
- **Mapping:**
  - Docker `running` → Status `running`
  - Docker `exited`, `dead` → Status `stopped`
  - Andere → Status `error`

## Troubleshooting

### Docker nicht verfügbar

Wenn Docker nicht verfügbar ist, wird folgende Warnung angezeigt:

```
⚠️  Docker service not available: ... Container management will be limited.
```

**Lösungen:**

- Prüfen ob Docker läuft: `docker ps`
- Socket-Berechtigung prüfen: `ls -la /var/run/docker.sock`
- Backend-Logs prüfen

### Container-ID ungültig

Wenn die Container-ID nicht gefunden wird:

```json
{
  "error": "Docker operation failed: Container not found"
}
```

**Lösungen:**

- Container existiert: `docker ps -a | grep <container-id>`
- Container-ID korrekt: Namen beginnen oft mit `/`, das muss in der DB ohne `/` gespeichert sein

### Keine Berechtigungen

```
permission denied while trying to connect to the Docker daemon socket
```

**Lösung:**

```bash
sudo usermod -aG docker $USER
newgrp docker
# Backend neu starten
```

## Beispiel: Nginx Container

1. **Container starten:**

```bash
docker run -d --name my-nginx -p 8080:80 nginx:latest
```

2. **Ressource in CSF-Core erstellen:**

```json
{
  "name": "Production Nginx",
  "resource_type": "docker-container",
  "description": "Main web server",
  "resource_group_id": "your-group-id",
  "container_id": "my-nginx",
  "configuration": {
    "image": "nginx:latest",
    "ports": [{ "container": 80, "host": 8080 }]
  },
  "tags": {
    "environment": "production",
    "service": "web"
  }
}
```

3. **Container über UI steuern:**
   - Öffne `/resources/{id}`
   - Nutze die Buttons: Starten, Stoppen, Neustarten
   - Bearbeite die Konfiguration nach Bedarf

## Roadmap

🔄 **In Arbeit:**

- Container-Logs anzeigen
- Container-Metrics (CPU, RAM) in Echtzeit
- Neue Container direkt erstellen (ohne docker run)
- Docker Compose Stack-Support

📋 **Geplant:**

- Volume-Management
- Network-Management
- Image-Management
- Container-Terminal (WebSocket)

## Backend-Architektur

```
backend/src/
├── docker_service.rs         # Docker-API-Client (Bollard)
├── routes/resources.rs        # REST-Endpunkte
└── main.rs                    # Docker-Service-Initialisierung

docker_service.rs:
- DockerService::new()         → Verbindung zum Socket
- start_container()            → Container starten
- stop_container()             → Container stoppen
- restart_container()          → Container neustarten
- inspect_container()          → Container-Info abrufen
- list_containers()            → Alle Container auflisten
```

## Sicherheit

⚠️ **Wichtig:**

- Docker-Socket-Zugriff gewährt Root-Rechte
- Nur vertrauenswürdige Benutzer sollten Container steuern können
- RBAC-Integration für Container-Management ist empfohlen
- In Produktion: Docker-Socket über TLS absichern

## Support

Bei Problemen:

1. Backend-Logs prüfen: Container starten mit `-v` für verbose logging
2. Docker-Status: `docker info`
3. Socket-Zugriff: `curl --unix-socket /var/run/docker.sock http://localhost/version`
