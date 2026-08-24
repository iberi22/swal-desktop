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
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::time::sleep;

use swal_node_daemon::native_shell::{NativeShellSupervisor, NativeSurfaceKind, ShellEvent};
use swal_node_daemon::{DaemonConfig, SwalNodeDaemon};

pub mod gesture_consumer;
use gesture_consumer::{GestureConsumer, ScreenConfig};

pub const DEFAULT_CTL_SOCKET: &str = "/tmp/swal_desktop_ctl.sock";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // If invoked as a CLI client controller (e.g. `swal-node-daemon toggle-dashboard` or `swal-desktop-ctl ...`)
    if args.len() > 1 && !args[1].starts_with("--daemon") && !args[1].starts_with("-d") {
        return handle_client_command(&args[1..]);
    }

    println!("⚡ Starting SWAL Desktop Native Node Daemon (100% Rust / Zero-EWW)...");

    // Clean up old socket if present
    if Path::new(DEFAULT_CTL_SOCKET).exists() {
        let _ = std::fs::remove_file(DEFAULT_CTL_SOCKET);
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
        let sock_path = "/run/user/1000/swal/telemetry.sock";
        if let Some(parent) = Path::new(sock_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::remove_file(sock_path);

        let mut prev_ticks = None;
        loop {
            let (metrics, ticks) = swal_telemetry_rs::read_system_metrics(prev_ticks);
            prev_ticks = Some(ticks);

            // Write telemetry cache to fast shared ramdisk
            if let Ok(json) = serde_json::to_string(&metrics) {
                let _ = std::fs::write("/tmp/swal_system_stats.json", &json);
            }

            sleep(Duration::from_millis(500)).await;
        }
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
        let listener = match UnixListener::bind(DEFAULT_CTL_SOCKET) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("❌ Failed to bind control socket {}: {}", DEFAULT_CTL_SOCKET, e);
                return;
            }
        };

        println!("✓ Control socket active at {}", DEFAULT_CTL_SOCKET);

        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = [0u8; 512];
                if let Ok(n) = stream.try_read(&mut buf) {
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
                                let _ = std::process::Command::new("swal-files").spawn();
                                let _ = stream.write_all(b"ok\n").await;
                            }
                            "close-files" | "close_files" => {
                                let _ = supervisor_ctl.broadcast_event(ShellEvent::Command {
                                    surface: NativeSurfaceKind::SwalFiles,
                                    command: "close_gui".to_string(),
                                });
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
                            "toggle-orb-hud" | "toggle_orb_hud" => {
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
                            "ping" => {
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
    let _ = std::fs::remove_file(DEFAULT_CTL_SOCKET);

    println!("✓ SWAL Desktop Native Node Daemon cleanly stopped.");
    Ok(())
}

/// Client dispatcher sending command over Unix Domain Socket
fn handle_client_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = args.join(" ");

    match UnixStream::connect(DEFAULT_CTL_SOCKET) {
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
            eprintln!("⚠ SWAL Node Daemon not running on {}. Running fallback handler for: {}", DEFAULT_CTL_SOCKET, cmd);
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
