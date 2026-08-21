# ⚡ SWAL Desktop: Autonomous Agentic Node & High-Refresh Shell

### NixOS + Hyprland + Native Rust Core (200Hz+) — The Official Desktop Environment for SouthWest AI Labs

<p align="center">
  <a href="https://nixos.org">
    <img src="https://img.shields.io/badge/NixOS-25.05-5277C3?style=for-the-badge&logo=nixos&logoColor=white" alt="NixOS"/>
  </a>
  <a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/badge/Rust-2021_Edition-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  </a>
  <a href="https://hyprland.org">
    <img src="https://img.shields.io/badge/Hyprland-200Hz+-00ff88?style=for-the-badge&logo=hyprland&logoColor=white" alt="Hyprland"/>
  </a>
  <a href="https://github.com/iberi22/swal-desktop">
    <img src="https://img.shields.io/badge/Version-v1.2.0--stable-06b6d4?style=for-the-badge" alt="Version"/>
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/iberi22/swal-desktop?style=for-the-badge&color=f97316" alt="License"/>
  </a>
</p>

---

> **"⚡ Execute, Automate, Resolve"** — SouthWest AI Labs

**SWAL Desktop** transforms any Linux workstation into a fully self-contained **SWAL Autonomous Node**. It merges cognitive long-term memory (**Xavier Core**), decentralized peer-to-peer communication (**Edge-Mesh**), high-frequency hardware telemetry, and generative **A2UI (Agent-to-UI)** interfaces into a unified 200Hz+ desktop shell.

---

## 🚀 Key Features

- **⚡ Native Rust Core (`crates/`)**: High-performance, zero-allocation workspace running at <0.2ms latency with sub-5.0ms frame budgeting (200Hz - 240Hz).
- **🎨 Dynamic Multi-Theme Engine (`swal-theme`)**: Declarative JSON design tokens conforming to `@swal/ui` standards (`SWAL Hive Dark`, `Cyber Neon`, `Nord Frost`) with instant Rofi selector and zero-downtime hot reloading.
- **✨ Generative A2UI Runtime (`swal-a2ui-engine`)**: AI agents (Hermes, Xavier) can construct and deploy interactive desktop widgets dynamically by emitting JSON blueprints into `~/.config/swal/widgets/` without writing or compiling code.
- **🌟 Ambient Orb Voice & Thought Visualizer (`swal-ambient-orb`)**: GLSL fragment shader surface reacting dynamically to microphone audio levels, agent voice, and background cognitive GraphRAG reasoning cycles.
- **🧠 Local Node Integration**: Auto-starting NixOS systemd services for Xavier Cognitive Memory Core (`:8006` / MCP `:8100`) and Edge-Mesh P2P daemon.
- **⚙️ Interactive Settings Modal & Gear Button (`swal_settings`)**: Comprehensive graphical control center for themes, hardware profiles, node services, and system self-healing.
- **🩺 SWAL Doctor & Linter (`swal-doctor`)**: Automated 5-point diagnostic CLI verifying SCSS encoding, YUCK syntax, log health, and service availability with `--fix` self-healing.

---

## 🏛️ System Architecture

```
+-------------------------------------------------------------------------+
|                       SWAL DESKTOP LAYER (200Hz+)                       |
+-------------------------------------------------------------------------+
|  🎨 @swal/ui Design System   |  🌟 Ambient Orb Shader  |  ⚙️ Settings UI |
|  (Hive Dark / Cyber Neon)    |  (GLSL Audio/Thought)   |  (Manual Config)|
+------------------------------+-------------------------+----------------+
|                     ✨ GENERATIVE A2UI RUNTIME                           |
|       (JSON Schema -> Native Component AST -> Widget Vault Indexer)     |
+-------------------------------------------------------------------------+
|                     ⚡ SWAL RUST CORE WORKSPACE                         |
|  • swal-telemetry-rs   : Zero-allocation /proc & GPU reader + Unix IPC  |
|  • swal-a2ui-engine    : Declarative JSON component tree AST compiler   |
|  • swal-ambient-orb    : Lock-free audio amplitude & GLSL shader state |
|  • swal-node-daemon    : Async Tokio supervisor & Xavier REST client    |
|  • swal-render-pipeline: 200Hz frame budget scheduler (<5.0ms per frame)|
|  • swal-widget-vault   : Agent widget persistence & export/import bundles|
+-------------------------------------------------------------------------+
|                    🧠 BACKEND AUTONOMOUS SERVICES                       |
|       - Xavier Cognitive Memory & GraphRAG (:8006 / MCP :8100)          |
|       - Edge-Mesh P2P Discovery & Yjs CRDT Synchronization             |
|       - Hermes / OpenCode Agent Orchestration Harness                   |
+-------------------------------------------------------------------------+
|                     🖥️ NIXOS FLAKE & HYPRLAND                           |
|       (Wayland Layer Shell, systemd user services, reproducible Flake)  |
+-------------------------------------------------------------------------+
```

