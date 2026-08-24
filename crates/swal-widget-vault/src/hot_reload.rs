//! hot_reload.rs
//! File system watcher and dynamic hot-reload engine for SWAL Widget Vault.

use notify::{
    Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvError, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::{VaultError, Widget, WidgetVault};

/// Represents events emitted by the WidgetVaultWatcher when widget files change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultEvent {
    /// A new widget was added to the vault.
    WidgetCreated(String),
    /// An existing widget was updated or modified.
    WidgetUpdated(String),
    /// A widget file was removed from the vault.
    WidgetRemoved(String),
}

impl VaultEvent {
    /// Returns the ID of the affected widget.
    pub fn widget_id(&self) -> &str {
        match self {
            VaultEvent::WidgetCreated(id) => id.as_str(),
            VaultEvent::WidgetUpdated(id) => id.as_str(),
            VaultEvent::WidgetRemoved(id) => id.as_str(),
        }
    }

    /// Returns true if this is a creation event.
    pub fn is_created(&self) -> bool {
        matches!(self, VaultEvent::WidgetCreated(_))
    }

    /// Returns true if this is an update event.
    pub fn is_updated(&self) -> bool {
        matches!(self, VaultEvent::WidgetUpdated(_))
    }

    /// Returns true if this is a deletion event.
    pub fn is_removed(&self) -> bool {
        matches!(self, VaultEvent::WidgetRemoved(_))
    }

    /// Returns the string representation of the event type.
    pub fn event_type(&self) -> &'static str {
        match self {
            VaultEvent::WidgetCreated(_) => "created",
            VaultEvent::WidgetUpdated(_) => "updated",
            VaultEvent::WidgetRemoved(_) => "removed",
        }
    }
}

impl fmt::Display for VaultEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultEvent::WidgetCreated(id) => write!(f, "WidgetCreated({})", id),
            VaultEvent::WidgetUpdated(id) => write!(f, "WidgetUpdated({})", id),
            VaultEvent::WidgetRemoved(id) => write!(f, "WidgetRemoved({})", id),
        }
    }
}

/// Parses or validates raw file content into a valid `Widget` struct.
/// Supports both SWAL `Widget` JSON format and declarative A2UI `WidgetDefinition` format.
/// Malformed JSON or invalid schema specifications are discarded gracefully.
pub fn parse_or_validate_widget_file(path: &Path, content: &str) -> Option<Widget> {
    // 1. Try parsing directly as a SWAL `Widget` struct
    if let Ok(widget) = serde_json::from_str::<Widget>(content) {
        // If payload specifies an A2UI schema URI, validate against A2UI schema rules
        if let Some(schema_val) = widget.payload.get("schema") {
            if schema_val.is_string() {
                let payload_str = serde_json::to_string(&widget.payload).unwrap_or_default();
                if swal_a2ui_engine::validate_widget_json(&payload_str).is_err() {
                    // Invalid A2UI schema payload; reject gracefully
                    return None;
                }
            }
        }
        return Some(widget);
    }

    // 2. Try validating as a standalone A2UI `WidgetDefinition`
    if let Ok(def) = swal_a2ui_engine::validate_widget_json(content) {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("widget")
            .to_string();
        let payload: Value = serde_json::from_str(content).unwrap_or(Value::Null);

        return Some(Widget {
            id: file_stem,
            name: def.title,
            description: None,
            author: "SWAL Agent".to_string(),
            version: "1.0.0".to_string(),
            pinned: false,
            payload,
        });
    }

    None
}

