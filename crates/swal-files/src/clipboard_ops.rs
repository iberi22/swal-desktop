//! Persistent JSON Clipboard Operations for SWAL Files
//! Manages file copy, cut, paste, and manifest serialization.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::session::SessionState;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClipboardManifest {
    pub op: String, // "copy" | "cut"
    pub paths: Vec<String>,
    pub ts: i64,
}

pub fn manifest_path() -> PathBuf {
    // Override por env: permite aislar tests (y correr dos instancias sin pisarse).
    if let Ok(p) = std::env::var("SWAL_FILES_CLIPBOARD_PATH") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    crate::config::FileManagerConfig::config_path()
        .parent()
        .map(|d| d.join("clipboard.json"))
        .unwrap_or_else(|| PathBuf::from("/tmp/swal_clipboard.json"))
}

pub fn load_manifest() -> Option<ClipboardManifest> {
    let path = manifest_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn set_clipboard(paths: &[PathBuf], cut: bool) -> io::Result<ClipboardManifest> {
    let mpath = manifest_path();
    if let Some(parent) = mpath.parent() {
        fs::create_dir_all(parent)?;
    }

    let abs_paths: Vec<String> = paths
        .iter()
        .map(|p| {
            if let Ok(canon) = fs::canonicalize(p) {
                canon.to_string_lossy().to_string()
            } else {
                p.to_string_lossy().to_string()
            }
        })
        .collect();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let manifest = ClipboardManifest {
        op: if cut { "cut".to_string() } else { "copy".to_string() },
        paths: abs_paths,
        ts,
    };

    let json_str = serde_json::to_string_pretty(&manifest)?;
    fs::write(&mpath, json_str)?;

    Ok(manifest)
}

pub fn clear_clipboard() -> io::Result<()> {
    let path = manifest_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn unique_destination(dir: &Path, name: &str) -> PathBuf {
    let target = dir.join(name);
    if !target.exists() {
        return target;
    }

    let path_obj = Path::new(name);
    let stem = path_obj
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let ext = path_obj
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 2;
    loop {
        let new_name = format!("{} (copia {}){}", stem, counter, ext);
        let candidate = dir.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// Helper function to copy file or directory recursively without following directory symlinks.
fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(src)?;
    let file_type = meta.file_type();

    if file_type.is_symlink() {
        // Read link target and recreate symlink if possible, or skip/copy target
        #[cfg(unix)]
        {
            let link_target = fs::read_link(src)?;
            let _ = std::os::unix::fs::symlink(link_target, dst);
            return Ok(());
        }
        #[cfg(not(unix))]
        {
            fs::copy(src, dst)?;
            return Ok(());
        }
    }

    if file_type.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = dst.join(entry.file_name());
            copy_recursive(&entry_path, &dest_path)?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }

    Ok(())
}

/// Helper function to move file or directory with fallback for cross-device moves.
fn move_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    // Attempt fast rename
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    // Fallback: copy recursively and then remove source
    copy_recursive(src, dst)?;
    let meta = fs::symlink_metadata(src)?;
    if meta.is_dir() {
        fs::remove_dir_all(src)?;
    } else {
        fs::remove_file(src)?;
    }

    Ok(())
}

pub fn paste_into(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let manifest = load_manifest().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "No clipboard manifest found")
    })?;

    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let mut pasted = Vec::new();
    let is_cut = manifest.op == "cut";
    let mut all_succeeded = true;

    for src_str in &manifest.paths {
        let src = PathBuf::from(src_str);
        if !src.exists() {
            all_succeeded = false;
            eprintln!("⚠ Clipboard source does not exist: {}", src_str);
            continue;
        }

        let file_name = match src.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                all_succeeded = false;
                continue;
            }
        };

        let dest = unique_destination(dir, &file_name);

        let res = if is_cut {
            move_recursive(&src, &dest)
        } else {
            copy_recursive(&src, &dest)
        };

        match res {
            Ok(_) => pasted.push(dest),
            Err(e) => {
                all_succeeded = false;
                eprintln!("⚠ Failed to process {}: {}", src_str, e);
            }
        }
    }

    // Clear manifest ONLY if it was a cut operation and all items moved successfully
    if is_cut && all_succeeded {
        let _ = clear_clipboard();
    }

    Ok(pasted)
}

