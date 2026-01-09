# 🔧 Manuelle Update-Fix Anleitung für Ubuntu Server

## Problem

- `sudo: The "no new privileges" flag is set` - sudo funktioniert nicht
- `/tmp/csf-core-update-status.json: Permission denied` - Schreibrechte-Problem

---

## ✅ Komplette Lösung (Schritt für Schritt)

### 🔴 WICHTIG: Als root/sudo ausführen!

```bash
# Root werden (falls nicht schon root)
sudo -i
```

---

## 1️⃣ Service-File anpassen

```bash
nano /etc/systemd/system/csf-core.service
```

**Ändere diese Zeilen:**

**VORHER:**

```ini
# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/csf-core
ReadWritePaths=/var/lib/csf-core
ReadWritePaths=/var/log/csf-core
SupplementaryGroups=docker
```

**NACHHER:**

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

**Änderungen:**

- ❌ **Entferne:** `NoNewPrivileges=true`
- ✅ **Füge hinzu:** `ReadWritePaths=/tmp`

Speichern: `Ctrl+O` → Enter → `Ctrl+X`

---

## 2️⃣ sudoers-Datei KOMPLETT NEU erstellen

```bash
nano /etc/sudoers.d/csf-core
```

**Lösche ALLES und ersetze mit:**

```sudoers
# Allow csf-core user to run update script without password (with nohup for detachment)
csf-core ALL=(ALL) NOPASSWD: /bin/bash /opt/csf-core/scripts/update.sh*
csf-core ALL=(ALL) NOPASSWD: /usr/bin/bash /opt/csf-core/scripts/update.sh*
csf-core ALL=(ALL) NOPASSWD: /usr/bin/nohup /bin/bash /opt/csf-core/scripts/update.sh*
csf-core ALL=(ALL) NOPASSWD: /usr/bin/nohup sudo /bin/bash /opt/csf-core/scripts/update.sh*
csf-core ALL=(ALL) NOPASSWD: /usr/bin/nohup /usr/bin/bash /opt/csf-core/scripts/update.sh*

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
csf-core ALL=(ALL) NOPASSWD: /bin/rm -f /tmp/csf-core-update-status.json
csf-core ALL=(ALL) NOPASSWD: /bin/mkdir -p *
csf-core ALL=(ALL) NOPASSWD: /bin/tar -xzf * -C /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/chmod +x /opt/csf-core*
csf-core ALL=(ALL) NOPASSWD: /bin/chmod 644 /tmp/csf-core-update-status.json
csf-core ALL=(ALL) NOPASSWD: /usr/bin/rsync -a * *

# Allow csf-core to preserve environment and run non-interactively
Defaults:csf-core !requiretty
Defaults:csf-core env_keep += "PATH HOME LANG LC_ALL"
```

Speichern: `Ctrl+O` → Enter → `Ctrl+X`

---

## 3️⃣ sudoers validieren und Berechtigungen setzen

```bash
# WICHTIG: Validieren (muss "parsed OK" ausgeben!)
visudo -c -f /etc/sudoers.d/csf-core
```

**Erwartete Ausgabe:**

```
/etc/sudoers.d/csf-core: parsed OK
```

❌ **Falls NICHT "parsed OK":** Es gibt einen Syntax-Fehler! Zurück zu Schritt 2 und nochmal prüfen!

✅ **Falls "parsed OK":**

```bash
# Berechtigungen setzen
chmod 0440 /etc/sudoers.d/csf-core
```

---

## 4️⃣ Update-Skript anpassen (Schreibrechte-Fix)

```bash
nano /opt/csf-core/scripts/update.sh
```

**Suche die Funktion `update_status()` (ca. Zeile 59-72) und ersetze mit:**

```bash
update_status() {
    local status="$1"
    local message="$2"
    local progress="${3:-0}"

    # Ensure status file directory exists
    mkdir -p "$(dirname "$STATUS_FILE")" 2>/dev/null

    # Remove old status file if it exists (to avoid permission issues)
    rm -f "$STATUS_FILE" 2>/dev/null

    # Write new status
    cat > "$STATUS_FILE" <<EOF
{
  "status": "$status",
  "message": "$message",
  "progress": $progress,
  "version": "$VERSION",
  "timestamp": "$(date -Iseconds 2>/dev/null || date +%Y-%m-%dT%H:%M:%S)"
}
EOF

    # Make status file readable by everyone (so the backend can read it)
    chmod 644 "$STATUS_FILE" 2>/dev/null || true
}
```

