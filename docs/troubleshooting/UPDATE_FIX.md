# Update System Fix - NoNewPrivileges Issue

## Problem

Der Update-Prozess schlug fehl mit dem Fehler:

```
sudo: The "no new privileges" flag is set, which prevents sudo from running as root.
```

## Ursache

Der systemd-Service hatte `NoNewPrivileges=true` gesetzt, was verhindert dass der Service sudo-Befehle ausführen kann.

## Lösung

### 1. Service-File angepasst ([csf-core.service](../csf-core.service))

- `NoNewPrivileges=true` entfernt
- `/tmp` zu `ReadWritePaths` hinzugefügt für Status-Datei

### 2. Erweiterte sudoers-Konfiguration

Die sudoers-Datei wurde erweitert um alle benötigten Befehle:

- Update-Skript Ausführung
- systemctl Befehle (start, stop, restart, status, etc.)
- Datei-Operationen (cp, mv, rm, mkdir, tar, rsync, chown, chmod)

Die Datei wird automatisch vom [install.sh](../scripts/install.sh) Skript erstellt.

### 3. Backend Update-Route angepasst

Die Command-Erstellung in [backend/src/routes/updates.rs](../backend/src/routes/updates.rs) wurde optimiert um immer den korrekten Pfad zu verwenden: `sudo /bin/bash /opt/csf-core/scripts/update.sh`

## Manuelle Aktualisierung auf bestehenden Systemen

Wenn das System bereits installiert ist, müssen die Änderungen manuell angewendet werden:

### Option 1: Neuinstallation (Empfohlen)

```bash
# Backup der Daten
sudo systemctl stop csf-core.service
sudo cp -r /opt/csf-core /opt/csf-core.backup

# Neuinstallation
curl -fsSL https://raw.githubusercontent.com/CS-Foundry/CSF-Core/main/scripts/install.sh | sudo bash
```

### Option 2: Manuelle Aktualisierung

#### 1. Service-File aktualisieren

```bash
sudo nano /etc/systemd/system/csf-core.service
```

Entferne die Zeile:

```
NoNewPrivileges=true
```

Und füge zu den `ReadWritePaths` hinzu:

```
ReadWritePaths=/tmp
```

Das Ergebnis sollte so aussehen:

```ini
# Security settings
# NoNewPrivileges disabled to allow sudo for updates
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/csf-core
ReadWritePaths=/var/lib/csf-core
ReadWritePaths=/var/log/csf-core
ReadWritePaths=/tmp
SupplementaryGroups=docker
```

#### 2. sudoers-Datei aktualisieren

```bash
sudo nano /etc/sudoers.d/csf-core
```

Ersetze den Inhalt mit:

```
# Allow csf-core user to run update script without password
csf-core ALL=(ALL) NOPASSWD: /bin/bash /opt/csf-core/scripts/update.sh*
csf-core ALL=(ALL) NOPASSWD: /usr/bin/bash /opt/csf-core/scripts/update.sh*

# Allow systemctl commands for service management
csf-core ALL=(ALL) NOPASSWD: /bin/systemctl daemon-reload
csf-core ALL=(ALL) NOPASSWD: /bin/systemctl start csf-core.service
csf-core ALL=(ALL) NOPASSWD: /bin/systemctl stop csf-core.service
csf-core ALL=(ALL) NOPASSWD: /bin/systemctl restart csf-core.service
csf-core ALL=(ALL) NOPASSWD: /bin/systemctl status csf-core.service
csf-core ALL=(ALL) NOPASSWD: /bin/systemctl is-active csf-core.service
csf-core ALL=(ALL) NOPASSWD: /usr/bin/systemctl daemon-reload
csf-core ALL=(ALL) NOPASSWD: /usr/bin/systemctl start csf-core.service
csf-core ALL=(ALL) NOPASSWD: /usr/bin/systemctl stop csf-core.service
csf-core ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart csf-core.service
csf-core ALL=(ALL) NOPASSWD: /usr/bin/systemctl status csf-core.service
csf-core ALL=(ALL) NOPASSWD: /usr/bin/systemctl is-active csf-core.service

# Additional file operations needed during update
csf-core ALL=(ALL) NOPASSWD: /bin/chown -R csf-core\:csf-core /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/cp -rp /opt/csf-core* *
csf-core ALL=(ALL) NOPASSWD: /bin/cp -rp * /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/mv * /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/rm -rf /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/rm -rf /tmp/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/rm -rf /var/tmp/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/mkdir -p *
csf-core ALL=(ALL) NOPASSWD: /bin/tar -xzf * -C /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/chmod +x /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /usr/bin/rsync -a * *

# Allow csf-core to preserve environment and run non-interactively
Defaults:csf-core !requiretty
Defaults:csf-core env_keep += "PATH HOME LANG LC_ALL"
```

Validieren und Berechtigungen setzen:

```bash
sudo visudo -c -f /etc/sudoers.d/csf-core
sudo chmod 0440 /etc/sudoers.d/csf-core
```

#### 3. Service neu laden und starten

```bash
sudo systemctl daemon-reload
sudo systemctl restart csf-core.service
sudo systemctl status csf-core.service
```

#### 4. Update testen

Jetzt sollte das Update über die Web-UI funktionieren oder kann getestet werden mit:

```bash
sudo -u csf-core sudo /bin/bash /opt/csf-core/scripts/update.sh 0.4.9
```

## Sicherheitshinweise

Das Entfernen von `NoNewPrivileges=true` reduziert die Sicherheit minimal, da der Service nun sudo-Befehle ausführen kann. Dies ist jedoch notwendig für das Update-System.

Die sudoers-Konfiguration ist auf spezifische Befehle beschränkt und erlaubt nur:

- Ausführung des Update-Skripts
- Service-Management via systemctl
- Datei-Operationen nur in /opt/csf-core und temporären Verzeichnissen

Dies minimiert das Sicherheitsrisiko.

## Verifikation

Nach der Aktualisierung sollte ein Update-Test durchgeführt werden:

1. Im Web-UI: Settings → Updates
2. Auf "Check for Updates" klicken
3. Falls Update verfügbar, auf "Install Update" klicken
4. Die Logs beobachten: `sudo journalctl -u csf-core.service -f`

Der Fehler "no new privileges" sollte nicht mehr auftreten.
