# [Ola 4.05] feat-swal-35 — GPU Typography & Glyph Rasterizer Engine (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- Typography currently depends on GTK3 Pango text layouts inside Eww.
- No native pure Rust GPU text layout and glyph caching engine exists in `swal-render-pipeline`.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/src/text.rs`:
  - Struct `GlyphRenderer`:
    - Manages font family resolution (`JetBrains Mono`, `Inter`, `Symbols Nerd Font Mono`).
    - Font metrics calculation: `measure_text(text: &str, font_size: f32) -> (f32, f32)`.
    - Text layout formatting: word wrapping, ellipsis truncation (`truncate_ellipsis`), line height, color attributes.
    - Vertex quad generator: converting laid-out text into colored texture quads for WGPU rendering.
  - **Embedded Unit Tests**: Include unit tests validating text dimension measurements, ellipsis truncation, multi-line wrapping, and font fallback resolution.
- **File Target**: `crates/swal-render-pipeline/src/text.rs`

## Web Research Required
1. search: "rust gpu text rendering measure text layout glyph quads"
2. search: "cosmic-text glyphon text rasterizer rust wgpu"
3. search: "font metrics text measure width height pure rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-render-pipeline` — 0 errors
- [ ] `cargo test -p swal-render-pipeline` — all unit tests pass
- [ ] `grep -rn "GlyphRenderer" crates/swal-render-pipeline/src/text.rs` >= 1 match
- [ ] `grep -rn "measure_text" crates/swal-render-pipeline/src/text.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/src/text.rs` | Non-existent | [NEW] GPU text layout and glyph measurement engine with full tests | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Implement pure Rust font measurement logic with fallback to monospace metrics calculation.
2. Include comprehensive unit tests for string measurement and truncation.

## Merge Order
- **Merge order within wave:** 5
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
