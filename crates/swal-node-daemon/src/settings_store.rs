//! SwalSystemSettings Store Engine for SWAL Desktop
//!
//! Provides canonical system settings JSON schema, serialization, deserialization,
//! file loading/saving, dot-notation key lookup, and mutation methods.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Canonical system settings root struct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SwalSystemSettings {
    pub appearance: AppearanceSettings,
    pub agent: AgentSettings,
    pub display: DisplaySettings,
    pub storage: StorageSettings,
    pub network: NetworkSettings,
    pub audio: AudioSettings,
}

/// System appearance and visual theme configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceSettings {
    pub theme: String,
    pub accent_color: String,
    pub corner_radius: f32,
    pub acrylic_opacity: f32,
    pub wallpaper_path: Option<String>,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "hive-dark".to_string(),
            accent_color: "#0078D4".to_string(),
            corner_radius: 8.0,
            acrylic_opacity: 0.8,
            wallpaper_path: None,
        }
    }
}

/// Agent cognition and AI interaction configurations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSettings {
    pub default_agent: String,
    pub model_routing: String,
    pub voice_orb_enabled: bool,
    pub audio_sensitivity: f32,
    pub auto_ui_generation: bool,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            default_agent: "hermes".to_string(),
            model_routing: "auto".to_string(),
            voice_orb_enabled: true,
            audio_sensitivity: 0.5,
            auto_ui_generation: true,
        }
    }
}

/// Display and compositor refresh rates / scaling settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplaySettings {
    pub target_fps: u32,
    pub hidpi_scale: f32,
    pub vsync: bool,
    pub compositor: String,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            target_fps: 120,
            hidpi_scale: 1.0,
            vsync: true,
            compositor: "niri".to_string(),
        }
    }
}

/// File manager dual pane and storage threshold settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageSettings {
    pub default_dual_pane: bool,
    pub show_hidden: bool,
    pub low_space_alert_gb: u32,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            default_dual_pane: false,
            show_hidden: false,
            low_space_alert_gb: 20,
        }
    }
}

/// Node mesh network discovery and Xavier bridge endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSettings {
    pub node_id: String,
    pub mesh_port: u16,
    pub xavier_endpoint: String,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            node_id: "swal-node-local".to_string(),
            mesh_port: 8900,
            xavier_endpoint: "http://127.0.0.1:8006".to_string(),
        }
    }
}

/// Audio pipeline sinks and input levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSettings {
    pub pipewire_sink: String,
    pub mic_gain: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            pipewire_sink: "default".to_string(),
            mic_gain: 1.0,
        }
    }
}

