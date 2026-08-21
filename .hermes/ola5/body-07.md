# [Ola 5.07] feat-swal-47 — Agent Real-Time Configuration Mutation IPC Protocol in Rust

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-ambient-orb/src/socket.rs` handles Hermes orb state packets.
- No dedicated Unix domain socket IPC daemon exists for querying and mutating system settings in real time from agents.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/agent_config_ipc.rs`:
  - Unix socket daemon on `/tmp/swal_settings.sock`.
  - JSON Request/Response protocol:
    - Request: `Get { key: String }`, `Set { key: String, value: String }`, `ListAll`, `SubscribeChanges`.
    - Response: `Value { key: String, value: String }`, `Ok { message: String }`, `AllSettings(SwalSystemSettings)`, `Error { error: String }`.
  - Broadcast channel notifying connected desktop surfaces when a setting changes.
  - **Embedded Unit Tests**: Include complete async unit tests verifying client-server IPC request/response roundtrip on temporary Unix sockets.
- **File Target**: `crates/swal-node-daemon/src/agent_config_ipc.rs`

## Web Research Required
1. search: "tokio unix listener request response ipc rust"
2. search: "broadcast channel live settings mutation rust"
3. search: "json ipc daemon unix domain socket rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "SettingsIpcServer" crates/swal-node-daemon/src/agent_config_ipc.rs` >= 1 match
- [ ] `grep -rn "SettingsIpcRequest" crates/swal-node-daemon/src/agent_config_ipc.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/agent_config_ipc.rs` | Non-existent | [NEW] Agent settings IPC protocol server and client with async tests | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-ambient-orb/src/socket.rs` for proven Tokio Unix socket patterns.
2. Use `tempfile::tempdir` for test sockets to avoid conflicts with running daemons.

## Merge Order
- **Merge order within wave:** 7
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
