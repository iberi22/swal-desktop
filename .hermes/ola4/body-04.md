# [Ola 4.04] feat-swal-34 — Hardware-Accelerated Hermes Ambient Orb Render Surface (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-ambient-orb/` contains Hermes state machine and GLSL shader strings.
- Rendering currently happens through external Eww windows.
- No direct GPU rendering loop exists in `swal-render-pipeline` for the Hermes Orb.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/orb_surface.rs`:
  - Struct `HermesOrbRenderSurface` holding WGPU pipeline, vertex buffer, and dynamic uniform buffer (`time`, `audio_level`, `thought_trigger`, `state_id`).
  - Methods: `update_uniforms(packet: &HermesOrbPacket)`, `render_to_texture(target_view, encoder)`, `tick_animation(delta_time)`.
  - WGSL port of `HERMES_COGNITION_VORTEX_SHADER` and `HERMES_A2UI_STREAM_SHADER`.
  - **Embedded Unit Tests**: Include complete unit tests verifying uniform updates, state color mappings, and shader pipeline generation with 100% test coverage.
- **File Target**: `crates/swal-render-pipeline/src/orb_surface.rs`

## Web Research Required
1. search: "wgpu render shader to texture buffer rust"
2. search: "animated voice assistant orb render loop wgpu"
3. search: "glsl to wgsl noise particle vortex port"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "HermesOrbRenderSurface" crates/swal-render-pipeline/src/orb_surface.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/orb_surface.rs` | Non-existent | [NEW] Native GPU Hermes Orb render surface and shader pipeline with tests | LOW |

## DO NOT touch
- `crates/swal-ambient-orb/src/hermes.rs` — Hermes agent protocol
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-ambient-orb/src/hermes.rs` and `hermes_shaders.rs` first.
2. Implement seamless interop with `HermesOrbPacket`.

## Merge Order
- **Merge order within wave:** 4
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
