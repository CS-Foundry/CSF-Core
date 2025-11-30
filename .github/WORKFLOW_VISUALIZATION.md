git push# GitHub Actions Workflows - Übersicht

## 🔄 Workflow-Struktur

```
Push to main
     │
     ▼
┌────────────────────────────────────────────┐
│     main-release.yml (Orchestrator)        │
│                                            │
│  ┌──────────────────────────────────────┐ │
│  │  Job 1: release.yml                  │ │
│  │  📦 Semantic Release                 │ │
│  │                                      │ │
│  │  ✓ Analyze commits                  │ │
│  │  ✓ Determine version                │ │
│  │  ✓ Create CHANGELOG                 │ │
│  │  ✓ Update versions                  │ │
│  │  ✓ Create Git Tag (v1.2.3)          │ │
│  │  ✓ Create GitHub Release            │ │
│  │                                      │ │
│  │  Output: new_release_published=true │ │
│  │          new_release_version=1.2.3  │ │
│  └──────────────────────────────────────┘ │
│                  │                         │
│                  ▼                         │
│  ┌──────────────────────────────────────┐ │
│  │  Job 2: summary                      │ │
│  │  📊 Display Release Info             │ │
│  └──────────────────────────────────────┘ │
└────────────────────────────────────────────┘
                  │
                  │ Git Tag Created (v1.2.3)
                  ▼
┌────────────────────────────────────────────┐
│  docker-build-push.yml (Triggered by Tag)  │
│                                            │
│  ┌─────────────────────────────┐          │
│  │  build-amd64                │          │
│  │  • Build AMD64 image        │          │
│  │  • Tag: v1.2.3-amd64        │          │
│  └─────────────────────────────┘          │
│                                            │
│  ┌─────────────────────────────┐          │
│  │  build-arm64                │          │
│  │  • Build ARM64 image        │          │
│  │  • Tag: v1.2.3-arm64        │          │
│  └─────────────────────────────┘          │
│                                            │
│  ┌─────────────────────────────┐          │
│  │  create-manifest            │          │
│  │  • Combine images           │          │
│  │  • Tag: v1.2.3, v1.2, v1   │          │
│  │  • Tag: latest              │          │
│  └─────────────────────────────┘          │
└────────────────────────────────────────────┘
                  │
                  ▼
         ✅ Complete!
```

## 📦 Workflows

### 1. `main-release.yml`

**Hauptpipeline** - Orchestriert Release und Build

- **Trigger:** Push auf `main`
- **Jobs:**
  - `release` - Ruft `release.yml` auf
  - `docker-build` - Ruft `docker-build-push.yml` auf (nur bei neuem Release)
  - `summary` - Zeigt Zusammenfassung

### 2. `release.yml`

**Semantic Release** - Erstellt Versionen und Releases

- **Trigger:** Wird von `main-release.yml` aufgerufen
- **Outputs:**
  - `new_release_published` - Boolean
  - `new_release_version` - Version string (z.B. "1.2.3")

### 3. `docker-build-push.yml`

**Docker Build** - Baut und published Images

- **Trigger:**
  - Wird von `main-release.yml` aufgerufen
  - Git Tags `v*`
- **Jobs:**
  - `build-amd64` - AMD64 Build
  - `build-arm64` - ARM64 Build
  - `create-manifest` - Multi-Arch Manifest

## 🎨 In GitHub Actions Ansicht

Wenn du in GitHub Actions schaust, siehst du **zwei separate Workflows**:

### Workflow 1: Main Release Pipeline

```
main-release.yml
├─ 📦 Semantic Release      ✓
└─ � Release Summary       ✓
```

### Workflow 2: Build and Push Docker Image (getriggert durch Tag)

```
docker-build-push.yml (triggered by v1.2.3)
├─ build-amd64           ✓
├─ build-arm64           ✓
└─ create-manifest       ✓
```

**Wichtig:** Der Docker Build erscheint als **separater Workflow-Run**, getriggert durch den Git Tag!

## 🚀 Verwendung

1. **Feature entwickeln:**

   ```bash
   git commit -m "feat(dashboard): add expense charts"
   ```

2. **Merge in main:**

   - PR erstellen und mergen

3. **Automatisch passiert:**

   - ✅ Release erstellt (v1.2.0)
   - ✅ Docker Images gebaut
   - ✅ Images gepushed mit korrekter Version

4. **Image verwenden:**
   ```bash
   docker pull ghcr.io/codemaster4711/financevault:1.2.0
   # oder
   docker pull ghcr.io/codemaster4711/financevault:latest
   ```

## 📊 Visualisierung

GitHub Actions zeigt dir:

- ✅ Grüne Häkchen für erfolgreiche Jobs
- 📊 Abhängigkeiten zwischen Jobs
- ⏱️ Laufzeiten für jeden Job
- 📝 Logs für jeden Schritt
- 📦 Artifacts und Outputs

Alles in **einer übersichtlichen Grafik**! 🎯
