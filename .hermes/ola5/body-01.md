# [Ola 5.01] feat-swal-41 — Canonical System Settings JSON Schema & Store Engine in Rust

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-node-daemon/` supervises mesh nodes and Xavier health.
- Configuration is partially spread across CLI args and specific submodules.
- No unified central settings store (`SwalSystemSettings`) exists that can be modified by both GUI and AI agents.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/settings_store.rs`:
  - Structs:
    - `SwalSystemSettings`: `appearance: AppearanceSettings`, `agent: AgentSettings`, `display: DisplaySettings`, `storage: StorageSettings`, `network: NetworkSettings`, `audio: AudioSettings`.
    - `AppearanceSettings`: `theme: String`, `accent_color: String`, `corner_radius: f32`, `acrylic_opacity: f32`, `wallpaper_path: Option<String>`.
    - `AgentSettings`: `default_agent: String`, `model_routing: String`, `voice_orb_enabled: bool`, `audio_sensitivity: f32`, `auto_ui_generation: bool`.
    - `DisplaySettings`: `target_fps: u32`, `hidpi_scale: f32`, `vsync: bool`, `compositor: String`.
    - `StorageSettings`: `default_dual_pane: bool`, `show_hidden: bool`, `low_space_alert_gb: u32`.
    - `NetworkSettings`: `node_id: String`, `mesh_port: u16`, `xavier_endpoint: String`.
    - `AudioSettings`: `pipewire_sink: String`, `mic_gain: f32`.
  - Methods: `load_from_path(path: &Path) -> Self`, `save_to_path(&self, path: &Path) -> Result<()>`, `get_value(&self, key_path: &str) -> Option<String>`, `set_value(&mut self, key_path: &str, value: &str) -> Result<()>`.
  - **Embedded Unit Tests**: Include comprehensive unit tests testing default initialization, JSON roundtrip, dot-notation key lookups (`get_value("appearance.theme")`), and setting mutations with 100% test coverage.
- **File Target**: `crates/swal-node-daemon/src/settings_store.rs`

## Web Research Required
1. search: "rust hierarchical configuration dot notation getter setter"
2. search: "atomic json file write rust tempfile"
3. search: "macos system settings schema design rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "SwalSystemSettings" crates/swal-node-daemon/src/settings_store.rs` >= 1 match
- [ ] `grep -rn "get_value" crates/swal-node-daemon/src/settings_store.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/settings_store.rs` | Non-existent | [NEW] Canonical system settings store engine with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-node-daemon/src/lib.rs` first.
2. Implement pure, safe Rust 2021 code with serde derives and complete unit tests.

## Merge Order
- **Merge order within wave:** 1
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
