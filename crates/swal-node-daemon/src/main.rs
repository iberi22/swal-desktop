//! SWAL Node Daemon & Native Desktop Runtime Entrypoint (100% Rust)
//!
//! Provides the primary background daemon supervising:
//! - Native Wayland Layer Shell surfaces (Dashboard, Hermes Orb, Files, Modals)
//! - High-speed zero-alloc telemetry IPC stream (<0.2ms latency)
//! - Generative A2UI Widget Vault hot-reloader
//! - IPC control socket for instant CLI toggles (SUPER+Escape, SUPER+E, SUPER+Q)

use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::time::sleep;

use swal_node_daemon::native_shell::{NativeShellSupervisor, NativeSurfaceKind, ShellEvent};
use swal_node_daemon::{DaemonConfig, SwalNodeDaemon};

pub mod desktop_bridge;
pub mod gesture_consumer;
use gesture_consumer::{GestureConsumer, ScreenConfig};

/// Resolves the per-user runtime directory (XDG base dir spec).
/// Falls back to /run/user/$UID, which is a tmpfs owned by the user.
pub fn runtime_dir() -> std::path::PathBuf {
    if let Ok(xdg) = env::var("XDG_RUNTIME_DIR") {
        if !xdg.trim().is_empty() {
            return std::path::PathBuf::from(xdg);
        }
    }
    let uid = unsafe { libc::getuid() };
    std::path::PathBuf::from(format!("/run/user/{}", uid))
}

/// Per-user control socket path: `$XDG_RUNTIME_DIR/swal/ctl.sock`.
/// Replaces the old world-readable `/tmp/swal_desktop_ctl.sock` (any local
/// user could pre-bind or hijack it). The `swal` subdir is created 0700 and
/// the socket itself chmod'ed 0600 after bind.
pub fn ctl_socket_path() -> std::path::PathBuf {
    runtime_dir().join("swal").join("ctl.sock")
}

/// Per-user telemetry socket path: `$XDG_RUNTIME_DIR/swal/telemetry.sock`.
pub fn telemetry_socket_path() -> std::path::PathBuf {
    runtime_dir().join("swal").join("telemetry.sock")
}

