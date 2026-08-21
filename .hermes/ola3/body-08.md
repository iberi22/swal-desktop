# [Ola 3.08] feat-swal-23 — Async Unix Domain Socket IPC Daemon for Hermes Ambient Orb

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-ambient-orb/` runs in-process only.
- External agents like Hermes CLI or background daemons cannot push state packets to the ambient orb surface in real-time.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-ambient-orb/src/socket.rs`:
  - Struct `HermesOrbIpcServer` listening on `/tmp/swal_hermes_orb.sock`.
  - Non-blocking async message handler using `tokio::net::UnixListener`.
  - JSON protocol supporting messages: `{"cmd": "set_state", "state": "Thinking", "prompt": "..."}`, `{"cmd": "set_audio", "level": 0.85}`.
  - Client helper function `send_hermes_orb_event(event: &HermesOrbPacket) -> Result<(), String>`.
- **File Target**: `crates/swal-ambient-orb/src/socket.rs`

## Web Research Required
1. search: "tokio unix listener unix domain socket json daemon rust"
2. search: "ipc agent daemon unix socket async rust"
3. search: "non-blocking unix socket server tokio lines codec"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-ambient-orb` — 0 errors
- [ ] `grep -rn "HermesOrbIpcServer" crates/swal-ambient-orb/src/socket.rs` >= 1 match
- [ ] `grep -rn "send_hermes_orb_event" crates/swal-ambient-orb/src/socket.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-ambient-orb/src/socket.rs` | Non-existent | [NEW] Async Unix Domain Socket IPC daemon for Hermes agent | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — node daemon core
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Use `tokio` asynchronous I/O primitives already present in the workspace.
2. Ensure graceful socket cleanup (`std::fs::remove_file`) on shutdown.

## Merge Order
- **Merge order within wave:** 8
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