/// Processes a single file path mutation and updates the in-memory widget cache.
/// Returns the resulting `VaultEvent` if a valid widget state change occurred.
pub fn process_file_path(
    path: &Path,
    _vault_dir: &Path,
    widgets: &Arc<RwLock<HashMap<String, Widget>>>,
) -> Option<VaultEvent> {
    // Filter only .json files
    if path.extension().and_then(|s| s.to_str()) != Some("json") {
        return None;
    }

    if path.exists() && path.is_file() {
        let content = fs::read_to_string(path).ok()?;
        let widget = parse_or_validate_widget_file(path, &content)?;

        let mut lock = widgets.write().unwrap();
        let existing = lock.get(&widget.id);

        if existing == Some(&widget) {
            // Already up-to-date with identical content; avoid redundant duplicate events
            return None;
        }

        let is_new = existing.is_none();
        let id = widget.id.clone();
        lock.insert(id.clone(), widget);

        if is_new {
            Some(VaultEvent::WidgetCreated(id))
        } else {
            Some(VaultEvent::WidgetUpdated(id))
        }
    } else {
        // File does not exist anymore (removed/deleted)
        let file_stem = path.file_stem().and_then(|s| s.to_str())?;
        let mut lock = widgets.write().unwrap();
        if lock.remove(file_stem).is_some() {
            Some(VaultEvent::WidgetRemoved(file_stem.to_string()))
        } else {
            None
        }
    }
}

/// Directory watcher and hot-reload engine for the SWAL Widget Vault.
pub struct WidgetVaultWatcher {
    vault_dir: PathBuf,
    widgets: Arc<RwLock<HashMap<String, Widget>>>,
    watcher: Option<RecommendedWatcher>,
    receiver: Receiver<VaultEvent>,
    sender: Sender<VaultEvent>,
}

impl WidgetVaultWatcher {
    /// Creates and starts watching a widget vault directory with a fresh in-memory widget map.
    pub fn new<P: AsRef<Path>>(vault_dir: P) -> Result<Self, VaultError> {
        let path = vault_dir.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        let widgets = Arc::new(RwLock::new(HashMap::new()));
        Self::with_shared_store(path, widgets)
    }

