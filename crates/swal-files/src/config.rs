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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagerConfig {
    pub theme_id: String,
    pub show_hidden: bool,
    pub dual_pane_enabled: bool,
    pub default_view_mode: String, // "grid" or "list"
    pub pinned_locations: Vec<SidebarPin>,
    pub agent_workspaces: Vec<PathBuf>,
    pub custom_tags: Vec<String>,
}

impl Default for FileManagerConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));
        Self {
            theme_id: "hive-dark".to_string(),
            show_hidden: false,
            dual_pane_enabled: false,
            default_view_mode: "list".to_string(),
            pinned_locations: vec![
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
            ],
            agent_workspaces: vec![
                home.join("proyectosSWAL"),
                home.join(".config/swal"),
            ],
            custom_tags: vec![
                "Agentic".to_string(),
                "SWAL Core".to_string(),
                "Memory/Xavier".to_string(),
                "WIP".to_string(),
            ],
        }
    }
}

impl FileManagerConfig {
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
}
