//! Async Unix Domain Socket IPC Daemon for Hermes Ambient Orb
//!
//! Listens on Unix domain socket `/tmp/swal_hermes_orb.sock` to process incoming JSON packets
//! from external agents like Hermes CLI or background daemons.

use crate::{LockFreeAudioConsumer, OrbState};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::net::UnixListener;
use tokio::time::sleep;

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/swal_hermes_orb.sock";

/// IPC Packet for Hermes Ambient Orb events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum HermesOrbPacket {
    /// Update Orb state (e.g. Listening, Thinking, Speaking) with an optional prompt message
    SetState {
        state: String,
        #[serde(default)]
        prompt: Option<String>,
    },
    /// Update microphone or speaker audio level [0.0, 1.0]
    SetAudio {
        level: f32,
    },
    /// Update Xavier thought trigger intensity [0.0, 1.0]
    SetThought {
        intensity: f32,
    },
}

/// Helper function to parse OrbState from a string representation
fn parse_orb_state(state_str: &str) -> OrbState {
    match state_str.to_lowercase().as_str() {
        "thinking" => OrbState::Thinking,
        "speaking" => OrbState::Speaking,
        _ => OrbState::Listening,
    }
}

/// Async Unix Domain Socket IPC Server for Hermes Orb
pub struct HermesOrbIpcServer {
    socket_path: PathBuf,
    consumer: LockFreeAudioConsumer,
}

impl HermesOrbIpcServer {
    pub fn new<P: AsRef<Path>>(socket_path: P, consumer: LockFreeAudioConsumer) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            consumer,
        }
    }

    pub fn default_server(consumer: LockFreeAudioConsumer) -> Self {
        Self::new(DEFAULT_SOCKET_PATH, consumer)
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn consumer(&self) -> &LockFreeAudioConsumer {
        &self.consumer
    }

    /// Process a received HermesOrbPacket and apply state/audio updates to the consumer
    pub fn process_packet(packet: &HermesOrbPacket, consumer: &LockFreeAudioConsumer) {
        match packet {
            HermesOrbPacket::SetState { state, .. } => {
                let orb_state = parse_orb_state(state);
                consumer.set_state(orb_state);
            }
            HermesOrbPacket::SetAudio { level } => {
                consumer.set_audio_amplitude(*level);
            }
            HermesOrbPacket::SetThought { intensity } => {
                consumer.set_thought_trigger(*intensity);
            }
        }
    }

    /// Runs the non-blocking async Unix domain socket listener loop
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(parent) = self.socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let consumer = self.consumer.clone();
                    tokio::spawn(async move {
                        let mut reader = tokio::io::BufReader::new(stream);
                        let mut line = String::new();
                        while let Ok(bytes_read) = reader.read_line(&mut line).await {
                            if bytes_read == 0 {
                                break;
                            }
                            if let Ok(packet) = serde_json::from_str::<HermesOrbPacket>(line.trim()) {
                                Self::process_packet(&packet, &consumer);
                            }
                            line.clear();
                        }
                    });
                }
                Err(_e) => {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }
}

impl Drop for HermesOrbIpcServer {
    fn drop(&mut self) {
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}

/// Client helper function to send a HermesOrbPacket to the IPC server at default socket path
pub fn send_hermes_orb_event(event: &HermesOrbPacket) -> Result<(), String> {
    send_hermes_orb_event_to_path(event, DEFAULT_SOCKET_PATH)
}

/// Client helper function to send a HermesOrbPacket to a specific socket path
pub fn send_hermes_orb_event_to_path<P: AsRef<Path>>(event: &HermesOrbPacket, path: P) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(path.as_ref())
        .map_err(|e| format!("Failed to connect to Hermes Orb socket at {:?}: {}", path.as_ref(), e))?;

    let json = serde_json::to_string(event)
        .map_err(|e| format!("Failed to serialize HermesOrbPacket: {}", e))?;

    writeln!(stream, "{}", json)
        .map_err(|e| format!("Failed to write to Hermes Orb socket: {}", e))?;

    stream
        .flush()
        .map_err(|e| format!("Failed to flush Hermes Orb socket: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let state_packet = HermesOrbPacket::SetState {
            state: "Thinking".to_string(),
            prompt: Some("Processing user request".to_string()),
        };
        let json = serde_json::to_string(&state_packet).unwrap();
        assert!(json.contains(r#""cmd":"set_state""#));
        assert!(json.contains(r#""state":"Thinking""#));
        assert!(json.contains(r#""prompt":"Processing user request""#));

        let deserialized: HermesOrbPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, state_packet);

        let audio_packet = HermesOrbPacket::SetAudio { level: 0.85 };
        let audio_json = serde_json::to_string(&audio_packet).unwrap();
        assert!(audio_json.contains(r#""cmd":"set_audio""#));
        assert!(audio_json.contains(r#""level":0.85"#));

        let deserialized_audio: HermesOrbPacket = serde_json::from_str(&audio_json).unwrap();
        assert_eq!(deserialized_audio, audio_packet);
    }

    #[test]
    fn test_parse_orb_state() {
        assert_eq!(parse_orb_state("Listening"), OrbState::Listening);
        assert_eq!(parse_orb_state("thinking"), OrbState::Thinking);
        assert_eq!(parse_orb_state("SPEAKING"), OrbState::Speaking);
        assert_eq!(parse_orb_state("unknown"), OrbState::Listening);
    }

    #[tokio::test]
    async fn test_hermes_orb_ipc_server() {
        let sock_path = format!("/tmp/test_hermes_orb_{}.sock", std::process::id());
        let consumer = LockFreeAudioConsumer::new();
        let server = HermesOrbIpcServer::new(&sock_path, consumer.clone());

        let server_path = server.socket_path().to_path_buf();

        // Spawn server in background
        let server_handle = tokio::spawn(async move {
            let _ = server.run().await;
        });

        // Give server time to start listening
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(server_path.exists());

        // Send set_audio packet via helper
        let audio_packet = HermesOrbPacket::SetAudio { level: 0.85 };
        let res = send_hermes_orb_event_to_path(&audio_packet, &server_path);
        assert!(res.is_ok(), "Failed to send audio packet: {:?}", res.err());

        // Give consumer time to process packet
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!((consumer.get_audio_amplitude() - 0.85).abs() < f32::EPSILON);

        // Send set_state packet via helper
        let state_packet = HermesOrbPacket::SetState {
            state: "Thinking".to_string(),
            prompt: Some("Analyzing query".to_string()),
        };
        let res = send_hermes_orb_event_to_path(&state_packet, &server_path);
        assert!(res.is_ok(), "Failed to send state packet: {:?}", res.err());

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(consumer.get_state(), OrbState::Thinking);

        // Shutdown server task
        server_handle.abort();
        let _ = std::fs::remove_file(&server_path);
    }
}
