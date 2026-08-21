# Changelog

All notable changes to the **SWAL Desktop** project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v1.2.0-stable] - 2026-08-20

### 🚀 Added
- **Native Rust Workspace (`crates/`)**:
  - `swal-telemetry-rs`: Zero-allocation `/proc` & GPU sysfs telemetry reader with Unix domain socket IPC streaming.
  - `swal-a2ui-engine`: Declarative JSON A2UI compiler translating agent blueprints into native component ASTs.
  - `swal-ambient-orb`: GLSL shader audio visualizer with lock-free amplitude consumer and thought state machine (`Listening`, `Thinking`, `Speaking`).
  - `swal-node-daemon`: Async Tokio supervisor connecting Xavier Memory Core (`:8006` / MCP `:8100`) and Edge-Mesh P2P network.
  - `swal-render-pipeline`: 200Hz - 240Hz frame budget scheduler with lock-free concurrency and atomic counters.
  - `swal-widget-vault`: Storage, watcher, and export/import bundle manager for dynamic agent widgets in `~/.config/swal/widgets/`.
- **Dynamic Multi-Theme Engine (`swal-theme`)**:
  - Canonical `@swal/ui` Hive Dark (`#020617`, `#06b6d4`, `#f97316`), Cyber Neon (`#00ff88`), and Nord Frost (`#88c0d0`).
  - Interactive Rofi Theme Picker (`swal-theme picker`).
  - Strict JSON Schema validation (`schemas/theme.schema.json`).
- **Interactive Manual Settings Panel (`swal_settings`)**:
  - Accessible via the orange Gear button (`⚙`) in the dashboard header.
  - 4 tabs: Apariencia & Temas, Nodo & Rust Core, Rendimiento, and Diagnóstico & Nix.
- **System Health & Diagnostic Linter (`swal-doctor`)**:
  - 5-point automated verification for SCSS encoding, YUCK syntax, log health, theme validation, and background daemons.
  - `--fix` flag for automatic hot-reloading and self-healing.
- **Autonomous Multi-Agent Wave Orchestration (GitCore v3.8.0)**:
  - Integration with Jules parallel waves (up to 15 concurrent micro-tasks with 100% disjoint file islands).
  - Complete architectural documentation: `ARCHITECTURE.md`, `AGENT_INDEX.md`, `SRS.md`, and `SESSION_HANDOFF.md`.

### 🔧 Fixed
- Fixed Eww SCSS compilation bug where UTF-8 characters caused `@charset "UTF-8";` injection, triggering GTK3 CSS parser rejections.
- Fixed non-blocking IPC re-entrancy deadlock in `swal-theme switch` when triggered from widget buttons.
- Fixed settings modal layout geometry and style persistence during runtime theme switching.
- Fixed NixOS OpenSSL dependency by migrating `reqwest` to `rustls-tls`.

---

## [v1.0.0-stable] - 2026-08-05

### 🚀 Initial Release
- Baseline NixOS Flake configuration with Hyprland window manager.
- Initial Eww system dashboard with CPU, RAM, GPU, storage, and calendar widgets.
- Process monitor with interactive kill actions.
- Initial AI canvas and agent CLI integration harness.
