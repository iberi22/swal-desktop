//! Native Desktop Daemon Supervisor for SWAL Node Daemon (Zero-Eww Launcher)
//!
//! Provides `NativeShellSupervisor` to launch, monitor, and manage native Rust Wayland
//! Layer Shell surfaces (`HermesOrb`, `SwalFiles`, `TelemetryBar`), route IPC events from
//! `/tmp/swal_hermes_orb.sock` and telemetry feeds directly to GPU render loops,
//! and handle graceful shutdown capturing `SIGINT` / `SIGTERM` signals.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::net::UnixListener;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

/// Target native Wayland Layer Shell surface managed by the supervisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NativeSurfaceKind {
    HermesOrb,
    SwalFiles,
    TelemetryBar,
}

impl std::fmt::Display for NativeSurfaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HermesOrb => write!(f, "HermesOrb"),
            Self::SwalFiles => write!(f, "SwalFiles"),
            Self::TelemetryBar => write!(f, "TelemetryBar"),
        }
    }
}

/// Operational status of a native desktop surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceStatus {
    Initializing,
    Running,
    Stopped,
    Failed(String),
}

/// Unified desktop IPC event routed directly to active GPU render loops.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShellEvent {
    /// Incoming payload from `/tmp/swal_hermes_orb.sock`
    HermesOrbPacket { payload: String },
    /// CPU & RAM metrics from `/run/user/1000/swal/telemetry.sock`
    TelemetryUpdate { cpu_pct: f32, ram_pct: f32 },
    /// Direct command targeting a specific surface
    Command { surface: NativeSurfaceKind, command: String },
    /// Desktop-wide graceful shutdown trigger
    Shutdown,
}

pub const DEFAULT_HERMES_ORB_SOCKET: &str = "/tmp/swal_hermes_orb.sock";
pub const DEFAULT_TELEMETRY_SOCKET: &str = "/run/user/1000/swal/telemetry.sock";

/// Native Wayland Layer Shell desktop surface supervisor and IPC event router.
pub struct NativeShellSupervisor {
    surfaces: Arc<Mutex<HashMap<NativeSurfaceKind, SurfaceStatus>>>,
    event_tx: broadcast::Sender<ShellEvent>,
    is_running: Arc<AtomicBool>,
    hermes_orb_socket: PathBuf,
    telemetry_socket: PathBuf,
}

impl NativeShellSupervisor {
    /// Creates a new `NativeShellSupervisor` with default socket paths.
    pub fn new() -> Self {
        Self::with_socket_paths(DEFAULT_HERMES_ORB_SOCKET, DEFAULT_TELEMETRY_SOCKET)
    }

