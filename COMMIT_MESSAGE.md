# Commit Message für diesen Fix

```
fix: include production node_modules in frontend package and add download stats

BREAKING CHANGE: None

Fixes:
- Frontend-Build enthält jetzt node_modules mit Production-Dependencies
- Behebt "Cannot find package 'jsonwebtoken'" Fehler bei Installation
- Download-Statistiken werden im GitHub Actions Workflow-Summary angezeigt

Changes:
1. .github/workflows/release.yml:
   - npm ci --omit=dev nach Build ausführen
   - node_modules/ ins Frontend-Tarball einpacken
   - Download-Stats Job hinzugefügt mit GitHub API

2. .github/workflows/main-release.yml:
   - Download-Statistiken im Summary anzeigen
   - Release-Asset-Info mit Größe und Download-Count

3. scripts/install.sh:
   - Kommentar hinzugefügt dass node_modules enthalten sind
   - Kein npm install mehr nötig nach Extraktion

4. Neue Dateien:
   - TROUBLESHOOTING.md: Komplette Anleitung für Log-Zugriff und Fehlersuche
   - INSTALLATION.md: Download-Statistiken Badge hinzugefügt

Solves:
- #<issue-number> (falls Issue existiert)
- 500 Internal Server Error bei Frontend
- Fehlende Transparenz über Download-Zahlen
```

## Was wurde geändert?

### Problem 1: Frontend-Fehler bei Installation

**Fehler:** `Cannot find package 'jsonwebtoken'`

**Ursache:** Das Frontend-Package enthielt nur `build/` und `package.json`, aber nicht `node_modules/`. Der SvelteKit-Server benötigt zur Laufzeit Zugriff auf Dependencies wie `jsonwebtoken`.

**Lösung:**

- GitHub Actions führt nach Build `npm ci --omit=dev` aus
- Tarball enthält jetzt: `build/`, `package.json`, `node_modules/`
- Install-Script muss kein `npm install` mehr ausführen

### Problem 2: Download-Statistiken nicht sichtbar

**Anforderung:** Sehen können, wie oft die Binaries heruntergeladen wurden

**Lösung:**

- Neuer `download-stats` Job in `.github/workflows/release.yml`
- GitHub API wird verwendet um Download-Counts abzurufen
- Statistiken werden im Workflow-Summary angezeigt:
  - Asset-Name
  - Größe in MB
  - Download-Count
- Zusätzlich Badge in `INSTALLATION.md`

### Neue Dokumentation

**TROUBLESHOOTING.md:**

- Komplette Log-Befehle für Backend und Frontend
- Service-Management (start, stop, restart, status)
- Häufige Fehler und deren Lösungen
- Debug-Modus aktivieren
- System-Informationen sammeln

## Testen

### 1. Frontend-Fix testen

```bash
# Nach dem nächsten Release:
curl -fsSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash

# Service sollte erfolgreich starten
sudo systemctl status csf-core

# Keine "Cannot find package" Fehler mehr im Log
sudo journalctl -u csf-core -n 50
```

### 2. Download-Stats testen

Nach dem Release:

1. Gehe zu GitHub Actions → Letzter Workflow-Run
2. Öffne "Pipeline Summary" oder "Display Download Statistics"
3. Sollte Tabelle mit Assets und Download-Counts zeigen

### 3. Troubleshooting-Docs testen

```bash
# Befehle aus TROUBLESHOOTING.md ausführen
sudo journalctl -u csf-core -f
sudo tail -f /var/log/csf-core/csf-core-error.log
```

## Deployment

1. Commit und Push zu `main`
2. GitHub Actions läuft automatisch
3. Semantic Release erstellt neues Release (z.B. v1.0.1)
4. Binaries werden gebaut mit neuem Frontend-Package
5. Download-Stats werden im Summary angezeigt
6. Nutzer können neue Version installieren:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
   ```
