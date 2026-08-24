# [Ola 6.05] feat-swal-55 — Terminal UI (TUI) Mode for Remote SSH & Headless Linux in Pure Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-files/` only outputs JSON or relies on GUI window rendering.
- When connected via SSH or in a minimal headless Linux/Server environment without any X11/Wayland display, running `swal-files --tui` should provide an interactive, high-speed terminal interface.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/tui.rs`:
  - Structs & Enums:
    - `TuiLayoutMode`: `SinglePane`, `DualPane`, `PreviewFocused`, `HelpModal`.
    - `TuiColorTheme`: `SwalDark`, `CyberNeon`, `Monochrome`.
    - `TuiViewport`: `width: u16`, `height: u16`, `cursor_row: usize`, `scroll_offset: usize`.
    - `TuiRenderBuffer`: Buffer holding character cells and ANSI color escapes.
    - `TuiFileManagerApp`: Terminal app struct with methods:
      - `new(initial_path: &Path) -> Self`
      - `handle_key_event(&mut self, key_code: &str, is_ctrl: bool, is_alt: bool) -> TuiActionResponse`
      - `render_to_buffer(&self, viewport: &TuiViewport) -> String` (Generates ANSI terminal output with breadcrumbs, dual pane file lists, git status flags, and status bar)
      - `get_preview_text(&self, max_lines: usize) -> Vec<String>`
      - `toggle_dual_pane(&mut self)`
      - `search_filter(&mut self, query: &str)`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing TUI key navigation (up/down/enter/backspace/tab), ANSI string formatting, dual-pane splitting, and preview clipping with 100% test coverage.
- **File Target**: `crates/swal-files/src/tui.rs`

## Web Research Required
1. search: "rust terminal ui tui cell buffer ansi escape rendering"
2. search: "minimal rust tui file manager dual pane architecture"
3. search: "crossterm compatible key code event state machine"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "TuiFileManagerApp" crates/swal-files/src/tui.rs` >= 1 match
- [ ] `grep -rn "render_to_buffer" crates/swal-files/src/tui.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/tui.rs` | Non-existent | [NEW] Terminal UI mode with ANSI buffer renderer and 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/dual_pane.rs` and `crates/swal-files/src/session.rs` first.
2. Implement pure, safe Rust 2021 code without external heavy TUI framework dependencies and with complete unit tests.

## Merge Order
- **Merge order within wave:** 5
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
