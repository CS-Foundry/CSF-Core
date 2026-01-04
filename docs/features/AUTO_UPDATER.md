# Auto-Updater Feature

## Übersicht

Das Auto-Updater-Feature ermöglicht es CSF-Core, automatisch nach Updates zu suchen und diese mit einem Klick zu installieren.

## Features

### 1. Automatische Update-Prüfung

- Prüft automatisch stündlich auf neue Versionen
- Nutzt GitHub Releases API
- Keine manuelle Aktion erforderlich

### 2. Sidebar-Benachrichtigung

- Zeigt eine Benachrichtigung in der Sidebar, wenn ein Update verfügbar ist
- Zeigt die aktuelle und neue Version an
- Link zum Changelog-Dialog

### 3. Changelog-Anzeige

- Vollständiger Changelog für neue Versionen
- Markdown-Formatierung wird unterstützt
- Link zu den vollständigen Release-Notes auf GitHub

### 4. Ein-Klick-Installation

- Update-Installation per Klick
- Automatischer Download und Installation
- Neustart der Anwendung nach der Installation

### 5. Einstellungsseite

- Manuelle Update-Prüfung
- Detaillierte Versionsinformationen
- Vollständige Changelog-Ansicht

## Technische Details

### Backend (Rust)

#### API-Endpunkte

1. **GET** `/api/updates/check`

   - Prüft die neueste Version von GitHub Releases
   - Gibt Versionsinformationen und Changelog zurück

2. **POST** `/api/updates/install`

   - Startet die Update-Installation
   - Akzeptiert die Zielversion als Parameter

3. **GET** `/api/updates/changelog/:version`
   - Ruft den Changelog für eine bestimmte Version ab

#### Implementierung

- Datei: `backend/src/routes/updates.rs`
- Nutzt `reqwest` für GitHub API-Aufrufe
- Asynchrone Update-Installation mit tokio

### Frontend (Svelte)

#### Komponenten

1. **UpdateNotification.svelte**

   - Sidebar-Benachrichtigung
   - Changelog-Dialog
   - Installation-Button

2. **UpdateSettings.svelte**
   - Einstellungsseite für Updates
   - Manuelle Update-Prüfung
   - Detaillierte Informationen

#### Services

- `lib/services/updates.ts`: API-Client für Update-Endpunkte
- `lib/stores/updates.ts`: Svelte-Store für Update-Status

### Update-Script

Das Update-Script (`scripts/update.sh`) führt folgende Schritte aus:

1. Backup der aktuellen Installation erstellen
2. CSF-Core-Service stoppen
3. Neue Version von GitHub herunterladen
4. Installation extrahieren und kopieren
5. Service neu starten
6. Erfolg verifizieren

## Verwendung

### Als Benutzer

1. **Automatische Benachrichtigung**: Eine Benachrichtigung erscheint in der Sidebar, wenn ein Update verfügbar ist
2. **Changelog ansehen**: Klicken Sie auf "Changelog anzeigen & Installieren"
3. **Installieren**: Klicken Sie auf "Jetzt installieren" im Dialog
4. **Warten**: Die Anwendung startet automatisch neu

### Manuelle Prüfung

1. Gehen Sie zu **Einstellungen** → **Updates**
2. Klicken Sie auf "Nach Updates suchen"
3. Wenn verfügbar, klicken Sie auf "Jetzt installieren"

## Sicherheit

- Updates werden nur von GitHub Releases heruntergeladen
- Versionsnummern werden validiert (keine Downgrades)
- Backup wird vor jedem Update erstellt
- Rollback-Anweisungen werden nach der Installation angezeigt

## Fehlerbehandlung

- Netzwerkfehler werden dem Benutzer angezeigt
- Bei Installationsfehlern bleibt die alte Version aktiv
- Backup ermöglicht manuelle Wiederherstellung

## Entwicklung

### Testen

Um das Update-Feature zu testen:

1. Erstellen Sie ein neues Release auf GitHub
2. Warten Sie auf die automatische Prüfung (oder manuell prüfen)
3. Installieren Sie das Update

### Anpassung

- **Prüfintervall ändern**: Passen Sie `60 * 60 * 1000` in `updates.ts` an
- **GitHub-Repository ändern**: Aktualisieren Sie `CS-Foundry/CSF-Core` in `updates.rs`
- **Update-Script anpassen**: Bearbeiten Sie `scripts/update.sh`

## Bekannte Einschränkungen

- Update-Installation erfordert Root-Rechte oder sudo
- Während des Updates ist die Anwendung nicht verfügbar
- Nur für Linux-Systeme mit systemd getestet

## Zukünftige Verbesserungen

- [ ] Beta/Alpha-Kanal-Unterstützung
- [ ] Automatische Installation planen
- [ ] Rollback-Funktion in der UI
- [ ] Update-Historie
- [ ] Differenzielle Updates (nur geänderte Dateien)