impl SwalSystemSettings {
    /// Loads settings from a JSON file path.
    /// If file does not exist or fails to parse, returns `Default::default()`.
    pub fn load_from_path(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    /// Saves settings to a JSON file path in pretty-printed JSON format.
    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json_str)
    }

    /// Retrieves a string representation of a setting value using dot-notation.
    ///
    /// Supported key paths:
    /// - `appearance.theme`
    /// - `appearance.accent_color`
    /// - `appearance.corner_radius`
    /// - `appearance.acrylic_opacity`
    /// - `appearance.wallpaper_path`
    /// - `agent.default_agent`
    /// - `agent.model_routing`
    /// - `agent.voice_orb_enabled`
    /// - `agent.audio_sensitivity`
    /// - `agent.auto_ui_generation`
    /// - `display.target_fps`
    /// - `display.hidpi_scale`
    /// - `display.vsync`
    /// - `display.compositor`
    /// - `storage.default_dual_pane`
    /// - `storage.show_hidden`
    /// - `storage.low_space_alert_gb`
    /// - `network.node_id`
    /// - `network.mesh_port`
    /// - `network.xavier_endpoint`
    /// - `audio.pipewire_sink`
    /// - `audio.mic_gain`
    pub fn get_value(&self, key_path: &str) -> Option<String> {
        match key_path {
            "appearance.theme" => Some(self.appearance.theme.clone()),
            "appearance.accent_color" => Some(self.appearance.accent_color.clone()),
            "appearance.corner_radius" => Some(self.appearance.corner_radius.to_string()),
            "appearance.acrylic_opacity" => Some(self.appearance.acrylic_opacity.to_string()),
            "appearance.wallpaper_path" => self.appearance.wallpaper_path.clone(),
            "agent.default_agent" => Some(self.agent.default_agent.clone()),
            "agent.model_routing" => Some(self.agent.model_routing.clone()),
            "agent.voice_orb_enabled" => Some(self.agent.voice_orb_enabled.to_string()),
            "agent.audio_sensitivity" => Some(self.agent.audio_sensitivity.to_string()),
            "agent.auto_ui_generation" => Some(self.agent.auto_ui_generation.to_string()),
            "display.target_fps" => Some(self.display.target_fps.to_string()),
            "display.hidpi_scale" => Some(self.display.hidpi_scale.to_string()),
            "display.vsync" => Some(self.display.vsync.to_string()),
            "display.compositor" => Some(self.display.compositor.clone()),
            "storage.default_dual_pane" => Some(self.storage.default_dual_pane.to_string()),
            "storage.show_hidden" => Some(self.storage.show_hidden.to_string()),
            "storage.low_space_alert_gb" => Some(self.storage.low_space_alert_gb.to_string()),
            "network.node_id" => Some(self.network.node_id.clone()),
            "network.mesh_port" => Some(self.network.mesh_port.to_string()),
            "network.xavier_endpoint" => Some(self.network.xavier_endpoint.clone()),
            "audio.pipewire_sink" => Some(self.audio.pipewire_sink.clone()),
            "audio.mic_gain" => Some(self.audio.mic_gain.to_string()),
            _ => None,
        }
    }

    /// Mutates a setting value matching the provided dot-notation key path.
    pub fn set_value(&mut self, key_path: &str, value: &str) -> Result<(), String> {
        match key_path {
            "appearance.theme" => {
                self.appearance.theme = value.to_string();
                Ok(())
            }
            "appearance.accent_color" => {
                self.appearance.accent_color = value.to_string();
                Ok(())
            }
            "appearance.corner_radius" => {
                self.appearance.corner_radius = value
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid float value for corner_radius: {}", e))?;
                Ok(())
            }
            "appearance.acrylic_opacity" => {
                self.appearance.acrylic_opacity = value
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid float value for acrylic_opacity: {}", e))?;
                Ok(())
            }
            "appearance.wallpaper_path" => {
                if value.is_empty() || value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("null") {
                    self.appearance.wallpaper_path = None;
                } else {
                    self.appearance.wallpaper_path = Some(value.to_string());
                }
                Ok(())
            }
            "agent.default_agent" => {
                self.agent.default_agent = value.to_string();
                Ok(())
            }
            "agent.model_routing" => {
                self.agent.model_routing = value.to_string();
                Ok(())
            }
            "agent.voice_orb_enabled" => {
                self.agent.voice_orb_enabled = value
                    .parse::<bool>()
                    .map_err(|e| format!("Invalid boolean value for voice_orb_enabled: {}", e))?;
                Ok(())
            }
            "agent.audio_sensitivity" => {
                self.agent.audio_sensitivity = value
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid float value for audio_sensitivity: {}", e))?;
                Ok(())
            }
            "agent.auto_ui_generation" => {
                self.agent.auto_ui_generation = value
                    .parse::<bool>()
                    .map_err(|e| format!("Invalid boolean value for auto_ui_generation: {}", e))?;
                Ok(())
            }
            "display.target_fps" => {
                self.display.target_fps = value
                    .parse::<u32>()
                    .map_err(|e| format!("Invalid u32 value for target_fps: {}", e))?;
                Ok(())
            }
            "display.hidpi_scale" => {
                self.display.hidpi_scale = value
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid float value for hidpi_scale: {}", e))?;
                Ok(())
            }
            "display.vsync" => {
                self.display.vsync = value
                    .parse::<bool>()
                    .map_err(|e| format!("Invalid boolean value for vsync: {}", e))?;
                Ok(())
            }
            "display.compositor" => {
                self.display.compositor = value.to_string();
                Ok(())
            }
            "storage.default_dual_pane" => {
                self.storage.default_dual_pane = value
                    .parse::<bool>()
                    .map_err(|e| format!("Invalid boolean value for default_dual_pane: {}", e))?;
                Ok(())
            }
            "storage.show_hidden" => {
                self.storage.show_hidden = value
                    .parse::<bool>()
                    .map_err(|e| format!("Invalid boolean value for show_hidden: {}", e))?;
                Ok(())
            }
            "storage.low_space_alert_gb" => {
                self.storage.low_space_alert_gb = value
                    .parse::<u32>()
                    .map_err(|e| format!("Invalid u32 value for low_space_alert_gb: {}", e))?;
                Ok(())
            }
            "network.node_id" => {
                self.network.node_id = value.to_string();
                Ok(())
            }
            "network.mesh_port" => {
                self.network.mesh_port = value
                    .parse::<u16>()
                    .map_err(|e| format!("Invalid u16 value for mesh_port: {}", e))?;
                Ok(())
            }
            "network.xavier_endpoint" => {
                self.network.xavier_endpoint = value.to_string();
                Ok(())
            }
            "audio.pipewire_sink" => {
                self.audio.pipewire_sink = value.to_string();
                Ok(())
            }
            "audio.mic_gain" => {
                self.audio.mic_gain = value
                    .parse::<f32>()
                    .map_err(|e| format!("Invalid float value for mic_gain: {}", e))?;
                Ok(())
            }
            _ => Err(format!("Unknown key path: {}", key_path)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_initialization() {
        let settings = SwalSystemSettings::default();
        assert_eq!(settings.appearance.theme, "hive-dark");
        assert_eq!(settings.appearance.accent_color, "#0078D4");
        assert_eq!(settings.appearance.corner_radius, 8.0);
        assert_eq!(settings.appearance.acrylic_opacity, 0.8);
        assert_eq!(settings.appearance.wallpaper_path, None);

        assert_eq!(settings.agent.default_agent, "hermes");
        assert_eq!(settings.agent.model_routing, "auto");
        assert!(settings.agent.voice_orb_enabled);
        assert_eq!(settings.agent.audio_sensitivity, 0.5);
        assert!(settings.agent.auto_ui_generation);

        assert_eq!(settings.display.target_fps, 120);
        assert_eq!(settings.display.hidpi_scale, 1.0);
        assert!(settings.display.vsync);
        assert_eq!(settings.display.compositor, "niri");

        assert!(!settings.storage.default_dual_pane);
        assert!(!settings.storage.show_hidden);
        assert_eq!(settings.storage.low_space_alert_gb, 20);

        assert_eq!(settings.network.node_id, "swal-node-local");
        assert_eq!(settings.network.mesh_port, 8900);
        assert_eq!(settings.network.xavier_endpoint, "http://127.0.0.1:8006");

        assert_eq!(settings.audio.pipewire_sink, "default");
        assert_eq!(settings.audio.mic_gain, 1.0);
    }

    #[test]
    fn test_json_roundtrip() {
        let settings = SwalSystemSettings::default();
        let serialized = serde_json::to_string_pretty(&settings).expect("Serialization failed");
        let deserialized: SwalSystemSettings = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_dot_notation_get_value() {
        let mut settings = SwalSystemSettings::default();
        settings.appearance.wallpaper_path = Some("/home/bela/wallpaper.png".to_string());

        assert_eq!(settings.get_value("appearance.theme"), Some("hive-dark".to_string()));
        assert_eq!(settings.get_value("appearance.accent_color"), Some("#0078D4".to_string()));
        assert_eq!(settings.get_value("appearance.corner_radius"), Some("8".to_string()));
        assert_eq!(settings.get_value("appearance.acrylic_opacity"), Some("0.8".to_string()));
        assert_eq!(settings.get_value("appearance.wallpaper_path"), Some("/home/bela/wallpaper.png".to_string()));

        assert_eq!(settings.get_value("agent.default_agent"), Some("hermes".to_string()));
        assert_eq!(settings.get_value("agent.model_routing"), Some("auto".to_string()));
        assert_eq!(settings.get_value("agent.voice_orb_enabled"), Some("true".to_string()));
        assert_eq!(settings.get_value("agent.audio_sensitivity"), Some("0.5".to_string()));
        assert_eq!(settings.get_value("agent.auto_ui_generation"), Some("true".to_string()));

        assert_eq!(settings.get_value("display.target_fps"), Some("120".to_string()));
        assert_eq!(settings.get_value("display.hidpi_scale"), Some("1".to_string()));
        assert_eq!(settings.get_value("display.vsync"), Some("true".to_string()));
        assert_eq!(settings.get_value("display.compositor"), Some("niri".to_string()));

        assert_eq!(settings.get_value("storage.default_dual_pane"), Some("false".to_string()));
        assert_eq!(settings.get_value("storage.show_hidden"), Some("false".to_string()));
        assert_eq!(settings.get_value("storage.low_space_alert_gb"), Some("20".to_string()));

        assert_eq!(settings.get_value("network.node_id"), Some("swal-node-local".to_string()));
        assert_eq!(settings.get_value("network.mesh_port"), Some("8900".to_string()));
        assert_eq!(settings.get_value("network.xavier_endpoint"), Some("http://127.0.0.1:8006".to_string()));

        assert_eq!(settings.get_value("audio.pipewire_sink"), Some("default".to_string()));
        assert_eq!(settings.get_value("audio.mic_gain"), Some("1".to_string()));

        assert_eq!(settings.get_value("invalid.key"), None);
        assert_eq!(settings.get_value("appearance"), None);
    }

    #[test]
    fn test_dot_notation_set_value() {
        let mut settings = SwalSystemSettings::default();

        assert!(settings.set_value("appearance.theme", "fluent-dark").is_ok());
        assert_eq!(settings.appearance.theme, "fluent-dark");

        assert!(settings.set_value("appearance.accent_color", "#FF0000").is_ok());
        assert_eq!(settings.appearance.accent_color, "#FF0000");

        assert!(settings.set_value("appearance.corner_radius", "12.5").is_ok());
        assert_eq!(settings.appearance.corner_radius, 12.5);

        assert!(settings.set_value("appearance.acrylic_opacity", "0.95").is_ok());
        assert_eq!(settings.appearance.acrylic_opacity, 0.95);

        assert!(settings.set_value("appearance.wallpaper_path", "/tmp/bg.png").is_ok());
        assert_eq!(settings.appearance.wallpaper_path, Some("/tmp/bg.png".to_string()));

        assert!(settings.set_value("appearance.wallpaper_path", "none").is_ok());
        assert_eq!(settings.appearance.wallpaper_path, None);

        assert!(settings.set_value("agent.default_agent", "xavier").is_ok());
        assert_eq!(settings.agent.default_agent, "xavier");

        assert!(settings.set_value("agent.model_routing", "local").is_ok());
        assert_eq!(settings.agent.model_routing, "local");

        assert!(settings.set_value("agent.voice_orb_enabled", "false").is_ok());
        assert!(!settings.agent.voice_orb_enabled);

        assert!(settings.set_value("agent.audio_sensitivity", "0.8").is_ok());
        assert_eq!(settings.agent.audio_sensitivity, 0.8);

        assert!(settings.set_value("agent.auto_ui_generation", "false").is_ok());
        assert!(!settings.agent.auto_ui_generation);

        assert!(settings.set_value("display.target_fps", "240").is_ok());
        assert_eq!(settings.display.target_fps, 240);

        assert!(settings.set_value("display.hidpi_scale", "2.0").is_ok());
        assert_eq!(settings.display.hidpi_scale, 2.0);

        assert!(settings.set_value("display.vsync", "false").is_ok());
        assert!(!settings.display.vsync);

        assert!(settings.set_value("display.compositor", "hyprland").is_ok());
        assert_eq!(settings.display.compositor, "hyprland");

        assert!(settings.set_value("storage.default_dual_pane", "true").is_ok());
        assert!(settings.storage.default_dual_pane);

        assert!(settings.set_value("storage.show_hidden", "true").is_ok());
        assert!(settings.storage.show_hidden);

        assert!(settings.set_value("storage.low_space_alert_gb", "50").is_ok());
        assert_eq!(settings.storage.low_space_alert_gb, 50);

        assert!(settings.set_value("network.node_id", "node-01").is_ok());
        assert_eq!(settings.network.node_id, "node-01");

        assert!(settings.set_value("network.mesh_port", "9000").is_ok());
        assert_eq!(settings.network.mesh_port, 9000);

        assert!(settings.set_value("network.xavier_endpoint", "http://10.0.0.1:8006").is_ok());
        assert_eq!(settings.network.xavier_endpoint, "http://10.0.0.1:8006");

        assert!(settings.set_value("audio.pipewire_sink", "alsa_output").is_ok());
        assert_eq!(settings.audio.pipewire_sink, "alsa_output");

        assert!(settings.set_value("audio.mic_gain", "1.5").is_ok());
        assert_eq!(settings.audio.mic_gain, 1.5);
    }

    #[test]
    fn test_set_value_errors() {
        let mut settings = SwalSystemSettings::default();

        assert!(settings.set_value("display.target_fps", "not_a_number").is_err());
        assert!(settings.set_value("agent.voice_orb_enabled", "not_a_bool").is_err());
        assert!(settings.set_value("network.mesh_port", "70000").is_err());
        assert!(settings.set_value("unknown.key", "value").is_err());
    }

    #[test]
    fn test_load_and_save_file() {
        let temp_dir = std::env::temp_dir().join("swal_settings_test");
        let file_path = temp_dir.join("sub_dir").join("settings.json");

        if file_path.exists() {
            let _ = std::fs::remove_file(&file_path);
        }

        // Test load when file doesn't exist returns default
        let loaded_default = SwalSystemSettings::load_from_path(&file_path);
        assert_eq!(loaded_default, SwalSystemSettings::default());

        // Save custom settings
        let mut custom = SwalSystemSettings::default();
        custom.appearance.theme = "custom-dark".to_string();
        custom.display.target_fps = 144;
        assert!(custom.save_to_path(&file_path).is_ok());

        // Load saved settings
        let loaded = SwalSystemSettings::load_from_path(&file_path);
        assert_eq!(loaded, custom);

        // Cleanup
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
