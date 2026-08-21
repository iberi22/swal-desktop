# [Ola 4.07] feat-swal-37 — Direct A2UI AST GPU Node Rasterizer (Zero-Eww)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-a2ui-engine/` compiles declarative JSON widgets to Yuck strings for Eww.
- No direct AST evaluator exists that converts `ComponentNode` trees directly into GPU draw commands and layout bounding boxes without generating `.yuck`.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-a2ui-engine/src/native_render.rs`:
  - Struct `LayoutRect` with `x: f32`, `y: f32`, `width: f32`, `height: f32`.
  - Enum `GpuDrawCommand`:
    - `DrawMicaCard { bounds: LayoutRect, bg_color: [f32; 4], border_color: [f32; 4], radius: f32 }`
    - `DrawText { text: String, x: f32, y: f32, font_size: f32, color: [f32; 4] }`
    - `DrawButton { bounds: LayoutRect, label: String, is_hovered: bool, action_id: String }`
    - `DrawProgressBar { bounds: LayoutRect, progress: f32, color: [f32; 4] }`
  - Function `evaluate_ast_to_gpu_commands(root: &ComponentNode, viewport: LayoutRect) -> Vec<GpuDrawCommand>`.
  - **Embedded Unit Tests**: Include complete unit tests evaluating all AST node variants (`Card`, `Button`, `ProgressBar`, `Box`) into exact GPU draw command sequences.
- **File Target**: `crates/swal-a2ui-engine/src/native_render.rs`

## Web Research Required
1. search: "declarative ui ast to draw commands rust"
2. search: "layout flexbox pure rust layout rect"
3. search: "ui component tree flatten draw list rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `cargo test -p swal-a2ui-engine` — all unit tests pass
- [ ] `grep -rn "GpuDrawCommand" crates/swal-a2ui-engine/src/native_render.rs` >= 1 match
- [ ] `grep -rn "evaluate_ast_to_gpu_commands" crates/swal-a2ui-engine/src/native_render.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/native_render.rs` | Non-existent | [NEW] Direct A2UI AST to GPU draw command evaluator with full tests | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/src/schema.rs` — schema definitions
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-a2ui-engine/src/lib.rs` and `schema.rs` before writing.
2. Support full recursive tree traversal over child nodes.

## Merge Order
- **Merge order within wave:** 7
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
