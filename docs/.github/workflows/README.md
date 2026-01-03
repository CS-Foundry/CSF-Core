# GitHub Actions Workflows

Dieses Verzeichnis enthält alle CI/CD Workflows für FinanceVault.

## Workflows

### 📦 `release.yml` - Semantic Release

Automatisches Versioning und Release-Erstellung basierend auf Conventional Commits.

**Trigger:** Push auf `main`

**Funktionen:**

- 🔍 Analysiert Commits
- 🏷️ Erstellt neue Version (semver)
- 📝 Generiert CHANGELOG
- 🔄 Aktualisiert package.json und Cargo.toml
- 📦 Erstellt GitHub Release
- 🏷️ Erstellt Git Tag

### 🐳 `docker-build-push.yml` - Docker Build & Push

Baut Multi-Arch Docker Images und pushed sie zu GitHub Container Registry.

**Trigger:** Aufgerufen durch Main Release Pipeline oder Git Tags (`v*`)

**Funktionen:**

- 🏗️ Multi-Architecture Build (AMD64, ARM64)
- 📤 Push zu ghcr.io
- 🏷️ **Saubere Tags** (nur Version, z.B. `0.1.0` und `latest`)
- 💾 Build Cache für schnellere Builds
- 🎯 **Automatische Architektur-Wahl** beim Pull

**Wichtig**: Es werden **keine** `-amd64` oder `-arm64` Tags mehr erstellt. Docker wählt automatisch die passende Architektur beim Pull.

### 🚀 `main-release.yml` - Main Release Pipeline

Orchestriert den gesamten Release-Prozess.

**Trigger:** Push auf `main` oder manuell

**Ablauf:**

1. **Semantic Release**: Erstellt Release und Version
2. **Docker Build**: Baut und pushed Multi-Arch Images (nur bei neuem Release)
3. **Summary**: Zeigt Pipeline-Übersicht

## 🐳 Docker Registry Struktur

### Erwartete Tags in GHCR

Nach einem erfolgreichen Release:

```
ghcr.io/codemaster4711/financevault:latest
ghcr.io/codemaster4711/financevault:0.1.0
```

**Keine** `-amd64` oder `-arm64` Suffixe!

### Pull Behavior

```bash
# Docker wählt automatisch die richtige Architektur
docker pull ghcr.io/codemaster4711/financevault:latest
```

## Scripts

### `scripts/update-versions.sh`

Helper-Script zum Aktualisieren der Versionen in:

- `package.json` (Root)
- `frontend/package.json`
- `backend/Cargo.toml`

## Dokumentation

Siehe [CICD.md](./CICD.md) für detaillierte Workflow-Dokumentation.
