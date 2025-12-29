# CSF-Core Auto-Update System

Dieses System ermöglicht automatische Updates für CSF-Core ohne manuelle Kompilierung.

## Komponenten

### 1. CI/CD Pipeline (`.github/workflows/build-artifacts.yml`)

Die GitHub Actions Workflow erstellt automatisch:

- **Backend Binary** (Rust)
- **Agent Binary** (Rust)
- **Frontend Build** (SvelteKit)
- **Release Artifacts** mit Checksums
- **Release Manifest** mit Versionsinformationen

**Trigger:**

- Push auf `main` oder `feat/server-monitoring` Branch
- Git Tags (`v*`)
- Releases
- Manueller Workflow Dispatch

**Artifacts:**

- `csf-core-linux-amd64.tar.gz`
- `csf-core-linux-arm64.tar.gz`
- Checksums für beide Archives
- `release-manifest.json`

### 2. Binary Installation Script (`scripts/install-binary.sh`)

Installiert CSF-Core ohne Kompilierung:

```bash
# Standard Installation (latest release)
sudo bash scripts/install-binary.sh

# Spezifische Version
sudo VERSION=v1.0.0 bash scripts/install-binary.sh

# Mit Branch
sudo BRANCH=feat/server-monitoring bash scripts/install-binary.sh
```

**Features:**

- Lädt vorkompilierte Binaries herunter
- Verifiziert Checksums
- Installiert als systemd Service
- Konfiguriert PostgreSQL oder SQLite
- Generiert Sicherheitsschlüssel

### 3. Backend Update-API (`backend/src/routes/updates.rs`)

Endpoint: `GET /api/system/version`

**Response:**

```json
{
  "current_version": "0.1.0",
  "current_commit": "abc123",
  "latest_version": "0.2.0",
  "latest_commit": "def456",
  "update_available": true,
  "download_url": "https://github.com/CS-Foundry/CSF-Core/releases/download/v0.2.0/csf-core-linux-amd64.tar.gz",
  "release_notes": "## Changes\n- Feature A\n- Bug fix B",
  "published_at": "2025-01-15T10:00:00Z"
}
```

**Features:**

- Prüft GitHub Releases API
- Vergleicht Versionen
- Gibt Download-URL zurück
- Inkludiert Release Notes

### 4. Frontend Update-Benachrichtigung (`frontend/src/lib/components/navbar/update-notifier.svelte`)

**Features:**

- Zeigt Banner in Sidebar wenn Update verfügbar
- Automatische Prüfung alle 30 Minuten
- "Herunterladen" Button öffnet Release-Seite
- "Später" Button zum Ausblenden

**Integration:**
Die Komponente ist in `app-sidebar.svelte` eingebunden.

### 5. Update Script (`scripts/update.sh`)

Aktualisiert eine bestehende Installation:

```bash
# Prüfe ob Update verfügbar
sudo ./scripts/update.sh --check

# Update installieren
sudo ./scripts/update.sh

# Update erzwingen (ohne Bestätigung)
sudo ./scripts/update.sh --force

# Backup wiederherstellen
sudo ./scripts/update.sh --restore
```

**Features:**

- Erstellt automatisch Backup vor Update
- Lädt neue Version herunter
- Verifiziert Checksums
- Stoppt/Startet Services automatisch
- Kann Backup wiederherstellen bei Fehler
- Behält letzte 5 Backups

**Update-Prozess:**

1. Prüft aktuelle Version
2. Ermittelt neueste Version
3. Erstellt Backup
4. Stoppt Services
5. Lädt neue Binaries herunter
6. Installiert Update
7. Startet Services
8. Verifiziert Installation

## Verwendung

### Erste Installation

```bash
# Clone Repository
git clone https://github.com/CS-Foundry/CSF-Core.git
cd CSF-Core

# Installation mit Binaries (empfohlen)
sudo bash scripts/install-binary.sh

# ODER: Installation aus Quellcode (wie bisher)
sudo bash scripts/install.sh
```

### Update durchführen

**Option 1: Über Frontend UI**

1. Update-Benachrichtigung erscheint in Sidebar
2. Klick auf "Herunterladen"
3. Führe Update-Script aus: `sudo /opt/csf-core/scripts/update.sh`

**Option 2: Manuell über CLI**

```bash
# Prüfe auf Updates
sudo /opt/csf-core/scripts/update.sh --check

# Update installieren
sudo /opt/csf-core/scripts/update.sh
```

### Automatische Updates (Optional)

