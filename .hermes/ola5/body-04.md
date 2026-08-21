# [Ola 5.04] feat-swal-44 — macOS-Inspired Centralized Settings Window Layout Builder

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-files/src/native_window.rs` builds file manager layouts.
- No dedicated window layout builder exists for the macOS-style centralized system settings panel.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/settings_window.rs`:
  - Enums: `SettingsCategory` (`General`, `Appearance`, `Agents`, `Display`, `Storage`, `Xavier`, `Keybinds`, `Audio`, `About`).
  - Struct `SettingsWindowBuilder`:
    - `build_settings_layout(active_category: SettingsCategory, settings: &SwalSystemSettings) -> ComponentNode`.
    - Left column: Sidebar with category icons, labels, and active selection indicator.
    - Right column: Content panel rendering categorized `SettingsGroup` cards (Theme pickers, sliders for corner radius/opacity, toggles for AI agents, storage bars, doctor diagnostic button).
  - **Embedded Unit Tests**: Include complete unit tests verifying layout generation across all 9 categories and active category highlighting.
- **File Target**: `crates/swal-node-daemon/src/settings_window.rs`

## Web Research Required
1. search: "macos system preferences sidebar detail layout ui"
2. search: "apple settings app layout design patterns"
3. search: "two column settings panel declarative rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "SettingsWindowBuilder" crates/swal-node-daemon/src/settings_window.rs` >= 1 match
- [ ] `grep -rn "SettingsCategory" crates/swal-node-daemon/src/settings_window.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/settings_window.rs` | Non-existent | [NEW] macOS-inspired centralized settings layout builder with unit tests | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-a2ui-engine/src/lib.rs` and `swal-files/src/native_window.rs` first.
2. Use `swal_a2ui_engine::ComponentNode` types.

## Merge Order
- **Merge order within wave:** 4
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
