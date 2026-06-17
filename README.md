<div align="center">

> [!CAUTION]
> **CSFX is under active development and not recommended for production use or as a secure system. Once ready, announcements will be made in this repository.**

# CSFX-Core

### Unified Infrastructure Management Platform

[![Release Pipeline](https://img.shields.io/github/actions/workflow/status/csfx-cloud/CSFX-Core/docker-build.yml?branch=main&label=Release-Pipeline&style=for-the-badge&logo=github)](https://github.com/csfx-cloud/CSFX-Core/actions)
[![Lint](https://img.shields.io/github/actions/workflow/status/csfx-cloud/CSFX-Core/lint.yml?branch=main&label=Lint&style=for-the-badge&logo=github&color=blueviolet)](https://github.com/csfx-cloud/CSFX-Core/actions)
[![Version](https://img.shields.io/github/v/release/csfx-cloud/CSFX-Core?style=for-the-badge&color=blue)](https://github.com/csfx-cloud/CSFX-Core/releases)
[![License](https://img.shields.io/badge/License-CSFX--Internal-purple?style=for-the-badge)](LICENSE)

</div>

---

## About

CSFX-Core is a distributed infrastructure management platform built with Rust and SvelteKit. It provides centralized control over nodes, workloads, volumes, and networks through a microservice control plane and a lightweight remote agent.

**Control Plane Services:**
- `api-gateway` — central HTTP API, JWT auth, frontend serving (port 8000)
- `registry` — node registration and agent tracking (port 8001)
- `scheduler` — workload scheduling via bin-packing (port 8002)
- `volume-manager` — persistent volume lifecycle (port 8003)
- `failover-controller` — node failure detection and workload rescheduling (port 8004)
- `sdn-controller` — overlay network and IPAM management (port 8005)

**Agent:** `csfx-agent` runs on each managed node and reports metrics, executes workloads, and manages local state.

---

## Alpha Testing

CSFX runs on **NixOS**. The full stack is distributed as a bootable ISO image.

**Installation:**
1. Download the latest ISO from [CSFX-Infra Releases](https://github.com/csfx-cloud/CSFX-Infra/releases)
2. Boot the target machine from the ISO and wait for installation to complete
3. Access the web interface at `http://<device-ip>:8000`

### Default Credentials

| Field    | Value           |
|----------|-----------------|
| Email    | admin@local.com |
| Password | admin           |

After first login you will be prompted to set a new password.

---

## License

Licensed under the **CSFX Internal Use License – Modified Shield License**.  
See [LICENSE](LICENSE) for full terms.

<div align="center">
<sub>&copy; 2026 CSFX. Built for scale.</sub>
</div>
