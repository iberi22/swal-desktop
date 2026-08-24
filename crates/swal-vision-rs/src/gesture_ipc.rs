use crate::hand_tracker::{GestureKind, HandState};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const GESTURE_SOCKET: &str = "/tmp/swal_gesture.sock";
pub const GESTURE_CONFIG_PATH: &str = "/home/belal/.config/swal/gesture.json";

/// Configuración global de visión, gestos y detección de presencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureConfig {
    /// Activa/desactiva el puntero virtual de ratón por gestos
    pub enabled: bool,
    /// Si true: el cursor gestual opera en toda la pantalla (como un ratón real).
    pub fullscreen_control: bool,
    /// Resolución de pantalla para mapeo absoluto
    pub screen_width: u32,
    pub screen_height: u32,
    /// Dispositivo V4L2 de cámara
    pub camera_device: String,
    /// Umbral mínimo de confianza para actuar [0.0, 1.0]
    pub confidence_threshold: f32,
    /// Factor de suavizado de movimiento del cursor (EMA) [0.0=sin suavizado, 1.0=máximo]
    pub cursor_smoothing: f32,
    /// Activa el bloqueo automático de sesión al alejarse del PC (walk-away lock)
    #[serde(default = "default_presence_auto_lock")]
    pub presence_auto_lock: bool,
    /// Segundos de ausencia de rostro antes de bloquear la sesión
    #[serde(default = "default_auto_lock_timeout")]
    pub auto_lock_timeout_secs: u64,
    /// Activa el auto-despertar de pantalla al detectar el regreso del usuario
    #[serde(default = "default_auto_unlock")]
    pub auto_unlock_on_presence: bool,
}

fn default_presence_auto_lock() -> bool {
    false
}
fn default_auto_lock_timeout() -> u64 {
    45
}
fn default_auto_unlock() -> bool {
    false
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fullscreen_control: true,
            screen_width: 2560,
            screen_height: 1440,
            camera_device: "/dev/video0".to_string(),
            confidence_threshold: 0.3,
            cursor_smoothing: 0.7,
            presence_auto_lock: false,
            auto_lock_timeout_secs: 45,
            auto_unlock_on_presence: false,
        }
    }
}

impl GestureConfig {
    /// Carga la configuración desde `~/.config/swal/gesture.json`.
    pub fn load_or_default() -> Self {
        let path = Path::new(GESTURE_CONFIG_PATH);
        if let Ok(content) = std::fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&config) {
                let _ = std::fs::write(path, json);
            }
            config
        }
    }

    /// Guarda la configuración actual al disco
    pub fn save(&self) -> std::io::Result<()> {
        let path = Path::new(GESTURE_CONFIG_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }
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

impl GestureEvent {
    pub fn from_hand_state(state: &HandState, ts: u64) -> Self {
        Self {
            kind: state.gesture.clone(),
            x_norm: state.x_norm,
            y_norm: state.y_norm,
            openness: state.openness,
            confidence: state.confidence,
            timestamp_ms: ts,
        }
    }

    pub fn none_event() -> Self {
        Self {
            kind: GestureKind::None,
            x_norm: 0.5,
            y_norm: 0.5,
            openness: 0.0,
            confidence: 0.0,
            timestamp_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_event_serialization() {
        let event = GestureEvent {
            kind: GestureKind::Open,
            x_norm: 0.5,
            y_norm: 0.5,
            openness: 0.8,
            confidence: 0.9,
            timestamp_ms: 123456789,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains(r#""kind":"open""#));
    }

    #[test]
    fn test_gesture_config_load_or_default() {
        let config = GestureConfig::load_or_default();
        assert!(config.screen_width > 0);
        assert!(config.screen_height > 0);
        assert_eq!(config.camera_device, "/dev/video0");
    }
}
