//! Declarative JSON configuration for SWAL Files

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarPin {
    pub name: String,
    pub path: PathBuf,
    pub icon: String,
    pub section: String, // "pinned", "workspaces", "drives", "tags"
}

/// A named filter+sort preset saved by the user for quick recall
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedFilterPreset {
    pub name: String,
    pub filter_type: String,
    pub sort_by: String,
    pub sort_order: String,
    pub group_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagerConfig {
    #[serde(default = "default_theme")]
    pub theme_id: String,
    #[serde(default = "default_view_mode")]
    pub default_view_mode: String, // "details" or "grid"
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default)]
    pub dual_pane_enabled: bool,
    #[serde(default = "default_sort_by")]
    pub sort_by: String, // "name", "date", "type", "size"
    #[serde(default = "default_sort_order")]
    pub sort_order: String, // "asc", "desc"
    #[serde(default = "default_group_by")]
    pub group_by: String, // "none", "type", "date", "size", "alphabetical"
    #[serde(default = "default_filter_type")]
    pub filter_type: String, // "all", "folders", "code", "documents", "images", "media", "archives"
    #[serde(default = "default_preview_mode")]
    pub preview_mode: String, // "sidebar", "window", "none"
    #[serde(default)]
    pub selected_path: Option<String>,
    #[serde(default = "default_pinned")]
    pub pinned_locations: Vec<SidebarPin>,
    #[serde(default = "default_workspaces")]
    pub agent_workspaces: Vec<PathBuf>,
    #[serde(default = "default_tags")]
    pub custom_tags: Vec<String>,
    /// Saved filter presets — restored across sessions
    #[serde(default)]
    pub saved_filter_presets: Vec<SavedFilterPreset>,
}


fn default_theme() -> String {
    "hive-dark".to_string()
}
fn default_view_mode() -> String {
    "details".to_string()
}
fn default_sort_by() -> String {
    "name".to_string()
}
fn default_sort_order() -> String {
    "asc".to_string()
}
fn default_group_by() -> String {
    "none".to_string()
}
fn default_filter_type() -> String {
    "all".to_string()
}
fn default_preview_mode() -> String {
    "sidebar".to_string()
}

fn default_pinned() -> Vec<SidebarPin> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        SidebarPin {
            name: "Home".to_string(),
            path: home.clone(),
            icon: "🏠".to_string(),
            section: "pinned".to_string(),
        },
        SidebarPin {
            name: "Proyectos SWAL".to_string(),
            path: home.join("proyectosSWAL"),
            icon: "⚡".to_string(),
            section: "workspaces".to_string(),
        },
        SidebarPin {
            name: "Descargas".to_string(),
            path: home.join("Downloads"),
            icon: "📥".to_string(),
            section: "pinned".to_string(),
        },
        SidebarPin {
            name: "Documentos".to_string(),
            path: home.join("Documents"),
            icon: "📁".to_string(),
            section: "pinned".to_string(),
        },
    ]
}

fn default_workspaces() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        home.join("proyectosSWAL"),
        home.join(".config/swal"),
    ]
}

fn default_tags() -> Vec<String> {
    vec![
        "Agentic".to_string(),
        "SWAL Core".to_string(),
        "Memory/Xavier".to_string(),
        "WIP".to_string(),
    ]
}

impl Default for FileManagerConfig {
    fn default() -> Self {
        Self {
            theme_id: default_theme(),
            default_view_mode: default_view_mode(),
            show_hidden: false,
            dual_pane_enabled: false,
            sort_by: default_sort_by(),
            sort_order: default_sort_order(),
            group_by: default_group_by(),
            filter_type: default_filter_type(),
            preview_mode: default_preview_mode(),
            selected_path: None,
            pinned_locations: default_pinned(),
            agent_workspaces: default_workspaces(),
            custom_tags: default_tags(),
            saved_filter_presets: Vec::new(),
        }
    }
}

impl FileManagerConfig {
    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(".config/swal/files/config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        Self::load_from_path(&path)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        self.save_to_path(&path)
    }

    pub fn load_from_path(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json_str)
    }

    pub fn is_pinned(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.pinned_locations.iter().any(|p| p.path == path || p.path.to_string_lossy() == path_str)
    }

    pub fn add_pin(&mut self, path: PathBuf, name: Option<String>, icon: Option<String>, section: Option<String>) -> bool {
        if self.is_pinned(&path) {
            return false;
        }
        let fallback_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Carpeta".to_string());

        let pin = SidebarPin {
            name: name.unwrap_or(fallback_name),
            path,
            icon: icon.unwrap_or_else(|| "📁".to_string()),
            section: section.unwrap_or_else(|| "pinned".to_string()),
        };
        self.pinned_locations.push(pin);
        true
    }

    pub fn remove_pin(&mut self, path: &Path) -> bool {
        let before_len = self.pinned_locations.len();
        let path_str = path.to_string_lossy();
        self.pinned_locations.retain(|p| p.path != path && p.path.to_string_lossy() != path_str);
        self.pinned_locations.len() < before_len
    }

    pub fn toggle_pin(&mut self, path: PathBuf) -> bool {
        if self.is_pinned(&path) {
            self.remove_pin(&path);
            false
        } else {
            self.add_pin(path, None, None, None);
            true
        }
    }
}


