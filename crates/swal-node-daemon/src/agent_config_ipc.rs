//! Agent Real-Time Configuration Mutation IPC Protocol Server & Client
//!
//! Provides a Unix domain socket server (`/tmp/swal_settings.sock`) allowing external
//! AI agents (e.g. Hermes, Xavier) and system components to query and mutate system settings
//! in real time, with broadcast notifications sent to subscribed desktop surfaces.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;

pub const DEFAULT_SETTINGS_SOCKET_PATH: &str = "/tmp/swal_settings.sock";

/// Appearance settings configuration.
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
            theme: "fluent-dark".to_string(),
            accent_color: "#0078D4".to_string(),
            corner_radius: 8.0,
            acrylic_opacity: 0.8,
            wallpaper_path: None,
        }
    }
}

/// Agent settings configuration.
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
            model_routing: "local-first".to_string(),
            voice_orb_enabled: true,
            audio_sensitivity: 0.75,
            auto_ui_generation: true,
        }
    }
}

/// Display settings configuration.
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
            target_fps: 144,
            hidpi_scale: 1.0,
            vsync: true,
            compositor: "hyprland".to_string(),
        }
    }
}

/// Storage settings configuration.
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
            low_space_alert_gb: 10,
        }
    }
}

/// Network settings configuration.
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

/// Audio settings configuration.
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

/// Canonical system settings store structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SwalSystemSettings {
    pub appearance: AppearanceSettings,
    pub agent: AgentSettings,
    pub display: DisplaySettings,
    pub storage: StorageSettings,
    pub network: NetworkSettings,
    pub audio: AudioSettings,
}

impl SwalSystemSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Hierarchical dot-notation key lookup.
    pub fn get_value(&self, key: &str) -> Option<String> {
        let json_val = serde_json::to_value(self).ok()?;
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &json_val;
        for part in parts {
            current = current.get(part)?;
        }
        match current {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Null => None,
            v => Some(v.to_string()),
        }
    }

    /// Mutates setting value given dot-notation key and string value representation.
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "appearance.theme" => self.appearance.theme = value.to_string(),
            "appearance.accent_color" => self.appearance.accent_color = value.to_string(),
            "appearance.corner_radius" => self
                .appearance
                .corner_radius = value.parse().map_err(|e| format!("Invalid float: {}", e))?,
            "appearance.acrylic_opacity" => self
                .appearance
                .acrylic_opacity = value.parse().map_err(|e| format!("Invalid float: {}", e))?,
            "appearance.wallpaper_path" => {
                self.appearance.wallpaper_path = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            "agent.default_agent" => self.agent.default_agent = value.to_string(),
            "agent.model_routing" => self.agent.model_routing = value.to_string(),
            "agent.voice_orb_enabled" => self
                .agent
                .voice_orb_enabled = value.parse().map_err(|e| format!("Invalid bool: {}", e))?,
            "agent.audio_sensitivity" => self
                .agent
                .audio_sensitivity = value.parse().map_err(|e| format!("Invalid float: {}", e))?,
            "agent.auto_ui_generation" => self
                .agent
                .auto_ui_generation = value.parse().map_err(|e| format!("Invalid bool: {}", e))?,
            "display.target_fps" => self
                .display
                .target_fps = value.parse().map_err(|e| format!("Invalid u32: {}", e))?,
            "display.hidpi_scale" => self
                .display
                .hidpi_scale = value.parse().map_err(|e| format!("Invalid float: {}", e))?,
            "display.vsync" => self
                .display
                .vsync = value.parse().map_err(|e| format!("Invalid bool: {}", e))?,
            "display.compositor" => self.display.compositor = value.to_string(),
            "storage.default_dual_pane" => self
                .storage
                .default_dual_pane = value.parse().map_err(|e| format!("Invalid bool: {}", e))?,
            "storage.show_hidden" => self
                .storage
                .show_hidden = value.parse().map_err(|e| format!("Invalid bool: {}", e))?,
            "storage.low_space_alert_gb" => self
                .storage
                .low_space_alert_gb = value.parse().map_err(|e| format!("Invalid u32: {}", e))?,
            "network.node_id" => self.network.node_id = value.to_string(),
            "network.mesh_port" => self
                .network
                .mesh_port = value.parse().map_err(|e| format!("Invalid u16: {}", e))?,
            "network.xavier_endpoint" => self.network.xavier_endpoint = value.to_string(),
            "audio.pipewire_sink" => self.audio.pipewire_sink = value.to_string(),
            "audio.mic_gain" => self
                .audio
                .mic_gain = value.parse().map_err(|e| format!("Invalid float: {}", e))?,
            _ => return Err(format!("Unknown setting key: {}", key)),
        }
        Ok(())
    }
}

