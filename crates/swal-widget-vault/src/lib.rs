use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tar::{Archive, Builder, Header};

pub mod hot_reload;
pub use hot_reload::{VaultEvent, WidgetVaultWatcher};

#[derive(Debug)]
pub enum VaultError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Notify(notify::Error),
    NotFound(String),
    PathError,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Io(e) => write!(f, "IO error: {}", e),
            VaultError::Json(e) => write!(f, "JSON error: {}", e),
            VaultError::Notify(e) => write!(f, "Notify error: {}", e),
            VaultError::NotFound(id) => write!(f, "Widget not found: {}", id),
            VaultError::PathError => write!(f, "Failed to resolve standard config directory"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(err: std::io::Error) -> Self {
        VaultError::Io(err)
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(err: serde_json::Error) -> Self {
        VaultError::Json(err)
    }
}

impl From<notify::Error> for VaultError {
    fn from(err: notify::Error) -> Self {
        VaultError::Notify(err)
    }
}

/// Widget representation conforming to SWAL widget specifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Widget {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub payload: Value,
}

fn default_author() -> String {
    "SWAL Agent".to_string()
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Agent Widget Vault & Addon Inventory Manager
pub struct WidgetVault {
    vault_dir: PathBuf,
    widgets: Arc<RwLock<HashMap<String, Widget>>>,
    _watcher: Option<RecommendedWatcher>,
}

impl WidgetVault {
    /// Resolves standard XDG widget directory (~/.config/swal/widgets)
    pub fn default_vault_dir() -> Result<PathBuf, VaultError> {
        dirs::config_dir()
            .map(|p| p.join("swal").join("widgets"))
            .ok_or(VaultError::PathError)
    }

    /// Initializes WidgetVault with default XDG directory
    pub fn new() -> Result<Self, VaultError> {
        let dir = Self::default_vault_dir()?;
        Self::new_with_dir(dir)
    }

    /// Initializes WidgetVault with custom directory (creates directory if missing)
    pub fn new_with_dir(vault_dir: PathBuf) -> Result<Self, VaultError> {
        if !vault_dir.exists() {
            fs::create_dir_all(&vault_dir)?;
        }

        let widgets = Arc::new(RwLock::new(HashMap::new()));
        Self::scan_directory(&vault_dir, &widgets)?;

        let widgets_clone = Arc::clone(&widgets);
        let vault_dir_clone = vault_dir.clone();

        let watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                if !event.kind.is_access()
                    && event
                        .paths
                        .iter()
                        .any(|p| {
                            let s = p.to_string_lossy();
                            s.ends_with(".json") && !s.ends_with(".tmp")
                        })
                {
                    let _ = Self::scan_directory(&vault_dir_clone, &widgets_clone);
                }
            }
        });

        let watcher = match watcher {
            Ok(mut w) => {
                let _ = w.watch(&vault_dir, RecursiveMode::NonRecursive);
                Some(w)
            }
            Err(_) => None,
        };

        Ok(Self {
            vault_dir,
            widgets,
            _watcher: watcher,
        })
    }

    /// Helper to scan directory and update internal widget map safely
    pub fn scan_directory(
        vault_dir: &Path,
        widgets: &Arc<RwLock<HashMap<String, Widget>>>,
    ) -> Result<(), VaultError> {
        if !vault_dir.is_dir() {
            return Ok(());
        }

        let mut disk_widgets = HashMap::new();

        if let Ok(entries) = fs::read_dir(vault_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(widget) = serde_json::from_str::<Widget>(&content) {
                            disk_widgets.insert(widget.id.clone(), widget);
                        }
                    }
                }
            }
        }

        let mut lock = widgets.write().unwrap();
        for (id, widget) in disk_widgets {
            lock.insert(id, widget);
        }
        lock.retain(|id, _| {
            let json_path = vault_dir.join(format!("{}.json", id));
            json_path.exists()
        });

        Ok(())
    }

    pub fn get_vault_dir(&self) -> &PathBuf {
        &self.vault_dir
    }

    /// Returns an Arc clone of the underlying widgets storage map
    pub fn widgets_handle(&self) -> Arc<RwLock<HashMap<String, Widget>>> {
        Arc::clone(&self.widgets)
    }

    /// Spawns a WidgetVaultWatcher connected to this vault
    pub fn start_watcher(&self) -> Result<WidgetVaultWatcher, VaultError> {
        WidgetVaultWatcher::for_vault(self)
    }

    /// Returns a list of all installed widgets
    pub fn list_installed_widgets(&self) -> Vec<Widget> {
        let lock = self.widgets.read().unwrap();
        lock.values().cloned().collect()
    }

    /// Alias for list_installed_widgets
    pub fn list_widgets(&self) -> Vec<Widget> {
        self.list_installed_widgets()
    }

    /// Retrieves a widget by ID if present
    pub fn get_widget(&self, id: &str) -> Option<Widget> {
        let lock = self.widgets.read().unwrap();
        lock.get(id).cloned()
    }

    /// Saves or creates a widget on disk in $VAULT_DIR/{id}.json
    pub fn save_widget(&self, widget: &Widget) -> Result<(), VaultError> {
        let file_path = self.vault_dir.join(format!("{}.json", widget.id));
        let content = serde_json::to_string_pretty(widget)?;

        fs::write(&file_path, content)?;

        let mut lock = self.widgets.write().unwrap();
        lock.insert(widget.id.clone(), widget.clone());
        Ok(())
    }

    /// Pins a widget on desktop
    pub fn pin_widget(&self, id: &str) -> Result<(), VaultError> {
        let mut widget = self
            .get_widget(id)
            .ok_or_else(|| VaultError::NotFound(id.to_string()))?;
        widget.pinned = true;
        self.save_widget(&widget)
    }

    /// Unpins a widget from desktop
    pub fn unpin_widget(&self, id: &str) -> Result<(), VaultError> {
        let mut widget = self
            .get_widget(id)
            .ok_or_else(|| VaultError::NotFound(id.to_string()))?;
        widget.pinned = false;
        self.save_widget(&widget)
    }

    /// Exports selected widget IDs (or all installed widgets if `widget_ids` is empty) into a bundle (.tar.gz)
    pub fn export_bundle(
        &self,
        widget_ids: &[&str],
        output_path: &Path,
    ) -> Result<(), VaultError> {
        let targets: Vec<Widget> = if widget_ids.is_empty() {
            self.list_installed_widgets()
        } else {
            let mut list = Vec::new();
            for id in widget_ids {
                if let Some(w) = self.get_widget(id) {
                    list.push(w);
                } else {
                    return Err(VaultError::NotFound((*id).to_string()));
                }
            }
            list
        };

        let file = File::create(output_path)?;
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(enc);

        for widget in targets {
            let json_bytes = serde_json::to_vec_pretty(&widget)?;
            let mut header = Header::new_gnu();
            header.set_size(json_bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            let filename = format!("{}.json", widget.id);
            tar.append_data(&mut header, Path::new(&filename), &json_bytes[..])?;
        }

        tar.finish()?;
        Ok(())
    }

    /// Imports widgets from a bundle archive (.tar.gz / bundle) into the vault
    pub fn import_bundle(&self, bundle_path: &Path) -> Result<Vec<Widget>, VaultError> {
        let file = File::open(bundle_path)?;
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        let mut imported = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;

                if let Ok(widget) = serde_json::from_str::<Widget>(&content) {
                    self.save_widget(&widget)?;
                    imported.push(widget);
                }
            }
        }

        Ok(imported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_vault_crud_and_pinning() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault = WidgetVault::new_with_dir(dir.path().to_path_buf())?;

        assert_eq!(vault.list_installed_widgets().len(), 0);

        let widget = Widget {
            id: "system-monitor".to_string(),
            name: "System Monitor".to_string(),
            description: Some("Displays CPU/RAM usage".to_string()),
            author: "SWAL Agent".to_string(),
            version: "1.0.0".to_string(),
            pinned: false,
            payload: json!({ "refresh_ms": 1000 }),
        };

        vault.save_widget(&widget)?;
        let widgets = vault.list_installed_widgets();
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].id, "system-monitor");
        assert!(!widgets[0].pinned);

        // Pin widget
        vault.pin_widget("system-monitor")?;
        let updated = vault.get_widget("system-monitor").unwrap();
        assert!(updated.pinned);

        // Unpin widget
        vault.unpin_widget("system-monitor")?;
        let unpinned = vault.get_widget("system-monitor").unwrap();
        assert!(!unpinned.pinned);

        Ok(())
    }

    #[test]
    fn test_vault_export_import_bundle() -> Result<(), Box<dyn std::error::Error>> {
        let vault_dir1 = tempdir()?;
        let vault1 = WidgetVault::new_with_dir(vault_dir1.path().to_path_buf())?;

        let w1 = Widget {
            id: "clock-widget".to_string(),
            name: "Digital Clock".to_string(),
            description: None,
            author: "SWAL Agent".to_string(),
            version: "1.0.0".to_string(),
            pinned: true,
            payload: json!({ "format": "24h" }),
        };

        let w2 = Widget {
            id: "weather-widget".to_string(),
            name: "Weather Forecast".to_string(),
            description: Some("Local weather".to_string()),
            author: "SWAL Agent".to_string(),
            version: "1.1.0".to_string(),
            pinned: false,
            payload: json!({ "unit": "C" }),
        };

        vault1.save_widget(&w1)?;
        vault1.save_widget(&w2)?;

        let bundle_dir = tempdir()?;
        let bundle_file = bundle_dir.path().join("widgets_bundle.tar.gz");

        vault1.export_bundle(&["clock-widget", "weather-widget"], &bundle_file)?;
        assert!(bundle_file.exists());

        let vault_dir2 = tempdir()?;
        let vault2 = WidgetVault::new_with_dir(vault_dir2.path().to_path_buf())?;
        assert_eq!(vault2.list_installed_widgets().len(), 0);

        let imported = vault2.import_bundle(&bundle_file)?;
        assert_eq!(imported.len(), 2);
        assert_eq!(vault2.list_installed_widgets().len(), 2);

        let imported_clock = vault2.get_widget("clock-widget").unwrap();
        assert_eq!(imported_clock.name, "Digital Clock");
        assert!(imported_clock.pinned);

        Ok(())
    }
}