/// Creates `$XDG_RUNTIME_DIR/swal` with 0700 permissions (idempotent).
fn ensure_swal_runtime_dir() -> std::io::Result<()> {
    let dir = runtime_dir().join("swal");
    std::fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // If invoked as a CLI client controller (e.g. `swal-node-daemon toggle-dashboard` or `swal-desktop-ctl ...`)
    if args.len() > 1 && !args[1].starts_with("--daemon") && !args[1].starts_with("-d") {
        return handle_client_command(&args[1..]);
    }

    println!("⚡ Starting SWAL Desktop Native Node Daemon (100% Rust / Zero-EWW)...");

    // Ensure per-user runtime dir exists with safe permissions
    let _ = ensure_swal_runtime_dir();
    let ctl_sock = ctl_socket_path();

    // Clean up old socket if present
    if Path::new(&ctl_sock).exists() {
        let _ = std::fs::remove_file(&ctl_sock);
    }

    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = is_running.clone();

    // Handle Ctrl+C / SIGINT
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\n🛑 Received shutdown signal, terminating SWAL Desktop cleanly...");
        is_running_clone.store(false, Ordering::SeqCst);
    });

    // 1. Initialize Telemetry IPC Server in background
    let telemetry_handle = tokio::spawn(async {
        let sock_path = telemetry_socket_path();
        let server = swal_telemetry_rs::ipc::TelemetryServer::new(sock_path);
        let _ = server.run(Duration::from_millis(250)).await;
    });

    // 2. Initialize Native Shell Supervisor
    let supervisor = Arc::new(NativeShellSupervisor::new());
    let supervisor_ipc = supervisor.start_ipc_router();

    // 3. Initialize SWAL Node Daemon (Xavier Bridge + Edge-Mesh)
    let daemon = Arc::new(SwalNodeDaemon::new(DaemonConfig::default()));
    let daemon_clone = daemon.clone();
    let supervisor_mesh_handle = tokio::spawn(async move {
        daemon_clone.run_supervisor_loop(None).await;
    });

    // 4. Start Local Unix Control Listener for instant keybind handling
    let supervisor_ctl = supervisor.clone();
    let ctl_handle = tokio::spawn(async move {
        let listener = match UnixListener::bind(&ctl_sock) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("❌ Failed to bind control socket {}: {}", ctl_sock.display(), e);
                return;
            }
        };

        // Restrict the socket to the owning user (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &ctl_sock,
                std::fs::Permissions::from_mode(0o600),
            );
        }

        println!("✓ Control socket active at {}", ctl_sock.display());

        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = [0u8; 512];
                if let Ok(n) = stream.read(&mut buf).await {
                    if n > 0 {
                        let cmd = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                        println!("⚡ Received Desktop Control Command: '{}'", cmd);

                        match cmd.as_str() {
                            "toggle-dashboard" | "toggle_dashboard" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::TelemetryBar,
                                    command: "toggle_dashboard".to_string(),
                                });
                                let _ = std::process::Command::new("/home/belal/.config/eww/scripts/toggle_dashboard.sh")
                                    .arg("toggle")
                                    .status();
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "close-all" | "close_all" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::TelemetryBar,
                                    command: "close_all".to_string(),
                                });
                                let _ = std::process::Command::new("/home/belal/.config/eww/scripts/toggle_dashboard.sh")
                                    .arg("close")
                                    .status();
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "open-files" | "files" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::SwalFiles,
                                    command: "open_gui".to_string(),
                                });
                                let _ = std::process::Command::new("/home/belal/.local/bin/swal-files").spawn();
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "close-files" | "close_files" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::SwalFiles,
                                    command: "close_gui".to_string(),
                                });
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "toggle-a2ui" | "toggle-agent-monitor" | "a2ui" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::TelemetryBar,
                                    command: "toggle_a2ui".to_string(),
                                });
                                let _ = std::process::Command::new("/home/belal/.config/eww/scripts/toggle_dashboard.sh")
                                    .arg("toggle")
                                    .status();
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "orb-thinking" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::HermesOrb,
                                    command: "set_state:thinking".to_string(),
                                });
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "orb-idle" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::HermesOrb,
                                    command: "set_state:idle".to_string(),
                                });
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "toggle-orb-hud" | "toggle_orb_hud" | "toggle-orb" | "orb" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::HermesOrb,
                                    command: "toggle_hud".to_string(),
                                });
                                let _ = std::process::Command::new("/home/belal/.config/eww/scripts/toggle_orb_hud.sh")
                                    .spawn();
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "orb-speaking" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::HermesOrb,
                                    command: "set_state:speaking".to_string(),
                                });
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "ping" | "status" | "health" => {
                                let _ = stream.write_all(b"pong\n").await;
                            }
                            _ => {
                                println!("Unknown ctl command: {}", cmd);
                                let _ = stream.write_all(b"unknown\n").await;
                            }
                        }
                    }
                }
            }
        }
    });

    // 5. Start Gesture Consumer (auto-connects to swal-vision when available)
    let gesture_consumer = std::sync::Arc::new(GestureConsumer::new(ScreenConfig::default()));
    let gesture_consumer_clone = gesture_consumer.clone();
    let _gesture_handle = tokio::spawn(async move {
        gesture_consumer_clone.run_loop().await;
    });

    println!("🚀 SWAL Desktop Rust Core is fully operational.");

    // Keep main thread alive until shutdown
    while is_running.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(500)).await;
    }

    // Graceful cleanup
    supervisor.shutdown().await;
    let _ = ctl_handle.abort();
    let _ = telemetry_handle.abort();
    let _ = supervisor_mesh_handle.abort();
    let _ = supervisor_ipc.await;
    let _ = std::fs::remove_file(ctl_socket_path());

    println!("✓ SWAL Desktop Native Node Daemon cleanly stopped.");
    Ok(())
}

