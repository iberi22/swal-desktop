//! session_orchestrator.rs
//! Unified Native Desktop Session Supervisor & Startup Orchestrator in Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionPhase {
    Initializing,
    StartingDaemon,
    LaunchingSurfaces,
    SessionReady,
    ShuttingDown,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedServiceConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub auto_restart: bool,
    pub max_restarts: u32,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_name: String,
    pub compositor: String,
    pub runtime_dir: PathBuf,
    pub services: Vec<ManagedServiceConfig>,
    pub heartbeat_interval_ms: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_name: "swal-desktop".to_string(),
            compositor: "hyprland".to_string(),
            runtime_dir: PathBuf::from("/run/user/1000/swal"),
            services: vec![
                ManagedServiceConfig {
                    name: "swal-node-daemon".to_string(),
                    command: "swal-node-daemon".to_string(),
                    args: vec!["--supervised".to_string()],
                    env_vars: HashMap::new(),
                    auto_restart: true,
                    max_restarts: 5,
                    critical: true,
                },
                ManagedServiceConfig {
                    name: "swal-orb".to_string(),
                    command: "swal-orb".to_string(),
                    args: vec![],
                    env_vars: HashMap::new(),
                    auto_restart: true,
                    max_restarts: 3,
                    critical: false,
                },
            ],
            heartbeat_interval_ms: 1000,
        }
    }
}

pub struct SessionOrchestrator {
    config: SessionConfig,
    phase: SessionPhase,
    running: Arc<AtomicBool>,
    service_restart_counts: HashMap<String, u32>,
}

impl SessionOrchestrator {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            phase: SessionPhase::Initializing,
            running: Arc::new(AtomicBool::new(false)),
            service_restart_counts: HashMap::new(),
        }
    }

    pub fn phase(&self) -> SessionPhase {
        self.phase
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    pub fn start_session(&mut self) -> Result<(), String> {
        self.phase = SessionPhase::StartingDaemon;
        self.running.store(true, Ordering::SeqCst);
        self.phase = SessionPhase::LaunchingSurfaces;
        self.phase = SessionPhase::SessionReady;
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.phase = SessionPhase::ShuttingDown;
        self.running.store(false, Ordering::SeqCst);
        self.phase = SessionPhase::Terminated;
    }

    pub fn verify_socket_health(&self, socket_path: &Path) -> bool {
        socket_path.exists()
    }

    pub fn restart_service(&mut self, service_name: &str) -> bool {
        let count = self.service_restart_counts.entry(service_name.to_string()).or_insert(0);
        if let Some(service) = self.config.services.iter().find(|s| s.name == service_name) {
            if *count < service.max_restarts {
                *count += 1;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_session_orchestrator_initialization_and_default_config() {
        let config = SessionConfig::default();
        let orchestrator = SessionOrchestrator::new(config);
        assert_eq!(orchestrator.phase(), SessionPhase::Initializing);
        assert!(!orchestrator.is_running());
        assert_eq!(orchestrator.config().session_name, "swal-desktop");
        assert_eq!(orchestrator.config().services.len(), 2);
    }

    #[test]
    fn test_session_lifecycle_state_transitions() {
        let mut orchestrator = SessionOrchestrator::new(SessionConfig::default());
        assert_eq!(orchestrator.phase(), SessionPhase::Initializing);

        let res = orchestrator.start_session();
        assert!(res.is_ok());
        assert_eq!(orchestrator.phase(), SessionPhase::SessionReady);
        assert!(orchestrator.is_running());

        orchestrator.shutdown();
        assert_eq!(orchestrator.phase(), SessionPhase::Terminated);
        assert!(!orchestrator.is_running());
    }

    #[test]
    fn test_socket_health_verification() {
        let dir = tempdir().expect("tempdir");
        let fake_sock = dir.path().join("test.sock");
        let orchestrator = SessionOrchestrator::new(SessionConfig::default());

        assert!(!orchestrator.verify_socket_health(&fake_sock));
        std::fs::write(&fake_sock, b"").expect("write");
        assert!(orchestrator.verify_socket_health(&fake_sock));
    }

    #[test]
    fn test_managed_service_config_serde() {
        let config = SessionConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: SessionConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.session_name, "swal-desktop");
        assert_eq!(deserialized.services[0].name, "swal-node-daemon");
    }

    #[test]
    fn test_service_restart_limit() {
        let mut orchestrator = SessionOrchestrator::new(SessionConfig::default());
        assert!(orchestrator.restart_service("swal-orb"));
        assert!(orchestrator.restart_service("swal-orb"));
        assert!(orchestrator.restart_service("swal-orb"));
        assert!(!orchestrator.restart_service("swal-orb")); // exceeded max_restarts (3)
    }
}
