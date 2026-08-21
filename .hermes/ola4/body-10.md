# [Ola 4.10] feat-swal-40 — E2E Integration Test Suite for Pure Rust Native Desktop Pipeline

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-render-pipeline/` has a basic frame budget unit test.
- No end-to-end integration test suite exists validating Wayland Layer Shell surface creation, WGPU context initialization, WGSL Mica shader execution, and input event routing in a unified pipeline.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-render-pipeline/tests/test_native_pipeline_e2e.rs`:
  - Comprehensive integration test suite:
    - `test_layer_surface_config_and_anchors`
    - `test_wgpu_context_headless_render_tick_200hz`
    - `test_mica_shader_uniform_packing`
    - `test_orb_surface_hermes_state_rendering`
    - `test_spatial_hit_testing_pointer_events`
    - `test_glyph_renderer_text_layout_measurements`
- **File Target**: `crates/swal-render-pipeline/tests/test_native_pipeline_e2e.rs`

## Web Research Required
1. search: "rust integration test multi-module pipeline"
2. search: "wgpu integration test headless render pipeline"
3. search: "wayland layer shell mock integration test rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo test -p swal-render-pipeline --test test_native_pipeline_e2e` — all tests pass
- [ ] `grep -rn "test_wgpu_context_headless" crates/swal-render-pipeline/tests/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-render-pipeline/tests/test_native_pipeline_e2e.rs` | Non-existent | [NEW] Unified E2E integration test suite for native desktop pipeline | LOW |

## DO NOT touch
- `crates/swal-render-pipeline/src/lib.rs` — core frame scheduler
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Import and test modules cleanly through `swal_render_pipeline` and `swal_a2ui_engine`.
2. Ensure tests run reliably in CI/sandbox without headless panics.

## Merge Order
- **Merge order within wave:** 10
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
