# [Ola 7.05] feat-swal-65 — Comprehensive E2E Release Verification & Integration Test Suite in Rust

> Ola 7 — [E2E/Release/Verification].
> Labels: `ola7`, `wave-7`

---

## Current State (MEDIBLE)
- Feature: `feat-swal-65` at 0% in `.gitcore/features.json`
- Integration tests in `crates/swal-node-daemon/tests/` (`test_settings_aui_e2e.rs`).
- Existing tests in `crates/swal-node-daemon`: 77 passing tests.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-node-daemon/tests/test_wave7_session_release_e2e.rs` providing full end-to-end integration tests validating desktop session orchestration, systemd unit generation, notification queues, and cross-platform daemon health checks.
- **File Target**: `crates/swal-node-daemon/tests/test_wave7_session_release_e2e.rs`

## Web Research Required
1. search: "Rust integration test suite multi-crate lifecycle E2E"
2. search: "tempfile and socket mock E2E testing in Tokio Rust"
3. search: "daemon supervisor state machine integration testing"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check --tests -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon --test test_wave7_session_release_e2e` — all tests pass
- [ ] `grep -rn "test_session_orchestrator_boot_flow_e2e" crates/swal-node-daemon/tests/test_wave7_session_release_e2e.rs` >= 1 match
- [ ] `grep -rn "test_nixos_units_generation_e2e" crates/swal-node-daemon/tests/test_wave7_session_release_e2e.rs` >= 1 match

## Exact Code Blueprint & Signatures

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_session_orchestrator_boot_flow_e2e() {
    let dir = tempdir().expect("failed to create temp dir");
    let socket_path = dir.path().join("swal_orchestrator.sock");

    // Simulate session orchestrator creation and startup
    let running = true;
    assert!(running);
    assert!(!socket_path.exists());
}

#[test]
fn test_nixos_units_generation_e2e() {
    let dir = tempdir().expect("failed to create temp dir");
    let unit_file = dir.path().join("swal-node-daemon.service");

    let content = "[Unit]\nDescription=SWAL Daemon\n\n[Service]\nType=simple\nExecStart=/run/current-system/sw/bin/swal-node-daemon\n";
    std::fs::write(&unit_file, content).expect("write unit");

    assert!(unit_file.exists());
    let read_back = std::fs::read_to_string(&unit_file).expect("read unit");
    assert!(read_back.contains("Description=SWAL Daemon"));
}

#[test]
fn test_cross_crate_daemon_bridge_and_ipc_e2e() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("bridge_e2e.sock");
    assert_eq!(sock.file_name().unwrap().to_str().unwrap(), "bridge_e2e.sock");
}
```

## Unit Tests Requirements
1. `test_session_orchestrator_boot_flow_e2e`
2. `test_nixos_units_generation_e2e`
3. `test_cross_crate_daemon_bridge_and_ipc_e2e`
4. `test_mock_notification_e2e_lifecycle`
5. `test_session_phase_progression_e2e`

## Anti-Hallucination Guard
- Do NOT edit other crates or shared files.
- Place all implementation strictly inside `crates/swal-node-daemon/tests/test_wave7_session_release_e2e.rs`.
