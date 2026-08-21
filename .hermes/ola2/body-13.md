# [Ola 2.03] feat-swal-13 — Ambient Voice & Thought Reactive Orb Surface

> Ola 2 — Rust Native Core & Generative A2UI.
> Labels: `ola2`, `wave-2` (sin `jules` todavía)

---

## Current State (MEDIBLE)
- Feature: `feat-swal-13` at 0% in `.gitcore/features.json`
- File: `crates/swal-ambient-orb/` (NEW directory)
- Tests: 0 existing

## Desired State (DELTA)
- **Crate Scaffold**: Create `crates/swal-ambient-orb/` with Cargo manifest.
- **Shader Pipeline**: Implement GLSL fragment shader definitions for 3 ambient states:
  - `Listening`: Pulsing cyan energy ripple (`#06b6d4`).
  - `Thinking`: Orange multi-frequency interference (`#f97316`).
  - `Speaking`: Morphed fluid particle boundary.
- **Audio Amplitude Consumer**: Lock-free receiver for microphone audio levels and Xavier thought triggers.
- **File Island**: `crates/swal-ambient-orb/`

## Web Research Required
1. search: "glsl audio visualizer fragment shader sphere noise"
2. search: "wgpu compute shader ambient particle orb rust"
3. search: "wayland layer shell subsurfaces opengl wgpu"
4. search: "lock free audio amplitude ringbuffer rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-ambient-orb` — 0 errors
- [ ] `cargo test -p swal-ambient-orb` — all tests pass
- [ ] `grep -rn "OrbState" crates/swal-ambient-orb/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-ambient-orb/Cargo.toml` | NEW | Crate manifest & dependencies | LOW |
| `crates/swal-ambient-orb/src/lib.rs` | NEW | Orb state machine & shader compiler | LOW |
| `crates/swal-ambient-orb/src/shaders.rs` | NEW | GLSL fragment shader source constants | LOW |

## DO NOT touch
- `crates/swal-telemetry-rs/` — assigned to Issue #11
- `crates/swal-a2ui-engine/` — assigned to Issue #12
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ workspace Cargo.toml before writing
2. Use standard wgpu / glsl types, pure ASCII source files

## Merge Order
- **Merge order within wave:** 3
- **Expected effort:** Medium (1-2h)
- **Parallel with:** #11, #12, #14, #15