---

## 🔄 Side-by-Side Coexistence Model

SWAL Desktop is designed with a **modular dual architecture**:
1. **Presentation Layer (Active Shell)**: Uses Eww Wayland Layer Shell widgets styled with pure ASCII SCSS and dynamic JSON tokens.
2. **Native Rust Backend**: The `crates/` workspace runs in parallel as a headless micro-daemon, broadcasting telemetry over `/run/user/$UID/swal/telemetry.sock` and validating generative A2UI schemas.
3. **Seamless Transition**: Users can switch between visual themes or activate the Rust A2UI engine on the fly without restarting their Wayland session.

---

## 🛠️ One-Line Installation

Deploy the complete SWAL Autonomous Node on any clean NixOS system:

```bash
curl -sSL https://raw.githubusercontent.com/iberi22/swal-desktop/main/scripts/install.sh | bash
```

To verify system health after installation:
```bash
swal-doctor --fix
```

---

## 🧪 Rust Workspace Test Suite

All 6 Rust crates include automated unit and integration tests:

```bash
cd ~/proyectosSWAL/periferia/swal-desktop
cargo test --workspace
```

```text
running 5 tests in swal_a2ui_engine      ... ok (Tokens, AST, Schemas)
running 8 tests in swal_ambient_orb      ... ok (GLSL Shaders, Audio Consumer, Uniforms)
running 7 tests in swal_node_daemon      ... ok (Tokio Supervisor, Xavier Client, MCP Health)
running 1 test  in swal_render_pipeline  ... ok (200Hz Frame Budget < 5.0ms)
running 5 tests in swal_telemetry_rs     ... ok (Direct /proc & IPC Server)
running 2 tests in swal_widget_vault     ... ok (CRUD, Pinning, Export/Import)

test result: ok. 28 passed; 0 failed in 0.12s
```

---

## 🎨 Design Tokens & Themes

SWAL Desktop uses the canonical `@swal/ui` design tokens:

| Token Name | Hex Code | Purpose |
|---|---|---|
| **`--swal-bg`** | `#020617` | Deep slate matte background |
| **`--swal-elevated`** | `#0f172a` | Container surface & cards |
| **`--swal-accent`** | `#06b6d4` | Primary Cyan accent |
| **`--swal-accent-orange`** | `#f97316` | Secondary Orange accent & alerts |
| **`--swal-success`** | `#10b981` | Node active & health indicators |
| **`--swal-text`** | `#f1f5f9` | High-contrast primary typography |

### Managing Themes via CLI & GUI:
```bash
swal-theme list              # List installed themes
swal-theme current           # Display active theme JSON
swal-theme switch hive-dark  # Switch to SWAL Hive Dark (@swal/ui)
swal-theme switch cyber-neon # Switch to Cyber Neon (Matrix)
swal-theme picker            # Open interactive Rofi GUI selector
```

---

## 🤖 Multi-Agent Orchestration Protocol

SWAL Desktop implements **GitCore v3.8.0** and autonomous multi-agent wave orchestration:
- **Hermes Orchestrator**: High-level wave planning, task decomposition, and local synthesis.
- **Jules Concurrent Execution**: Autonomous parallel delivery of up to **15 micro-tasks** across disjoint file islands.
- **Continuous Documentation**: Real-time progress metrics tracked in `.gitcore/features.json` and `.gitcore/SESSION_HANDOFF.md`.

---

## 📄 License

Distributed under the **MIT License**. Created with ⚡ by **SouthWest AI Labs**.

*Repository: [github.com/iberi22/swal-desktop](https://github.com/iberi22/swal-desktop)*
