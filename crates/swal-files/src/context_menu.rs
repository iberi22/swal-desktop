//! Context menu action registry and JSON builder for SWAL Files

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::session::{load_session, SessionState};

/// Context menu action item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextAction {
    pub id: String,        // "copy-path", "cut", "paste", ...
    pub label: String,     // "Copiar ruta"
    pub icon: String,      // emoji o nerd-font
    pub enabled: bool,
    pub shortcut: String,  // documental: "Ctrl+Shift+C"
}

/// Checks whether a valid clipboard manifest exists in temp or user config directory
pub fn has_clipboard_manifest() -> bool {
    let tmp_manifest = Path::new("/tmp/swal_clipboard.json");
    if tmp_manifest.exists() {
        if let Ok(meta) = fs::metadata(tmp_manifest) {
            if meta.len() > 0 {
                return true;
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let cfg_manifest = home.join(".config/swal/files/clipboard.json");
        if cfg_manifest.exists() {
            if let Ok(meta) = fs::metadata(&cfg_manifest) {
                if meta.len() > 0 {
                    return true;
                }
            }
        }
    }
    false
}

/// Builds the contextual action menu for a given target path and session state.
/// Contractual order of actions:
/// open, open-with, copy-path, copy-name, copy-content,
/// cut, copy, paste, duplicate, rename, new-folder, trash, properties, reveal-terminal.
pub fn build_context_menu(target: Option<&Path>, session: &SessionState) -> Vec<ContextAction> {
    let resolved_target: Option<PathBuf> = target
        .map(|p| p.to_path_buf())
        .or_else(|| session.selected_path.as_ref().map(PathBuf::from))
        .or_else(|| {
            session
                .tabs
                .iter()
                .find(|t| t.id == session.active_tab_id)
                .map(|t| PathBuf::from(&t.path))
        });

    let target_exists = resolved_target.as_ref().map(|p| p.exists()).unwrap_or(false);
    let is_file = resolved_target.as_ref().map(|p| p.is_file()).unwrap_or(false);
    let is_dir = resolved_target.as_ref().map(|p| p.is_dir()).unwrap_or(false);
    let clipboard_manifest = has_clipboard_manifest();

    vec![
        ContextAction {
            id: "open".to_string(),
            label: "Abrir".to_string(),
            icon: "󰏌".to_string(),
            enabled: target_exists,
            shortcut: "Enter".to_string(),
        },
        ContextAction {
            id: "open-with".to_string(),
            label: "Abrir con...".to_string(),
            icon: "󰅍".to_string(),
            enabled: target_exists && is_file,
            shortcut: "Ctrl+O".to_string(),
        },
        ContextAction {
            id: "copy-path".to_string(),
            label: "Copiar ruta".to_string(),
            icon: "󰆏".to_string(),
            enabled: target_exists,
            shortcut: "Ctrl+Shift+C".to_string(),
        },
        ContextAction {
            id: "copy-name".to_string(),
            label: "Copiar nombre".to_string(),
            icon: "󰈔".to_string(),
            enabled: target_exists,
            shortcut: "Ctrl+Alt+C".to_string(),
        },
        ContextAction {
            id: "copy-content".to_string(),
            label: "Copiar contenido".to_string(),
            icon: "󰈙".to_string(),
            enabled: target_exists && is_file,
            shortcut: "Ctrl+C".to_string(),
        },
        ContextAction {
            id: "cut".to_string(),
            label: "Cortar".to_string(),
            icon: "󰆐".to_string(),
            enabled: target_exists,
            shortcut: "Ctrl+X".to_string(),
        },
        ContextAction {
            id: "copy".to_string(),
            label: "Copiar".to_string(),
            icon: "󰆏".to_string(),
            enabled: target_exists,
            shortcut: "Ctrl+C".to_string(),
        },
        ContextAction {
            id: "paste".to_string(),
            label: "Pegar".to_string(),
            icon: "󰆒".to_string(),
            enabled: clipboard_manifest && (is_dir || target_exists),
            shortcut: "Ctrl+V".to_string(),
        },
        ContextAction {
            id: "duplicate".to_string(),
            label: "Duplicar".to_string(),
            icon: "󰆑".to_string(),
            enabled: target_exists,
            shortcut: "Ctrl+D".to_string(),
        },
        ContextAction {
            id: "rename".to_string(),
            label: "Renombrar".to_string(),
            icon: "󰑕".to_string(),
            enabled: target_exists,
            shortcut: "F2".to_string(),
        },
        ContextAction {
            id: "new-folder".to_string(),
            label: "Nueva carpeta".to_string(),
            icon: "󰉋".to_string(),
            enabled: is_dir || target_exists,
            shortcut: "Ctrl+Shift+N".to_string(),
        },
        ContextAction {
            id: "trash".to_string(),
            label: "Mover a la papelera".to_string(),
            icon: "󰩹".to_string(),
            enabled: target_exists,
            shortcut: "Delete".to_string(),
        },
        ContextAction {
            id: "properties".to_string(),
            label: "Propiedades".to_string(),
            icon: "󰋼".to_string(),
            enabled: target_exists,
            shortcut: "Alt+Enter".to_string(),
        },
        ContextAction {
            id: "reveal-terminal".to_string(),
            label: "Abrir en terminal".to_string(),
            icon: "󰞷".to_string(),
            enabled: target_exists,
            shortcut: "Ctrl+Alt+T".to_string(),
        },
    ]
}

/// Serializes the context menu actions for a given target path to JSON string
pub fn menu_json(target: Option<&Path>) -> String {
    let session = load_session();
    let actions = build_context_menu(target, &session);
    serde_json::to_string_pretty(&actions).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::collections::HashSet;

    #[test]
    fn test_context_menu_directory_vs_file() {
        let dir = tempdir().expect("Failed to create tempdir");
        let sub_dir = dir.path().join("test_folder");
        fs::create_dir_all(&sub_dir).expect("Failed to create sub_dir");
        let test_file = sub_dir.join("test_file.txt");
        fs::write(&test_file, "Hello SWAL").expect("Failed to write test file");

        let session = SessionState::default();

        let menu_dir = build_context_menu(Some(&sub_dir), &session);
        let menu_file = build_context_menu(Some(&test_file), &session);

        let copy_content_dir = menu_dir.iter().find(|a| a.id == "copy-content").unwrap();
        let copy_content_file = menu_file.iter().find(|a| a.id == "copy-content").unwrap();

        assert!(!copy_content_dir.enabled, "copy-content should be disabled for directories");
        assert!(copy_content_file.enabled, "copy-content should be enabled for files");

        let open_with_dir = menu_dir.iter().find(|a| a.id == "open-with").unwrap();
        let open_with_file = menu_file.iter().find(|a| a.id == "open-with").unwrap();

        assert!(!open_with_dir.enabled, "open-with should be disabled for directories");
        assert!(open_with_file.enabled, "open-with should be enabled for files");
    }

    #[test]
    fn test_context_menu_paste_with_and_without_manifest() {
        let dir = tempdir().expect("Failed to create tempdir");
        let temp_manifest = Path::new("/tmp/swal_clipboard.json");
        let _ = fs::remove_file(temp_manifest);

        let session = SessionState::default();
        let menu_no_manifest = build_context_menu(Some(dir.path()), &session);
        let paste_no_manifest = menu_no_manifest.iter().find(|a| a.id == "paste").unwrap();
        assert!(!paste_no_manifest.enabled, "paste should be disabled when no clipboard manifest exists");

        // Create temporary clipboard manifest
        fs::write(temp_manifest, r#"{"items":["/tmp/dummy.txt"]}"#).expect("Failed to write manifest");

        let menu_with_manifest = build_context_menu(Some(dir.path()), &session);
        let paste_with_manifest = menu_with_manifest.iter().find(|a| a.id == "paste").unwrap();
        assert!(paste_with_manifest.enabled, "paste should be enabled when clipboard manifest exists");

        // Clean up
        let _ = fs::remove_file(temp_manifest);
    }

    #[test]
    fn test_context_menu_unique_ids_and_non_empty_labels() {
        let session = SessionState::default();
        let actions = build_context_menu(None, &session);

        assert_eq!(actions.len(), 14, "Context menu must have exactly 14 actions");

        let mut ids = HashSet::new();
        for action in &actions {
            assert!(!action.id.is_empty(), "Action ID must not be empty");
            assert!(!action.label.is_empty(), "Action label must not be empty");
            assert!(!action.icon.is_empty(), "Action icon must not be empty");
            assert!(!action.shortcut.is_empty(), "Action shortcut must not be empty");
            assert!(ids.insert(&action.id), "Duplicate action ID found: {}", action.id);
        }

        let expected_order = vec![
            "open", "open-with", "copy-path", "copy-name", "copy-content",
            "cut", "copy", "paste", "duplicate", "rename", "new-folder",
            "trash", "properties", "reveal-terminal",
        ];
        let actual_order: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(actual_order, expected_order, "Context menu actions must follow contractual order");
    }
}
