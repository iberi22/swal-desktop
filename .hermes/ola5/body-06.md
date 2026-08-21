# [Ola 5.06] feat-swal-46 — Interactive Settings Hit-Testing & Value Mutation Controller

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-render-pipeline/src/events.rs` handles general spatial hit-testing for static rectangles.
- No specialized controller exists for mutating setting values on mouse drags (sliders), toggle clicks, and swatch clicks.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/settings_controller.rs`:
  - Struct `SettingsInteractionController`:
    - Tracks active drag state (`active_slider_drag: Option<(String, LayoutRect)>`).
    - Methods:
      - `handle_click(x: f64, y: f64, hitboxes: &[InteractiveControlHitBox]) -> Option<SettingMutationEvent>`.
      - `handle_drag(x: f64, y: f64) -> Option<SettingMutationEvent>`.
      - `end_drag()`.
  - Enum `SettingMutationEvent`:
    - `ToggleSwitched { key: String, new_value: bool }`
    - `SliderChanged { key: String, new_value: f32 }`
    - `OptionSelected { key: String, selected: String }`
    - `SwatchPicked { key: String, hex: String }`
  - **Embedded Unit Tests**: Include complete unit tests simulating click-to-toggle, drag-to-slide percentage calculation, and swatch pick events.
- **File Target**: `crates/swal-render-pipeline/src/settings_controller.rs`

## Web Research Required
1. search: "slider drag percentage calculation math rust"
2. search: "ui interaction state machine rust"
3. search: "event to mutation event dispatcher rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "SettingsInteractionController" crates/swal-render-pipeline/src/settings_controller.rs` >= 1 match
- [ ] `grep -rn "SettingMutationEvent" crates/swal-render-pipeline/src/settings_controller.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/settings_controller.rs` | Non-existent | [NEW] Interactive settings hit-testing and mutation controller with tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use robust clamping for slider drag values (`((x - bounds.x) / bounds.width).clamp(0.0, 1.0)`).
2. Include comprehensive unit test assertions.

## Merge Order
- **Merge order within wave:** 6
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
