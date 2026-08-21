# [Ola 4.02] feat-swal-32 — WGPU Graphics Context & Hardware Surface Initializer (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-render-pipeline/` tracks frame budgets in CPU time.
- No GPU graphics backend context (`wgpu::Instance`, `Adapter`, `Device`, `Queue`) exists for hardware-accelerated desktop rendering.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/wgpu_context.rs`:
  - Struct `WgpuSurfaceContext` with `instance: wgpu::Instance`, `adapter: wgpu::Adapter`, `device: wgpu::Device`, `queue: wgpu::Queue`, `surface_config: wgpu::SurfaceConfiguration`.
  - Methods: `new_headless() -> Self`, `configure_surface(width, height, format, present_mode)`, `create_command_encoder()`, `render_frame_target()`.
  - Support for `PresentMode::Immediate` (200Hz+ unlocked) and `PresentMode::Fifo` (VSync).
  - **Embedded Unit Tests**: Include complete unit tests in `wgpu_context.rs` verifying headless adapter creation, surface configuration math, and queue command submission.
- **File Target**: `crates/swal-render-pipeline/src/wgpu_context.rs`

## Web Research Required
1. search: "wgpu surface configuration present mode rust"
2. search: "wgpu headless test context device queue rust"
3. search: "wgpu hardware acceleration wayland surface linux"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "WgpuSurfaceContext" crates/swal-render-pipeline/src/wgpu_context.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/wgpu_context.rs` | Non-existent | [NEW] WGPU hardware graphics context and surface manager with unit tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use `wgpu` types with proper async/poll abstractions.
2. Provide a mock/headless initialization path for test environments without physical display servers.

## Merge Order
- **Merge order within wave:** 2
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
