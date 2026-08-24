# [Ola 6.06] feat-swal-56 — Standalone Cross-Platform Window Frame & Titlebar Node in Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-a2ui-engine/` provides component nodes (`Card`, `Button`, `Tabs`, `Grid`, etc.).
- When running outside Wayland Layer Shell (e.g. on Windows 11 DWM or standard X11/macOS), windows need standard window frame decorations (minimize, maximize/restore, close buttons, draggable titlebar area, and custom menu bar).

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-a2ui-engine/src/standalone_window.rs`:
  - Structs & Enums:
    - `TitlebarStyle`: `FluentMica`, `MacOSTrafficLights`, `MinimalistFrameless`, `CustomSkin(String)`.
    - `WindowButtonKind`: `Close`, `Minimize`, `Maximize`, `Restore`, `PinToTop`, `Settings`.
    - `StandaloneWindowFrame`: Frame model struct holding:
      - `title: String`, `app_icon: String`, `style: TitlebarStyle`, `is_maximized: bool`, `is_focused: bool`, `show_breadcrumbs: bool`, `custom_actions: Vec<ComponentNode>`.
    - Methods:
      - `new(title: &str, style: TitlebarStyle) -> Self`
      - `with_custom_action(mut self, node: ComponentNode) -> Self`
      - `wrap_content_tree(&self, content_root: ComponentNode) -> ComponentNode` (Wraps any arbitrary A2UI widget tree with the standalone window titlebar, controls, and status footer)
      - `handle_caption_hit_test(x: f32, y: f32, width: f32) -> Option<WindowButtonKind>`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing frame wrapping, button hit testing, titlebar style serialization, and AST tree hierarchy with 100% test coverage.
- **File Target**: `crates/swal-a2ui-engine/src/standalone_window.rs`

## Web Research Required
1. search: "custom window frame titlebar hit test wgpu"
2. search: "fluent design system titlebar caption buttons"
3. search: "declarative ast window wrapper pattern rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `cargo test -p swal-a2ui-engine` — all unit tests pass
- [ ] `grep -rn "StandaloneWindowFrame" crates/swal-a2ui-engine/src/standalone_window.rs` >= 1 match
- [ ] `grep -rn "wrap_content_tree" crates/swal-a2ui-engine/src/standalone_window.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/standalone_window.rs` | Non-existent | [NEW] Standalone window frame builder with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-a2ui-engine/src/lib.rs` and `crates/swal-a2ui-engine/src/native_render.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 6
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
