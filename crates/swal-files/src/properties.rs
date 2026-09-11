//! Item Properties extraction module for SWAL Files
//! Provides full metadata inspection (stat in JSON) for files, directories, and symlinks.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemProperties {
    pub path: String,
    pub name: String,
    pub kind: String,        // "file" | "dir" | "symlink"
    pub size_bytes: u64,
    pub size_human: String,
    pub size_on_disk: u64,
    pub items_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    pub mode_octal: String,   // "0644"
    pub owner: String,
    pub group: String,
    pub modified: String,
    pub accessed: String,
    pub created: String,
    pub mime: String,
    pub symlink_target: Option<String>,
    pub inode: u64,
    pub is_readonly: bool,
    pub is_executable: bool,
}

pub fn compute_properties(path: &Path) -> io::Result<ItemProperties> {
    let meta = fs::symlink_metadata(path)?;

    let is_symlink = meta.file_type().is_symlink();
    let is_dir = meta.is_dir();

    let kind = if is_symlink {
        "symlink".to_string()
    } else if is_dir {
        "dir".to_string()
    } else {
        "file".to_string()
    };

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    let abs_path = fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    let symlink_target = if is_symlink {
        fs::read_link(path)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };

    let mut items_count: Option<u64> = None;
    let mut truncated: Option<bool> = None;
    let size_bytes: u64;
    let size_on_disk: u64;

    if kind == "dir" {
        let mut count: u64 = 0;
        let mut dir_bytes: u64 = 0;
        let mut dir_disk: u64 = 0;
        let mut is_truncated = false;

        for entry_res in walkdir::WalkDir::new(path).follow_links(false).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let entry_path = entry.path();
            if entry_path == path {
                if let Ok(m) = entry.metadata() {
                    dir_bytes += m.len();
                    #[cfg(unix)]
                    {
                        let b = m.blocks() * 512;
                        dir_disk += if b > 0 { b } else { m.len() };
                    }
                    #[cfg(not(unix))]
                    {
                        dir_disk += m.len();
                    }
                }
                continue;
            }

            count += 1;

            if let Ok(m) = entry.metadata() {
                dir_bytes += m.len();
                #[cfg(unix)]
                {
                    let b = m.blocks() * 512;
                    dir_disk += if b > 0 { b } else { m.len() };
                }
                #[cfg(not(unix))]
                {
                    dir_disk += m.len();
                }
            }

            if count >= 50_000 {
                is_truncated = true;
                break;
            }
        }

        items_count = Some(count);
        if is_truncated {
            truncated = Some(true);
        }
        size_bytes = dir_bytes;
        size_on_disk = dir_disk;
    } else {
        size_bytes = meta.len();
        #[cfg(unix)]
        {
            let b = meta.blocks() * 512;
            size_on_disk = if b > 0 { b } else { size_bytes };
        }
        #[cfg(not(unix))]
        {
            size_on_disk = size_bytes;
        }
    }

    let size_human = format_bytes_human(size_bytes);

    let (inode, mode_octal, uid, gid, is_executable) = extract_unix_metadata(&meta);
    let owner = resolve_owner(uid);
    let group = resolve_group(gid);

    let modified = format_system_time(meta.modified());
    let accessed = format_system_time(meta.accessed());
    let created = format_system_time(meta.created());

    let mime = resolve_mime(path, is_dir, is_symlink);

    let is_readonly = meta.permissions().readonly();

    Ok(ItemProperties {
        path: abs_path,
        name,
        kind,
        size_bytes,
        size_human,
        size_on_disk,
        items_count,
        truncated,
        mode_octal,
        owner,
        group,
        modified,
        accessed,
        created,
        mime,
        symlink_target,
        inode,
        is_readonly,
        is_executable,
    })
}

pub fn handle_cli(args: &[String], session: &mut crate::session::SessionState) -> Option<String> {
    let target_str = if args.len() > 2 {
        args[2].clone()
    } else if let Some(sel) = &session.selected_path {
        sel.clone()
    } else {
        session
            .tabs
            .iter()
            .find(|t| t.id == session.active_tab_id)
            .map(|t| t.path.clone())
            .unwrap_or_else(|| ".".to_string())
    };

    let path = Path::new(&target_str);
    match compute_properties(path) {
        Ok(props) => serde_json::to_string(&props).ok(),
        Err(_) => None,
    }
}

