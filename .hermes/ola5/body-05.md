# [Ola 5.05] feat-swal-45 — GPU Rasterizer Extension for Interactive Settings Controls

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-render-pipeline/src/mica_shader.rs` and `text.rs` rasterize static cards and text.
- No specialized GPU draw command generator exists for interactive controls: animated toggle switches, slider tracks with thumbs, and color palette swatches.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/controls_render.rs`:
  - Struct `ControlsRenderer`:
    - `generate_toggle_draw_commands(bounds: LayoutRect, enabled: bool, thumb_pos: f32, accent_color: [f32; 4]) -> Vec<GpuDrawCommand>`.
    - `generate_slider_draw_commands(bounds: LayoutRect, progress: f32, accent_color: [f32; 4]) -> Vec<GpuDrawCommand>`.
    - `generate_swatch_draw_commands(bounds: LayoutRect, color_hex: &str, is_selected: bool) -> Vec<GpuDrawCommand>`.
  - Methods for calculating control hitboxes and thumb interpolation.
  - **Embedded Unit Tests**: Include complete unit tests validating vertex calculations, toggle thumb bounds, slider track fills, and color conversions.
- **File Target**: `crates/swal-render-pipeline/src/controls_render.rs`

## Web Research Required
1. search: "gpu toggle switch slider track quad draw commands rust"
2. search: "ui control rasterizer rounded track thumb wgpu"
3. search: "hex to rgba float array rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "ControlsRenderer" crates/swal-render-pipeline/src/controls_render.rs` >= 1 match
- [ ] `grep -rn "generate_toggle_draw_commands" crates/swal-render-pipeline/src/controls_render.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/controls_render.rs` | Non-existent | [NEW] Interactive settings controls GPU rasterizer with unit tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use pure Rust geometric math (`LayoutRect`, bounds calculations).
2. Include comprehensive unit test assertions.

## Merge Order
- **Merge order within wave:** 5
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
