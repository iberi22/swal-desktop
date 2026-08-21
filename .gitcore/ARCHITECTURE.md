# SWAL Desktop — Canonical Architecture Specification (GitCore v3.8.0)

## 1. System Mission & Node Role
`swal-desktop` is the desktop application shell and autonomous node management system for the SouthWest AI Labs (SWAL) ecosystem. It turns any workstation into a fully self-contained **SWAL Autonomous Node**, integrating cognitive memory (Xavier Core), decentralized communication (Edge-Mesh), and generative agentic UI.

---

## 2. Core Architectural Pillars

```
+-------------------------------------------------------------------------+
|                          SWAL DESKTOP LAYER                             |
+-------------------------------------------------------------------------+
|  🎨 @swal/ui Tokens     |  🌟 Ambient Orb Chat   |  📊 Dynamic Dashboard|
|  (Hive Dark / Matrix)   |  (GLSL Shader Audio)   |  (Telemetry & Memory)|
+-------------------------+------------------------+----------------------+
|                     ✨ GENERATIVE A2UI RUNTIME                           |
|       (JSON Schema -> Native Component Tree -> ~/.config/swal/widgets)  |
+-------------------------------------------------------------------------+
|                  ⚡ SWAL RUST CORE DAEMON (swal-node)                    |
|       (Tokio Async Loop, Wayland Layer Shell, Sysfs Telemetry, IPC)     |
+-------------------------------------------------------------------------+
|                    🧠 LOCAL BACKEND SERVICES                            |
|       - Xavier Cognitive Memory & GraphRAG (:8006 / MCP :8100)          |
|       - Edge-Mesh P2P Discovery & Yjs CRDT Synchronization             |
|       - Hermes / OpenCode Agent Orchestration Harness                   |
+-------------------------------------------------------------------------+
|                     🖥️ NIXOS FLAKE INFRASTRUCTURE                       |
|       (Reproducible system packages, systemd user services, Hyprland)   |
+-------------------------------------------------------------------------+
```

---

## 3. Non-Negotiable Architectural Invariants

1. **Design System Canonical Tokens**:
   - All visual elements must adhere to `@swal/ui` design tokens (`--swal-bg: #020617`, `--swal-elevated: #0f172a`, `--swal-accent: #06b6d4`, `--swal-accent-orange: #f97316`).
   - Alternative themes (`cyber-neon`, `nord-swal`) must conform to `schemas/theme.schema.json`.

2. **Zero-Overhead Native Telemetry**:
   - All hardware and process metrics must be read directly from `/proc` without spawning intermediate shell sub-processes.

3. **Autonomous Agentic Rails**:
   - AI agents (Hermes, Jules, Antigravity) must be able to create themes and widgets purely by emitting declarative JSON files without touching C/Rust compilation directly.

4. **Multi-Agent Disjoint File Islands**:
   - Every development Wave executed by autonomous Jules agents must maintain 100% disjoint file boundaries to prevent git merge conflicts.
