use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use swal_ambient_orb::socket::{HermesOrbIpcServer, DEFAULT_SOCKET_PATH};
use swal_ambient_orb::{LockFreeAudioConsumer, OrbController, OrbState};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("daemon");

    match command {
        "toggle" | "toggle-hud" => {
            send_ctl_command("toggle-orb-hud")?;
            println!("✓ Hermes Ambient Orb toggled via Native Rust Core.");
        }
        "thinking" | "think" => {
            send_ctl_command("orb-thinking")?;
            println!("✓ Hermes Ambient Orb state set to: Thinking");
        }
        "speaking" | "speak" => {
            send_ctl_command("orb-speaking")?;
            println!("✓ Hermes Ambient Orb state set to: Speaking");
        }
        "idle" | "listening" => {
            send_ctl_command("orb-idle")?;
            println!("✓ Hermes Ambient Orb state set to: Listening / Idle");
        }
        "status" => {
            let response = send_ctl_command("ping")?;
            println!("✓ SWAL Orb Daemon Status: {}", response.trim());
        }
        "daemon" | _ => {
            println!("🔮 Starting SWAL Ambient Orb 100% Native Rust Surface...");
            let audio_consumer = LockFreeAudioConsumer::new();
            let _controller = OrbController::new(audio_consumer.clone());

            let ipc_server = HermesOrbIpcServer::default_server(audio_consumer.clone());
            let server_handle = tokio::spawn(async move {
                if let Err(e) = ipc_server.run().await {
                    eprintln!("⚠ Orb IPC Server error: {}", e);
                }
            });

            println!("✓ Ambient Orb listening on {}", DEFAULT_SOCKET_PATH);
            println!("🚀 Ambient Orb Native Rust Render Loop Active (200Hz+ / Zero-EWW)");

            // Main event and animation loop (200Hz tick cycle)
            loop {
                tokio::select! {
                    _ = sleep(Duration::from_millis(5)) => {
                        let _state = audio_consumer.get_state();
                        let _amp = audio_consumer.get_audio_amplitude();
                        let _thought = audio_consumer.get_thought_trigger();
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("🛑 Shutting down SWAL Ambient Orb");
                        server_handle.abort();
                        let _ = std::fs::remove_file(DEFAULT_SOCKET_PATH);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn send_ctl_command(cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect("/tmp/swal_desktop_ctl.sock")?;
    stream.write_all(cmd.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    Ok(response)
}
