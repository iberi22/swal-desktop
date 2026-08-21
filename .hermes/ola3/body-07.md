# [Ola 3.07] feat-swal-22 — GLSL Shaders for Hermes Thinking Particle Vortex & A2UI Waves

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-ambient-orb/src/shaders.rs` contains basic shaders (`CYAN_CYBER`, `ORANGE_THOUGHT`, `EMERALD_PARTICLE`).
- Lacks high-fidelity multi-octave simplex noise shader for Hermes Cognition Vortex and A2UI reactive energy stream.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-ambient-orb/src/hermes_shaders.rs`:
  - `HERMES_COGNITION_VORTEX_SHADER`: Multi-octave swirling particle vortex in deep indigo & electric cyan (`#06b6d4` & `#8b5cf6`).
  - `HERMES_A2UI_STREAM_SHADER`: Fluid dynamic sine wave ripple reflecting dynamic UI generation.
  - `HERMES_IDLE_BREATHE_SHADER`: Organic soft breathing gradient for ambient desktop state.
  - Public getter function `get_hermes_shader(state: &str) -> &'static str`.
- **File Target**: `crates/swal-ambient-orb/src/hermes_shaders.rs`

## Web Research Required
1. search: "glsl simplex noise 2d swirling vortex shader"
2. search: "ambient voice assistant glowing orb glsl fragment shader"
3. search: "raymarching glowing fluid sphere shader glsl"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-ambient-orb` — 0 errors
- [ ] `grep -rn "HERMES_COGNITION_VORTEX_SHADER" crates/swal-ambient-orb/src/hermes_shaders.rs` >= 1 match
- [ ] `grep -rn "get_hermes_shader" crates/swal-ambient-orb/src/hermes_shaders.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-ambient-orb/src/hermes_shaders.rs` | Non-existent | [NEW] GLSL shaders for Hermes cognition, A2UI stream, and idle breathe | LOW |

## DO NOT touch
- `crates/swal-ambient-orb/src/shaders.rs` — legacy shaders
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-ambient-orb/src/shaders.rs` for uniform interface compatibility.
2. Validate GLSL syntax strings for valid 330 core / ES 3.0 fragment shaders.

## Merge Order
- **Merge order within wave:** 7
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
