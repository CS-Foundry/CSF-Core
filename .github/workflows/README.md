# GitHub Actions Workflows

## Übersicht

### Release & Deployment Workflows

#### `main-release.yml` (Haupt-Release-Pipeline)

Läuft automatisch bei jedem Push auf `main`:

1. **Semantic Release** - Erstellt neue Releases basierend auf Conventional Commits
2. **Docker Build Backend** - Baut und pusht Backend-Image nach ghcr.io
3. **Docker Build Frontend** - Baut und pusht Frontend-Image nach ghcr.io
4. **Summary** - Zeigt Übersicht aller Artefakte

**Outputs:**

- GitHub Release mit Binaries
- Docker Images: `ghcr.io/cs-foundry/csf-core-backend:latest` & `:version`
- Docker Images: `ghcr.io/cs-foundry/csf-core-frontend:latest` & `:version`

#### `release.yml` (Wiederverwendbarer Release-Workflow)

Wird von `main-release.yml` aufgerufen:

- Führt Semantic Release aus
- Baut Backend-Binaries (Linux/macOS)
- Baut Frontend-Package
- Lädt alle Artefakte zum Release hoch

#### `docker-build-manual.yml` (Manuelles Docker-Build)

Manueller Workflow für Docker-Builds:

- Auswahl: Backend, Frontend oder beides
- Eigene Versionsnummer angeben
- Erstellt Tags: `<version>` und `manual-latest`

### Weitere Workflows

#### `beta-release.yml`

Release-Pipeline für Beta-Versionen auf dem `beta` Branch

#### `docker-build-push.yml`

Legacy-Workflow für das vereinigte Backend+Frontend Image

#### `build-artifacts.yml`

Standalone-Workflow für Binary-Builds

#### `lint.yml`

Code-Quality-Checks (Rust, TypeScript, etc.)

## Verwendung

### Automatischer Release (main)

```bash
git commit -m "feat: neue Feature"
git push origin main
# → Automatischer Release + Docker Images
```

### Manueller Docker-Build

1. GitHub Actions → **Manual Docker Build**
2. **Run workflow** klicken
3. Version eingeben (z.B. `1.2.3`)
4. Target auswählen (backend/frontend/both)
5. **Run workflow** ausführen

## Image-URLs

Nach erfolgreichem Build sind die Images verfügbar unter:

```bash
# Backend
ghcr.io/cs-foundry/csf-core-backend:latest
ghcr.io/cs-foundry/csf-core-backend:<version>

# Frontend
ghcr.io/cs-foundry/csf-core-frontend:latest
ghcr.io/cs-foundry/csf-core-frontend:<version>
```

## Permissions

Die Workflows benötigen folgende Permissions:

- `contents: write` - Für Releases
- `packages: write` - Für Docker Registry
- `issues: write` - Für Issue-Updates
- `pull-requests: write` - Für PR-Updates

## Secrets

Keine zusätzlichen Secrets erforderlich - verwendet `GITHUB_TOKEN` automatisch.

## Weitere Dokumentation

- [Docker Registry Integration](../docs/deployment/DOCKER_REGISTRY.md)
- [NixOS Deployment](../docs/deployment/DEPLOYMENT.md)
- [Installation Guide](../docs/deployment/INSTALLATION.md)