Erstelle einen Cron-Job für automatische Update-Prüfung:

```bash
# Editiere Crontab
sudo crontab -e

# Füge hinzu (prüft täglich um 3 Uhr morgens)
0 3 * * * /opt/csf-core/scripts/update.sh --force > /var/log/csf-core/update.log 2>&1
```

## Release-Prozess

### 1. Code ändern und committen

```bash
git add .
git commit -m "feat: neue feature"
git push
```

### 2. Release erstellen

```bash
# Tag erstellen
git tag v1.0.0
git push origin v1.0.0

# Oder: GitHub Release UI verwenden
```

### 3. GitHub Actions baut automatisch

- Workflow startet automatisch
- Erstellt Binaries für amd64 und arm64
- Uploaded Artifacts zu Release
- Erstellt Release Manifest

### 4. User werden benachrichtigt

- Backend prüft alle 30 Minuten auf neue Version
- Update-Banner erscheint in Frontend
- User können Update herunterladen und installieren

## Architektur-Details

### Version-Tracking

Versionen werden gespeichert in:

- `/opt/csf-core/VERSION` - Version-String (z.B. "1.0.0")
- `/opt/csf-core/COMMIT` - Git Commit Hash

### Backups

Backups werden gespeichert unter:

- `/var/backups/csf-core/csf-core-backup-YYYYMMDD-HHMMSS.tar.gz`
- Letzte 5 Backups werden behalten

### Service-Management

Services:

- `csf-core.service` - Backend + Frontend
- `csf-core-agent.service` - Monitoring Agent

Kommandos:

```bash
# Status prüfen
systemctl status csf-core
systemctl status csf-core-agent

# Logs anzeigen
journalctl -u csf-core -f
journalctl -u csf-core-agent -f

# Neustart
systemctl restart csf-core
systemctl restart csf-core-agent
```

## Fehlerbehebung

### Update fehlgeschlagen

```bash
# Backup wiederherstellen
sudo /opt/csf-core/scripts/update.sh --restore

# Services prüfen
systemctl status csf-core
journalctl -u csf-core -n 50
```

### Update-Check funktioniert nicht

```bash
# API manuell testen
curl http://localhost:8080/api/system/version

# Oder von außen
curl https://your-domain.com/api/system/version
```

### Download schlägt fehl

Mögliche Ursachen:

- Keine Internetverbindung
- GitHub API Rate Limit erreicht
- Release existiert nicht

```bash
# Prüfe GitHub API
curl https://api.github.com/repos/CS-Foundry/CSF-Core/releases/latest

# Prüfe Rate Limit
curl https://api.github.com/rate_limit
```

## Sicherheit

### Checksum-Verifikation

Alle Downloads werden mit SHA256 Checksums verifiziert:

```bash
sha256sum -c csf-core-linux-amd64.tar.gz.sha256
```

### HTTPS

Alle Downloads erfolgen über HTTPS (GitHub):

- `https://github.com/CS-Foundry/CSF-Core/releases/download/...`

### Backup-Strategie

- Automatisches Backup vor jedem Update
- Letzte 5 Backups werden behalten
- Manuelles Restore möglich

## Testing auf Test-Branch

Das System ist aktuell auf dem `feat/server-monitoring` Branch verfügbar:

```bash
# Clone und wechsle zu Test-Branch
git clone https://github.com/CS-Foundry/CSF-Core.git
cd CSF-Core
git checkout feat/server-monitoring

# Push um Build zu triggern
git push origin feat/server-monitoring
```

Die GitHub Actions Workflow baut automatisch für diesen Branch und erstellt Pre-Release Artifacts.

## Migration von alter Installation

Wenn du bereits eine Installation mit `install.sh` hast:

```bash
# 1. Backup erstellen
sudo tar -czf /tmp/csf-core-backup.tar.gz /opt/csf-core /var/lib/csf-core

# 2. Update-Script kopieren
sudo cp scripts/update.sh /opt/csf-core/scripts/

# 3. Version-Datei erstellen (falls nicht vorhanden)
echo "0.1.0" | sudo tee /opt/csf-core/VERSION

# 4. Update durchführen
sudo /opt/csf-core/scripts/update.sh
```

## Zukünftige Erweiterungen

- [ ] Automatische Rollback bei fehlgeschlagenem Update
- [ ] Email-Benachrichtigung bei verfügbaren Updates
- [ ] Update-History in Frontend anzeigen
- [ ] Staging-Updates (Beta-Versionen testen)
- [ ] Delta-Updates (nur geänderte Dateien)
