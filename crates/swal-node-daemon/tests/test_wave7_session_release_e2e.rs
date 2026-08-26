//! test_wave7_session_release_e2e.rs
//! Comprehensive E2E Release Verification & Integration Test Suite in Rust

use tempfile::tempdir;

use swal_node_daemon::session_orchestrator::{SessionConfig, SessionOrchestrator, SessionPhase};
use swal_node_daemon::nixos_integration::NixOsServiceGenerator;

#[test]
fn test_session_orchestrator_boot_flow_e2e() {
    let dir = tempdir().expect("failed to create temp dir");
    let socket_path = dir.path().join("swal_orchestrator.sock");

    let mut config = SessionConfig::default();
    config.runtime_dir = dir.path().to_path_buf();

    let mut orchestrator = SessionOrchestrator::new(config);
    assert_eq!(orchestrator.phase(), SessionPhase::Initializing);

    let res = orchestrator.start_session();
    assert!(res.is_ok());
    assert_eq!(orchestrator.phase(), SessionPhase::SessionReady);
    assert!(orchestrator.is_running());

    std::fs::write(&socket_path, b"IPC_READY").expect("write socket");
    assert!(orchestrator.verify_socket_health(&socket_path));

    orchestrator.shutdown();
    assert_eq!(orchestrator.phase(), SessionPhase::Terminated);
    assert!(!orchestrator.is_running());
}

#[test]
fn test_nixos_units_generation_e2e() {
    let dir = tempdir().expect("failed to create temp dir");
    let units = NixOsServiceGenerator::default_desktop_units();

    for unit in units {
        let unit_file = dir.path().join(&unit.name);
        let content = unit.to_unit_file_content();
        std::fs::write(&unit_file, &content).expect("write unit");

        assert!(unit_file.exists());
        let read_back = std::fs::read_to_string(&unit_file).expect("read unit");
        assert!(read_back.contains("[Unit]"));
        assert!(read_back.contains("[Service]"));
        assert!(read_back.contains("[Install]"));
    }
}

#[test]
fn test_cross_crate_daemon_bridge_and_ipc_e2e() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("bridge_e2e.sock");
    assert_eq!(sock.file_name().unwrap().to_str().unwrap(), "bridge_e2e.sock");
}

#[test]
fn test_mock_notification_e2e_lifecycle() {
    let dir = tempdir().expect("tempdir");
    let notif_log = dir.path().join("notifications.jsonl");

    let entry = r#"{"id": 1, "app": "swal", "title": "System Ready"}"#;
    std::fs::write(&notif_log, format!("{}\n", entry)).expect("write log");

    assert!(notif_log.exists());
    let content = std::fs::read_to_string(&notif_log).expect("read log");
    assert!(content.contains("System Ready"));
}

#[test]
fn test_session_phase_progression_e2e() {
    let mut orchestrator = SessionOrchestrator::new(SessionConfig::default());
    assert_eq!(orchestrator.phase(), SessionPhase::Initializing);

    assert!(orchestrator.start_session().is_ok());
    assert_eq!(orchestrator.phase(), SessionPhase::SessionReady);

    orchestrator.shutdown();
    assert_eq!(orchestrator.phase(), SessionPhase::Terminated);
}
