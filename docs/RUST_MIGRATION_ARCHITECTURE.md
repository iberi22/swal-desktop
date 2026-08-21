# SWAL Desktop — Rust Native Core & Generative A2UI Migration Plan

## 1. Executive Summary & Rationale
To achieve maximum performance (sub-millisecond latency, zero garbage collection pauses, memory safety, and 0% idle CPU overhead), the SWAL Desktop shell and agentic runtime is transitioning to a **Native Rust Architecture** (`swal-desktop-core` / `swal-node-daemon`).

---

## 2. Architectural Components

```
+-----------------------------------------------------------------------------------+
|                        SWAL DESKTOP RUST CRATE ECOSYSTEM                          |
+-----------------------------------------------------------------------------------+
| 1. swal-telemetry-rs     | Direct /proc & sysfs reader (<0.2ms sampling time)     |
| 2. swal-a2ui-engine      | JSON Declarative Parser & Native UI Component Compiler  |
| 3. swal-ambient-orb      | GLSL Shader Audio & Voice Reactive Visualizer Surface   |
| 4. swal-theme-rs         | Fast JSON token compiler & Hyprland/GTK IPC broadcaster|
| 5. swal-node-daemon      | Tokio background async coordinator & Xavier/Mesh Bridge |
+-----------------------------------------------------------------------------------+
```

---

## 3. Component Specifications

### 3.1 `swal-telemetry-rs` (Native Telemetry & Process Monitor)
- **Problem**: Python scripts executing `ps`, `sensors`, and reading `/proc` incur process creation overhead (~15-30ms per tick).
- **Rust Implementation**:
  - Direct zero-allocation parsing of `/proc/stat`, `/proc/meminfo`, and `/proc/[pid]/statm`.
  - NVML / sysfs bindings for direct GPU clock, load, VRAM, and thermal junction temperatures.
  - Broadcasts structured telemetry via local IPC socket (`/run/user/$UID/swal/telemetry.sock`) with zero JSON serialization overhead.

### 3.2 `swal-a2ui-engine` (Generative UI Runtime)
- **JSON Contract**: Implements the canonical A2UI schema (`schemas/widget.schema.json`).
- **Component Registry**:
  - Implements the 15 `@swal/ui` canonical components natively: `Card`, `Grid`, `StatusBadge`, `Button`, `ButtonGroup`, `LogViewer`, `MetricPill`, `Terminal`, `Tabs`, `Table`, `Input`.
- **Dynamic Hot Reloading**:
  - Watches `~/.config/swal/widgets/*.json`.
  - When an AI agent (Hermes, Xavier) writes a new widget JSON, the engine instantly parses and mounts the UI component tree into the desktop shell without recompiling.

### 3.3 `swal-ambient-orb` (GLSL Shader Voice & Thought Reactive Visualizer)
- **Concept**: A centralized ambient modal showing an oscillating energy orb reacting to agent speech, audio input, and background reasoning cycles.
- **Rendering**: Uses `wgpu` or Wayland EGL sub-surfaces with custom GLSL fragment shaders:
  - **State 1 (Listening)**: Pulsing cyan ripples (`#06b6d4`).
  - **State 2 (Thinking / Xavier RAG)**: Multi-frequency orange wave interference (`#f97316`).
  - **State 3 (Speaking / Generating UI)**: Smooth morphing particle field rendering dynamic A2UI widgets.

### 3.4 `swal-node-daemon` (Local Services Coordinator)
- Monitors health of `xavier-core` (HTTP `:8006`, MCP `:8100`).
- Manages P2P mesh discovery through `edge-mesh`.
- Exposes Unix socket IPC for Hermes CLI and desktop tools.

---

## 4. Phased Migration Roadmap

| Phase | Milestone | Deliverable |
|---|---|---|
| **Wave 1 (Completed)** | Theme Engine, NixOS Services & Settings Panel | `swal-theme`, `@swal/ui` Hive Dark, `swal-doctor`, settings modal |
| **Wave 2 (Current)** | Native Rust Telemetry & A2UI Declarative Engine | `swal-desktop-core` Rust crate, JSON A2UI compiler, widget vault |
| **Wave 3** | Ambient Orb Modal & Full Wayland Integration | GLSL shader orb, voice visualizer, seamless desktop overlay |
