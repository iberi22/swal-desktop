//! Hermes Live Reasoning SSE Stream Listener and State Broadcaster
//!
//! Subscribes to real-time agent reasoning tokens, speaking states, and idle events
//! from the Hermes Gateway (`/v1/events` or `/events`) over Server-Sent Events (SSE),
//! parsing `AgentThinking`, `AgentSpeaking`, and `AgentIdle` packets, and broadcasting
//! `ShellEvent::OrbStateChanged` to `NativeShellSupervisor` in real-time.
//! Includes non-blocking stream consumption and exponential/stepped automatic reconnect backoff.

use crate::native_shell::{NativeShellSupervisor, ShellEvent};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

/// Default Hermes SSE endpoint URL.
pub const DEFAULT_HERMES_EVENTS_URL: &str = "http://127.0.0.1:8000/v1/events";

/// Default reconnect backoff delays in seconds (1s, 2s, 5s).
pub const DEFAULT_BACKOFF_STEPS: [u64; 3] = [1, 2, 5];

/// Real-time agent event parsed from the Hermes SSE stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HermesEvent {
    /// Agent is actively generating reasoning tokens / text chunks.
    AgentThinking(String),
    /// Agent is speaking or outputting an audio stream.
    AgentSpeaking(String),
    /// Agent has finished execution or is idle awaiting user input.
    AgentIdle,
}

/// Type alias for compatibility with stream event naming.
pub type HermesStreamEvent = HermesEvent;

impl HermesEvent {
    /// Returns the canonical state string (e.g. "thinking", "speaking", "idle").
    pub fn state_name(&self) -> &'static str {
        match self {
            Self::AgentThinking(_) => "thinking",
            Self::AgentSpeaking(_) => "speaking",
            Self::AgentIdle => "idle",
        }
    }

    /// Converts this event into a `ShellEvent::OrbStateChanged` instance.
    pub fn to_shell_event(&self) -> ShellEvent {
        match self {
            Self::AgentThinking(token) => ShellEvent::OrbStateChanged {
                state: "thinking".to_string(),
                details: if token.is_empty() {
                    None
                } else {
                    Some(token.clone())
                },
            },
            Self::AgentSpeaking(audio_url) => ShellEvent::OrbStateChanged {
                state: "speaking".to_string(),
                details: if audio_url.is_empty() {
                    None
                } else {
                    Some(audio_url.clone())
                },
            },
            Self::AgentIdle => ShellEvent::OrbStateChanged {
                state: "idle".to_string(),
                details: None,
            },
        }
    }
}

/// Helper JSON schema for deserializing JSON data payloads.
#[derive(Debug, Deserialize)]
struct SsePayloadHelper {
    #[serde(alias = "token", alias = "chunk", alias = "text", alias = "content")]
    token_chunk: Option<String>,
    #[serde(alias = "url", alias = "audio", alias = "audio_path")]
    audio_url: Option<String>,
    #[serde(alias = "type", alias = "event", alias = "state")]
    event_type: Option<String>,
}

