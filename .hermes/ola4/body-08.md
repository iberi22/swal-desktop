# [Ola 4.08] feat-swal-38 — Native SWAL Files GPU Window Layout Builder (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-files/` generates a JSON payload for Eww (`GuiPayload`).
- No native layout builder exists in `swal-files` that converts the active file manager session directly into an A2UI component tree for GPU rendering.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/native_window.rs`:
  - Struct `NativeFilesWindowBuilder`:
    - Builds header section: Tab strip with active indicator, close buttons, and new tab `+`.
    - Builds toolbar section: Nav buttons (Back, Up, Refresh), segmented breadcrumb chevrons, search bar, preview toggle.
    - Builds sidebar section: Favorites, Workspaces, Drive storage capacity progress bars.
    - Builds main content grid/list: File rows with icons, names, size, git badge, and selection highlights.
    - Builds right-side panel: QuickLook code/markdown/image preview card.
  - Method `build_native_a2ui_tree(session: &SessionState) -> ComponentNode`.
  - **Embedded Unit Tests**: Include complete unit tests verifying tree generation, sidebar drive entries, and tab count consistency.
- **File Target**: `crates/swal-files/src/native_window.rs`

## Web Research Required
1. search: "files-community/Files modern window layout structure"
2. search: "file manager ui component tree builder rust"
3. search: "fluent design 2 layout composition rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "NativeFilesWindowBuilder" crates/swal-files/src/native_window.rs` >= 1 match
- [ ] `grep -rn "build_native_a2ui_tree" crates/swal-files/src/native_window.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/native_window.rs` | Non-existent | [NEW] Native SWAL Files window layout builder for GPU rendering | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/session.rs`, `gui.rs`, and `storage.rs` first.
2. Use `swal_a2ui_engine::ComponentNode` types.

## Merge Order
- **Merge order within wave:** 8
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
