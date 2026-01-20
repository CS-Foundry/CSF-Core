# CSF-Core Master Node NixOS ISO

Diese Konfiguration erstellt ein bootfähiges NixOS ISO-Image mit vorinstalliertem und automatisch startendem CSF-Core Master Node.

## 🚀 ISO bauen

### Mit Flakes (empfohlen)

```bash
cd nixos/
nix build .#nixosConfigurations.iso.config.system.build.isoImage
```

Das ISO-Image wird unter `./result/iso/` erstellt.

### Ohne Flakes (klassisch)

```bash
nix-build '<nixpkgs/nixos>' -A config.system.build.isoImage -I nixos-config=./iso-configuration.nix
```

## 📦 Was ist enthalten?

- **Docker & Docker Compose** - Container-Management und Orchestrierung
- **Test Container** - Nginx-Beispiel-Container auf Port 8080
- **Rust & Cargo** - Zum Bauen des Backends
- **Node.js 20** - Für das Frontend
- **CSF-Core Service** - Automatisch gestartet beim Booten

## 🔧 Konfiguration

### Ports

- `8000` - CSF-Core Backend API
- `3000` - CSF-Core Frontend
- `8443` - P2P Agent Kommunikation
- `5432` - PostgreSQL

### Verzeichnisse

- `/opt/csf-core` - Hauptverzeichnis
- `/var/lib/csf-core` - Datenspeicher
- `/var/log/csf-core` - Logs

### Umgebungsvariablen

Die Konfiguration kann in `/opt/csf-core/config.env` angepasst werden:

```bash
DATABASE_URL=postgres://csf_core@localhost/csf_core
JWT_SECRET=change-this-in-production
RUST_LOG=info
NODE_ENV=production
PORT=3000
FRONTEND_URL=http://localhost:3000
ORIGIN=http://localhost:8000
```

## 🎯 Verwendung

### 1. ISO auf USB-Stick schreiben

```bash
sudo dd if=result/iso/*.iso of=/dev/sdX bs=4M status=progress
```

### 2. System booten

Boote von dem USB-Stick. Das System startet automatisch als Root und CSF-Core wird gestartet.

### 3. Service-Status prüfen

```bash
systemctl status csf-core
systemctl status postgresql
```

### 4. Logs anschauen

```bash
journalctl -u csf-core -f
```

## 🔐 Sicherheit

**WICHTIG:** Diese Konfiguration ist für Development/Testing gedacht!

Für Production:
- Ändere `JWT_SECRET` in eine sichere Random-String
- Konfiguriere PostgreSQL mit sicheren Passwörtern
- Aktiviere HTTPS mit eigenen Zertifikaten
- Deaktiviere Auto-Login
- Setze strikte Firewall-Regeln

## 🛠️ Anpassungen

### Eigene CSF-Core Binaries verwenden

1. Baue deine CSF-Core Binaries:
```bash
# Backend
cd backend/
cargo build --release

# Frontend
cd frontend/
npm run build
```

2. Kopiere sie ins ISO während des Builds, indem du in `iso-configuration.nix` ergänzt:

```nix
system.activationScripts.csf-core-setup = {
  text = ''
    # ... existing setup ...
    
    # Copy binaries
    cp ${./path/to/backend/binary} /opt/csf-core/csf-core
    cp -r ${./path/to/frontend/build} /opt/csf-core/frontend
    
    chmod +x /opt/csf-core/csf-core
  '';
};
```

### Hostname ändern

In `iso-configuration.nix`:
```nix
networking.hostName = "mein-master-node";
```

### Zusätzliche Packages

In `environment.systemPackages`:
```nix
environment.systemPackages = with pkgs; [
  # ... existing packages ...
  neovim
  ripgrep
  fd
];
```

## 📝 Troubleshooting

### Service startet nicht

```bash
# Logs prüfen
journalctl -u csf-core -xe

# Manuell starten
sudo systemctl start csf-core

# Status prüfen
sudo systemctl status csf-core
```

### PostgreSQL Verbindungsprobleme

```bash
# PostgreSQL Status
sudo systemctl status postgresql

# PostgreSQL neu starten
sudo systemctl restart postgresql

# Verbindung testen
psql -U csf_core -d csf_core -h localhost
```

### Firewall blockiert Ports

```bash
# Firewall Status
sudo nft list ruleset

# Port öffnen (temporär)
sudo iptables -A INPUT -p tcp --dport 8000 -j ACCEPT
```

## 🔄 Updates

Um das ISO mit Updates zu bauen:

1. Aktualisiere Flake-Inputs:
```bash
nix flake update
```

2. Baue das ISO neu:
```bash
nix build .#nixosConfigurations.iso.config.system.build.isoImage
```

## 📚 Weitere Dokumentationen

- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [NixOS ISO Building](https://nixos.wiki/wiki/Creating_a_NixOS_live_CD)
- [CSF-Core Dokumentation](../docs/README.md)
