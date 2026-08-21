# [Ola 4.01] feat-swal-31 — Wayland Layer Shell Protocol Surface Manager in Rust (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-render-pipeline/` contains `FrameScheduler` for 200Hz frame timing.
- Desktop UI currently relies on Eww (`gtk-layer-shell`).
- No native Wayland Layer Shell (`zwlr_layer_shell_v1`) surface manager exists in Rust.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/layershell.rs`:
  - Enums: `LayerType` (`Background`, `Bottom`, `Top`, `Overlay`), `AnchorEdge` (`Top`, `Bottom`, `Left`, `Right`), `KeyboardInteractivity` (`None`, `Exclusive`, `OnDemand`).
  - Struct `LayerSurfaceConfig` with `width: u32`, `height: u32`, `margin: (i32, i32, i32, i32)`, `anchors: Vec<AnchorEdge>`, `layer: LayerType`.
  - Struct `WaylandLayerSurface` handling surface configuration, commit states, resize ack, and protocol requests.
  - **Embedded Unit Tests**: Include comprehensive unit tests inside `layershell.rs` verifying config builders, anchor bitflags, and mock protocol states with 100% test coverage.
- **File Target**: `crates/swal-render-pipeline/src/layershell.rs`

## Web Research Required
1. search: "wayland layer shell protocol zwlr_layer_shell_v1 rust"
2. search: "smithay client toolkit layer shell surface rust"
3. search: "wayland layer surface anchor margin configuration rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "WaylandLayerSurface" crates/swal-render-pipeline/src/layershell.rs` >= 1 match
- [ ] `grep -rn "LayerSurfaceConfig" crates/swal-render-pipeline/src/layershell.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/layershell.rs` | Non-existent | [NEW] Native Wayland Layer Shell surface manager with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-render-pipeline/src/lib.rs` first.
2. Implement pure, safe Rust 2021 code with no unsafe pointer bugs. Include comprehensive unit test assertions.

## Merge Order
- **Merge order within wave:** 1
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
