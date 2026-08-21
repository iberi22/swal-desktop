# [Ola 4.06] feat-swal-36 — Wayland Pointer, Keyboard Focus & Drag Input Dispatcher (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- User interaction events (click, scroll, hotkeys) are currently processed through GTK3 event loops in Eww.
- No direct Wayland input decoder and spatial hit-testing engine exists in Rust.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/events.rs`:
  - Enums: `PointerEvent` (`Motion(f64, f64)`, `ButtonPress(u32)`, `ButtonRelease(u32)`, `AxisScroll(f64, f64)`), `KeyEvent` (`Press(u32, u32)`, `Release(u32)`), `WindowEvent` (`FocusIn`, `FocusOut`, `CloseRequested`).
  - Struct `HitTestManager`:
    - Registers interactive rectangular areas (`HitBox { id, rect, cursor_shape, on_click }`).
    - Method `hit_test(x: f64, y: f64) -> Option<&HitBox>`.
    - Handles hover transitions (`MouseEnter`, `MouseLeave`).
  - **Embedded Unit Tests**: Include complete unit tests in `events.rs` testing spatial hit-testing math, multi-layer hitbox resolution, and keyboard shortcut event parsing.
- **File Target**: `crates/swal-render-pipeline/src/events.rs`

## Web Research Required
1. search: "wayland pointer input event handling rust"
2. search: "spatial hit testing ui rectangles rust"
3. search: "keyboard shortcut matcher rust wayland keycode"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "HitTestManager" crates/swal-render-pipeline/src/events.rs` >= 1 match
- [ ] `grep -rn "PointerEvent" crates/swal-render-pipeline/src/events.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/events.rs` | Non-existent | [NEW] Wayland input dispatcher and spatial hit-test manager with tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use robust, pure Rust spatial bounds collision testing (`x >= left && x <= right && y >= top && y <= bottom`).
2. Provide complete test suites for overlapping and nested hitboxes.

## Merge Order
- **Merge order within wave:** 6
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
