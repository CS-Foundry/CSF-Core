# Branch-spezifische Installation

## Verfügbare Branches

Du kannst CSF-Core von jedem Branch oder Tag installieren:

### 🟢 Main Branch (Stable/Production)

```bash
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
```

**Empfohlen für:** Production, Stable Deployments

---

### 🔵 Development/Feature Branches

```bash
# Aktueller Feature Branch (Docker Management)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash

# Beliebiger Branch
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/BRANCH_NAME/scripts/install.sh | sudo bash
```

**Empfohlen für:** Testing, Development, neue Features

---

### 🏷️ Spezifische Version/Tag

```bash
# Bestimmtes Release
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/v1.2.3/scripts/install.sh | sudo bash

# Mit Versions-Parameter
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo VERSION=1.2.3 bash
```

**Empfohlen für:** Reproduzierbare Deployments, pinned versions

---

## URL-Struktur

Die GitHub Raw URL folgt diesem Schema:

```
https://raw.githubusercontent.com/{owner}/{repo}/{branch_or_tag}/path/to/file
```

### Beispiele:

```bash
# Main Branch
https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh

# Feature Branch
https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh

# Commit Hash
https://raw.githubusercontent.com/CS-Foundry/CSF-Core/a1b2c3d4/scripts/install.sh

# Release Tag
https://raw.githubusercontent.com/CS-Foundry/CSF-Core/v1.0.0/scripts/install.sh
```

---

## Docker Images von spezifischen Branches

Für Docker Builds von Feature Branches:

```bash
# Manual Build von einem Branch
git clone -b feat/docker-managment https://github.com/CS-Foundry/CSF-Core.git
cd CSF-Core
docker build -t csf-core:dev .
docker run -d -p 8000:8000 -v csf_data:/data csf-core:dev
```

---

## Environment Variable Override

Du kannst auch spezifische Versionen via Environment Variables setzen:

```bash
# Installiere spezifische Version
VERSION=1.2.3 curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash

# Oder mit zusätzlichen Optionen
SKIP_POSTGRES=true VERSION=latest curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
```

---

## Verfügbare Branches in diesem Repository

Um alle verfügbaren Branches zu sehen:

```bash
# Alle Remote Branches
git ls-remote --heads https://github.com/CS-Foundry/CSF-Core.git

# Oder auf GitHub
https://github.com/CS-Foundry/CSF-Core/branches
```

---

## Testing verschiedener Branches

```bash
# Branch 1 testen
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
sudo systemctl stop csf-core

# Branch 2 testen (überschreibt Installation)
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash
sudo systemctl restart csf-core
```

---

## Best Practices

### ✅ Production

```bash
# Immer main oder spezifische Version
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
# oder
VERSION=1.2.3 curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
```

### 🧪 Development/Testing

```bash
# Feature Branches
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/feat/docker-managment/scripts/install.sh | sudo bash
```

### 🔒 Audit/Security

```bash
# Download Script zuerst prüfen
curl -sSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh -o install.sh
cat install.sh  # oder less/nano zum Review
sudo bash install.sh
```

---

## Fallback: Lokale Installation

Wenn GitHub nicht erreichbar ist:

```bash
# Clone Repository
git clone -b BRANCH_NAME https://github.com/CS-Foundry/CSF-Core.git
cd CSF-Core

# Script lokal ausführen
sudo bash scripts/install.sh
```

---

## Siehe auch

- [INSTALLATION.md](./INSTALLATION.md) - Vollständige Installationsanleitung
- [DEPLOYMENT.md](./DEPLOYMENT.md) - Quick Deployment Guide
- [GitHub Releases](https://github.com/CS-Foundry/CSF-Core/releases) - Verfügbare Versionen
