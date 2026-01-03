# CSF-Core Documentation

Willkommen in der CSF-Core Dokumentation! Diese Dokumentation ist in thematische Ordner strukturiert für bessere Übersichtlichkeit.

## 📁 Ordner-Struktur

```
docs/
├── README.md              # Diese Datei
├── architecture/          # System-Architektur & Design
├── development/           # Entwicklung & Testing
├── deployment/            # Installation & Deployment
├── troubleshooting/       # Fehlerbehebung & Debugging
├── internal/              # Interne Dokumentation
├── frontend/              # Frontend-spezifische Docs
└── .github/               # CI/CD & GitHub Workflows
```

## 🚀 Schnellzugriff

### Für neue Nutzer:

1. **[Installation](./deployment/INSTALLATION.md)** - Erste Schritte
2. **[Troubleshooting](./troubleshooting/TROUBLESHOOTING.md)** - Bei Problemen

### Für Entwickler:

1. **[Lokale Entwicklung](./development/LOCAL_DEVELOPMENT.md)** - Setup
2. **[Architektur](./architecture/ARCHITECTURE_AGENT_SYSTEM.md)** - System-Design
3. **[Agent Testing](./development/agent/TESTING.md)** - Agent-Entwicklung

### Für Deployment:

1. **[Deployment Guide](./deployment/DEPLOYMENT.md)** - Produktions-Setup
2. **[Docker Integration](./deployment/DOCKER_INTEGRATION_PLAN.md)** - Container-Setup

## 📊 Projekt-Status

- ✅ **Backend**: Rust + Axum, PostgreSQL/SQLite
- ✅ **Frontend**: SvelteKit + TailwindCSS
- ✅ **Agent**: Rust Binary für Remote-Management
- ✅ **Deployment**: Single systemd Service
- ✅ **Docker**: Vollständige Container-Unterstützung
- 🔄 **Testing**: Agent-Testing implementiert
- 📈 **Monitoring**: Self-Monitoring integriert

## 🔗 Wichtige Links

- **Repository**: https://github.com/CS-Foundry/CSF-Core
- **Issues**: https://github.com/CS-Foundry/CSF-Core/issues
- **Releases**: https://github.com/CS-Foundry/CSF-Core/releases
- **Main README**: [../README.md](../README.md)

## 📞 Support

Bei Fragen oder Problemen:

1. Prüfe die **[Troubleshooting](./troubleshooting/)** Dokumentation
2. Schaue in die **[Installation](./deployment/INSTALLATION.md)** Anleitung
3. Öffne ein [GitHub Issue](https://github.com/CS-Foundry/CSF-Core/issues)

## 🤝 Beitragen

Dokumentation beitragen:

1. Änderungen in den entsprechenden `docs/` Ordnern vornehmen
2. Links zu anderen Dokumenten aktualisieren
3. Neue Dokumente in passenden Ordnern platzieren
4. Diese `docs/README.md` bei neuen Ordnern aktualisieren

---

**Hinweis**: Alle Links sind relativ zum `docs/` Ordner. Verwende `../` um zum Projekt-Root zu navigieren.
