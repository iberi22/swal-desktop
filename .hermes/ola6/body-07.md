# [Ola 6.07] feat-swal-57 — Unified Cross-Platform WGPU Surface Backend in Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-render-pipeline/src/layershell.rs` handles Wayland Layer Shell surface protocols directly.
- On Windows (DX12/Vulkan), macOS (Metal), or standard X11 Linux, `swal-render-pipeline` needs an abstracted cross-platform rendering surface (`CrossPlatformSurfaceAdapter`).

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/cross_platform_surface.rs`:
  - Structs & Enums:
    - `GpuBackendType`: `Auto`, `Vulkan`, `Dx12`, `Metal`, `Gl`.
    - `SurfacePresenterMode`: `WaylandLayerShell`, `StandardDesktopWindow`, `OffscreenHeadless`, `TuiBuffer`.
    - `SurfaceDescriptorConfig`: `width: u32`, `height: u32`, `scale_factor: f32`, `vsync: bool`, `backend: GpuBackendType`, `presenter: SurfacePresenterMode`.
    - `CrossPlatformSurfaceAdapter`: Adapter struct with methods:
      - `new(config: SurfaceDescriptorConfig) -> Self`
      - `resize(&mut self, new_width: u32, new_height: u32, scale_factor: f32)`
      - `is_vsync_enabled(&self) -> bool`
      - `format_target_fps(&self) -> u32`
      - `get_render_texture_bounds(&self) -> (u32, u32)`
      - `create_fallback_headless_context() -> Self`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing backend negotiation, resolution scaling math, headless fallback creation, and presenter state transitions with 100% test coverage.
- **File Target**: `crates/swal-render-pipeline/src/cross_platform_surface.rs`

## Web Research Required
1. search: "wgpu cross platform surface adapter vulkan dx12 metal"
2. search: "wgpu surface configuration vsync present mode"
3. search: "headless offscreen texture rendering wgpu rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "CrossPlatformSurfaceAdapter" crates/swal-render-pipeline/src/cross_platform_surface.rs` >= 1 match
- [ ] `grep -rn "SurfacePresenterMode" crates/swal-render-pipeline/src/cross_platform_surface.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/cross_platform_surface.rs` | Non-existent | [NEW] Unified cross-platform WGPU surface backend with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-render-pipeline/src/wgpu_context.rs` and `crates/swal-render-pipeline/src/layershell.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 7
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