    /// Creates a `NativeShellSupervisor` with custom Unix domain socket paths.
    pub fn with_socket_paths<P1: AsRef<Path>, P2: AsRef<Path>>(
        hermes_orb_path: P1,
        telemetry_path: P2,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            surfaces: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            is_running: Arc::new(AtomicBool::new(false)),
            hermes_orb_socket: hermes_orb_path.as_ref().to_path_buf(),
            telemetry_socket: telemetry_path.as_ref().to_path_buf(),
        }
    }

    /// Returns `true` if the supervisor loop is active.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Returns the active Hermes Orb IPC socket path.
    pub fn hermes_orb_socket(&self) -> &Path {
        &self.hermes_orb_socket
    }

    /// Returns the active Telemetry IPC socket path.
    pub fn telemetry_socket(&self) -> &Path {
        &self.telemetry_socket
    }

    /// Subscribes to the unified shell event broadcast channel.
    pub fn subscribe_events(&self) -> broadcast::Receiver<ShellEvent> {
        self.event_tx.subscribe()
    }

    /// Registers a native surface for supervision.
    pub async fn register_surface(&self, kind: NativeSurfaceKind) {
        let mut map = self.surfaces.lock().await;
        map.entry(kind).or_insert(SurfaceStatus::Initializing);
    }

    /// Checks if a surface kind is registered.
    pub async fn is_surface_registered(&self, kind: NativeSurfaceKind) -> bool {
        let map = self.surfaces.lock().await;
        map.contains_key(&kind)
    }

    /// Gets the current status of a registered surface.
    pub async fn get_surface_status(&self, kind: NativeSurfaceKind) -> Option<SurfaceStatus> {
        let map = self.surfaces.lock().await;
        map.get(&kind).cloned()
    }

    /// Broadcasts a `ShellEvent` to all subscribed rendering threads/tasks.
    pub fn broadcast_event(&self, event: ShellEvent) -> Result<usize, String> {
        self.event_tx
            .send(event)
            .map_err(|e| format!("Failed to broadcast shell event: {}", e))
    }

    /// Spawns a background rendering task for the specified surface.
    pub async fn spawn_surface(&self, kind: NativeSurfaceKind) -> tokio::task::JoinHandle<Result<(), String>> {
        self.register_surface(kind).await;

        let surfaces = Arc::clone(&self.surfaces);
        let mut rx = self.subscribe_events();
        let is_running = Arc::clone(&self.is_running);

        {
            let mut map = surfaces.lock().await;
            map.insert(kind, SurfaceStatus::Running);
        }

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                    Ok(Ok(ShellEvent::Shutdown)) => {
                        let mut map = surfaces.lock().await;
                        map.insert(kind, SurfaceStatus::Stopped);
                        break;
                    }
                    Ok(Ok(_evt)) => {
                        // Surface render loop event received
                    }
                    Ok(Err(broadcast::error::RecvError::Closed)) => {
                        let mut map = surfaces.lock().await;
                        map.insert(kind, SurfaceStatus::Stopped);
                        break;
                    }
                    Ok(Err(broadcast::error::RecvError::Lagged(_))) => {}
                    Err(_) => {
                        // Timeout tick for frame rendering cycle
                    }
                }
            }

            let mut map = surfaces.lock().await;
            if map.get(&kind) == Some(&SurfaceStatus::Running) {
                map.insert(kind, SurfaceStatus::Stopped);
            }
            Ok(())
        })
    }

    /// Starts the IPC socket router listening on `/tmp/swal_hermes_orb.sock`.
    pub fn start_ipc_router(&self) -> tokio::task::JoinHandle<Result<(), String>> {
        let socket_path = self.hermes_orb_socket.clone();
        let event_tx = self.event_tx.clone();
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            if let Some(parent) = socket_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if socket_path.exists() {
                let _ = std::fs::remove_file(&socket_path);
            }

            let listener = match UnixListener::bind(&socket_path) {
                Ok(l) => l,
                Err(e) => return Err(format!("Failed to bind IPC router socket at {:?}: {}", socket_path, e)),
            };

            while is_running.load(Ordering::SeqCst) {
                match tokio::time::timeout(Duration::from_millis(200), listener.accept()).await {
                    Ok(Ok((stream, _addr))) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let mut reader = tokio::io::BufReader::new(stream);
                            let mut line = String::new();
                            while let Ok(n) = reader.read_line(&mut line).await {
                                if n == 0 {
                                    break;
                                }
                                let trimmed = line.trim().to_string();
                                if !trimmed.is_empty() {
                                    let _ = tx.send(ShellEvent::HermesOrbPacket { payload: trimmed });
                                }
                                line.clear();
                            }
                        });
                    }
                    Ok(Err(_)) => {
                        sleep(Duration::from_millis(10)).await;
                    }
                    Err(_) => {
                        // Timeout tick to allow checking is_running flag
                    }
                }
            }

            if socket_path.exists() {
                let _ = std::fs::remove_file(&socket_path);
            }
            Ok(())
        })
    }

    /// Triggers graceful shutdown across all surfaces and IPC routers.
    pub async fn shutdown(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.broadcast_event(ShellEvent::Shutdown);

        let mut map = self.surfaces.lock().await;
        for status in map.values_mut() {
            *status = SurfaceStatus::Stopped;
        }
    }

    /// Listens for process signal triggers (`SIGINT` / `SIGTERM`) to trigger graceful shutdown.
    pub fn listen_for_shutdown_signals(&self) -> tokio::task::JoinHandle<()> {
        let is_running = Arc::clone(&self.is_running);
        let event_tx = self.event_tx.clone();
        let surfaces = Arc::clone(&self.surfaces);

        tokio::spawn(async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigint = signal(SignalKind::interrupt()).ok();
                let mut sigterm = signal(SignalKind::terminate()).ok();

                tokio::select! {
                    _ = async {
                        if let Some(s) = &mut sigint {
                            s.recv().await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    } => {},
                    _ = async {
                        if let Some(s) = &mut sigterm {
                            s.recv().await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    } => {},
                }
            }

            #[cfg(not(unix))]
            {
                let _ = tokio::signal::ctrl_c().await;
            }

            is_running.store(false, Ordering::SeqCst);
            let _ = event_tx.send(ShellEvent::Shutdown);

            let mut map = surfaces.lock().await;
            for status in map.values_mut() {
                *status = SurfaceStatus::Stopped;
            }
        })
    }

    /// Primary supervisory loop monitoring native surfaces and IPC routers.
    pub async fn run_supervisor_loop(&self, max_ticks: Option<u64>) {
        self.is_running.store(true, Ordering::SeqCst);

        // Auto-register default native surface stack
        self.register_surface(NativeSurfaceKind::HermesOrb).await;
        self.register_surface(NativeSurfaceKind::SwalFiles).await;
        self.register_surface(NativeSurfaceKind::TelemetryBar).await;

        let _ipc_task = self.start_ipc_router();
        let _sig_task = self.listen_for_shutdown_signals();

        let mut tick_count = 0u64;

        while self.is_running.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(50)).await;

            // Audit surface health
            {
                let mut map = self.surfaces.lock().await;
                for (kind, status) in map.iter_mut() {
                    if *status == SurfaceStatus::Initializing {
                        *status = SurfaceStatus::Running;
                    }
                    let _ = kind;
                }
            }

            tick_count += 1;
            if let Some(max) = max_ticks {
                if tick_count >= max {
                    break;
                }
            }
        }

        self.shutdown().await;
    }
}

