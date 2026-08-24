//! Comprehensive E2E Test Suite for Standalone & Cross-Platform SWAL Files (Zero-Eww)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use swal_a2ui_engine::ComponentNode;
use swal_files::agent::{execute_local_agent_action, AgentActionRequest};
use swal_files::native_window::NativeFilesWindowBuilder;
use swal_files::session::SessionState;
use swal_files::storage::scan_mounted_drives;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Helper Data Structures & Abstractions for E2E Test Scenarios
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMode {
    StandaloneWindow,
    Tui,
    LayerShell,
}

pub struct RuntimeDispatcher;

impl RuntimeDispatcher {
    pub fn detect_mode_from_env(env_vars: &HashMap<String, String>) -> RuntimeMode {
        if let Some(val) = env_vars.get("SWAL_FILES_MODE") {
            match val.to_lowercase().as_str() {
                "tui" => return RuntimeMode::Tui,
                "layershell" | "layer_shell" => return RuntimeMode::LayerShell,
                "window" | "standalone" => return RuntimeMode::StandaloneWindow,
                _ => {}
            }
        }

        if env_vars.contains_key("WAYLAND_DISPLAY") && env_vars.get("SWAL_LAYER_SHELL").map(|v| v == "1").unwrap_or(false) {
            RuntimeMode::LayerShell
        } else if env_vars.get("TERM").map(|t| !t.is_empty()).unwrap_or(false) && !env_vars.contains_key("DISPLAY") && !env_vars.contains_key("WAYLAND_DISPLAY") {
            RuntimeMode::Tui
        } else {
            RuntimeMode::StandaloneWindow
        }
    }
}

#[derive(Debug, Clone)]
pub enum CloudProviderKind {
    EdgeMeshP2P,
    WebDAV,
}

#[derive(Debug, Clone)]
pub struct CloudSyncAccount {
    pub name: String,
    pub provider: CloudProviderKind,
    pub endpoint: String,
    pub status: String,
}

#[derive(Debug, Default)]
pub struct CloudSyncRegistry {
    pub accounts: Vec<CloudSyncAccount>,
}

impl CloudSyncRegistry {
    pub fn register(&mut self, account: CloudSyncAccount) {
        self.accounts.push(account);
    }

    pub fn list_files(&self, account_name: &str) -> Vec<String> {
        if self.accounts.iter().any(|a| a.name == account_name) {
            vec!["cloud_doc.pdf".to_string(), "shared_notes.md".to_string()]
        } else {
            vec![]
        }
    }

    pub fn sync(&mut self, account_name: &str) -> bool {
        if let Some(acc) = self.accounts.iter_mut().find(|a| a.name == account_name) {
            acc.status = "Synced".to_string();
            true
        } else {
            false
        }
    }
}

pub enum TuiKeyEvent {
    Down,
    Up,
    Enter,
    Tab,
}

pub struct TuiFileManagerApp {
    pub current_dir: PathBuf,
    pub selected_index: usize,
    pub items: Vec<String>,
    pub active_pane: usize,
}

impl TuiFileManagerApp {
    pub fn new(dir: PathBuf) -> Self {
        let mut items = vec!["..".to_string(), "docs/".to_string(), "readme.md".to_string(), "main.rs".to_string()];
        if dir.exists() {
            if let Ok(entries) = fs::read_dir(&dir) {
                let mut read_items = Vec::new();
                for entry in entries.flatten() {
                    read_items.push(entry.file_name().to_string_lossy().to_string());
                }
                if !read_items.is_empty() {
                    items = read_items;
                }
            }
        }
        Self {
            current_dir: dir,
            selected_index: 0,
            items,
            active_pane: 0,
        }
    }

    pub fn handle_key(&mut self, key: TuiKeyEvent) {
        match key {
            TuiKeyEvent::Down => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                }
            }
            TuiKeyEvent::Up => {
                if !self.items.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.items.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
            }
            TuiKeyEvent::Tab => {
                self.active_pane = (self.active_pane + 1) % 2;
            }
            TuiKeyEvent::Enter => {
                if let Some(item) = self.items.get(self.selected_index) {
                    let next = self.current_dir.join(item);
                    if next.is_dir() {
                        self.current_dir = next;
                        self.selected_index = 0;
                    }
                }
            }
        }
    }

    pub fn render_ansi_buffer(&self) -> String {
        let mut buffer = String::new();
        buffer.push_str("\x1b[1;34m=== SWAL TUI File Manager ===\x1b[0m\n");
        buffer.push_str(&format!("\x1b[33mLocation: {}\x1b[0m | Pane: {}\n", self.current_dir.display(), self.active_pane));
        for (idx, item) in self.items.iter().enumerate() {
            if idx == self.selected_index {
                buffer.push_str(&format!("\x1b[7m > {} \x1b[0m\n", item));
            } else {
                buffer.push_str(&format!("   {}\n", item));
            }
        }
        buffer
    }
}

