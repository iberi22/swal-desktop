//! Trash & Permanent Delete Operations with Guard for SWAL Files
//! Handles sending files to trash, permanent deletion with confirmation token,
//! system path protection blacklists, and CLI dispatching.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::platform::PlatformAbstraction;
use crate::session::SessionState;

/// Report summarizing the outcome of a trash or permanent delete batch operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TrashReport {
    pub ok: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub items_count: usize,
}

/// Status summary of the system trash location.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TrashStatus {
    pub total_items: usize,
    pub total_bytes: u64,
    pub items: Vec<String>,
}

/// Checks whether a given path is protected by the system safety guard.
/// Safety rule: Never allow trashing or permanent deletion of `/`, `$HOME`, or core system directories (`/etc`, `/usr`, `/nix`, etc.).
pub fn is_blacklisted_path(path: &Path) -> bool {
    let normalized = PlatformAbstraction::normalize_path(path);

    let mut protected_paths: Vec<PathBuf> = vec![
        PathBuf::from("/"),
        PathBuf::from("/etc"),
        PathBuf::from("/usr"),
        PathBuf::from("/nix"),
        PathBuf::from("/boot"),
        PathBuf::from("/dev"),
        PathBuf::from("/proc"),
        PathBuf::from("/sys"),
        PathBuf::from("/run"),
        PathBuf::from("/var"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/lib"),
        PathBuf::from("/lib64"),
    ];

    if let Some(home) = dirs::home_dir() {
        protected_paths.push(home.clone());
        protected_paths.push(PlatformAbstraction::normalize_path(&home));
        if let Ok(canon_home) = home.canonicalize() {
            protected_paths.push(canon_home);
        }
    }

    if protected_paths.contains(&normalized) {
        return true;
    }

    if let Ok(canon) = path.canonicalize() {
        if protected_paths.contains(&canon) {
            return true;
        }
    }

    false
}

/// Sends multiple items to the system trash directory using `PlatformAbstraction::move_to_trash`.
pub fn trash_items(paths: &[PathBuf]) -> TrashReport {
    let mut report = TrashReport::default();

    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        if is_blacklisted_path(path) {
            report.failed.push((
                path_str,
                "Path is protected by system safety guard".to_string(),
            ));
            continue;
        }

        match PlatformAbstraction::move_to_trash(path) {
            Ok(()) => {
                report.ok.push(path_str);
            }
            Err(e) => {
                report.failed.push((path_str, e.to_string()));
            }
        }
    }

    report.items_count = report.ok.len();
    report
}

/// Permanently deletes multiple items from disk.
/// Exiges the literal confirmation token `"BORRAR"`. Without this token, no files are modified.
/// Guarantees that symlinks are unlinked directly without following target paths.
pub fn delete_permanently(paths: &[PathBuf], confirm_token: &str) -> TrashReport {
    let mut report = TrashReport::default();

    if confirm_token != "BORRAR" {
        for path in paths {
            let path_str = path.to_string_lossy().to_string();
            report.failed.push((
                path_str,
                "Error: Missing or invalid confirm token (expected --confirm=BORRAR)".to_string(),
            ));
        }
        return report;
    }

    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        if is_blacklisted_path(path) {
            report.failed.push((
                path_str,
                "Error: Path is protected by system safety guard".to_string(),
            ));
            continue;
        }

        let meta_res = fs::symlink_metadata(path);
        match meta_res {
            Ok(meta) => {
                // If it's a symlink or file, use remove_file (never follow symlinks)
                let res = if meta.file_type().is_symlink() || meta.is_file() {
                    fs::remove_file(path)
                } else if meta.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };

                match res {
                    Ok(()) => {
                        report.ok.push(path_str);
                    }
                    Err(e) => {
                        report.failed.push((path_str, e.to_string()));
                    }
                }
            }
            Err(e) => {
                report.failed.push((path_str, e.to_string()));
            }
        }
    }

    report.items_count = report.ok.len();
    report
}

