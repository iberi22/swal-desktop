# [Ola 1.04] feat-render: Multi-Touch Gesture Recognizer and Pointer Smooth Scrolling

> Ola 1 — Render Pipeline & Wayland Input.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-render` in `crates/swal-render-pipeline`
- File: `crates/swal-render-pipeline/src/events.rs` (36 tests passing)
- Problem: Wayland layer-shell pointer events only handle basic single click/hover without kinetic scroll or gesture pinch-to-zoom.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-render-pipeline/src/gestures.rs` containing `GestureRecognizer` for kinetic inertial scrolling (`scroll_friction: 0.92`) and pinch zoom delta calculation.
- Connect `GestureState` transitions (Began, Updated, Ended, Cancelled) with `PointerEvent`.

## Web Research Required
1. search: "rust gesture recognition touch event state machine"
2. search: "kinetic inertial scrolling math decay formula rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all tests pass
- [ ] `grep -rn "GestureRecognizer" crates/swal-render-pipeline/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/gestures.rs` | None (New) | Implement gesture recognition & kinetic scroll | LOW |
| `crates/swal-render-pipeline/src/lib.rs` | 70 lines | Export `pub mod gestures;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-telemetry-rs/` — assigned to Issue #3

## Anti-Hallucination Guard
1. READ before write: inspect `events.rs` and `layershell.rs`
2. Follow lock-free time calculations using `Instant`

## Merge Order
- **Merge order within wave:** 4
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
