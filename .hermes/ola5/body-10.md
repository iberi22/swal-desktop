# [Ola 5.10] feat-swal-50 — E2E Integration Test Suite for Centralized Settings & Generative AUI

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-node-daemon/` has unit tests for supervisor and Xavier client.
- No end-to-end integration test suite exists verifying the full cycle: settings mutation via IPC, macOS layout generation, agent action card dynamic streaming, and doctor self-healing.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/tests/test_settings_aui_e2e.rs`:
  - Comprehensive integration test suite:
    - `test_settings_store_mutation_and_persistence`
    - `test_settings_window_layout_all_categories`
    - `test_agent_action_card_generation_and_payload_parsing`
    - `test_settings_ipc_server_client_roundtrip`
    - `test_settings_cli_runner_subcommands`
    - `test_doctor_engine_diagnostics_and_auto_fix`
- **File Target**: `crates/swal-node-daemon/tests/test_settings_aui_e2e.rs`

## Web Research Required
1. search: "rust integration test multi-module tokio unix ipc"
2. search: "e2e settings mutation test rust"
3. search: "agent action card roundtrip integration test"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo test -p swal-node-daemon --test test_settings_aui_e2e` — all tests pass
- [ ] `grep -rn "test_settings_store_mutation" crates/swal-node-daemon/tests/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/tests/test_settings_aui_e2e.rs` | Non-existent | [NEW] Unified E2E integration test suite for settings & AUI | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Import and test modules cleanly through `swal_node_daemon` and `swal_a2ui_engine`.
2. Ensure tests run reliably in CI/sandbox without port or socket conflicts.

## Merge Order
- **Merge order within wave:** 10
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
