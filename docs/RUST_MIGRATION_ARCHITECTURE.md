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

## 4. Zero-Complexity Architectural Principles (State of the Art)

1. **Zero-Allocation Stack Buffers**:
   - File reads from `/sys/class/hwmon/` and `/sys/class/drm/` execute into pre-allocated stack slices (`[u8; 64]`) with byte-level parsing in-place (`0% allocation overhead`, `<0.1ms latency`).
2. **Unified Single Daemon Model**:
   - One background Tokio supervisor (`swal-node-daemon`) eliminates multi-process IPC collisions, zombie sockets, and GTK layer-shell desynchronizations.
3. **Wayland zwlr_layer_shell_v1 Protocol Native Surface**:
   - Direct surface state toggling without process teardown or GTK window destruction, guaranteeing zero orphan surfaces.
4. **Declarative A2UI Hot-Reloading**:
   - Widgets are parsed directly from canonical JSON schemas (`~/.config/swal/widgets/*.json`), allowing autonomous agents (Hermes/Xavier) to deploy dynamic UI cards at runtime.

---

## 5. Phased Migration Roadmap & Feature Tracking

| Phase | Milestone | Deliverable | Status |
|---|---|---|---|
| **Phase 1** | Native Telemetry & Zero-Alloc Metric Readers | `swal-telemetry-rs` direct `/proc` & `hwmon` reader (<0.1ms) | 🟢 90% |
| **Phase 2** | A2UI Generative Engine & Widget Vault | `swal-a2ui-engine`, `swal-widget-vault`, JSON AST compiler | 🟢 85% |
| **Phase 3** | Native Wayland Host & Render Loop | `swal-render-pipeline`, `wgpu` Mica shader, `swal-desktop-ctl` | 🟢 90% |
| **Phase 4** | Complete Zero-EWW Cutover & NixOS Switch | Full retirement of `eww daemon` and `eww.yuck` | 🟡 80% |

