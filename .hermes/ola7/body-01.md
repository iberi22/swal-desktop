# [Ola 7.01] feat-swal-61 — Unified Native Desktop Session Supervisor & Startup Orchestrator in Rust

> Ola 7 — [Core/Session/Zero-Eww].
> Labels: `ola7`, `wave-7`

---

## Current State (MEDIBLE)
- Feature: `feat-swal-61` at 0% in `.gitcore/features.json`
- Supervisor module: `crates/swal-node-daemon/src/native_shell.rs` exists and manages surface lifecycles.
- Existing tests in `crates/swal-node-daemon`: 77 passing tests.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-node-daemon/src/session_orchestrator.rs` providing a pure Rust session bootstrapper that starts and monitors the entire SWAL Desktop environment (daemon, visualizer orb, status bar, files manager, and desktop widgets) without relying on shell scripts.
- **File Target**: `crates/swal-node-daemon/src/session_orchestrator.rs`

## Web Research Required
1. search: "Rust async process supervision tokio Command Child wait restart policy"
2. search: "Wayland compositors session startup orchestration in pure Rust"
3. search: "desktop session lifecycle management IPC socket health probe"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all tests pass
- [ ] `grep -rn "SessionOrchestrator" crates/swal-node-daemon/src/session_orchestrator.rs` >= 1 match
- [ ] `grep -rn "SessionPhase" crates/swal-node-daemon/src/session_orchestrator.rs` >= 1 match
- [ ] `grep -rn "SessionConfig" crates/swal-node-daemon/src/session_orchestrator.rs` >= 1 match

## Exact Code Blueprint & Signatures

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

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
}
```

## Unit Tests Requirements
1. `test_session_orchestrator_initialization_and_default_config`
2. `test_session_lifecycle_state_transitions`
3. `test_socket_health_verification`
4. `test_managed_service_config_serde`
5. `test_session_shutdown_behavior`

## Anti-Hallucination Guard
- Do NOT edit other crates or shared files.
- Place all implementation strictly inside `crates/swal-node-daemon/src/session_orchestrator.rs`.
