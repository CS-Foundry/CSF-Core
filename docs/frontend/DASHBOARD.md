# CSF-Core Dashboard

Ein umfassendes Echtzeit-Dashboard für die Überwachung und Verwaltung deiner Cloud-Infrastruktur.

## Features

### 🎯 Hauptfunktionen

#### 1. **Echtzeit-Hardware-Health**

- **CPU Usage**: Live CPU-Auslastung mit farbcodierter Statusanzeige
- **Memory Usage**: RAM-Nutzung mit detaillierter Anzeige (verwendet/verfügbar)
- **Storage Usage**: Speicherauslastung über alle Datenträger
- **Network Traffic**: RX/TX Datenübertragung in Echtzeit

#### 2. **System Health Card**

Detaillierte Hardware-Metriken mit:

- Prozentualer Auslastung für CPU, Memory und Storage
- Progress Bars für visuelle Übersicht
- Status-Indikatoren (Normal/Elevated/High)
- Byte-formatierte Anzeigen für Speicher

#### 3. **Uptime & Availability**

- **Current Uptime**: Betriebszeit seit letztem Neustart
- **Availability**: 30-Tage Verfügbarkeit in Prozent
- **SLA-Tracking**: Vergleich mit 99.9% Target
- **Status Indicators**: API und Database Health

#### 4. **Traffic World Map** 🌍

- Echtzeit-Visualisierung globaler Anfragen
- Top 5 Regionen nach Request-Volume
- Geografische Verteilung der Traffic-Quellen
- Requests pro 5-Minuten-Fenster

#### 5. **Resource Distribution**

- Übersicht über alle Resource Groups
- Container-Verteilung nach Gruppen
- Ressourcen-Auslastung pro Gruppe
- Durchschnittliche Ressourcennutzung

#### 6. **Activity Feed**

- Echtzeit-Ereignisprotokoll
- Nutzeraktionen (Login, Deployments, etc.)
- Systemereignisse (Backups, Updates, etc.)
- Warnungen und Alerts
- Zeitstempel mit relativer Formatierung

## Komponenten-Struktur

```
frontend/src/
├── routes/
│   └── +page.svelte                    # Haupt-Dashboard
└── lib/components/dashboard/
    ├── SystemHealthCard.svelte         # Hardware-Metriken
    ├── UptimeCard.svelte               # Verfügbarkeit & Uptime
    ├── TrafficMapCard.svelte           # Globale Traffic-Map
    ├── ResourceDistributionCard.svelte # Ressourcen-Gruppen
    └── ActivityFeedCard.svelte         # Ereignis-Feed
```

## Backend API Endpoints

Das Dashboard nutzt folgende Backend-Endpunkte:

```rust
GET /api/system/info      # Statische System-Informationen
GET /api/system/metrics   # Echtzeit-Metriken
```

### Response Format

```json
{
  "metrics": {
    "timestamp": "2026-01-05T20:00:00Z",
    "cpu_usage_percent": 45.2,
    "memory_total_bytes": 17179869184,
    "memory_used_bytes": 8589934592,
    "memory_usage_percent": 50.0,
    "disk_total_bytes": 1099511627776,
    "disk_used_bytes": 549755813888,
    "disk_usage_percent": 50.0,
    "network_rx_bytes": 1073741824,
    "network_tx_bytes": 536870912,
    "hostname": "csf-core-prod",
    "uptime_seconds": 864000
  }
}
```

## Echtzeit-Updates

Das Dashboard aktualisiert sich automatisch:

```typescript
// Update-Intervall: 5 Sekunden
onMount(() => {
  fetchMetrics();
  updateInterval = window.setInterval(fetchMetrics, 5000);
});
```

## Status-Indikatoren

### CPU/Memory/Disk Farb-Codes

```typescript
- 0-60%:   🟢 Grün  (Normal)
- 60-80%:  🟡 Gelb  (Elevated)
- 80-100%: 🔴 Rot   (High/Critical)
```

### Activity Types

```typescript
user:     👤 Blau    - Nutzeraktionen
system:   ⚙️  Grau    - Systemereignisse
warning:  ⚠️  Gelb    - Warnungen
success:  ✓  Grün    - Erfolgreiche Aktionen
info:     ℹ️  Lila    - Informationen
```

## Responsives Design

Das Dashboard passt sich automatisch an verschiedene Bildschirmgrößen an:

```svelte
<!-- 4 Spalten auf großen Bildschirmen -->
<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4">

<!-- 3 Spalten für mittlere Sektion -->
<div class="grid gap-6 lg:grid-cols-3">

<!-- 2 Spalten für untere Sektion -->
<div class="grid gap-6 lg:grid-cols-2">
```

## Verwendete UI-Komponenten

- **Card**: shadcn/ui Card-Komponenten
- **Progress**: Progress Bars für Auslastung
- **Badge**: Status-Badges und Labels
- **Skeleton**: Loading-Zustände
- **Lucide Icons**: Moderne Icon-Bibliothek

## Zukünftige Features

- [ ] Interaktive World Map mit echten Geo-Daten
- [ ] Historische Metriken und Graphen
- [ ] Konfigurierbare Alert-Schwellwerte
- [ ] Export von Metriken (CSV/JSON)
- [ ] Custom Dashboard-Layouts
- [ ] Echtzeit-Websocket-Updates
- [ ] Mobile App Integration
- [ ] Multi-Tenant Support

## Performance-Optimierung

### Lazy Loading

```typescript
// Komponenten werden nur bei Bedarf geladen
import SystemHealthCard from "$lib/components/dashboard/SystemHealthCard.svelte";
```

### Memoization

```typescript
// Berechnete Werte werden gecached
$: totalRequests = trafficData.reduce((sum, t) => sum + t.requests, 0);
```

### Efficient Updates

- Nur geänderte Metriken werden neu gerendert
- Svelte's reactive system optimiert Updates automatisch

## Development

### Starten

```bash
cd frontend
npm install
npm run dev
```

### Build

```bash
npm run build
```

### Type-Check

```bash
npm run check
```

## Fehlerbehandlung

Das Dashboard zeigt informative Fehlermeldungen:

```svelte
{#if error}
  <Card.Root class="border-destructive">
    <Card.Content class="pt-6">
      <div class="flex items-center gap-2 text-destructive">
        <AlertCircle class="h-5 w-5" />
        <p>{error}</p>
      </div>
    </Card.Content>
  </Card.Root>
{/if}
```

## Accessibility

- Semantisches HTML
- ARIA-Labels für Screenreader
- Keyboard-Navigation
- Hoher Kontrast für Farben
- Relative Zeitstempel für bessere UX

## Browser-Kompatibilität

- Chrome/Edge: ✅ Vollständig unterstützt
- Firefox: ✅ Vollständig unterstützt
- Safari: ✅ Vollständig unterstützt
- Mobile Browsers: ✅ Responsive Design

## Lizenz

MIT License - siehe LICENSE-Datei für Details