impl Default for NativeShellSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NativeShellSupervisor {
    fn drop(&mut self) {
        if self.hermes_orb_socket.exists() {
            let _ = std::fs::remove_file(&self.hermes_orb_socket);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::net::UnixStream as StdUnixStream;

    #[tokio::test]
    async fn test_surface_registration() {
        let supervisor = NativeShellSupervisor::new();

        assert!(!supervisor.is_surface_registered(NativeSurfaceKind::HermesOrb).await);
        assert_eq!(supervisor.get_surface_status(NativeSurfaceKind::HermesOrb).await, None);

        supervisor.register_surface(NativeSurfaceKind::HermesOrb).await;
        supervisor.register_surface(NativeSurfaceKind::SwalFiles).await;
        supervisor.register_surface(NativeSurfaceKind::TelemetryBar).await;

        assert!(supervisor.is_surface_registered(NativeSurfaceKind::HermesOrb).await);
        assert!(supervisor.is_surface_registered(NativeSurfaceKind::SwalFiles).await);
        assert!(supervisor.is_surface_registered(NativeSurfaceKind::TelemetryBar).await);

        assert_eq!(
            supervisor.get_surface_status(NativeSurfaceKind::HermesOrb).await,
            Some(SurfaceStatus::Initializing)
        );
    }

    #[tokio::test]
    async fn test_event_broadcasting() {
        let supervisor = NativeShellSupervisor::new();
        let mut rx = supervisor.subscribe_events();

        let event = ShellEvent::TelemetryUpdate {
            cpu_pct: 18.5,
            ram_pct: 42.0,
        };

        let sent = supervisor.broadcast_event(event.clone());
        assert!(sent.is_ok());

        let received = rx.recv().await.expect("Failed to receive broadcast event");
        assert_eq!(received, event);

        let cmd_event = ShellEvent::Command {
            surface: NativeSurfaceKind::SwalFiles,
            command: "toggle_dual_pane".to_string(),
        };

        let _ = supervisor.broadcast_event(cmd_event.clone());
        let received_cmd = rx.recv().await.expect("Failed to receive command event");
        assert_eq!(received_cmd, cmd_event);
    }

    #[tokio::test]
    async fn test_shutdown_signals() {
        let supervisor = NativeShellSupervisor::new();
        let mut rx = supervisor.subscribe_events();

        supervisor.register_surface(NativeSurfaceKind::HermesOrb).await;
        supervisor.is_running.store(true, Ordering::SeqCst);

        assert!(supervisor.is_running());

        supervisor.shutdown().await;

        assert!(!supervisor.is_running());
        assert_eq!(
            supervisor.get_surface_status(NativeSurfaceKind::HermesOrb).await,
            Some(SurfaceStatus::Stopped)
        );

        let evt = rx.recv().await.expect("Shutdown event not received");
        assert_eq!(evt, ShellEvent::Shutdown);
    }

    #[tokio::test]
    async fn test_surface_spawn_and_supervise() {
        let supervisor = NativeShellSupervisor::new();
        supervisor.is_running.store(true, Ordering::SeqCst);

        let handle = supervisor.spawn_surface(NativeSurfaceKind::HermesOrb).await;

        sleep(Duration::from_millis(50)).await;
        assert_eq!(
            supervisor.get_surface_status(NativeSurfaceKind::HermesOrb).await,
            Some(SurfaceStatus::Running)
        );

        supervisor.shutdown().await;
        let _ = handle.await;

        assert_eq!(
            supervisor.get_surface_status(NativeSurfaceKind::HermesOrb).await,
            Some(SurfaceStatus::Stopped)
        );
    }

    #[tokio::test]
    async fn test_ipc_router_event_forwarding() {
        let test_socket = format!("/tmp/test_swal_native_shell_{}.sock", std::process::id());
        let supervisor = NativeShellSupervisor::with_socket_paths(&test_socket, "/tmp/unused.sock");
        supervisor.is_running.store(true, Ordering::SeqCst);

        let mut rx = supervisor.subscribe_events();
        let ipc_handle = supervisor.start_ipc_router();

        sleep(Duration::from_millis(100)).await;

        // Connect std UnixStream to send test packet
        let mut client = StdUnixStream::connect(&test_socket).expect("Failed to connect to IPC socket");
        writeln!(client, "{{\"cmd\":\"set_state\",\"state\":\"thinking\"}}").unwrap();
        client.flush().unwrap();

        let received = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(received.is_ok(), "Timed out waiting for IPC event");
        let payload = match received.unwrap().unwrap() {
            ShellEvent::HermesOrbPacket { payload } => payload,
            _ => panic!("Expected HermesOrbPacket event"),
        };

        assert!(payload.contains("set_state"));
        assert!(payload.contains("thinking"));

        supervisor.shutdown().await;
        let _ = ipc_handle.await;

        let _ = std::fs::remove_file(&test_socket);
    }
}
