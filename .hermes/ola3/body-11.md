# [Ola 3.11] feat-swal-26 — E2E Integration Test Suite for Hermes Orb & A2UI Streamer

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-ambient-orb/` has basic unit tests.
- No end-to-end integration test exists validating Hermes state transitions, IPC socket messages, and A2UI streaming payloads together.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-ambient-orb/tests/test_hermes_orb.rs`:
  - Test suite covering:
    - `test_hermes_state_transitions_all_variants`
    - `test_hermes_shader_lookup_and_fallback`
    - `test_hermes_ipc_packet_serialization_roundtrip`
    - `test_hermes_orb_progress_and_audio_clamping`
- **File Target**: `crates/swal-ambient-orb/tests/test_hermes_orb.rs`

## Web Research Required
1. search: "rust integration test tokio unix socket mock"
2. search: "serde json roundtrip test enum variants rust"
3. search: "cargo test integration test best practices"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo test -p swal-ambient-orb --test test_hermes_orb` — all tests pass
- [ ] `grep -rn "test_hermes_state_transitions" crates/swal-ambient-orb/tests/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-ambient-orb/tests/test_hermes_orb.rs` | Non-existent | [NEW] Integration test suite for Hermes Orb pipeline | LOW |

## DO NOT touch
- `crates/swal-ambient-orb/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use `tempfile` for any test sockets.
2. Assert strict state boundaries without unwrap panics.

## Merge Order
- **Merge order within wave:** 11
- **Expected effort:** Small (<20m)
- **Parallel with:** All other wave issues (disjoint file islands)
