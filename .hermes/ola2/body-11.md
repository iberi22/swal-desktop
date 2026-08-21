# [Ola 2.01] feat-swal-11 — Native Rust Telemetry Core & Unix Socket IPC

> Ola 2 — Rust Native Core & Generative A2UI.
> Labels: `ola2`, `wave-2` (sin `jules` todavía)

---

## Current State (MEDIBLE)
- Feature: `feat-swal-11` at 25% in `.gitcore/features.json`
- File: `crates/swal-telemetry-rs/src/lib.rs` (58 lines, basic /proc/meminfo reader)
- Tests: 1 existing, 1 passing (`test_memory_reading`)

## Desired State (DELTA)
- **Direct /proc & sysfs parsing**: Read CPU ticks from `/proc/stat`, GPU load from sysfs/NVML without spawning bash subprocesses.
- **Unix Domain Socket Server**: Bind to `/run/user/1000/swal/telemetry.sock` and broadcast `SystemMetrics` stream using non-blocking Tokio loop.
- **File Island**: `crates/swal-telemetry-rs/`

## Web Research Required
1. search: "tokio UnixListener nonblocking broadcast rust 2026"
2. search: "parse /proc/stat cpu usage zero allocation rust"
3. search: "sysfs gpu usage telemetry linux rust"
4. search: "crossbeam channel low latency telemetry rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-telemetry-rs` — 0 errors
- [ ] `cargo test -p swal-telemetry-rs` — all tests pass
- [ ] `grep -rn "UnixListener" crates/swal-telemetry-rs/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-telemetry-rs/src/lib.rs` | 58 lines | Add CPU/GPU parser and metrics broadcaster | LOW |
| `crates/swal-telemetry-rs/src/ipc.rs` | NEW | Unix socket listener & streaming handler | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #12
- `crates/swal-ambient-orb/` — assigned to Issue #13
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ before write: read `crates/swal-telemetry-rs/src/lib.rs` completely
2. Use native `/proc` file paths, avoid hardcoding non-standard locations
3. Zero shell subprocess spawning (`std::process::Command` is prohibited in telemetry hot loop)

## Merge Order
- **Merge order within wave:** 1
- **Expected effort:** Medium (1-2h)
- **Parallel with:** #12, #13, #14, #15
