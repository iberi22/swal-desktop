# [Ola 4.03] feat-swal-33 — Mica Acrylic Blur & Rounded Geometry Quad Shader in WGSL (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `eww/files-fluent.scss` and `themes/fluent-mica.json` define Mica colors in SCSS/JSON.
- No hardware GPU shader exists to rasterize rounded corner quads with Mica acrylic tint, 1px highlight borders, and smooth antialiased edges.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/mica_shader.rs`:
  - Embedded WGSL shader string `MICA_QUAD_WGSL_SHADER`:
    - Uniforms: `rect_bounds: vec4<f32>`, `corner_radius: f32`, `border_width: f32`, `border_color: vec4<f32>`, `bg_tint: vec4<f32>`, `blur_intensity: f32`.
    - Signed Distance Field (SDF) box calculation with smoothstep antialiasing.
  - Struct `MicaPipelineDescriptor` building WGPU render pipeline, bind group layouts, and vertex buffers.
  - **Embedded Unit Tests**: Include unit tests validating WGSL shader syntax compilation, uniform buffer packing, and SDF distance calculations.
- **File Target**: `crates/swal-render-pipeline/src/mica_shader.rs`

## Web Research Required
1. search: "wgsl rounded rectangle signed distance field shader"
2. search: "mica acrylic blur shader wgsl wgpu rust"
3. search: "sdf box border antialiasing smoothstep wgsl"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "MICA_QUAD_WGSL_SHADER" crates/swal-render-pipeline/src/mica_shader.rs` >= 1 match
- [ ] `grep -rn "MicaPipelineDescriptor" crates/swal-render-pipeline/src/mica_shader.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/mica_shader.rs` | Non-existent | [NEW] WGSL Mica Acrylic rounded rectangle shader with unit tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Ensure WGSL adheres to WebGPU standard syntax.
2. Validate uniform alignment (16-byte boundary alignment in WGSL).

## Merge Order
- **Merge order within wave:** 3
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