    /// Creates and starts watching using a shared in-memory widgets map.
    pub fn with_shared_store<P: AsRef<Path>>(
        vault_dir: P,
        widgets: Arc<RwLock<HashMap<String, Widget>>>,
    ) -> Result<Self, VaultError> {
        let path = vault_dir.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        // Perform initial directory scan and population
        Self::scan_and_populate(&path, &widgets);

        let (tx, rx) = channel();
        let tx_clone = tx.clone();
        let widgets_clone = Arc::clone(&widgets);
        let vault_dir_clone = path.clone();

        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                if event.kind.is_access() {
                    return;
                }
                for p in event.paths {
                    if let Some(vault_event) =
                        Self::handle_fs_change(&p, &vault_dir_clone, &widgets_clone)
                    {
                        let _ = tx_clone.send(vault_event);
                    }
                }
            }
        })?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        Ok(Self {
            vault_dir: path,
            widgets,
            watcher: Some(watcher),
            receiver: rx,
            sender: tx,
        })
    }

    /// Creates a watcher attached to an existing WidgetVault.
    pub fn for_vault(vault: &WidgetVault) -> Result<Self, VaultError> {
        Self::with_shared_store(vault.get_vault_dir().clone(), vault.widgets_handle())
    }

    fn scan_and_populate(
        vault_dir: &Path,
        widgets: &Arc<RwLock<HashMap<String, Widget>>>,
    ) {
        if let Ok(entries) = fs::read_dir(vault_dir) {
            let mut disk_widgets = HashMap::new();
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&p) {
                        if let Some(widget) = parse_or_validate_widget_file(&p, &content) {
                            disk_widgets.insert(widget.id.clone(), widget);
                        }
                    }
                }
            }
            let mut lock = widgets.write().unwrap();
            for (id, widget) in disk_widgets {
                lock.insert(id, widget);
            }
        }
    }

    fn handle_fs_change(
        path: &Path,
        vault_dir: &Path,
        widgets: &Arc<RwLock<HashMap<String, Widget>>>,
    ) -> Option<VaultEvent> {
        process_file_path(path, vault_dir, widgets)
    }

    /// Returns the watched directory path.
    pub fn vault_dir(&self) -> &Path {
        &self.vault_dir
    }

    /// Returns a cloned reference to the shared in-memory widgets map.
    pub fn widgets(&self) -> Arc<RwLock<HashMap<String, Widget>>> {
        Arc::clone(&self.widgets)
    }

    /// Returns all widgets currently held in memory.
    pub fn list_widgets(&self) -> Vec<Widget> {
        let lock = self.widgets.read().unwrap();
        lock.values().cloned().collect()
    }

    /// Retrieves a specific widget by ID.
    pub fn get_widget(&self, id: &str) -> Option<Widget> {
        let lock = self.widgets.read().unwrap();
        lock.get(id).cloned()
    }

    /// Returns true if the underlying notify watcher is active.
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Stops watching the directory.
    pub fn unwatch(&mut self) -> Result<(), VaultError> {
        if let Some(mut watcher) = self.watcher.take() {
            let _ = watcher.unwatch(&self.vault_dir);
        }
        Ok(())
    }

    /// Non-blocking check for the next vault event.
    pub fn try_recv(&self) -> Option<VaultEvent> {
        self.receiver.try_recv().ok()
    }

    /// Drains all available pending vault events without blocking.
    pub fn poll_events(&self) -> Vec<VaultEvent> {
        let mut events = Vec::new();
        while let Ok(evt) = self.receiver.try_recv() {
            events.push(evt);
        }
        events
    }

    /// Blocking wait for the next vault event up to the specified duration.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<VaultEvent, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    /// Blocking wait for the next vault event.
    pub fn recv(&self) -> Result<VaultEvent, RecvError> {
        self.receiver.recv()
    }

    /// Returns a reference to the event receiver channel.
    pub fn receiver(&self) -> &Receiver<VaultEvent> {
        &self.receiver
    }

    /// Returns a clone of the event sender channel.
    pub fn sender(&self) -> Sender<VaultEvent> {
        self.sender.clone()
    }

    /// Explicit manual reload triggering events for newly detected or modified files.
    pub fn reload_all(&self) -> Vec<VaultEvent> {
        let mut events = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.vault_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(evt) = Self::handle_fs_change(&p, &self.vault_dir, &self.widgets) {
                    let _ = self.sender.send(evt.clone());
                    events.push(evt);
                }
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_vault_event_helpers_and_serialization() {
        let created = VaultEvent::WidgetCreated("clock-1".to_string());
        let updated = VaultEvent::WidgetUpdated("clock-1".to_string());
        let removed = VaultEvent::WidgetRemoved("clock-1".to_string());

        assert_eq!(created.widget_id(), "clock-1");
        assert!(created.is_created());
        assert!(!created.is_updated());
        assert!(!created.is_removed());
        assert_eq!(created.event_type(), "created");
        assert_eq!(format!("{}", created), "WidgetCreated(clock-1)");

        assert_eq!(updated.widget_id(), "clock-1");
        assert!(updated.is_updated());
        assert_eq!(updated.event_type(), "updated");
        assert_eq!(format!("{}", updated), "WidgetUpdated(clock-1)");

        assert_eq!(removed.widget_id(), "clock-1");
        assert!(removed.is_removed());
        assert_eq!(removed.event_type(), "removed");
        assert_eq!(format!("{}", removed), "WidgetRemoved(clock-1)");

        // Serde check
        let json_str = serde_json::to_string(&updated).expect("serialize");
        let parsed: VaultEvent = serde_json::from_str(&json_str).expect("deserialize");
        assert_eq!(parsed, updated);
    }

    #[test]
    fn test_widget_vault_watcher_create_modify_delete() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let watcher = WidgetVaultWatcher::new(dir.path())?;

        assert!(watcher.is_watching());
        assert_eq!(watcher.list_widgets().len(), 0);

        // 1. Create a widget file
        let widget1_path = dir.path().join("sys_info.json");
        let w1 = Widget {
            id: "sys_info".to_string(),
            name: "System Info".to_string(),
            description: Some("CPU/RAM monitor".to_string()),
            author: "SWAL Agent".to_string(),
            version: "1.0.0".to_string(),
            pinned: false,
            payload: json!({ "interval_ms": 500 }),
        };
        fs::write(&widget1_path, serde_json::to_string_pretty(&w1)?)?;

        // Wait for created event
        let mut got_create = false;
        for _ in 0..30 {
            if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                if evt.widget_id() == "sys_info" && (evt.is_created() || evt.is_updated()) {
                    got_create = true;
                    break;
                }
            }
        }
        assert!(got_create, "Expected create or initial update event for sys_info");
        assert_eq!(watcher.get_widget("sys_info").unwrap().name, "System Info");

        // Drain any transient events before modification
        let _ = watcher.poll_events();

        // 2. Modify the widget file
        let mut w1_mod = w1.clone();
        w1_mod.name = "System Info Pro".to_string();
        w1_mod.pinned = true;
        fs::write(&widget1_path, serde_json::to_string_pretty(&w1_mod)?)?;

        // Wait for update event
        let mut got_update = false;
        for _ in 0..30 {
            if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                if evt.widget_id() == "sys_info" && evt.is_updated() {
                    got_update = true;
                    break;
                }
            }
        }
        assert!(got_update, "Expected update event for sys_info");
        let fetched = watcher.get_widget("sys_info").unwrap();
        assert_eq!(fetched.name, "System Info Pro");
        assert!(fetched.pinned);

        // Drain any transient events before deletion
        let _ = watcher.poll_events();

        // 3. Delete the widget file
        fs::remove_file(&widget1_path)?;

        let mut got_delete = false;
        for _ in 0..30 {
            if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                if evt.widget_id() == "sys_info" && evt.is_removed() {
                    got_delete = true;
                    break;
                }
            }
        }
        assert!(got_delete, "Expected remove event for sys_info");
        assert!(watcher.get_widget("sys_info").is_none());

        Ok(())
    }

    #[test]
    fn test_widget_vault_watcher_a2ui_schema_validation() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let watcher = WidgetVaultWatcher::new(dir.path())?;

        // Write a valid A2UI declarative widget
        let a2ui_path = dir.path().join("xavier_metrics.json");
        let a2ui_json = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "Xavier Cluster Metrics",
            "root": {
                "type": "Card",
                "children": [
                    { "type": "StatusBadge", "status": "healthy", "label": "Cluster Online" }
                ]
            }
        }"#;

        fs::write(&a2ui_path, a2ui_json)?;

        let mut got_event = false;
        for _ in 0..30 {
            if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                if evt.widget_id() == "xavier_metrics" {
                    got_event = true;
                    break;
                }
            }
        }
        assert!(got_event, "Expected event for xavier_metrics A2UI widget");
        let widget = watcher.get_widget("xavier_metrics").expect("Widget must exist");
        assert_eq!(widget.name, "Xavier Cluster Metrics");

        Ok(())
    }

    #[test]
    fn test_widget_vault_watcher_discard_malformed_and_non_json() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let watcher = WidgetVaultWatcher::new(dir.path())?;

        // 1. Non-JSON file (.txt)
        let txt_path = dir.path().join("notes.txt");
        fs::write(&txt_path, "Just some notes")?;

        // 2. Malformed JSON file
        let bad_json_path = dir.path().join("broken.json");
        fs::write(&bad_json_path, "{ broken json ...")?;

        // 3. Invalid A2UI JSON (empty title)
        let invalid_a2ui_path = dir.path().join("invalid_schema.json");
        let invalid_schema = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "",
            "root": { "type": "StatusBadge", "status": "ok", "label": "Test" }
        }"#;
        fs::write(&invalid_a2ui_path, invalid_schema)?;

        // Verify none of these invalid files were loaded into memory
        assert_eq!(watcher.list_widgets().len(), 0);
        assert!(watcher.get_widget("broken").is_none());
        assert!(watcher.get_widget("notes").is_none());
        assert!(watcher.get_widget("invalid_schema").is_none());

        Ok(())
    }

    #[test]
    fn test_widget_vault_watcher_shared_with_vault() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault = WidgetVault::new_with_dir(dir.path().to_path_buf())?;
        let watcher = WidgetVaultWatcher::for_vault(&vault)?;

        let w = Widget {
            id: "ambient_clock".to_string(),
            name: "Ambient Clock".to_string(),
            description: None,
            author: "SWAL Agent".to_string(),
            version: "1.0.0".to_string(),
            pinned: false,
            payload: json!({ "tz": "UTC" }),
        };

        vault.save_widget(&w)?;
        assert_eq!(watcher.get_widget("ambient_clock").unwrap().name, "Ambient Clock");

        // Pin via vault
        vault.pin_widget("ambient_clock")?;
        assert!(watcher.get_widget("ambient_clock").unwrap().pinned);

        Ok(())
    }
}