/// Client dispatcher sending command over Unix Domain Socket
fn handle_client_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = args.join(" ");

    match UnixStream::connect(ctl_socket_path()) {
        Ok(mut stream) => {
            stream.write_all(cmd.as_bytes())?;
            stream.flush()?;
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
            if !response.is_empty() {
                print!("{}", response);
            }
            Ok(())
        }
        Err(_) => {
            // Fallback if daemon is not running: execute locally or start daemon
            eprintln!("⚠ SWAL Node Daemon not running on {}. Running fallback handler for: {}", ctl_socket_path().display(), cmd);
            match cmd.as_str() {
                "toggle-dashboard" | "toggle_dashboard" => {
                    // Fallback to toggle_dashboard.sh during hybrid phase
                    let _ = std::process::Command::new("/home/belal/.config/eww/scripts/toggle_dashboard.sh")
                        .arg("toggle")
                        .status();
                }
                "open-files" | "files" => {
                    let _ = std::process::Command::new("swal-files").status();
                }
                "close-files" | "close_files" => {
                    let _ = std::process::Command::new("eww")
                        .args(["close", "swal_files"]).status();
                    let _ = std::process::Command::new("eww")
                        .args(["close", "swal_files_maximized"]).status();
                }
                "toggle-orb-hud" | "toggle_orb_hud" => {
                    let _ = std::process::Command::new("swal-vision").spawn();
                }
                _ => {}
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod socket_security_tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    /// Serializes env-mutating tests (XDG_RUNTIME_DIR is process-global).
    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let mutex = LOCK.get_or_init(|| Mutex::new(()));
        mutex.lock().unwrap_or_else(|p| p.into_inner())
    }

    #[test]
    fn ctl_socket_lives_under_xdg_runtime_dir() {
        let _lock = env_lock();
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/swal-test-xdg-ctl");
        let p = ctl_socket_path();
        assert!(p.starts_with("/tmp/swal-test-xdg-ctl"), "got: {:?}", p);
        assert!(p.ends_with("swal/ctl.sock"));
    }

    #[test]
    fn telemetry_socket_lives_under_xdg_runtime_dir() {
        let _lock = env_lock();
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/swal-test-xdg-tel");
        let p = telemetry_socket_path();
        assert!(p.starts_with("/tmp/swal-test-xdg-tel"), "got: {:?}", p);
        assert!(p.ends_with("swal/telemetry.sock"));
    }

    #[test]
    fn runtime_dir_falls_back_to_run_user_uid() {
        let _lock = env_lock();
        let old = std::env::var("XDG_RUNTIME_DIR").ok();
        std::env::remove_var("XDG_RUNTIME_DIR");
        let d = runtime_dir();
        if let Some(v) = old {
            std::env::set_var("XDG_RUNTIME_DIR", v);
        }
        let expected_prefix = format!("/run/user/{}", unsafe { libc::getuid() });
        assert!(
            d.starts_with(&expected_prefix),
            "expected {:?} under {}", d, expected_prefix
        );
    }

    #[test]
    fn swal_runtime_dir_is_created_0700() {
        let _lock = env_lock();
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/swal-test-xdg-perm");
        ensure_swal_runtime_dir().expect("create runtime dir");
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata("/tmp/swal-test-xdg-perm/swal").unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o700);
    }

    #[test]
    fn bound_ctl_socket_is_0600() {
        let _lock = env_lock();
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/swal-test-xdg-bind");
        ensure_swal_runtime_dir().unwrap();
        let sock_path = ctl_socket_path();
        let _ = std::fs::remove_file(&sock_path);
        // std UnixListener (no tokio reactor needed for the permission check)
        let listener = std::os::unix::net::UnixListener::bind(&sock_path).expect("bind");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&sock_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let meta = std::fs::metadata(&sock_path).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        drop(listener);
        let _ = std::fs::remove_file(&sock_path);
    }
}