/// JSON IPC Request protocol variant enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum SettingsIpcRequest {
    Get { key: String },
    Set { key: String, value: String },
    ListAll,
    SubscribeChanges,
}

/// JSON IPC Response protocol variant enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SettingsIpcResponse {
    Value { key: String, value: String },
    Ok { message: String },
    Error { error: String },
    AllSettings(SwalSystemSettings),
}

/// Event broadcast when a setting mutation occurs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingChangeEvent {
    pub key: String,
    pub value: String,
}

/// IPC Server handling settings mutations and live subscription broadcasts over Unix sockets.
pub struct SettingsIpcServer {
    socket_path: PathBuf,
    settings: Arc<RwLock<SwalSystemSettings>>,
    broadcast_tx: broadcast::Sender<SettingChangeEvent>,
}

impl SettingsIpcServer {
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        let (broadcast_tx, _) = broadcast::channel(128);
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            settings: Arc::new(RwLock::new(SwalSystemSettings::default())),
            broadcast_tx,
        }
    }

    pub fn default_server() -> Self {
        Self::new(DEFAULT_SETTINGS_SOCKET_PATH)
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn settings(&self) -> Arc<RwLock<SwalSystemSettings>> {
        Arc::clone(&self.settings)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SettingChangeEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Process a single `SettingsIpcRequest` and produce a corresponding `SettingsIpcResponse`.
    pub async fn process_request(
        req: SettingsIpcRequest,
        settings_lock: &Arc<RwLock<SwalSystemSettings>>,
        broadcast_tx: &broadcast::Sender<SettingChangeEvent>,
    ) -> SettingsIpcResponse {
        match req {
            SettingsIpcRequest::Get { key } => {
                let settings = settings_lock.read().await;
                if let Some(val) = settings.get_value(&key) {
                    SettingsIpcResponse::Value { key, value: val }
                } else {
                    SettingsIpcResponse::Error {
                        error: format!("Setting key not found: {}", key),
                    }
                }
            }
            SettingsIpcRequest::Set { key, value } => {
                let mut settings = settings_lock.write().await;
                match settings.set_value(&key, &value) {
                    Ok(()) => {
                        let _ = broadcast_tx.send(SettingChangeEvent {
                            key: key.clone(),
                            value: value.clone(),
                        });
                        SettingsIpcResponse::Ok {
                            message: format!("Setting '{}' set to '{}'", key, value),
                        }
                    }
                    Err(err) => SettingsIpcResponse::Error { error: err },
                }
            }
            SettingsIpcRequest::ListAll => {
                let settings = settings_lock.read().await;
                SettingsIpcResponse::AllSettings(settings.clone())
            }
            SettingsIpcRequest::SubscribeChanges => SettingsIpcResponse::Ok {
                message: "Subscribed to live configuration changes".to_string(),
            },
        }
    }

    /// Runs the non-blocking async Unix domain socket listener loop.
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
                    let settings_lock = Arc::clone(&self.settings);
                    let broadcast_tx = self.broadcast_tx.clone();
                    tokio::spawn(async move {
                        let (read_half, mut write_half) = stream.into_split();
                        let mut reader = BufReader::new(read_half);
                        let mut line = String::new();

                        while let Ok(bytes_read) = reader.read_line(&mut line).await {
                            if bytes_read == 0 {
                                break;
                            }

                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                match serde_json::from_str::<SettingsIpcRequest>(trimmed) {
                                    Ok(SettingsIpcRequest::SubscribeChanges) => {
                                        // Respond ok, then stream live changes
                                        let initial_resp = SettingsIpcResponse::Ok {
                                            message: "Subscribed to live configuration changes"
                                                .to_string(),
                                        };
                                        if let Ok(resp_json) = serde_json::to_string(&initial_resp)
                                        {
                                            let _ = write_half
                                                .write_all(format!("{}\n", resp_json).as_bytes())
                                                .await;
                                            let _ = write_half.flush().await;
                                        }

                                        let mut rx = broadcast_tx.subscribe();
                                        while let Ok(change) = rx.recv().await {
                                            let resp = SettingsIpcResponse::Value {
                                                key: change.key,
                                                value: change.value,
                                            };
                                            if let Ok(resp_json) = serde_json::to_string(&resp) {
                                                if write_half
                                                    .write_all(format!("{}\n", resp_json).as_bytes())
                                                    .await
                                                    .is_err()
                                                {
                                                    break;
                                                }
                                                let _ = write_half.flush().await;
                                            }
                                        }
                                        break;
                                    }
                                    Ok(req) => {
                                        let resp = Self::process_request(
                                            req,
                                            &settings_lock,
                                            &broadcast_tx,
                                        )
                                        .await;
                                        if let Ok(resp_json) = serde_json::to_string(&resp) {
                                            let _ = write_half
                                                .write_all(format!("{}\n", resp_json).as_bytes())
                                                .await;
                                            let _ = write_half.flush().await;
                                        }
                                    }
                                    Err(e) => {
                                        let err_resp = SettingsIpcResponse::Error {
                                            error: format!("Invalid JSON request: {}", e),
                                        };
                                        if let Ok(resp_json) = serde_json::to_string(&err_resp) {
                                            let _ = write_half
                                                .write_all(format!("{}\n", resp_json).as_bytes())
                                                .await;
                                            let _ = write_half.flush().await;
                                        }
                                    }
                                }
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

impl Drop for SettingsIpcServer {
    fn drop(&mut self) {
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}

/// Client helper function to send a request to default IPC socket path.
pub fn send_settings_ipc_request(
    request: &SettingsIpcRequest,
) -> Result<SettingsIpcResponse, String> {
    send_settings_ipc_request_to_path(request, DEFAULT_SETTINGS_SOCKET_PATH)
}

/// Async client helper function to send a request to a specific socket path.
pub async fn send_settings_ipc_request_to_path_async<P: AsRef<Path>>(
    request: &SettingsIpcRequest,
    path: P,
) -> Result<SettingsIpcResponse, String> {
    use tokio::net::UnixStream;

    let mut stream = UnixStream::connect(path.as_ref()).await.map_err(|e| {
        format!(
            "Failed to connect to Settings IPC socket at {:?}: {}",
            path.as_ref(),
            e
        )
    })?;

    let json = serde_json::to_string(request)
        .map_err(|e| format!("Failed to serialize SettingsIpcRequest: {}", e))?;

    stream
        .write_all(format!("{}\n", json).as_bytes())
        .await
        .map_err(|e| format!("Failed to write to Settings IPC socket: {}", e))?;

    stream
        .flush()
        .await
        .map_err(|e| format!("Failed to flush Settings IPC socket: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| format!("Failed to read response from socket: {}", e))?;

    let response: SettingsIpcResponse = serde_json::from_str(response_line.trim())
        .map_err(|e| format!("Failed to deserialize SettingsIpcResponse: {}", e))?;

    Ok(response)
}

/// Client helper function to send a request to a specific socket path.
pub fn send_settings_ipc_request_to_path<P: AsRef<Path>>(
    request: &SettingsIpcRequest,
    path: P,
) -> Result<SettingsIpcResponse, String> {
    use std::io::{BufRead, Write};
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(path.as_ref()).map_err(|e| {
        format!(
            "Failed to connect to Settings IPC socket at {:?}: {}",
            path.as_ref(),
            e
        )
    })?;

    let json = serde_json::to_string(request)
        .map_err(|e| format!("Failed to serialize SettingsIpcRequest: {}", e))?;

    writeln!(stream, "{}", json)
        .map_err(|e| format!("Failed to write to Settings IPC socket: {}", e))?;

    stream
        .flush()
        .map_err(|e| format!("Failed to flush Settings IPC socket: {}", e))?;

    let mut reader = std::io::BufReader::new(&stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Failed to read response from socket: {}", e))?;

    let response: SettingsIpcResponse = serde_json::from_str(response_line.trim())
        .map_err(|e| format!("Failed to deserialize SettingsIpcResponse: {}", e))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req_get = SettingsIpcRequest::Get {
            key: "appearance.theme".to_string(),
        };
        let json_get = serde_json::to_string(&req_get).unwrap();
        assert!(json_get.contains(r#""cmd":"get""#));
        assert!(json_get.contains(r#""key":"appearance.theme""#));

        let req_set = SettingsIpcRequest::Set {
            key: "display.target_fps".to_string(),
            value: "240".to_string(),
        };
        let json_set = serde_json::to_string(&req_set).unwrap();
        assert!(json_set.contains(r#""cmd":"set""#));
        assert!(json_set.contains(r#""value":"240""#));

        let req_list = SettingsIpcRequest::ListAll;
        let json_list = serde_json::to_string(&req_list).unwrap();
        assert!(json_list.contains(r#""cmd":"list_all""#));
    }

    #[test]
    fn test_response_serialization() {
        let val_resp = SettingsIpcResponse::Value {
            key: "appearance.theme".to_string(),
            value: "fluent-dark".to_string(),
        };
        let val_json = serde_json::to_string(&val_resp).unwrap();
        let val_deser: SettingsIpcResponse = serde_json::from_str(&val_json).unwrap();
        assert_eq!(val_deser, val_resp);

        let ok_resp = SettingsIpcResponse::Ok {
            message: "Updated".to_string(),
        };
        let ok_json = serde_json::to_string(&ok_resp).unwrap();
        let ok_deser: SettingsIpcResponse = serde_json::from_str(&ok_json).unwrap();
        assert_eq!(ok_deser, ok_resp);

        let err_resp = SettingsIpcResponse::Error {
            error: "Key not found".to_string(),
        };
        let err_json = serde_json::to_string(&err_resp).unwrap();
        let err_deser: SettingsIpcResponse = serde_json::from_str(&err_json).unwrap();
        assert_eq!(err_deser, err_resp);

        let all_resp = SettingsIpcResponse::AllSettings(SwalSystemSettings::default());
        let all_json = serde_json::to_string(&all_resp).unwrap();
        let all_deser: SettingsIpcResponse = serde_json::from_str(&all_json).unwrap();
        assert_eq!(all_deser, all_resp);
    }

    #[test]
    fn test_swal_system_settings_get_set() {
        let mut settings = SwalSystemSettings::default();
        assert_eq!(
            settings.get_value("appearance.theme"),
            Some("fluent-dark".to_string())
        );

        assert!(settings
            .set_value("appearance.theme", "fluent-light")
            .is_ok());
        assert_eq!(
            settings.get_value("appearance.theme"),
            Some("fluent-light".to_string())
        );

        assert!(settings.set_value("display.target_fps", "240").is_ok());
        assert_eq!(
            settings.get_value("display.target_fps"),
            Some("240".to_string())
        );

        assert!(settings.set_value("invalid.key", "value").is_err());
    }

    #[tokio::test]
    async fn test_settings_ipc_server_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sock_path = temp_dir.path().join("test_swal_settings.sock");

        let server = SettingsIpcServer::new(&sock_path);
        let server_path = server.socket_path().to_path_buf();

        let server_handle = tokio::spawn(async move {
            let _ = server.run().await;
        });

        // Give server time to bind
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(server_path.exists());

        // Get default theme
        let req_get = SettingsIpcRequest::Get {
            key: "appearance.theme".to_string(),
        };
        let resp = send_settings_ipc_request_to_path_async(&req_get, &server_path).await.unwrap();
        assert_eq!(
            resp,
            SettingsIpcResponse::Value {
                key: "appearance.theme".to_string(),
                value: "fluent-dark".to_string()
            }
        );

        // Set theme
        let req_set = SettingsIpcRequest::Set {
            key: "appearance.theme".to_string(),
            value: "mica-dark".to_string(),
        };
        let resp_set = send_settings_ipc_request_to_path_async(&req_set, &server_path).await.unwrap();
        assert_eq!(
            resp_set,
            SettingsIpcResponse::Ok {
                message: "Setting 'appearance.theme' set to 'mica-dark'".to_string()
            }
        );

        // Verify set theme
        let resp_verify = send_settings_ipc_request_to_path_async(&req_get, &server_path).await.unwrap();
        assert_eq!(
            resp_verify,
            SettingsIpcResponse::Value {
                key: "appearance.theme".to_string(),
                value: "mica-dark".to_string()
            }
        );

        // List all settings
        let req_list = SettingsIpcRequest::ListAll;
        let resp_list = send_settings_ipc_request_to_path_async(&req_list, &server_path).await.unwrap();
        match resp_list {
            SettingsIpcResponse::AllSettings(s) => {
                assert_eq!(s.appearance.theme, "mica-dark");
            }
            _ => panic!("Expected AllSettings response"),
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_subscribe_changes_broadcast() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sock_path = temp_dir.path().join("test_subscribe_settings.sock");

        let server = SettingsIpcServer::new(&sock_path);
        let server_path = server.socket_path().to_path_buf();

        let server_handle = tokio::spawn(async move {
            let _ = server.run().await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connect a subscriber stream
        let stream = tokio::net::UnixStream::connect(&server_path)
            .await
            .unwrap();
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        let sub_req = SettingsIpcRequest::SubscribeChanges;
        let req_json = serde_json::to_string(&sub_req).unwrap();
        write_half
            .write_all(format!("{}\n", req_json).as_bytes())
            .await
            .unwrap();

        // Read initial ack
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let init_resp: SettingsIpcResponse = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(
            init_resp,
            SettingsIpcResponse::Ok {
                message: "Subscribed to live configuration changes".to_string()
            }
        );

        // Send a mutation setting change from another client
        let req_set = SettingsIpcRequest::Set {
            key: "audio.mic_gain".to_string(),
            value: "1.5".to_string(),
        };
        let _ = send_settings_ipc_request_to_path_async(&req_set, &server_path).await.unwrap();

        // Read broadcast event line from subscriber
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        let broadcast_event: SettingsIpcResponse = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(
            broadcast_event,
            SettingsIpcResponse::Value {
                key: "audio.mic_gain".to_string(),
                value: "1.5".to_string()
            }
        );

        server_handle.abort();
    }
}