**Was wurde geändert:**

- ✅ `rm -f "$STATUS_FILE"` - Alte Datei löschen (Permission-Problem beheben)
- ✅ `chmod 644 "$STATUS_FILE"` - Datei für alle lesbar machen

Speichern: `Ctrl+O` → Enter → `Ctrl+X`

---

## 5️⃣ Alte Status-Datei aufräumen (falls vorhanden)

```bash
# Alte Status-Datei löschen
rm -f /tmp/csf-core-update-status.json

# Sicherstellen dass /tmp beschreibbar ist
chmod 1777 /tmp
```

---

## 6️⃣ Service neu laden und starten

```bash
# Systemd neu laden
systemctl daemon-reload

# Service neu starten
systemctl restart csf-core.service

# Status prüfen (muss "active (running)" sein)
systemctl status csf-core.service
```

**Erwartete Ausgabe:**

```
● csf-core.service - CSF Core Backend and Frontend Service
     Loaded: loaded (/etc/systemd/system/csf-core.service; enabled; vendor preset: enabled)
     Active: active (running) since ...
```

---

## 7️⃣ Tests durchführen

### Test 1: sudo ohne Passwort

```bash
sudo -u csf-core sudo /bin/bash -c "echo 'sudo works!'"
```

**Erwartete Ausgabe:** `sudo works!` (OHNE Passwort-Abfrage!)

### Test 2: Status-Datei Test

```bash
# Als csf-core User testen
sudo -u csf-core bash -c '
  echo "test" > /tmp/test-status.json
  sudo rm -f /tmp/test-status.json
  sudo chmod 644 /tmp/test-status.json
  echo "Status file operations work!"
'
```

### Test 3: Update-Skript Test (Dry-Run)

```bash
# ACHTUNG: Startet echtes Update! Nur wenn du bereit bist.
# sudo -u csf-core sudo /bin/bash /opt/csf-core/scripts/update.sh 0.4.11
```

---

## 8️⃣ Live-Logs beobachten (in separatem Terminal)

```bash
# In einem zweiten SSH-Terminal
journalctl -u csf-core.service -f
```

Jetzt kannst du in der Web-UI das Update triggern und die Logs live sehen!

---

## 🎯 Verifikation

Nach erfolgreicher Durchführung:

1. ✅ Service läuft: `systemctl status csf-core.service`
2. ✅ Web-UI erreichbar
3. ✅ Update über Web-UI funktioniert (Settings → Updates → Install Update)
4. ✅ Keine Fehler mehr in Logs

---

## 🔍 Troubleshooting

### Problem: "parsed OK" schlägt fehl bei sudoers

**Lösung:** Syntax-Fehler in `/etc/sudoers.d/csf-core`

- Prüfe auf fehlende Leerzeichen
- Prüfe auf Tippfehler in Pfaden
- Kopiere nochmal den kompletten Text aus Schritt 2

### Problem: Service startet nicht

**Lösung:**

```bash
# Logs prüfen
journalctl -u csf-core.service -n 100 --no-pager

# Konfiguration testen
systemd-analyze verify csf-core.service
```

### Problem: "Permission denied" bleibt

**Lösung:**

```bash
# Alle alten Status-Dateien löschen
rm -f /tmp/csf-core-update-status.json
rm -f /var/tmp/csf-core-update-status.json

# Update-Skript nochmal prüfen (Schritt 4)
cat /opt/csf-core/scripts/update.sh | grep -A 15 "update_status()"
```

---

## 📋 Checkliste

- [ ] Service-File angepasst (NoNewPrivileges entfernt, /tmp hinzugefügt)
- [ ] sudoers-Datei komplett neu erstellt
- [ ] sudoers validiert (`visudo -c` → "parsed OK")
- [ ] sudoers Berechtigungen gesetzt (0440)
- [ ] Update-Skript angepasst (update_status Funktion)
- [ ] Alte Status-Datei gelöscht
- [ ] systemctl daemon-reload ausgeführt
- [ ] Service neu gestartet
- [ ] sudo-Test erfolgreich (ohne Passwort)
- [ ] Service läuft (systemctl status)
- [ ] Update über Web-UI getestet

---

## 🎉 Fertig!

Nach diesen Schritten sollte das Update-System vollständig funktionieren. Du kannst jetzt Updates über die Web-UI installieren, ohne dass der "no new privileges" oder "Permission denied" Fehler auftritt.

**Bei Problemen:** Logs prüfen mit `journalctl -u csf-core.service -n 100`
