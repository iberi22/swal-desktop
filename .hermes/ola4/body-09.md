# [Ola 4.09] feat-swal-39 — Native Desktop Daemon Supervisor in swal-node-daemon (Zero-Eww Launcher)

> Ola 4 — Pure Rust Native Wayland Layer Shell & GPU Rendering Engine.
> Labels: `ola4`, `wave-4`

---

## Current State (MEDIBLE)
- `crates/swal-node-daemon/` supervises Xavier GraphRAG and Edge-Mesh sync.
- Starting the GUI currently requires executing `eww daemon` and `eww open`.
- No supervisor exists in `swal-node-daemon` to launch and monitor native Rust Wayland Layer Shell surfaces.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/native_shell.rs`:
  - Struct `NativeShellSupervisor`:
    - Manages active native surfaces: `HermesOrb`, `SwalFiles`, `TelemetryBar`.
    - Spawns and supervises rendering threads.
    - IPC event router: connects `/tmp/swal_hermes_orb.sock` and telemetry feeds directly to the active GPU render loops.
    - Graceful shutdown handler capturing SIGINT / SIGTERM.
  - **Embedded Unit Tests**: Include unit tests validating surface registration, event broadcasting, and shutdown signals.
- **File Target**: `crates/swal-node-daemon/src/native_shell.rs`

## Web Research Required
1. search: "tokio actor supervisor thread manager rust"
2. search: "wayland shell surface lifecycle supervisor rust"
3. search: "zero-overhead desktop daemon supervisor rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "NativeShellSupervisor" crates/swal-node-daemon/src/native_shell.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/native_shell.rs` | Non-existent | [NEW] Native Layer Shell desktop supervisor and event router | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — node daemon core
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-node-daemon/src/lib.rs` and `xavier.rs` first.
2. Use async Tokio channels and atomic state flags.

## Merge Order
- **Merge order within wave:** 9
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
