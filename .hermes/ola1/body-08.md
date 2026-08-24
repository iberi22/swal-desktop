# [Ola 1.08] feat-render: Windows Fluent Dark Acrylic Shader Palette Integration

> Ola 1 — Render Pipeline & Mica Acrylic Shaders.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-render` in `crates/swal-render-pipeline`
- File: `crates/swal-render-pipeline/src/mica_shader.rs` (36 tests passing)
- Problem: The mica shader is hardcoded to Hive Dark palette values and does not support dynamic acrylic noise / tint blend factors for Windows Fluent Dark mode.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-render-pipeline/src/fluent_acrylic.rs` with `FluentAcrylicUniforms` supporting:
  - `tint_color: [f32; 4]`
  - `tint_opacity: f32` (default 0.70 for Fluent Dark)
  - `blur_radius: f32`
  - `noise_factor: f32` (0.02 procedural frosted texture)
- Add shader WGSL code and test verifying blend math.

## Web Research Required
1. search: "windows 11 acrylic material formula tint opacity noise"
2. search: "wgsl procedural noise texture hash without memory lookup"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all tests pass
- [ ] `grep -rn "FluentAcrylicUniforms" crates/swal-render-pipeline/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/fluent_acrylic.rs` | None (New) | Implement Fluent Acrylic shader and uniforms | LOW |
| `crates/swal-render-pipeline/src/lib.rs` | 70 lines | Export `pub mod fluent_acrylic;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-ambient-orb/` — assigned to Issue #7

## Anti-Hallucination Guard
1. Maintain 16-byte uniform alignment for WGSL compatibility
2. Verify shader passes headless pipeline validation in tests

## Merge Order
- **Merge order within wave:** 8
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