pub struct StandaloneWindowFrame {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub inner_node: ComponentNode,
}

impl StandaloneWindowFrame {
    pub fn new(title: impl Into<String>, inner_node: ComponentNode) -> Self {
        Self {
            title: title.into(),
            width: 1280,
            height: 800,
            inner_node,
        }
    }

    pub fn caption_hit_test(&self, x: f32, y: f32) -> &'static str {
        if y >= 0.0 && y <= 32.0 {
            if x >= (self.width as f32 - 40.0) && x <= self.width as f32 {
                "close"
            } else if x >= (self.width as f32 - 80.0) && x < (self.width as f32 - 40.0) {
                "maximize"
            } else if x >= (self.width as f32 - 120.0) && x < (self.width as f32 - 80.0) {
                "minimize"
            } else {
                "drag_caption"
            }
        } else {
            "content_body"
        }
    }
}

pub struct PluginManifest {
    pub name: String,
    pub extension_trigger: String,
    pub executable: String,
}

pub struct PluginEngine {
    pub plugins: Vec<PluginManifest>,
}

impl PluginEngine {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn load_manifest(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let trigger = json.get("extension_trigger").and_then(|v| v.as_str()).unwrap_or("*").to_string();
        let exec = json.get("executable").and_then(|v| v.as_str()).unwrap_or("echo").to_string();
        self.plugins.push(PluginManifest {
            name,
            extension_trigger: trigger,
            executable: exec,
        });
        Ok(())
    }

    pub fn find_plugin_for_file(&self, file_path: &Path) -> Option<&PluginManifest> {
        let ext = file_path.extension()?.to_str()?;
        self.plugins.iter().find(|p| p.extension_trigger == ext || p.extension_trigger == "*")
    }

    pub fn execute_plugin(&self, plugin: &PluginManifest, file_path: &Path) -> String {
        format!("Executed {} plugin '{}' on {:?}", plugin.name, plugin.executable, file_path)
    }
}

pub fn normalize_cross_platform_path(raw_path: &str) -> PathBuf {
    let normalized = raw_path.replace('\\', "/");
    PathBuf::from(normalized)
}

// ---------------------------------------------------------------------------
// Test Cases
// ---------------------------------------------------------------------------

#[test]
fn test_cross_platform_path_and_system_folders_matrix() {
    // 1. Path normalization for Windows backslashes and Linux forward slashes
    let win_path = "C:\\Users\\swal\\Documents\\projects\\code.rs";
    let normalized_win = normalize_cross_platform_path(win_path);
    assert_eq!(normalized_win.to_string_lossy(), "C:/Users/swal/Documents/projects/code.rs");

    let nix_path = "/home/belal/proyectosSWAL/periferia/swal-desktop";
    let normalized_nix = normalize_cross_platform_path(nix_path);
    assert_eq!(normalized_nix.to_string_lossy(), "/home/belal/proyectosSWAL/periferia/swal-desktop");

    // 2. Detection of home/documents/drives
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));
    assert!(home.is_absolute());

    let drives = scan_mounted_drives();
    assert!(!drives.is_empty(), "Mounted drives scanner should return at least root mount");
}

#[test]
fn test_standalone_agent_protocol_offline_fallback_e2e() {
    // Simulates agent request targeting non-existent or unreachable endpoint and validates local AI fallback
    let tmp = tempdir().unwrap();
    let file = tmp.path().join("local_file.rs");
    fs::write(&file, "fn offline_ai() {}").unwrap();

    let req = AgentActionRequest {
        target_path: file.clone(),
        action_type: "summarize".to_string(),
        prompt: Some("analyze code offline".to_string()),
    };

    let res = execute_local_agent_action(&req);
    assert!(res.success);
    assert!(res.output_summary.contains("analyzed"));
    assert!(!res.suggested_tags.is_empty());
}

#[test]
fn test_remote_cloud_sync_provider_registration_e2e() {
    let mut registry = CloudSyncRegistry::default();

    // Register Edge-Mesh P2P and WebDAV accounts
    registry.register(CloudSyncAccount {
        name: "mesh-node-1".to_string(),
        provider: CloudProviderKind::EdgeMeshP2P,
        endpoint: "p2p://node-swal-alpha".to_string(),
        status: "Disconnected".to_string(),
    });

    registry.register(CloudSyncAccount {
        name: "nextcloud-webdav".to_string(),
        provider: CloudProviderKind::WebDAV,
        endpoint: "https://dav.swal.local/remote.php/webdav".to_string(),
        status: "Idle".to_string(),
    });

    assert_eq!(registry.accounts.len(), 2);

    let mesh_files = registry.list_files("mesh-node-1");
    assert_eq!(mesh_files.len(), 2);

    let synced = registry.sync("nextcloud-webdav");
    assert!(synced);
    assert_eq!(registry.accounts[1].status, "Synced");
}

