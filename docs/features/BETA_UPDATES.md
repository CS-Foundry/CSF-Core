# CSF-Core Update-System mit Beta-Support

## Übersicht

Das Update-System wurde erweitert, um sowohl stabile als auch Beta-Versionen zu unterstützen.

## Änderungen

### 1. Semantic Release Konfiguration

**Datei:** `.releaserc.json`

- Beta-Branch wurde hinzugefügt mit `prerelease: true`
- Main-Branch erstellt weiterhin normale Releases
- Beta-Branch erstellt Pre-Release Versionen mit Beta-Tag

### 2. GitHub Actions Workflows

**Neue Datei:** `.github/workflows/beta-release.yml`

- Pipeline für Beta-Branch mit eigenem Release-Workflow
- Warnung in der Summary über Beta-Status
- Separate Installationsanweisungen für Beta-Versionen

**Bestehend:** `.github/workflows/main-release.yml`

- Bleibt unverändert für stabile Releases auf Main-Branch

### 3. Backend API Erweiterungen

**Datei:** `backend/src/routes/updates.rs`

#### Bugfixes:

- PathBuf Type-Mismatch behoben
- Iterator Type-Annotation hinzugefügt

#### Neue Features:

- `VersionInfo` erweitert um:
  - `is_prerelease: bool`
  - `latest_beta_version: Option<String>`
  - `beta_release_url: Option<String>`
- `GitHubRelease` struct erweitert um `prerelease: bool`
- `/api/updates/check` ruft nun alle Releases ab und findet die neueste Beta-Version
- Beta-Versionen werden nur zurückgegeben, wenn verfügbar

### 4. Frontend Update-Settings

**Datei:** `frontend/src/lib/components/settings/UpdateSettings.svelte`

#### Neue Features:

- **Beta-Updates Toggle**: Aktiviert/deaktiviert Beta-Update-Prüfung
- **Sicherheitswarnung**:
  - Zeigt deutliche Warnung bei Aktivierung von Beta-Updates
  - Listet Risiken auf (Instabilität, Datenverlust, etc.)
  - Benutzer muss Risiken explizit bestätigen
- **Beta-Version Anzeige**:
  - Zeigt verfügbare Beta-Version mit orangem Badge
  - Separater "Beta installieren" Button
  - Funktioniert unabhängig von stabilen Updates

#### UI-Komponenten:

- Switch-Komponente für Beta-Toggle
- AlertTriangle-Icon für Warnungen
- Separate Button-Styles für Beta-Installation

### 5. Services und Stores

**Datei:** `frontend/src/lib/services/updates.ts`

- `VersionInfo` Interface erweitert um Beta-Felder

**Datei:** `frontend/src/lib/stores/updates.ts`

- Keine Änderungen nötig, da bereits flexibel für neue VersionInfo-Felder

### 6. Update-Script

**Datei:** `scripts/update.sh`

- Unterstützt bereits Beta-Versionen (Version als Parameter)
- Lädt Release von GitHub basierend auf Tag
- Funktioniert für v1.0.0 und v1.0.0-beta.1 gleichermaßen

## Verwendung

### Für Entwickler

#### Beta-Release erstellen:

```bash
# Auf Beta-Branch wechseln
git checkout beta

# Conventional Commits verwenden
git commit -m "feat: neue experimentelle Funktion"

# Push triggert Beta-Release
git push origin beta
```

#### Stabiles Release erstellen:

```bash
# Auf Main-Branch wechseln
git checkout main

# Conventional Commits verwenden
git commit -m "feat: neue stabile Funktion"

# Push triggert stabiles Release
git push origin main
```

### Für Benutzer

#### Beta-Updates aktivieren:

1. Öffne Settings → Updates
2. Aktiviere "Beta-Updates aktivieren"
3. Lies und bestätige die Sicherheitswarnung
4. Beta-Version wird angezeigt (falls verfügbar)
5. Klicke "Beta installieren"

#### Normale Updates:

- Funktioniert wie bisher
- Zeigt nur stabile Versionen
- Keine Beta-Versionen sichtbar (außer Beta-Toggle ist an)

## Sicherheitshinweise

### ⚠️ Beta-Versionen Warnung

Beta-Versionen sind experimentell und können:

- Instabil sein und Fehler enthalten
- Unerwartetes Verhalten zeigen
- Datenverlust verursachen
- Funktionen die sich ohne Vorankündigung ändern

**Nicht für Produktionsumgebungen empfohlen!**

### API-Verhalten

- `/api/updates/check` gibt immer die neueste stabile Version zurück
- Beta-Informationen sind optional und nur für aktivierte Beta-Updates relevant
- Backend prüft keine User-Präferenzen (Beta-Toggle ist nur Frontend)

## Technische Details

### Version-Tagging

- Stabile Versionen: `v1.0.0`, `v1.0.1`, etc.
- Beta-Versionen: `v1.1.0-beta.1`, `v1.1.0-beta.2`, etc.

### Semantic Release

- Main-Branch: Normal releases
- Beta-Branch: Pre-releases mit `-beta.X` Suffix
- Conventional Commits für beide Branches

### GitHub Releases

- API filtert nach `prerelease` Flag
- Backend findet automatisch neueste Beta-Version
- Releases haben separate URLs für Stable und Beta

## Testing

### Backend:

```bash
cd backend
cargo check  # Sollte ohne Fehler durchlaufen
cargo test   # Tests ausführen
```

### Frontend:

```bash
cd frontend
npm run check  # TypeScript Prüfung
npm run build  # Build Test
```

## Migration bestehender Installationen

Keine Änderungen an bestehenden Installationen nötig:

- Beta-Updates sind standardmäßig deaktiviert
- API ist abwärtskompatibel
- Neue Felder in VersionInfo sind optional

## Rollback

Falls Probleme auftreten, siehe Update-Script Backup-Anweisungen:

```bash
sudo rm -rf /opt/csf-core
sudo mv /tmp/csf-core-backup-TIMESTAMP/csf-core /opt/csf-core
sudo systemctl restart csf-core.service
```
