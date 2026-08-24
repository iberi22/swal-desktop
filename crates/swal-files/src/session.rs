//! Session state persistence and management for SWAL Files

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::config::FileManagerConfig;

pub fn get_session_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));
    let dir = home.join(".config/swal/files");
    let _ = fs::create_dir_all(&dir);
    dir.join("session.json")
}

pub fn get_editor_session_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));
    let dir = home.join(".config/swal/files");
    let _ = fs::create_dir_all(&dir);
    dir.join("editor_session.json")
}

pub const STATE_FILE: &str = "/home/belal/.config/swal/files/session.json";
pub const EDITOR_STATE_FILE: &str = "/home/belal/.config/swal/files/editor_session.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: usize,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub active_tab_id: usize,
    pub tabs: Vec<TabState>,
    pub view_mode: String,
    pub show_hidden: bool,
    pub dual_pane: bool,
    pub search_query: String,
    pub is_maximized: bool,
    pub sort_by: String,
    pub sort_order: String,
    pub group_by: String,
    pub filter_type: String,
    pub preview_mode: String,
    pub selected_path: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        let cfg = FileManagerConfig::load();
        let home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/home/belal"))
            .to_string_lossy()
            .to_string();

        Self {
            active_tab_id: 1,
            tabs: vec![
                TabState {
                    id: 1,
                    title: "swal-desktop".to_string(),
                    path: format!("{}/proyectosSWAL/periferia/swal-desktop", home),
                    active: true,
                },
                TabState {
                    id: 2,
                    title: "proyectosSWAL".to_string(),
                    path: format!("{}/proyectosSWAL", home),
                    active: false,
                },
            ],
            view_mode: cfg.default_view_mode,
            show_hidden: cfg.show_hidden,
            dual_pane: cfg.dual_pane_enabled,
            search_query: String::new(),
            is_maximized: false,
            sort_by: cfg.sort_by,
            sort_order: cfg.sort_order,
            group_by: cfg.group_by,
            filter_type: cfg.filter_type,
            preview_mode: cfg.preview_mode,
            selected_path: None,
        }
    }
}

pub fn load_session() -> SessionState {
    load_session_from_path(Path::new(STATE_FILE))
}

pub fn load_session_from_path(path: &Path) -> SessionState {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(session) = serde_json::from_str::<SessionState>(&content) {
            return session;
        }
    }
    SessionState::default()
}

pub fn save_session(session: &SessionState) {
    save_session_to_path(session, Path::new(STATE_FILE));
}

pub fn save_session_to_path(session: &SessionState, path: &Path) {
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = fs::write(path, json);
    }
    // Also sync preferences to persistent user config
    let mut cfg = FileManagerConfig::load();
    cfg.default_view_mode = session.view_mode.clone();
    cfg.show_hidden = session.show_hidden;
    cfg.sort_by = session.sort_by.clone();
    cfg.sort_order = session.sort_order.clone();
    cfg.group_by = session.group_by.clone();
    cfg.filter_type = session.filter_type.clone();
    cfg.preview_mode = session.preview_mode.clone();
    cfg.selected_path = session.selected_path.clone();
    let _ = cfg.save();
}
