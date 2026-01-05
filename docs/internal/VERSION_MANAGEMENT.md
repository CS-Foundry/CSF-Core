# Version Management in Release Pipeline

## Problem

Die Binary-Builds zeigten die alte Version (v0.1.3), obwohl Semantic Release bereits v0.2.0 erstellt hatte.

## Ursache

Der Release-Workflow lief in dieser Reihenfolge:

1. ✅ Semantic Release erstellt Release v0.2.0
2. ✅ Semantic Release updated Cargo.toml zu 0.2.0
3. ❌ Semantic Release versucht zu committen (scheitert an Repository Rules)
4. ❌ Build-Job checked alte Version aus (vor dem Update)
5. ❌ Binaries werden mit alter Version 0.1.3 gebaut

## Lösung

Der Build-Job updated nun die Version explizit VOR dem Build:

```yaml
- name: Update version in Cargo.toml
  run: |
    VERSION="${{ needs.release.outputs.new_release_version }}"

    # Update backend Cargo.toml (kompatibel mit Linux und macOS)
    if [[ "$OSTYPE" == "darwin"* ]]; then
      sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" backend/Cargo.toml
    else
      sed -i "s/^version = \".*\"/version = \"$VERSION\"/" backend/Cargo.toml
    fi
```

## Workflow

Neue Reihenfolge:

1. ✅ Semantic Release läuft und erstellt Tag/Release
2. ✅ Semantic Release gibt `new_release_version` Output zurück
3. ✅ Build-Job checked Code aus
4. ✅ Build-Job pulled neueste Änderungen
5. ✅ **Build-Job updated Version in Cargo.toml**
6. ✅ Build-Job baut Binaries mit korrekter Version
7. ✅ Binaries werden zum Release hochgeladen

## Vorteile

- ✅ Binaries haben immer die korrekte Version
- ✅ Funktioniert auch wenn Semantic Release Commit fehlschlägt
- ✅ Kompatibel mit Linux und macOS Runners
- ✅ Keine zusätzlichen Dependencies nötig

## Testen

Nach dem nächsten Release:

```bash
# Download binary
curl -L -o csf-backend \
  https://github.com/CS-Foundry/CSF-Core/releases/download/vX.X.X/csf-backend-linux-amd64

# Check Version
chmod +x csf-backend
./csf-backend --version
# Sollte vX.X.X ausgeben (nicht die alte Version)
```

## Alternative Ansätze

### 1. Cargo-edit verwenden

```yaml
- run: cargo install cargo-edit
- run: cargo set-version ${{ needs.release.outputs.new_release_version }}
```

Nachteil: Zusätzliche Dependency, langsamer

### 2. sed mit Backup

```yaml
- run: sed -i.bak "s/^version = .*/version = \"$VERSION\"/" backend/Cargo.toml
- run: rm backend/Cargo.toml.bak
```

Nachteil: Funktioniert nicht einheitlich auf allen Plattformen

### 3. Python/Node Script

Nachteil: Mehr Code, mehr Komplexität

## Unsere Wahl: sed mit Platform-Detection

- ✅ Einfach
- ✅ Schnell
- ✅ Keine zusätzlichen Dependencies
- ✅ Funktioniert auf Linux und macOS

## Verwandte Dateien

- [.github/workflows/release.yml](../../.github/workflows/release.yml) - Hauptworkflow mit Version-Update
- [backend/Cargo.toml](../../backend/Cargo.toml) - Version wird hier gesetzt
- [.github/scripts/update-versions.sh](../../.github/scripts/update-versions.sh) - Script für Semantic Release