/// Parses an SSE message block composed of `event:` and `data:` fields into a `HermesEvent`.
pub fn parse_sse_message(event_type: Option<&str>, data: &str) -> Option<HermesEvent> {
    let trimmed_data = data.trim();
    let norm_event = event_type.map(|e| e.trim().to_lowercase());

    // 1. Check if event header explicitly specifies the type
    if let Some(ref ev) = norm_event {
        match ev.as_str() {
            "agentthinking" | "agent_thinking" | "thinking" | "reasoning" => {
                if let Ok(helper) = serde_json::from_str::<SsePayloadHelper>(trimmed_data) {
                    if let Some(token) = helper.token_chunk {
                        return Some(HermesEvent::AgentThinking(token));
                    }
                }
                return Some(HermesEvent::AgentThinking(trimmed_data.to_string()));
            }
            "agentspeaking" | "agent_speaking" | "speaking" | "speech" | "audio" => {
                if let Ok(helper) = serde_json::from_str::<SsePayloadHelper>(trimmed_data) {
                    if let Some(url) = helper.audio_url {
                        return Some(HermesEvent::AgentSpeaking(url));
                    }
                }
                return Some(HermesEvent::AgentSpeaking(trimmed_data.to_string()));
            }
            "agentidle" | "agent_idle" | "idle" | "ready" | "done" => {
                return Some(HermesEvent::AgentIdle);
            }
            _ => {}
        }
    }

    // 2. If event header was missing or generic ("message"), inspect data payload as JSON
    if !trimmed_data.is_empty() {
        if let Ok(helper) = serde_json::from_str::<SsePayloadHelper>(trimmed_data) {
            if let Some(ref ev_type) = helper.event_type {
                match ev_type.to_lowercase().as_str() {
                    "agentthinking" | "agent_thinking" | "thinking" | "reasoning" => {
                        let token = helper.token_chunk.unwrap_or_default();
                        return Some(HermesEvent::AgentThinking(token));
                    }
                    "agentspeaking" | "agent_speaking" | "speaking" | "speech" | "audio" => {
                        let url = helper.audio_url.unwrap_or_default();
                        return Some(HermesEvent::AgentSpeaking(url));
                    }
                    "agentidle" | "agent_idle" | "idle" | "ready" | "done" => {
                        return Some(HermesEvent::AgentIdle);
                    }
                    _ => {}
                }
            }

            // Check field presence directly if event_type wasn't specified
            if let Some(token) = helper.token_chunk {
                return Some(HermesEvent::AgentThinking(token));
            }
            if let Some(url) = helper.audio_url {
                return Some(HermesEvent::AgentSpeaking(url));
            }
        }

        // 3. Fallback heuristic on trimmed data string
        let lower = trimmed_data.to_lowercase();
        if lower == "idle" || lower == "agentidle" || lower == "agent_idle" {
            return Some(HermesEvent::AgentIdle);
        }
    } else if let Some(ref ev) = norm_event {
        if ev == "agentidle" || ev == "agent_idle" || ev == "idle" {
            return Some(HermesEvent::AgentIdle);
        }
    }

    None
}

/// Parses full SSE text chunk or stream buffer, extracting all complete messages.
/// Modifies `buffer` in-place, retaining incomplete trailing lines.
pub fn parse_sse_chunk(buffer: &mut String) -> Vec<HermesEvent> {
    let mut events = Vec::new();

    loop {
        let boundary = match (buffer.find("\r\n\r\n"), buffer.find("\n\n")) {
            (Some(crlf), Some(lf)) => {
                if crlf <= lf {
                    Some((crlf, 4))
                } else {
                    Some((lf, 2))
                }
            }
            (Some(crlf), None) => Some((crlf, 4)),
            (None, Some(lf)) => Some((lf, 2)),
            (None, None) => None,
        };

        if let Some((pos, delimiter_len)) = boundary {
            let block = buffer[..pos].to_string();
            buffer.drain(..pos + delimiter_len);

            let mut event_type: Option<String> = None;
            let mut data_lines: Vec<String> = Vec::new();

            for line in block.lines() {
                let trimmed = line.trim_end_matches('\r');
                if trimmed.is_empty() || trimmed.starts_with(':') {
                    continue; // Comment or empty line
                }

                if let Some(stripped) = trimmed.strip_prefix("event:") {
                    event_type = Some(stripped.trim().to_string());
                } else if let Some(stripped) = trimmed.strip_prefix("data:") {
                    let data_part = stripped.strip_prefix(' ').unwrap_or(stripped);
                    data_lines.push(data_part.to_string());
                }
            }

            let combined_data = data_lines.join("\n");
            if let Some(parsed) = parse_sse_message(event_type.as_deref(), &combined_data) {
                events.push(parsed);
            }
        } else {
            break;
        }
    }

    events
}

