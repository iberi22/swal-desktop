# [Ola 6.10] feat-swal-60 — Comprehensive E2E Test Suite for Standalone & Cross-Platform SWAL Files

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-files/tests/` contains tests for archive operations, directory watcher, and dual-pane views.
- No unified end-to-end integration test exists validating standalone execution, platform path normalization, agent protocol fallback, TUI rendering, plugin execution, and zero-Eww compatibility across OS targets.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/tests/test_standalone_crossplatform_e2e.rs`:
  - Test Cases:
    1. `test_cross_platform_path_and_system_folders_matrix`: Validates home/documents/drives detection and normalization across Windows/Linux path formats.
    2. `test_standalone_agent_protocol_offline_fallback_e2e`: Simulates query to offline agent endpoint and verifies deterministic local AI fallback response.
    3. `test_remote_cloud_sync_provider_registration_e2e`: Tests registering Edge-Mesh P2P and WebDAV accounts, mock listing, and sync state transitions.
    4. `test_standalone_runtime_mode_dispatcher_e2e`: Tests auto-detection of standalone window vs TUI vs layer shell based on environment mocks.
    5. `test_tui_file_manager_interactive_flow_e2e`: Instantiates `TuiFileManagerApp`, simulates key events (down, enter, tab), and asserts ANSI buffer content.
    6. `test_standalone_window_frame_a2ui_wrapping_e2e`: Verifies wrapping files component tree in `StandaloneWindowFrame` and caption hit testing.
    7. `test_file_plugin_system_registration_and_execution_e2e`: Creates temporary plugin manifest, tests extension matching (`.rs` / `.json`), and simulates execution.
  - **Embedded Unit Tests**: Minimum 7 high-coverage integration tests passing with 0 warnings.
- **File Target**: `crates/swal-files/tests/test_standalone_crossplatform_e2e.rs`

## Web Research Required
1. search: "rust integration test suite cross platform matrix"
2. search: "tempfile cross platform testing pattern rust"
3. search: "rust assert ansi string formatting in test"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files --tests` — 0 errors
- [ ] `cargo test --test test_standalone_crossplatform_e2e` — 7/7 tests pass
- [ ] `grep -rn "test_cross_platform_path" crates/swal-files/tests/test_standalone_crossplatform_e2e.rs` >= 1 match
- [ ] `grep -rn "test_tui_file_manager" crates/swal-files/tests/test_standalone_crossplatform_e2e.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/tests/test_standalone_crossplatform_e2e.rs` | Non-existent | [NEW] Full E2E integration test suite for standalone cross-platform files | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ existing integration tests in `crates/swal-files/tests/` first.
2. Implement robust integration tests using `tempfile` and deterministic mocks.

## Merge Order
- **Merge order within wave:** 10
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
