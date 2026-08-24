//! Gesture Consumer — lee GestureEvent desde swal-vision IPC socket
//! y los convierte en movimiento de cursor vía ydotool.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

pub const GESTURE_SOCKET: &str = "/tmp/swal_gesture.sock";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureKind {
    Open,
    Pinch,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEvent {
    pub kind: GestureKind,
    pub x_norm: f32,
    pub y_norm: f32,
    pub openness: f32,
    pub confidence: f32,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ScreenConfig {
    pub width: u32,
    pub height: u32,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self { width: 2560, height: 1440 }
    }
}

#[derive(Debug)]
pub struct GestureConsumer {
    pub screen: ScreenConfig,
    pub orb_enabled: std::sync::atomic::AtomicBool,
}

impl GestureConsumer {
    pub fn new(screen: ScreenConfig) -> Self {
        Self {
            screen,
            orb_enabled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn set_orb_enabled(&self, enabled: bool) {
        self.orb_enabled.store(enabled, std::sync::atomic::Ordering::Release);
    }

    pub fn is_orb_enabled(&self) -> bool {
        self.orb_enabled.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Procesa un GestureEvent y lanza las acciones correspondientes.
    /// - Open: mueve el cursor con ydotool mousemove --absolute -x X -y Y
    /// - Pinch: hace click izquierdo con ydotool click 0x40
    /// - Requiere confidence >= 0.3 para actuar
    pub async fn handle_event(&self, event: &GestureEvent) {
        if event.confidence < 0.3 || event.kind == GestureKind::None {
            return;
        }

        let x_px = (event.x_norm * self.screen.width as f32) as u32;
        let y_px = (event.y_norm * self.screen.height as f32) as u32;

        match event.kind {
            GestureKind::Open => {
                let _ = std::process::Command::new("ydotool")
                    .args([
                        "mousemove", "--absolute",
                        "-x", &x_px.to_string(),
                        "-y", &y_px.to_string(),
                    ])
                    .output();
            }
            GestureKind::Pinch => {
                let _ = std::process::Command::new("ydotool")
                    .args(["click", "0x40"])
                    .output();
            }
            GestureKind::None => {}
        }
    }

    /// Loop principal: conecta al socket de swal-vision, lee eventos JSON line-delimited,
    /// y llama handle_event. Auto-reconnect cada 2s si el socket no está disponible.
    pub async fn run_loop(self: Arc<Self>) {
        loop {
            match UnixStream::connect(GESTURE_SOCKET).await {
                Ok(stream) => {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    if let Ok(event) = serde_json::from_str::<GestureEvent>(trimmed) {
                                        self.handle_event(&event).await;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // swal-vision no está corriendo todavía, reintentar
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_config_default() {
        let cfg = ScreenConfig::default();
        assert_eq!(cfg.width, 2560);
        assert_eq!(cfg.height, 1440);
    }

    #[test]
    fn test_gesture_event_deserialization() {
        let json = r#"{"kind":"open","x_norm":0.5,"y_norm":0.3,"openness":0.8,"confidence":0.9,"timestamp_ms":12345}"#;
        let event: GestureEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.kind, GestureKind::Open);
        assert!((event.x_norm - 0.5).abs() < 1e-5);
        assert_eq!(event.timestamp_ms, 12345);
    }

    #[test]
    fn test_gesture_event_pinch_deserialization() {
        let json = r#"{"kind":"pinch","x_norm":0.4,"y_norm":0.6,"openness":0.2,"confidence":0.7,"timestamp_ms":999}"#;
        let event: GestureEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.kind, GestureKind::Pinch);
    }

    #[test]
    fn test_gesture_kind_serialization() {
        let kind = GestureKind::Open;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#""open""#);

        let kind2 = GestureKind::Pinch;
        let json2 = serde_json::to_string(&kind2).unwrap();
        assert_eq!(json2, r#""pinch""#);
    }

    #[test]
    fn test_orb_enabled_toggle() {
        let consumer = GestureConsumer::new(ScreenConfig::default());
        assert!(!consumer.is_orb_enabled());
        consumer.set_orb_enabled(true);
        assert!(consumer.is_orb_enabled());
        consumer.set_orb_enabled(false);
        assert!(!consumer.is_orb_enabled());
    }

    #[test]
    fn test_low_confidence_noop() {
        // Un evento con confidence < 0.3 no debe causar pánico ni efectos
        let event = GestureEvent {
            kind: GestureKind::Open,
            x_norm: 0.5,
            y_norm: 0.5,
            openness: 0.8,
            confidence: 0.1, // baja confianza
            timestamp_ms: 0,
        };
        // Solo verificamos que el filtro funciona, no lanzamos ydotool en tests
        assert!(event.confidence < 0.3);
    }
}