pub fn format_bytes_human(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn extract_unix_metadata(meta: &fs::Metadata) -> (u64, String, u32, u32, bool) {
    #[cfg(unix)]
    {
        let inode = meta.ino();
        let mode = meta.mode() & 0o7777;
        let mode_octal = format!("{:04o}", mode);
        let uid = meta.uid();
        let gid = meta.gid();
        let is_exec = (meta.mode() & 0o111) != 0;
        (inode, mode_octal, uid, gid, is_exec)
    }
    #[cfg(not(unix))]
    {
        (0, "0644".to_string(), 1000, 1000, false)
    }
}

fn resolve_owner(uid: u32) -> String {
    #[cfg(unix)]
    unsafe {
        let pwd = libc::getpwuid(uid);
        if !pwd.is_null() && !(*pwd).pw_name.is_null() {
            if let Ok(name) = std::ffi::CStr::from_ptr((*pwd).pw_name).to_str() {
                return name.to_string();
            }
        }
    }
    uid.to_string()
}

fn resolve_group(gid: u32) -> String {
    #[cfg(unix)]
    unsafe {
        let grp = libc::getgrgid(gid);
        if !grp.is_null() && !(*grp).gr_name.is_null() {
            if let Ok(name) = std::ffi::CStr::from_ptr((*grp).gr_name).to_str() {
                return name.to_string();
            }
        }
    }
    gid.to_string()
}

fn format_system_time(time_res: io::Result<SystemTime>) -> String {
    match time_res {
        Ok(sys_time) => {
            let dt: chrono::DateTime<chrono::Local> = sys_time.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        Err(_) => "--".to_string(),
    }
}

fn resolve_mime(path: &Path, is_dir: bool, is_symlink: bool) -> String {
    if is_dir {
        return "inode/directory".to_string();
    }
    if is_symlink {
        return "inode/symlink".to_string();
    }
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "txt" | "log" | "hostname" => "text/plain".to_string(),
        "md" => "text/markdown".to_string(),
        "html" | "htm" => "text/html".to_string(),
        "css" => "text/css".to_string(),
        "js" => "application/javascript".to_string(),
        "json" => "application/json".to_string(),
        "toml" => "application/toml".to_string(),
        "yaml" | "yml" => "application/yaml".to_string(),
        "rs" => "text/x-rust".to_string(),
        "py" => "text/x-python".to_string(),
        "sh" | "bash" => "application/x-sh".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "webp" => "image/webp".to_string(),
        "pdf" => "application/pdf".to_string(),
        "zip" => "application/zip".to_string(),
        "gz" | "tgz" => "application/gzip".to_string(),
        "tar" => "application/x-tar".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        "mp4" => "video/mp4".to_string(),
        "" => {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if file_name == "hostname" || file_name == "hosts" || file_name == "passwd" {
                "text/plain".to_string()
            } else {
                "text/plain".to_string()
            }
        }
        _ => format!("application/x-{}", ext),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_file_properties() {
        let dir = tempdir().expect("Failed to create tempdir");
        let file_path = dir.path().join("test_file.txt");
        let content = b"Hello SWAL Properties!";

        {
            let mut f = File::create(&file_path).expect("Failed to create file");
            f.write_all(content).expect("Failed to write file");
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o644);
            fs::set_permissions(&file_path, perms).expect("Failed to set perms");
        }

        let props = compute_properties(&file_path).expect("compute_properties failed");
        assert_eq!(props.kind, "file");
        assert_eq!(props.name, "test_file.txt");
        assert_eq!(props.size_bytes, content.len() as u64);
        assert!(props.items_count.is_none());
        assert_eq!(props.mode_octal, "0644");
        assert_eq!(props.mime, "text/plain");
        assert!(props.symlink_target.is_none());
        assert!(!props.owner.is_empty());
        assert!(!props.group.is_empty());
    }

    #[test]
    fn test_directory_properties() {
        let dir = tempdir().expect("Failed to create tempdir");
        let sub_dir = dir.path().join("sub_folder");
        fs::create_dir(&sub_dir).expect("Failed to create sub_folder");

        let file1 = dir.path().join("file1.rs");
        let file2 = dir.path().join("file2.json");
        let file3 = sub_dir.join("file3.txt");

        fs::write(&file1, "fn main() {}").expect("write file1");
        fs::write(&file2, "{}").expect("write file2");
        fs::write(&file3, "data").expect("write file3");

        let props = compute_properties(dir.path()).expect("compute_properties failed for dir");
        assert_eq!(props.kind, "dir");
        assert_eq!(props.items_count, Some(4)); // sub_dir, file1, file2, file3
        assert!(props.size_bytes > 0);
        assert!(props.truncated.is_none());
    }

    #[test]
    fn test_symlink_properties() {
        let dir = tempdir().expect("Failed to create tempdir");
        let target_file = dir.path().join("target.txt");
        fs::write(&target_file, "target content").expect("write target");

        let symlink_file = dir.path().join("link.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_file, &symlink_file).expect("create symlink");

        #[cfg(unix)]
        {
            let props = compute_properties(&symlink_file).expect("compute_properties failed for symlink");
            assert_eq!(props.kind, "symlink");
            assert!(props.symlink_target.is_some());
            assert_eq!(
                props.symlink_target.unwrap(),
                target_file.to_string_lossy().to_string()
            );
            assert!(props.items_count.is_none());
        }
    }
}