/// Non-blocking Hermes live reasoning SSE listener with automatic backoff reconnection.
pub struct HermesEventListener {
    url: String,
    event_tx: broadcast::Sender<HermesEvent>,
    is_running: Arc<AtomicBool>,
    backoff_steps: Vec<Duration>,
    supervisor: Option<Arc<NativeShellSupervisor>>,
    client: reqwest::Client,
}

impl HermesEventListener {
    /// Creates a new `HermesEventListener` targeting the given endpoint URL.
    pub fn new(url: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(512);
        let backoff_steps = DEFAULT_BACKOFF_STEPS
            .iter()
            .map(|&secs| Duration::from_secs(secs))
            .collect();

        Self {
            url: url.into(),
            event_tx,
            is_running: Arc::new(AtomicBool::new(false)),
            backoff_steps,
            supervisor: None,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Creates a `HermesEventListener` attached to a `NativeShellSupervisor`.
    pub fn with_supervisor(url: impl Into<String>, supervisor: Arc<NativeShellSupervisor>) -> Self {
        let mut listener = Self::new(url);
        listener.supervisor = Some(supervisor);
        listener
    }

    /// Configures custom reconnect backoff durations.
    pub fn with_backoff(mut self, backoff_steps: Vec<Duration>) -> Self {
        if !backoff_steps.is_empty() {
            self.backoff_steps = backoff_steps;
        }
        self
    }

    /// Configures custom reqwest HTTP client.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    /// Returns the target SSE events URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns true if the listener loop is active.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Stops the listener loop cleanly.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Subscribes to the broadcast channel receiving parsed `HermesEvent`s.
    pub fn subscribe(&self) -> broadcast::Receiver<HermesEvent> {
        self.event_tx.subscribe()
    }

    /// Broadcasts an event to subscribers and forwards `ShellEvent::OrbStateChanged` to supervisor if present.
    pub fn broadcast_event(&self, event: HermesEvent) -> Result<usize, String> {
        if let Some(ref supervisor) = self.supervisor {
            let shell_evt = event.to_shell_event();
            let _ = supervisor.broadcast_event(shell_evt);
        }

        self.event_tx
            .send(event)
            .map_err(|e| format!("Failed to broadcast Hermes event: {}", e))
    }

    /// Connects to the HTTP SSE endpoint and streams events until disconnection or error.
    pub async fn read_stream_once(&self) -> Result<(), String> {
        let resp = self
            .client
            .get(&self.url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Hermes SSE endpoint at {}: {}", self.url, e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Hermes SSE endpoint returned HTTP status error: {}",
                resp.status()
            ));
        }

        let mut stream_resp = resp;
        let mut buffer = String::new();

        while self.is_running.load(Ordering::SeqCst) {
            match stream_resp.chunk().await {
                Ok(Some(chunk)) => {
                    let text = String::from_utf8_lossy(&chunk);
                    buffer.push_str(&text);

                    let events = parse_sse_chunk(&mut buffer);
                    for event in events {
                        let _ = self.broadcast_event(event);
                    }
                }
                Ok(None) => {
                    // EOF reached cleanly
                    break;
                }
                Err(e) => {
                    return Err(format!("Error reading chunk from Hermes SSE stream: {}", e));
                }
            }
        }

        Ok(())
    }

    /// Starts the listener loop in a spawned non-blocking Tokio task.
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let is_running = Arc::clone(&self.is_running);
        let url = self.url.clone();
        let event_tx = self.event_tx.clone();
        let backoff_steps = self.backoff_steps.clone();
        let supervisor = self.supervisor.clone();
        let client = self.client.clone();

        is_running.store(true, Ordering::SeqCst);

