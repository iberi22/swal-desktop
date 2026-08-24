# [Ola 6.04] feat-swal-54 — Cross-Platform Standalone Window & App Runtime Dispatcher in Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-files/src/cli.rs` and `native_window.rs` assume execution either as an Eww IPC companion or a Hyprland Layer Shell window.
- When invoked on Windows, macOS, or generic X11/Wayland Linux desktop, the application needs an autonomous runtime dispatcher (`AppRuntimeMode`) that selects the appropriate window backend.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/app_runtime.rs`:
  - Structs & Enums:
    - `AppRuntimeMode`: `WaylandLayerShell`, `StandaloneWindow`, `TuiTerminal`, `HeadlessDaemon`, `WebCanvas`.
    - `WindowSettings`: `title: String`, `width: u32`, `height: u32`, `min_width: u32`, `min_height: u32`, `resizable: bool`, `decorations: bool`, `transparent: bool`, `always_on_top: bool`.
    - `AppRuntimeContext`: Context struct holding:
      - `mode: AppRuntimeMode`, `window_settings: WindowSettings`, `is_swal_desktop_present: bool`, `active_theme: String`.
    - `AppRuntimeDispatcher`: Dispatcher with methods:
      - `detect_optimal_mode() -> AppRuntimeMode` (Checks environment variables `WAYLAND_DISPLAY`, `DISPLAY`, `OS`, `SWAL_DESKTOP_ACTIVE`, TTY status)
      - `new_context(mode_override: Option<AppRuntimeMode>) -> AppRuntimeContext`
      - `build_initial_payload(&self) -> String`
      - `handle_window_lifecycle_event(&mut self, event: &str) -> bool`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing mode auto-detection, fallback order, window settings customization, and lifecycle events with 100% test coverage.
- **File Target**: `crates/swal-files/src/app_runtime.rs`

## Web Research Required
1. search: "rust detect wayland display x11 tty windows desktop environment"
2. search: "rust window lifecycle event dispatcher"
3. search: "cross platform desktop app runtime mode selection"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "AppRuntimeDispatcher" crates/swal-files/src/app_runtime.rs` >= 1 match
- [ ] `grep -rn "AppRuntimeMode" crates/swal-files/src/app_runtime.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/app_runtime.rs` | Non-existent | [NEW] Cross-platform standalone window and app runtime dispatcher with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/native_window.rs` and `crates/swal-files/src/cli.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 4
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
