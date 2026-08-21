# [Ola 3.13] feat-swal-28 — Integration Test Suite for Dual-Pane and Storage Engine

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-files/tests/` contains `test_file_viewer_formats.rs` and `test_full_core_coverage.rs`.
- No dedicated integration test exists validating dual-pane state synchronization and Linux disk usage scanning.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/tests/test_dual_pane_storage.rs`:
  - Test suite covering:
    - `test_dual_pane_split_and_focus_switching`
    - `test_dual_pane_path_synchronization`
    - `test_disk_usage_scanner_root_and_mounts`
    - `test_extended_tab_reordering_and_duplication`
- **File Target**: `crates/swal-files/tests/test_dual_pane_storage.rs`

## Web Research Required
1. search: "rust file manager dual pane integration test"
2. search: "statvfs mock testing rust linux"
3. search: "cargo integration test multi-module assertions"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo test -p swal-files --test test_dual_pane_storage` — all tests pass
- [ ] `grep -rn "test_dual_pane_split" crates/swal-files/tests/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/tests/test_dual_pane_storage.rs` | Non-existent | [NEW] Integration test suite for dual-pane layout and storage scanner | LOW |

## DO NOT touch
- `crates/swal-files/tests/test_full_core_coverage.rs` — core coverage suite
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use `tempfile::tempdir()` for all test directory fixtures.
2. Ensure tests run reliably in CI and sandbox environments.

## Merge Order
- **Merge order within wave:** 13
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
