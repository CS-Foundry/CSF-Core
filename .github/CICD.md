# CI/CD Workflow Documentation 🚀

## Überblick

Das FinanceVault Projekt nutzt einen **automatisierten Release- und Build-Prozess** mit semantic-release und GitHub Actions.

## Workflow-Ablauf

### 1. Release Pipeline (`release.yml`)

**Trigger:** Push auf den `main` Branch

**Ablauf:**
1. ✅ Commit-Analyzer prüft Commit-Messages (Conventional Commits)
2. 📊 Bestimmt die neue Version (major/minor/patch)
3. 📝 Generiert CHANGELOG.md
4. 🔄 Aktualisiert Versionen in:
   - `package.json` (Root)
   - `frontend/package.json`
   - `backend/Cargo.toml`
5. 🏷️ Erstellt Git Tag (z.B. `v1.2.3`)
6. 📦 Erstellt GitHub Release
7. ⚙️ Committed Änderungen zurück (`[skip ci]`)

**Ausgabe:**
- GitHub Release mit Changelog
- Versionierte Dateien im Repository
- Git Tag für Docker Build

### 2. Docker Build Pipeline (`docker-build-push.yml`)

**Trigger:** 
- Git Tag `v*` (wird vom Release-Workflow erstellt)
- Manueller Trigger (`workflow_dispatch`)

**Ablauf:**
1. 🏗️ Baut AMD64 Image
2. 🏗️ Baut ARM64 Image  
3. 🔗 Erstellt Multi-Arch Manifest
4. 📤 Pushed zu GitHub Container Registry

**Tags:**
- `v1.2.3` (exakte Version vom Git Tag)
- `v1.2` (Major.Minor)
- `latest` (nur für main branch)
- `main-sha123-amd64` / `main-sha123-arm64` (Arch-spezifisch)

## Commit Message Format

Wir verwenden [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types und ihre Auswirkung:

| Type       | Beschreibung              | Release | Beispiel                                    |
|------------|---------------------------|---------|---------------------------------------------|
| `feat:`    | Neues Feature            | MINOR   | `feat(auth): add OAuth2 support`           |
| `fix:`     | Bugfix                   | PATCH   | `fix(api): correct expense calculation`    |
| `perf:`    | Performance              | PATCH   | `perf(db): optimize query performance`     |
| `refactor:`| Code-Umstrukturierung   | PATCH   | `refactor(ui): restructure components`     |
| `build:`   | Build-System            | PATCH   | `build(docker): update base image`         |
| `docs:`    | Dokumentation           | NONE    | `docs(readme): update installation guide`  |
| `style:`   | Code-Style              | NONE    | `style(lint): fix formatting issues`       |
| `test:`    | Tests                   | NONE    | `test(auth): add unit tests`               |
| `ci:`      | CI/CD                   | NONE    | `ci(actions): update workflow`             |
| `chore:`   | Wartungsarbeiten        | NONE    | `chore(deps): update dependencies`         |

### Breaking Changes

Für **Major Version** Bump:

```
feat(api)!: redesign authentication API

BREAKING CHANGE: The authentication endpoint now requires OAuth2 tokens instead of JWT
```

## Beispiel-Workflow

### Feature entwickeln und releasen:

```bash
# 1. Feature Branch erstellen
git checkout -b feat/new-dashboard

# 2. Änderungen entwickeln
# ... Code ändern ...

# 3. Committen mit Conventional Commits
git commit -m "feat(dashboard): add expense overview widget"
git commit -m "feat(ui): add dark mode toggle"

# 4. Push und PR erstellen
git push origin feat/new-dashboard

# 5. PR Review und Merge in main
# Nach dem Merge:
# ✅ Release Pipeline läuft
# ✅ Neue Version wird erstellt (z.B. v0.2.0)
# ✅ Git Tag wird erstellt
# ✅ Docker Build wird getriggert
# ✅ Image wird mit v0.2.0 getaggt und gepushed
```

### Bugfix releasen:

```bash
git checkout -b fix/expense-calculation

# Bugfix entwickeln
git commit -m "fix(api): correct decimal rounding in expenses"

# Merge in main → Release v0.2.1
```

## Version Management

Die Version wird automatisch in folgenden Dateien aktualisiert:

- ✅ `/package.json`
- ✅ `/frontend/package.json`
- ✅ `/backend/Cargo.toml`
- ✅ `/CHANGELOG.md`

**Wichtig:** Versionen **NIE manuell** ändern! Immer über semantic-release.

## Docker Images abrufen

Nach einem erfolgreichen Release:

```bash
# Latest Version
docker pull ghcr.io/codemaster4711/financevault:latest

# Spezifische Version
docker pull ghcr.io/codemaster4711/financevault:v0.2.0

# Major.Minor Version
docker pull ghcr.io/codemaster4711/financevault:v0.2

# Spezifische Architektur (falls nötig)
docker pull ghcr.io/codemaster4711/financevault:v0.2.0-amd64
docker pull ghcr.io/codemaster4711/financevault:v0.2.0-arm64
```

## Troubleshooting

### Release wird nicht erstellt

**Ursache:** Keine relevanzbaren Commits seit letztem Release

**Lösung:** 
- Prüfe ob commits `feat:`, `fix:`, etc. verwenden
- Prüfe GitHub Actions Log

### Docker Build schlägt fehl

**Ursache:** Build-Fehler oder fehlende Permissions

**Lösung:**
1. Check `docker-build-push.yml` Actions Log
2. Teste Build lokal: `docker build -t test .`
3. Prüfe ob `GITHUB_TOKEN` Permissions hat

### Version wird nicht aktualisiert

**Ursache:** Update-Script schlägt fehl

**Lösung:**
1. Check `release.yml` Actions Log (Schritt "Run semantic-release")
2. Prüfe ob `.github/scripts/update-versions.sh` ausführbar ist
3. Teste Script lokal

## Manueller Release

Falls nötig, kann ein Release manuell getriggert werden:

1. Gehe zu Actions → Release
2. Klicke "Run workflow"
3. Wähle Branch `main`
4. Klicke "Run workflow"

## Weitere Informationen

- 📖 [Semantic Release Docs](https://semantic-release.gitbook.io/)
- 📖 [Conventional Commits](https://www.conventionalcommits.org/)
- 📖 [GitHub Actions Docs](https://docs.github.com/actions)
