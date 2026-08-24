# [Ola 6.08] feat-swal-58 — Standalone-to-Desktop Discovery & IPC Bridge in Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-node-daemon/src/native_shell.rs` manages local desktop Wayland surfaces.
- When `swal-files` is running on a secondary laptop, Windows machine, or separate Linux workstation, there is no discovery bridge to automatically pair the standalone instance with the master SWAL Node Daemon.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/desktop_bridge.rs`:
  - Structs & Enums:
    - `RemoteClientInfo`: `client_id: String`, `app_name: String`, `os: String`, `ip_address: String`, `port: u16`, `last_heartbeat: u64`, `protocol_version: String`.
    - `BridgeCommand`: `Ping`, `SyncSessionState { session_json: String }`, `RequestAgentInference { query: String }`, `NotifyDesktop { title: String, message: String }`.
    - `DesktopBridgeManager`: Manager struct with methods:
      - `new(listen_port: u16) -> Self`
      - `register_remote_client(&mut self, info: RemoteClientInfo)`
      - `prune_stale_clients(&mut self, timeout_secs: u64)`
      - `list_active_clients(&self) -> Vec<RemoteClientInfo>`
      - `broadcast_event_to_clients(&self, event_name: &str, payload: &str) -> usize`
      - `process_client_command(&self, client_id: &str, cmd: BridgeCommand) -> Result<String, String>`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing client registration, heartbeat renewal, stale client pruning, command dispatch, and serialization with 100% test coverage.
- **File Target**: `crates/swal-node-daemon/src/desktop_bridge.rs`

## Web Research Required
1. search: "udp broadcast beacon discovery rust"
2. search: "rust remote client session heartbeat supervisor"
3. search: "lan peer discovery json protocol"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "DesktopBridgeManager" crates/swal-node-daemon/src/desktop_bridge.rs` >= 1 match
- [ ] `grep -rn "RemoteClientInfo" crates/swal-node-daemon/src/desktop_bridge.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/desktop_bridge.rs` | Non-existent | [NEW] Standalone-to-desktop IPC discovery bridge with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-node-daemon/src/native_shell.rs` and `crates/swal-node-daemon/src/lib.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 8
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