/// Queries and calculates current statistics for system trash location.
pub fn get_trash_status() -> TrashStatus {
    let mut status = TrashStatus::default();
    let home = dirs::home_dir();
    let trash_dir = if cfg!(target_os = "linux") {
        home.map(|h| h.join(".local/share/Trash/files"))
    } else if cfg!(target_os = "macos") {
        home.map(|h| h.join(".Trash"))
    } else {
        dirs::data_dir().or(home).map(|p| p.join("SWAL/Trash"))
    };

    if let Some(td) = trash_dir {
        if td.exists() && td.is_dir() {
            if let Ok(entries) = fs::read_dir(&td) {
                for entry in entries.flatten() {
                    status.total_items += 1;
                    if let Ok(meta) = entry.metadata() {
                        status.total_bytes += meta.len();
                    }
                    status.items.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }

    status
}

/// CLI command dispatcher for trash and delete operations.
pub fn handle_cli(args: &[String], _session: &mut SessionState) -> Option<String> {
    if args.len() < 2 {
        return Some("Error: missing trash sub-command".to_string());
    }

    let cmd = args[1].as_str();

    match cmd {
        "trash-item" | "trash_item" => {
            if args.len() < 3 {
                return Some("Error: trash-item requires at least one path argument".to_string());
            }
            let paths: Vec<PathBuf> = args[2..].iter().map(PathBuf::from).collect();
            let report = trash_items(&paths);
            if report.ok.is_empty() && !report.failed.is_empty() {
                let err_msg = report.failed.iter().map(|(p, e)| format!("{}: {}", p, e)).collect::<Vec<_>>().join("; ");
                return Some(format!("Error: Failed to move items to trash: {}", err_msg));
            }
            serde_json::to_string_pretty(&report).ok()
        }
        "delete-item" | "delete_item" => {
            let mut paths = Vec::new();
            let mut confirm_token = String::new();

            let mut i = 2;
            while i < args.len() {
                let arg = &args[i];
                if arg.starts_with("--confirm=") {
                    confirm_token = arg.trim_start_matches("--confirm=").to_string();
                } else if arg == "--confirm" {
                    if i + 1 < args.len() {
                        confirm_token = args[i + 1].clone();
                        i += 1;
                    }
                } else if !arg.starts_with('-') {
                    paths.push(PathBuf::from(arg));
                }
                i += 1;
            }

            if paths.is_empty() {
                return Some("Error: delete-item requires at least one path argument".to_string());
            }

            if confirm_token != "BORRAR" {
                return Some(format!(
                    "Error: Permanent deletion requires --confirm=BORRAR token. Given: '{}'",
                    confirm_token
                ));
            }

            let report = delete_permanently(&paths, &confirm_token);
            if report.ok.is_empty() && !report.failed.is_empty() {
                let err_msg = report.failed.iter().map(|(p, e)| format!("{}: {}", p, e)).collect::<Vec<_>>().join("; ");
                return Some(format!("Error: Delete permanently failed: {}", err_msg));
            }

            serde_json::to_string_pretty(&report).ok()
        }
        "trash-status" | "trash_status" => {
            let status = get_trash_status();
            serde_json::to_string_pretty(&status).ok()
        }
        _ => None, // no es nuestro: que siga la cadena de despacho
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_trash_single_file() {
        let dir = tempdir().expect("tempdir creation failed");
        let file_path = dir.path().join("file_to_trash.txt");
        fs::write(&file_path, "hello trash").expect("write file failed");

        assert!(file_path.exists());
        let report = trash_items(&[file_path.clone()]);

        assert_eq!(report.items_count, 1);
        assert_eq!(report.ok.len(), 1);
        assert!(report.failed.is_empty());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_trash_directory() {
        let temp = tempdir().expect("tempdir creation failed");
        let sub_dir = temp.path().join("folder_to_trash");
        fs::create_dir_all(&sub_dir).expect("create_dir_all failed");
        let inner = sub_dir.join("inner.txt");
        fs::write(&inner, "inner content").expect("write inner failed");

        assert!(sub_dir.exists());
        let report = trash_items(&[sub_dir.clone()]);

        assert_eq!(report.items_count, 1);
        assert_eq!(report.ok.len(), 1);
        assert!(report.failed.is_empty());
        assert!(!sub_dir.exists());
    }

    #[test]
    fn test_delete_permanently_without_confirm_token() {
        let temp = tempdir().expect("tempdir creation failed");
        let file_path = temp.path().join("permanent_file.txt");
        fs::write(&file_path, "do not delete me without token").expect("write failed");

        let report_no = delete_permanently(&[file_path.clone()], "NO");
        assert_eq!(report_no.items_count, 0);
        assert_eq!(report_no.ok.len(), 0);
        assert_eq!(report_no.failed.len(), 1);
        assert!(file_path.exists());

        let report_invalid = delete_permanently(&[file_path.clone()], "CONFIRM");
        assert_eq!(report_invalid.items_count, 0);
        assert!(file_path.exists());
    }

    #[test]
    fn test_blacklisted_paths_rejected() {
        let root = PathBuf::from("/");
        let etc = PathBuf::from("/etc");
        let usr = PathBuf::from("/usr");
        let nix = PathBuf::from("/nix");

        assert!(is_blacklisted_path(&root));
        assert!(is_blacklisted_path(&etc));
        assert!(is_blacklisted_path(&usr));
        assert!(is_blacklisted_path(&nix));

        if let Some(home) = dirs::home_dir() {
            assert!(is_blacklisted_path(&home));
        }

        let trash_rep = trash_items(&[root.clone(), etc.clone()]);
        assert_eq!(trash_rep.items_count, 0);
        assert_eq!(trash_rep.failed.len(), 2);

        let del_rep = delete_permanently(&[usr.clone(), nix.clone()], "BORRAR");
        assert_eq!(del_rep.items_count, 0);
        assert_eq!(del_rep.failed.len(), 2);
    }

    #[test]
    fn test_trash_report_partial_failures() {
        let temp = tempdir().expect("tempdir creation failed");
        let existing = temp.path().join("existing.txt");
        fs::write(&existing, "valid").expect("write failed");

        let ghost = temp.path().join("non_existent_ghost.txt");

        let report = trash_items(&[existing.clone(), ghost.clone()]);

        assert_eq!(report.items_count, 1);
        assert_eq!(report.ok.len(), 1);
        assert_eq!(report.failed.len(), 1);
        assert!(!existing.exists());
    }

    #[test]
    fn test_delete_symlink_does_not_follow() {
        let temp = tempdir().expect("tempdir creation failed");
        let target_dir = temp.path().join("real_target_dir");
        fs::create_dir_all(&target_dir).expect("create dir failed");
        let target_file = target_dir.join("important.txt");
        fs::write(&target_file, "critical data").expect("write file failed");

        let symlink_path = temp.path().join("link_to_target");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_dir, &symlink_path).expect("symlink creation failed");

        #[cfg(unix)]
        {
            assert!(symlink_path.exists());
            let report = delete_permanently(&[symlink_path.clone()], "BORRAR");
            assert_eq!(report.items_count, 1);
            assert!(!symlink_path.exists(), "Symlink itself should be deleted");
            assert!(target_dir.exists(), "Target directory must NOT be deleted");
            assert!(target_file.exists(), "File inside target directory must NOT be deleted");
        }
    }
}