        tokio::spawn(async move {
            let mut backoff_idx = 0;

            while is_running.load(Ordering::SeqCst) {
                let listener_instance = HermesEventListener {
                    url: url.clone(),
                    event_tx: event_tx.clone(),
                    is_running: Arc::clone(&is_running),
                    backoff_steps: backoff_steps.clone(),
                    supervisor: supervisor.clone(),
                    client: client.clone(),
                };

                match listener_instance.read_stream_once().await {
                    Ok(()) => {
                        // Connection ended cleanly, reset backoff
                        backoff_idx = 0;
                    }
                    Err(_err) => {
                        // Error occurred, apply backoff delay
                        let delay = if !backoff_steps.is_empty() {
                            let idx = backoff_idx.min(backoff_steps.len() - 1);
                            backoff_steps[idx]
                        } else {
                            Duration::from_secs(1)
                        };

                        backoff_idx += 1;
                        sleep(delay).await;
                    }
                }
            }

            is_running.store(false, Ordering::SeqCst);
        })
    }
}

impl Default for HermesEventListener {
    fn default() -> Self {
        Self::new(DEFAULT_HERMES_EVENTS_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermes_event_state_name_and_conversion() {
        let thinking = HermesEvent::AgentThinking("Analyzing AST".to_string());
        assert_eq!(thinking.state_name(), "thinking");
        assert_eq!(
            thinking.to_shell_event(),
            ShellEvent::OrbStateChanged {
                state: "thinking".to_string(),
                details: Some("Analyzing AST".to_string()),
            }
        );

        let speaking = HermesEvent::AgentSpeaking("http://127.0.0.1:8000/audio.wav".to_string());
        assert_eq!(speaking.state_name(), "speaking");
        assert_eq!(
            speaking.to_shell_event(),
            ShellEvent::OrbStateChanged {
                state: "speaking".to_string(),
                details: Some("http://127.0.0.1:8000/audio.wav".to_string()),
            }
        );

        let idle = HermesEvent::AgentIdle;
        assert_eq!(idle.state_name(), "idle");
        assert_eq!(
            idle.to_shell_event(),
            ShellEvent::OrbStateChanged {
                state: "idle".to_string(),
                details: None,
            }
        );
    }

    #[test]
    fn test_parse_sse_message_typed_events() {
        // AgentThinking with JSON payload
        let ev1 = parse_sse_message(
            Some("AgentThinking"),
            r#"{"token_chunk":"Checking codebase"}"#,
        );
        assert_eq!(
            ev1,
            Some(HermesEvent::AgentThinking("Checking codebase".to_string()))
        );

        // AgentThinking with raw text payload
        let ev2 = parse_sse_message(Some("AgentThinking"), "Calculating optimal layout");
        assert_eq!(
            ev2,
            Some(HermesEvent::AgentThinking(
                "Calculating optimal layout".to_string()
            ))
        );

        // AgentSpeaking with JSON payload
        let ev3 = parse_sse_message(
            Some("AgentSpeaking"),
            r#"{"audio_url":"http://localhost:8000/resp.wav"}"#,
        );
        assert_eq!(
            ev3,
            Some(HermesEvent::AgentSpeaking(
                "http://localhost:8000/resp.wav".to_string()
            ))
        );

        // AgentSpeaking with raw string payload
        let ev4 = parse_sse_message(Some("AgentSpeaking"), "/tmp/out.wav");
        assert_eq!(
            ev4,
            Some(HermesEvent::AgentSpeaking("/tmp/out.wav".to_string()))
        );

        // AgentIdle with empty data
        let ev5 = parse_sse_message(Some("AgentIdle"), "{}");
        assert_eq!(ev5, Some(HermesEvent::AgentIdle));

        let ev6 = parse_sse_message(Some("idle"), "");
        assert_eq!(ev6, Some(HermesEvent::AgentIdle));
    }

    #[test]
    fn test_parse_sse_message_self_describing_json() {
        let json_thinking = r#"{"type":"AgentThinking","token_chunk":"Decomposing wave plan"}"#;
        let ev1 = parse_sse_message(None, json_thinking);
        assert_eq!(
            ev1,
            Some(HermesEvent::AgentThinking(
                "Decomposing wave plan".to_string()
            ))
        );

        let json_speaking =
            r#"{"type":"AgentSpeaking","audio_url":"https://swal.dev/audio/1.mp3"}"#;
        let ev2 = parse_sse_message(None, json_speaking);
        assert_eq!(
            ev2,
            Some(HermesEvent::AgentSpeaking(
                "https://swal.dev/audio/1.mp3".to_string()
            ))
        );

        let json_idle = r#"{"type":"AgentIdle"}"#;
        let ev3 = parse_sse_message(None, json_idle);
        assert_eq!(ev3, Some(HermesEvent::AgentIdle));
    }

    #[test]
    fn test_parse_sse_chunk_streaming() {
        let mut buffer = String::new();

        buffer.push_str(
            "event: AgentThinking\r\ndata: {\"token_chunk\":\"First token\"}\r\n\r\n\
             event: AgentSpeaking\r\ndata: {\"audio_url\":\"/audio/speech.wav\"}\r\n\r\n\
             event: AgentIdle\r\ndata: {}\r\n\r\n",
        );

        let events = parse_sse_chunk(&mut buffer);
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            HermesEvent::AgentThinking("First token".to_string())
        );
        assert_eq!(
            events[1],
            HermesEvent::AgentSpeaking("/audio/speech.wav".to_string())
        );
        assert_eq!(events[2], HermesEvent::AgentIdle);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_parse_sse_chunk_fragmented() {
        let mut buffer = String::new();

        // First partial chunk
        buffer.push_str("event: AgentThinking\ndata: Partial tok");
        let events1 = parse_sse_chunk(&mut buffer);
        assert_eq!(events1.len(), 0);
        assert_eq!(buffer, "event: AgentThinking\ndata: Partial tok");

        // Second chunk completing the message
        buffer.push_str("en completed\n\n");
        let events2 = parse_sse_chunk(&mut buffer);
        assert_eq!(events2.len(), 1);
        assert_eq!(
            events2[0],
            HermesEvent::AgentThinking("Partial token completed".to_string())
        );
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_listener_broadcast_and_supervisor_integration() {
        let supervisor = Arc::new(NativeShellSupervisor::new());
        let mut shell_rx = supervisor.subscribe_events();

        let listener =
            HermesEventListener::with_supervisor("http://127.0.0.1:8000/events", supervisor.clone());
        let mut hermes_rx = listener.subscribe();

        // Broadcast thinking event
        let event = HermesEvent::AgentThinking("Executing step 1".to_string());
        let sent = listener.broadcast_event(event.clone());
        assert!(sent.is_ok());

        let recv_hermes = hermes_rx.recv().await.expect("Failed to receive hermes event");
        assert_eq!(recv_hermes, event);

        let recv_shell = shell_rx.recv().await.expect("Failed to receive shell event");
        assert_eq!(
            recv_shell,
            ShellEvent::OrbStateChanged {
                state: "thinking".to_string(),
                details: Some("Executing step 1".to_string()),
            }
        );

        // Broadcast idle event
        let idle_event = HermesEvent::AgentIdle;
        let _ = listener.broadcast_event(idle_event.clone());

        let recv_idle_hermes = hermes_rx.recv().await.unwrap();
        assert_eq!(recv_idle_hermes, idle_event);

        let recv_idle_shell = shell_rx.recv().await.unwrap();
        assert_eq!(
            recv_idle_shell,
            ShellEvent::OrbStateChanged {
                state: "idle".to_string(),
                details: None,
            }
        );
    }

    #[test]
    fn test_listener_configuration() {
        let listener = HermesEventListener::new("http://127.0.0.1:8006/events").with_backoff(vec![
            Duration::from_millis(50),
            Duration::from_millis(100),
        ]);

        assert_eq!(listener.url(), "http://127.0.0.1:8006/events");
        assert_eq!(listener.backoff_steps.len(), 2);
        assert!(!listener.is_running());
        listener.stop();
        assert!(!listener.is_running());
    }
}
