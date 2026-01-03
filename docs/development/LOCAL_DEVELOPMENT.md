# CSF Core - Local Development auf macOS

## Setup für lokale Entwicklung

### Option 1: PostgreSQL in Docker, Backend & Frontend lokal (Empfohlen)

Dies ist ideal für die Entwicklung, da du schnelle Rebuild-Zyklen hast und direkt debuggen kannst.

#### 1. PostgreSQL starten

```bash
# Starte nur PostgreSQL in Docker
docker-compose -f docker-compose.local.yml up -d

# Überprüfe ob PostgreSQL läuft
docker-compose -f docker-compose.local.yml ps
```

#### 2. Backend starten

```bash
# Terminal 1: Backend
cd backend
cargo run
```

Das Backend läuft nun auf `http://localhost:8000`

#### 3. Frontend starten

```bash
# Terminal 2: Frontend
cd frontend
npm install  # Falls noch nicht geschehen
npm run dev
```

Das Frontend läuft nun auf `http://localhost:5173`

#### 4. Testen

```bash
# System-Metriken abrufen
# Erst einloggen
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin"}'

# Token speichern (aus der Response)
export TOKEN="dein-token-hier"

# System Info
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/system/info | jq

# System Metriken
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/api/system/metrics | jq
```

#### 5. Aufräumen

```bash
# PostgreSQL stoppen
docker-compose -f docker-compose.local.yml down

# PostgreSQL stoppen und Daten löschen
docker-compose -f docker-compose.local.yml down -v
```

---

### Option 2: Alles in Docker (Wie im Production)

Wenn du das komplette Setup in Docker testen willst:

```bash
# Development-Stack starten
docker-compose -f docker-compose.dev.yml up --build

# Backend: http://localhost:8000
# Frontend: http://localhost:3000
# PostgreSQL: localhost:5432
```

**Wichtig:** Bei dieser Option läuft das Backend im Container und sammelt die System-Metriken **des Containers**, nicht deines Mac-Systems!

---

## Schnell-Start (Empfohlener Workflow)

```bash
# 1. PostgreSQL starten
docker-compose -f docker-compose.local.yml up -d

# 2. Backend in einem Terminal
cd backend && cargo run

# 3. Frontend in anderem Terminal
cd frontend && npm run dev

# 4. Browser öffnen
open http://localhost:5173/local-system
```

---

## Umgebungsvariablen

Die `.env` Datei im `backend/` Verzeichnis ist für lokale Entwicklung konfiguriert:

```dotenv
DATABASE_URL=postgres://csf_user:csf_password@localhost:5432/csf_core
RUST_LOG=debug,tower_http=debug,backend=info
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
FRONTEND_URL=http://localhost:5173
```

- `DATABASE_URL`: Zeigt auf `localhost:5432` (PostgreSQL im Docker)
- `FRONTEND_URL`: Zeigt auf Vite Dev Server (`localhost:5173`)

---

## Troubleshooting

### PostgreSQL Connection Error

```
error communicating with database: failed to lookup address information
```

**Lösung:** PostgreSQL ist nicht gestartet

```bash
docker-compose -f docker-compose.local.yml up -d
```

### Port bereits in Verwendung

Wenn Port 8000 oder 3000 bereits verwendet wird:

```bash
# Finde Prozess auf Port 8000
lsof -ti:8000 | xargs kill -9

# Finde Prozess auf Port 3000
lsof -ti:3000 | xargs kill -9
```

### Frontend kann Backend nicht erreichen

Überprüfe die CORS-Einstellungen im Backend. Das Backend sollte `FRONTEND_URL=http://localhost:5173` haben.

### Docker-Metriken auf macOS

Da du auf macOS entwickelst, zeigt das System-Monitoring die Metriken deines **Mac-Systems** an. Das ist völlig normal und funktioniert dank der `sysinfo` crate plattformübergreifend.

Wenn du später Docker-Container verwalten willst, stelle sicher dass Docker Desktop für Mac läuft:

```bash
# Überprüfe Docker
docker ps
```

---

## Datenbank-Management

### Datenbank zurücksetzen

```bash
# Stoppe PostgreSQL
docker-compose -f docker-compose.local.yml down -v

# Starte neu (erstellt neue Datenbank)
docker-compose -f docker-compose.local.yml up -d

# Warte kurz, dann Backend starten
cd backend && cargo run
```

### Datenbank direkt zugreifen

```bash
# Verbinde zu PostgreSQL
docker exec -it csf-core-postgres-1 psql -U csf_user -d csf_core

# Oder mit psql auf deinem Mac
psql -h localhost -U csf_user -d csf_core
```

---

## Hot Reload

- **Backend**: Nutze `cargo-watch` für Auto-Reload

  ```bash
  cargo install cargo-watch
  cargo watch -x run
  ```

- **Frontend**: Vite hat automatisch Hot-Reload aktiviert

---

## Next Steps

1. ✅ PostgreSQL in Docker läuft
2. ✅ Backend lokal läuft und sammelt Mac-Metriken
3. ✅ Frontend läuft und zeigt Dashboard
4. 🔄 Docker-Integration für Container-Management hinzufügen

Siehe [DOCKER_INTEGRATION_PLAN.md](../deployment/DOCKER_INTEGRATION_PLAN.md) für die nächsten Schritte!
