# [Ola 1.07] feat-orb: Fluid Particle Morphing Shader for Speaking State

> Ola 1 — Ambient Orb & Shaders.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-orb` in `crates/swal-ambient-orb`
- File: `crates/swal-ambient-orb/src/shaders.rs` (15 tests passing, Listening and Thinking shaders)
- Problem: The `Speaking` state shader currently falls back to basic sine ripples instead of a high-density morphing particle field.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-ambient-orb/src/particle_shader.rs` with procedural WGSL particle dispersion and fluid SDF boundary morphing.
- Add `get_speaking_wgsl_shader() -> &'static str` and uniform bindings reacting to `audio_amplitude` (>0.4 triggering boundary bloom).
- Include headless shader compilation test.

## Web Research Required
1. search: "wgsl procedural particle field sdf distance field smoothstep"
2. search: "wgpu audio reactive particle shader uniforms math"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-ambient-orb` — 0 errors
- [ ] `cargo test -p swal-ambient-orb` — all tests pass
- [ ] `grep -rn "get_speaking_wgsl_shader" crates/swal-ambient-orb/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-ambient-orb/src/particle_shader.rs` | None (New) | Implement WGSL fluid particle shader | LOW |
| `crates/swal-ambient-orb/src/lib.rs` | 260 lines | Export `pub mod particle_shader;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-node-daemon/` — assigned to Issue #5

## Anti-Hallucination Guard
1. Use WGSL compliant syntax without non-standard extensions
2. Keep math zero-alloc and execution safe within 200Hz render budget (<2ms GPU cost)

## Merge Order
- **Merge order within wave:** 7
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
