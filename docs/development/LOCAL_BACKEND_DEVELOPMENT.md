# Lokales Backend Development Setup

Dieses Setup ermöglicht es, das Backend lokal zu entwickeln, während Frontend und Postgres in Docker-Containern laufen.

## Vorteile

- Schnellere Backend-Kompilierung und Hot-Reload
- Einfacheres Debugging mit IDE
- Direkter Zugriff auf Backend-Prozess
- Frontend und Postgres bleiben isoliert in Docker

## Setup

### 1. Voraussetzungen

```bash
# Rust installiert
cargo --version

# Docker läuft
docker --version
```

### 2. Container starten

```bash
# Option A: Mit Skript
./scripts/start-local-backend.sh

# Option B: Manuell
docker-compose -f docker-compose.local-backend.yml up -d
```

Das startet:

- ✅ Postgres Container (Port 5432)
- ✅ Frontend Container (Port 3000)

### 3. Backend lokal starten

```bash
cd backend

# Database URL für lokale Entwicklung
export DATABASE_URL="postgres://csf_user:csf_password@localhost:5432/csf_core"
export RUST_LOG=debug

# Backend starten
cargo run

# Oder mit Auto-Reload
cargo watch -x run
```

### 4. Zugriff

- **Frontend**: http://localhost:3000
- **Backend**: http://localhost:8000
- **Postgres**: localhost:5432

## CORS Konfiguration

Das Backend ist so konfiguriert, dass es Requests von folgenden Origins akzeptiert:

- `http://localhost:3000` (Frontend im Docker)
- `http://localhost:8000` (Backend lokal)
- `http://127.0.0.1:3000`
- `http://127.0.0.1:8000`
- Konfigurierbarer `FRONTEND_URL` aus `.env`

## Umgebungsvariablen

Stelle sicher, dass deine `.env` Datei folgendes enthält:

```env
DATABASE_URL=postgres://csf_user:csf_password@localhost:5432/csf_core
JWT_SECRET=dein-geheimer-jwt-key
RUST_LOG=debug
```

## Troubleshooting

### CORS Fehler

Stelle sicher, dass das Frontend `host.docker.internal:8000` als Backend-URL verwendet:

```bash
docker-compose -f docker-compose.local-backend.yml logs frontend-dev | grep PUBLIC_API_BASE_URL
```

### Backend kann nicht auf Postgres zugreifen

Überprüfe, ob der Postgres Container läuft und Port 5432 exposed ist:

```bash
docker ps | grep postgres
psql -h localhost -U csf_user -d csf_core
```

### Frontend kann Backend nicht erreichen

1. Prüfe, ob das Backend auf Port 8000 läuft:

   ```bash
   curl http://localhost:8000/api/system/health
   ```

2. Überprüfe die Frontend-Logs:
   ```bash
   docker-compose -f docker-compose.local-backend.yml logs frontend-dev
   ```

## Container stoppen

```bash
docker-compose -f docker-compose.local-backend.yml down

# Mit Volumes löschen
docker-compose -f docker-compose.local-backend.yml down -v
```

## Zurück zu vollständigem Docker Setup

```bash
# Container stoppen
docker-compose -f docker-compose.local-backend.yml down

# Normal starten
docker-compose -f docker-compose.dev.yml up -d
```