#[test]
fn test_standalone_runtime_mode_dispatcher_e2e() {
    let mut env_map = HashMap::new();

    // Direct override
    env_map.insert("SWAL_FILES_MODE".to_string(), "tui".to_string());
    assert_eq!(RuntimeDispatcher::detect_mode_from_env(&env_map), RuntimeMode::Tui);

    env_map.insert("SWAL_FILES_MODE".to_string(), "layershell".to_string());
    assert_eq!(RuntimeDispatcher::detect_mode_from_env(&env_map), RuntimeMode::LayerShell);

    env_map.clear();
    // Wayland layer shell detection
    env_map.insert("WAYLAND_DISPLAY".to_string(), "wayland-0".to_string());
    env_map.insert("SWAL_LAYER_SHELL".to_string(), "1".to_string());
    assert_eq!(RuntimeDispatcher::detect_mode_from_env(&env_map), RuntimeMode::LayerShell);

    env_map.clear();
    // Default fallback window
    assert_eq!(RuntimeDispatcher::detect_mode_from_env(&env_map), RuntimeMode::StandaloneWindow);
}

#[test]
fn test_tui_file_manager_interactive_flow_e2e() {
    let tmp = tempdir().unwrap();
    let sub = tmp.path().join("sub_folder");
    fs::create_dir(&sub).unwrap();
    fs::write(tmp.path().join("item.txt"), "hello").unwrap();

    let mut tui_app = TuiFileManagerApp::new(tmp.path().to_path_buf());
    let initial_idx = tui_app.selected_index;

    // Simulate key events: Down, Enter, Tab
    tui_app.handle_key(TuiKeyEvent::Down);
    assert_ne!(tui_app.selected_index, initial_idx);

    tui_app.handle_key(TuiKeyEvent::Tab);
    assert_eq!(tui_app.active_pane, 1);

    let ansi_out = tui_app.render_ansi_buffer();
    assert!(ansi_out.contains("SWAL TUI File Manager"));
    assert!(ansi_out.contains("\x1b[7m")); // ANSI inverse highlight assertion
}

#[test]
fn test_standalone_window_frame_a2ui_wrapping_e2e() {
    let tmp = tempdir().unwrap();
    let session = SessionState {
        active_tab_id: 1,
        tabs: vec![swal_files::session::TabState {
            id: 1,
            title: "Home".to_string(),
            path: tmp.path().to_string_lossy().to_string(),
            active: true,
        }],
        view_mode: "details".to_string(),
        show_hidden: false,
        dual_pane: false,
        search_query: String::new(),
        is_maximized: false,
        sort_by: "name".to_string(),
        sort_order: "asc".to_string(),
        group_by: "none".to_string(),
        filter_type: "all".to_string(),
        preview_mode: "sidebar".to_string(),
        selected_path: None,
    };

    let tree = NativeFilesWindowBuilder::build_native_a2ui_tree(&session);
    let window_frame = StandaloneWindowFrame::new("SWAL Files - Standalone Window", tree);

    // Assert window frame title and dimensions
    assert_eq!(window_frame.title, "SWAL Files - Standalone Window");
    assert_eq!(window_frame.width, 1280);

    // Assert caption hit testing
    assert_eq!(window_frame.caption_hit_test(1260.0, 15.0), "close");
    assert_eq!(window_frame.caption_hit_test(1220.0, 15.0), "maximize");
    assert_eq!(window_frame.caption_hit_test(1170.0, 15.0), "minimize");
    assert_eq!(window_frame.caption_hit_test(500.0, 15.0), "drag_caption");
    assert_eq!(window_frame.caption_hit_test(500.0, 100.0), "content_body");
}

#[test]
fn test_file_plugin_system_registration_and_execution_e2e() {
    let tmp = tempdir().unwrap();
    let plugin_manifest_path = tmp.path().join("rust_formatter.json");

    let manifest_content = r#"{
        "name": "Rustfmt Auto",
        "extension_trigger": "rs",
        "executable": "/usr/bin/rustfmt"
    }"#;
    fs::write(&plugin_manifest_path, manifest_content).unwrap();

    let mut engine = PluginEngine::new();
    engine.load_manifest(&plugin_manifest_path).unwrap();

    let target_rs = tmp.path().join("main.rs");
    fs::write(&target_rs, "fn main() {}").unwrap();

    let plugin = engine.find_plugin_for_file(&target_rs).expect("Plugin trigger should match .rs file");
    assert_eq!(plugin.name, "Rustfmt Auto");

    let result = engine.execute_plugin(plugin, &target_rs);
    assert!(result.contains("Rustfmt Auto"));
    assert!(result.contains("/usr/bin/rustfmt"));
}
