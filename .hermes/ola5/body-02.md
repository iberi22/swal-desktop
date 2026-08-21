# [Ola 5.02] feat-swal-42 — A2UI Rich Settings Component Nodes (Toggle, Slider, Select, ColorSwatch)

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-a2ui-engine/` supports `Card`, `Grid`, `StatusBadge`, `MetricPill`, `Button`, `LogViewer`, `Terminal`, `Tabs`.
- No specialized declarative settings components (`Toggle`, `Slider`, `Select`, `ColorSwatch`, `SettingsGroup`) exist for building macOS-style preferences panels.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-a2ui-engine/src/settings_components.rs`:
  - Structs and builders for rich settings nodes:
    - `SettingsToggle { label: String, key: String, enabled: bool, description: Option<String> }`
    - `SettingsSlider { label: String, key: String, value: f32, min: f32, max: f32, step: f32, unit: String }`
    - `SettingsSelect { label: String, key: String, selected: String, options: Vec<String> }`
    - `SettingsColorSwatch { label: String, key: String, selected_hex: String, swatches: Vec<String> }`
    - `SettingsGroupNode { title: String, description: Option<String>, items: Vec<SettingsItemNode> }`
    - `SettingsItemNode` enum wrapping the above variants.
  - Serialization to/from JSON and token resolution support.
  - **Embedded Unit Tests**: Include complete unit tests verifying JSON serialization, token resolution, and value clamping on sliders and toggles.
- **File Target**: `crates/swal-a2ui-engine/src/settings_components.rs`

## Web Research Required
1. search: "macos preferences ui component declarative rust"
2. search: "settings card toggle slider component schema"
3. search: "color swatch palette picker rust ui"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `cargo test -p swal-a2ui-engine` — all unit tests pass
- [ ] `grep -rn "SettingsToggle" crates/swal-a2ui-engine/src/settings_components.rs` >= 1 match
- [ ] `grep -rn "SettingsSlider" crates/swal-a2ui-engine/src/settings_components.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/settings_components.rs` | Non-existent | [NEW] Declarative rich settings components with unit tests | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-a2ui-engine/src/schema.rs` and `lib.rs` first.
2. Ensure full serde derive support on all structs.

## Merge Order
- **Merge order within wave:** 2
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
