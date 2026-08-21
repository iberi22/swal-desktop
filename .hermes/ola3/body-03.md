# [Ola 3.03] feat-swal-18 — Windows Files Dual-Pane Layout Controller in Rust (swal-files)

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-files/` provides single-pane tab manager and session state.
- `dual_pane` flag exists in `SessionState` but lacks a dedicated controller for synchronized dual-pane navigation and split view management.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/dual_pane.rs` with `DualPaneController`:
  - Struct `DualPaneState` with `left_path: PathBuf`, `right_path: PathBuf`, `active_pane: PaneSide`, `split_ratio: f32`.
  - Methods: `toggle_dual_pane()`, `focus_left()`, `focus_right()`, `sync_panes()`, `swap_panes()`.
  - Serde serialization support for GUI payload integration.
- **File Target**: `crates/swal-files/src/dual_pane.rs`

## Web Research Required
1. search: "files-community/Files dual pane file manager layout"
2. search: "rust file manager dual pane state machine"
3. search: "yazi dual pane navigation rust implementation"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -rn "DualPaneController" crates/swal-files/src/dual_pane.rs` >= 1 match
- [ ] `grep -rn "PaneSide" crates/swal-files/src/dual_pane.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/dual_pane.rs` | Non-existent | [NEW] Dual pane controller and split-view state manager | LOW |

## DO NOT touch
- `crates/swal-files/src/main.rs` — CLI entry point
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/session.rs` and `crates/swal-files/src/lib.rs` first.
2. Implement standard Rust unit tests inside `dual_pane.rs` (`#[cfg(test)] mod tests`).

## Merge Order
- **Merge order within wave:** 3
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