pub fn handle_cli(args: &[String], session: &mut SessionState) -> Option<String> {
    if args.len() < 2 {
        return None;
    }

    let cmd = args[1].as_str();
    match cmd {
        "clip-copy" | "clip_copy" => {
            let paths: Vec<PathBuf> = if args.len() > 2 {
                args[2..].iter().map(PathBuf::from).collect()
            } else if let Some(selected) = &session.selected_path {
                vec![PathBuf::from(selected)]
            } else {
                Vec::new()
            };

            if paths.is_empty() {
                return Some(serde_json::json!({ "error": "No paths specified" }).to_string());
            }

            match set_clipboard(&paths, false) {
                Ok(manifest) => Some(serde_json::to_string(&manifest).unwrap_or_default()),
                Err(e) => Some(serde_json::json!({ "error": e.to_string() }).to_string()),
            }
        }
        "clip-cut" | "clip_cut" => {
            let paths: Vec<PathBuf> = if args.len() > 2 {
                args[2..].iter().map(PathBuf::from).collect()
            } else if let Some(selected) = &session.selected_path {
                vec![PathBuf::from(selected)]
            } else {
                Vec::new()
            };

            if paths.is_empty() {
                return Some(serde_json::json!({ "error": "No paths specified" }).to_string());
            }

            match set_clipboard(&paths, true) {
                Ok(manifest) => Some(serde_json::to_string(&manifest).unwrap_or_default()),
                Err(e) => Some(serde_json::json!({ "error": e.to_string() }).to_string()),
            }
        }
        "clip-paste" | "clip_paste" => {
            let target_dir = if args.len() > 2 {
                PathBuf::from(&args[2])
            } else {
                let active_path = session
                    .tabs
                    .iter()
                    .find(|t| t.id == session.active_tab_id)
                    .map(|t| t.path.clone())
                    .unwrap_or_else(|| {
                        dirs::home_dir()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    });
                PathBuf::from(active_path)
            };

            match paste_into(&target_dir) {
                Ok(pasted) => {
                    let pasted_strs: Vec<String> = pasted
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    Some(serde_json::json!({ "ok": pasted_strs, "failed": [] }).to_string())
                }
                Err(e) => Some(serde_json::json!({ "ok": [], "failed": [e.to_string()] }).to_string()),
            }
        }
        "clip-clear" | "clip_clear" => {
            match clear_clipboard() {
                Ok(_) => Some(serde_json::json!({ "status": "cleared" }).to_string()),
                Err(e) => Some(serde_json::json!({ "error": e.to_string() }).to_string()),
            }
        }
        "clip-status" | "clip_status" => {
            if let Some(manifest) = load_manifest() {
                Some(serde_json::to_string(&manifest).unwrap_or_default())
            } else {
                Some(serde_json::json!({ "status": "empty" }).to_string())
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test() -> std::sync::MutexGuard<'static, ()> {
        let guard = TEST_MUTEX.lock().unwrap();
        let _ = clear_clipboard();
        guard
    }

    #[test]
    fn test_file_copy() {
        let _guard = setup_test();
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("source.txt");
        fs::write(&src_file, "hello world").unwrap();

        set_clipboard(&[src_file.clone()], false).unwrap();
        let manifest = load_manifest().unwrap();
        assert_eq!(manifest.op, "copy");
        assert_eq!(manifest.paths.len(), 1);

        let dst_dir = dir.path().join("destination");
        let pasted = paste_into(&dst_dir).unwrap();

        assert_eq!(pasted.len(), 1);
        assert!(dst_dir.join("source.txt").exists());
        assert_eq!(fs::read_to_string(dst_dir.join("source.txt")).unwrap(), "hello world");
        assert!(src_file.exists()); // Source still exists after copy
    }

    #[test]
    fn test_folder_copy_recursive() {
        let _guard = setup_test();
        let dir = tempdir().unwrap();
        let src_folder = dir.path().join("sub_folder");
        fs::create_dir_all(src_folder.join("nested")).unwrap();
        fs::write(src_folder.join("file1.txt"), "content 1").unwrap();
        fs::write(src_folder.join("nested").join("file2.txt"), "content 2").unwrap();

        set_clipboard(&[src_folder.clone()], false).unwrap();

        let dst_dir = dir.path().join("target");
        let pasted = paste_into(&dst_dir).unwrap();

        assert_eq!(pasted.len(), 1);
        let pasted_folder = dst_dir.join("sub_folder");
        assert!(pasted_folder.join("file1.txt").exists());
        assert!(pasted_folder.join("nested").join("file2.txt").exists());
        assert_eq!(fs::read_to_string(pasted_folder.join("nested").join("file2.txt")).unwrap(), "content 2");
    }

    #[test]
    fn test_name_collision() {
        let _guard = setup_test();
        let dir = tempdir().unwrap();
        let target_dir = dir.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();

        fs::write(target_dir.join("data.txt"), "existing").unwrap();

        let unique1 = unique_destination(&target_dir, "data.txt");
        assert_eq!(unique1, target_dir.join("data (copia 2).txt"));

        fs::write(&unique1, "copia 2 content").unwrap();

        let unique2 = unique_destination(&target_dir, "data.txt");
        assert_eq!(unique2, target_dir.join("data (copia 3).txt"));
    }

    #[test]
    fn test_cut_moves_and_cleans_manifest() {
        let _guard = setup_test();
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("cut_me.txt");
        fs::write(&src_file, "cut payload").unwrap();

        set_clipboard(&[src_file.clone()], true).unwrap();
        let manifest = load_manifest().unwrap();
        assert_eq!(manifest.op, "cut");

        let dst_dir = dir.path().join("dst");
        let pasted = paste_into(&dst_dir).unwrap();

        assert_eq!(pasted.len(), 1);
        assert!(dst_dir.join("cut_me.txt").exists());
        assert!(!src_file.exists()); // Source moved
        assert!(load_manifest().is_none()); // Manifest cleared
    }
}
