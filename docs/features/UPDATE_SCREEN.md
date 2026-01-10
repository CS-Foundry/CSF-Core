# Update Screen Feature

## Übersicht

Das Update-Screen-Feature zeigt dem Benutzer während eines laufenden System-Updates einen vollständigen Bildschirm-Overlay mit Fortschrittsanzeige, Logs und Status-Informationen an.

## Implementierung

### Backend (Rust)

Der Update-Prozess wird als eigener, unabhängiger Prozess gestartet (nicht als Child-Prozess), damit er nicht vom Backend-Prozess selbst beendet wird, wenn dieser während des Updates neustartet.

**Datei:** [`backend/src/routes/updates.rs`](../../backend/src/routes/updates.rs)

#### Status-Tracking

Der Update-Status wird in einer JSON-Datei gespeichert:

- **Pfad:** `/tmp/csf-core-update-status.json`
- **Format:**
  ```json
  {
    "status": "in_progress",
    "message": "Downloading updates...",
    "progress": 45,
    "version": "1.2.0",
    "timestamp": "1673456789"
  }
  ```

#### API-Endpunkt

**GET** `/api/updates/status`

- Gibt den aktuellen Update-Status zurück
- Wird vom Frontend alle Sekunde gepollt
- Gibt `idle` zurück, wenn kein Update läuft

### Frontend (Svelte)

#### 1. Update Screen Komponente

**Datei:** [`frontend/src/lib/components/UpdateScreen.svelte`](../../frontend/src/lib/components/UpdateScreen.svelte)

**Features:**

- ✅ Vollbild-Overlay mit z-index 9999
- ✅ Fortschrittsbalken mit Prozentanzeige
- ✅ Echtzeit-Logs mit Auto-Scroll
- ✅ Versionsinformationen
- ✅ Verbindungsstatus-Anzeige während Backend-Restart
- ✅ Automatisches Reconnect (30 Versuche)
- ✅ Automatischer Reload nach erfolgreichem Update
- ✅ Fehlerbehandlung mit manueller Reload-Option
- ✅ Deutsche Übersetzung aller Texte

**Polling-Mechanismus:**

- Pollt `/api/updates/status` jede Sekunde
- Erkennt Verbindungsabbrüche (z.B. während Backend-Restart)
- Zeigt Warnung bei Verbindungsverlust an
- Führt automatischen Reload durch, wenn Update abgeschlossen

#### 2. Update Store

**Datei:** [`frontend/src/lib/stores/update.ts`](../../frontend/src/lib/stores/update.ts)

```typescript
export const updateInProgress = writable(false);
export const updateVersion = writable<string | null>(null);
```

#### 3. Layout Integration

**Datei:** [`frontend/src/routes/+layout.svelte`](../../frontend/src/routes/+layout.svelte)

**Features:**

- ✅ Zeigt UpdateScreen-Overlay an, wenn `updateInProgress` true ist
- ✅ Prüft beim App-Start automatisch, ob ein Update läuft
- ✅ Stellt Update-Screen wieder her, auch nach Seiten-Reload

**Auto-Detection beim Start:**

```typescript
async function checkForOngoingUpdate() {
  const response = await fetch("/api/updates/status");
  if (response.ok) {
    const status = await response.json();
    if (status.status === "in_progress") {
      updateInProgress.set(true);
    }
  }
}
```

#### 4. Update Settings Integration

**Datei:** [`frontend/src/lib/components/settings/UpdateSettings.svelte`](../../frontend/src/lib/components/settings/UpdateSettings.svelte)

Beim Starten eines Updates:

```typescript
async function installUpdate(version: string) {
  const response = await updateStore.installUpdate(version);

  // Trigger update screen
  updateVersion.set(version);
  updateInProgress.set(true);
}
```

## Ablauf eines Updates

### 1. User startet Update

```
User klickt "Update installieren"
  → POST /api/updates/install
  → Backend startet update.sh als eigener Prozess
  → updateInProgress.set(true)
  → UpdateScreen erscheint
```

### 2. Update läuft

```
UpdateScreen pollt /api/updates/status
  → Zeigt Fortschritt an
  → Sammelt Logs
  → Backend schreibt Status in /tmp/csf-core-update-status.json
```

### 3. Backend Restart während Update

```
Backend neustart
  → Verbindung unterbrochen
  → UpdateScreen zeigt Warning: "Backend wird neu gestartet..."
  → Reconnect-Versuche starten
  → Verbindung wiederhergestellt
  → Log: "✅ Connection restored"
```

### 4. Update abgeschlossen

```
Status = "completed"
  → Log: "✅ Update completed successfully! Reloading..."
  → 2 Sekunden Delay
  → window.location.reload()
  → App läuft mit neuer Version
```

### 5. Update fehlgeschlagen

```
Status = "error"
  → Polling stoppt
  → Error-Box wird angezeigt
  → User kann manuell reload durchführen
```

## Besondere Eigenschaften

### 🔄 Resilience

- **Verbindungsabbrüche:** UpdateScreen erkennt, wenn das Backend während des Updates neustartet
- **Auto-Reconnect:** Versucht automatisch 30 Sekunden lang, die Verbindung wiederherzustellen
- **Status-Persistenz:** Prüft beim App-Start, ob ein Update läuft (auch nach Seiten-Reload)

### 🎨 User Experience

- **Vollbild-Overlay:** Verhindert User-Interaktion während des Updates
- **Echtzeit-Feedback:** Zeigt Fortschritt und Logs in Echtzeit an
- **Deutsche Lokalisierung:** Alle Texte auf Deutsch
- **Automatischer Reload:** Keine manuelle Aktion nötig nach erfolgreichem Update

### 🛡️ Error Handling

- **Verbindungsfehler:** Zeigt Warning statt Error bei temporären Verbindungsproblemen
- **Update-Fehler:** Zeigt detaillierte Fehlermeldung mit Reload-Button
- **Timeout-Schutz:** Führt automatischen Reload durch, wenn Backend nach 30 Sekunden nicht zurückkommt

## Testing

### Manueller Test

1. Update starten: Settings → Updates → "Update installieren"
2. UpdateScreen sollte erscheinen
3. Fortschritt und Logs sollten sichtbar sein
4. Bei Backend-Restart: Warning "Backend wird neu gestartet..." erscheint
5. Nach Abschluss: Automatischer Reload zur neuen Version

### Edge Cases

- **Seite während Update neu laden:** UpdateScreen sollte automatisch wieder erscheinen
- **Backend während Update abstürzen:** Warning erscheint, Auto-Reconnect versucht Verbindung
- **Mehrere Browser-Tabs:** Jeder Tab zeigt UpdateScreen unabhängig

## Bekannte Limitierungen

- Update-Status geht bei Server-Neustart verloren (liegt in `/tmp`)
- Keine Pause/Resume-Funktion für Updates
- Keine Rollback-Funktion bei fehlgeschlagenen Updates

## Zukünftige Verbesserungen

- [ ] Update-Status in Datenbank statt `/tmp` speichern
- [ ] WebSocket-Verbindung statt Polling
- [ ] Detailliertere Fortschrittsanzeige (Download, Install, Cleanup)
- [ ] Update-Historie mit Changelog
- [ ] Automatischer Rollback bei Fehlern
