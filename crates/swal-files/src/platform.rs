//! Cross-Platform OS & Filesystem Abstraction Layer for SWAL Files
//! Handles OS detection, system folder resolution, drive listing, path normalization,
//! default application launching, and trash operations.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported Operating System Platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsPlatform {
    Linux,
    Windows,
    MacOS,
    Unknown,
}

impl OsPlatform {
    /// Detects the current target operating system at compile time or runtime
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            OsPlatform::Linux
        } else if cfg!(target_os = "windows") {
            OsPlatform::Windows
        } else if cfg!(target_os = "macos") {
            OsPlatform::MacOS
        } else {
            OsPlatform::Unknown
        }
    }
}

/// Known System Special Folders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemFolder {
    Home,
    Documents,
    Downloads,
    Pictures,
    Music,
    Videos,
    Desktop,
    Config,
}

/// Information about a mounted drive or disk partition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriveInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub drive_type: String,
    pub is_removable: bool,
}

impl DriveInfo {
    pub fn new(
        mount_point: impl Into<String>,
        total_bytes: u64,
        available_bytes: u64,
        drive_type: impl Into<String>,
        is_removable: bool,
    ) -> Self {
        Self {
            mount_point: mount_point.into(),
            total_bytes,
            available_bytes,
            drive_type: drive_type.into(),
            is_removable,
        }
    }
}

/// Unified Cross-Platform Operating System and Filesystem Abstraction
pub struct PlatformAbstraction;

impl PlatformAbstraction {
    /// Detects the active operating system platform
    pub fn detect_os() -> OsPlatform {
        OsPlatform::current()
    }

    /// Resolves the absolute path to a standard system folder
    pub fn get_system_folder(folder: SystemFolder) -> Option<PathBuf> {
        match folder {
            SystemFolder::Home => dirs::home_dir(),
            SystemFolder::Documents => dirs::document_dir(),
            SystemFolder::Downloads => dirs::download_dir(),
            SystemFolder::Pictures => dirs::picture_dir(),
            SystemFolder::Music => dirs::audio_dir(),
            SystemFolder::Videos => dirs::video_dir(),
            SystemFolder::Desktop => dirs::desktop_dir(),
            SystemFolder::Config => dirs::config_dir(),
        }
    }

    /// Scans and lists active system drives and mount points
    pub fn list_system_drives() -> Vec<DriveInfo> {
        let mut drives = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/proc/mounts") {
                let mut seen = std::collections::HashSet::new();
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 3 {
                        continue;
                    }
                    let device = parts[0];
                    let mount_point = parts[1];
                    let fstype = parts[2];

                    if is_linux_pseudo_fs(fstype, device, mount_point) {
                        continue;
                    }
                    if !seen.insert(mount_point.to_string()) {
                        continue;
                    }

                    if let Some((total, avail)) = get_posix_statvfs(mount_point) {
                        let is_removable = mount_point.starts_with("/media")
                            || mount_point.starts_with("/run/media")
                            || mount_point.starts_with("/mnt")
                            || device.contains("usb");
                        drives.push(DriveInfo::new(
                            mount_point,
                            total,
                            avail,
                            fstype,
                            is_removable,
                        ));
                    }
                }
            }
        }

        // Fallback or generic drive resolution
        if drives.is_empty() {
            if let Some(home) = dirs::home_dir() {
                if let Some((total, avail)) = get_posix_statvfs(&home) {
                    drives.push(DriveInfo::new(
                        home.to_string_lossy().to_string(),
                        total,
                        avail,
                        "ext4",
                        false,
                    ));
                }
            }
            if drives.is_empty() {
                if let Some((total, avail)) = get_posix_statvfs("/") {
                    drives.push(DriveInfo::new("/", total, avail, "rootfs", false));
                } else {
                    // Windows default drive fallback or mock
                    drives.push(DriveInfo::new(
                        "C:\\",
                        512_000_000_000,
                        256_000_000_000,
                        "NTFS",
                        false,
                    ));
                }
            }
        }

        drives
    }

    /// Normalizes path representations across Windows (drive letters C:\, UNC \\server\share, backslashes)
    /// and Unix root (`/`), collapsing `.` and `..` components.
    pub fn normalize_path(path: &Path) -> PathBuf {
        let s = path.to_string_lossy();
        if s.is_empty() {
            return PathBuf::from(".");
        }

        // Check for UNC path prefix (e.g. \\server\share or //server/share)
        let is_unc = s.starts_with("\\\\") || s.starts_with("//");

        // Convert backslashes to forward slashes for unified parsing
        let normalized_slashes = s.replace('\\', "/");

        let (prefix, rest) = if is_unc {
            let parts: Vec<&str> = normalized_slashes
                .split('/')
                .filter(|p| !p.is_empty())
                .collect();
            if parts.len() >= 2 {
                let unc_prefix = format!("\\\\{}\\{}", parts[0], parts[1]);
                let rest_str = parts[2..].join("/");
                (Some(unc_prefix), rest_str)
            } else {
                (Some(format!("\\\\{}", parts.join("\\"))), String::new())
            }
        } else if normalized_slashes.len() >= 2
            && normalized_slashes.as_bytes()[1] == b':'
            && (normalized_slashes.as_bytes()[0] as char).is_ascii_alphabetic()
        {
            let drive = (normalized_slashes.as_bytes()[0] as char).to_ascii_uppercase();
            let rest_str = &normalized_slashes[2..];
            (Some(format!("{}:", drive)), rest_str.to_string())
        } else if normalized_slashes.starts_with('/') {
            (Some("/".to_string()), normalized_slashes[1..].to_string())
        } else {
            (None, normalized_slashes.to_string())
        };

        let components: Vec<&str> = rest
            .split('/')
            .filter(|c| !c.is_empty() && *c != ".")
            .collect();

        let mut stack: Vec<&str> = Vec::new();
        for comp in components {
            if comp == ".." {
                if !stack.is_empty() {
                    stack.pop();
                } else if prefix.is_none() {
                    stack.push("..");
                }
            } else {
                stack.push(comp);
            }
        }

        match prefix {
            Some(p) => {
                if p == "/" {
                    if stack.is_empty() {
                        PathBuf::from("/")
                    } else {
                        PathBuf::from(format!("/{}", stack.join("/")))
                    }
                } else if p.starts_with("\\\\") {
                    if stack.is_empty() {
                        PathBuf::from(p)
                    } else {
                        PathBuf::from(format!("{}\\{}", p, stack.join("\\")))
                    }
                } else if p.ends_with(':') {
                    if stack.is_empty() {
                        PathBuf::from(format!("{}\\", p))
                    } else {
                        PathBuf::from(format!("{}\\{}", p, stack.join("\\")))
                    }
                } else {
                    PathBuf::from(format!("{}/{}", p, stack.join("/")))
                }
            }
            None => {
                if stack.is_empty() {
                    PathBuf::from(".")
                } else {
                    PathBuf::from(stack.join("/"))
                }
            }
        }
    }

    /// Dispatches opening a file or URI to the default OS desktop application
    pub fn open_with_default_app(path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Path not found: {}", path.display()),
            ));
        }

        let os = Self::detect_os();
        let status = match os {
            OsPlatform::Linux => Command::new("xdg-open").arg(path).status(),
            OsPlatform::Windows => Command::new("cmd")
                .args(["/C", "start", "", path.to_str().unwrap_or_default()])
                .status(),
            OsPlatform::MacOS => Command::new("open").arg(path).status(),
            OsPlatform::Unknown => Command::new("xdg-open").arg(path).status(),
        };

        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(Error::new(
                ErrorKind::Other,
                format!("Default app launcher exited with status: {}", s),
            )),
            Err(e) => Err(e),
        }
    }

    /// Moves a file or directory to the OS system trash location
    pub fn move_to_trash(path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Target path does not exist: {}", path.display()),
            ));
        }

        let trash_dir = resolve_trash_directory()?;
        fs::create_dir_all(&trash_dir)?;

        let file_name = path
            .file_name()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Invalid target filename"))?;

        let mut dest_path = trash_dir.join(file_name);
        if dest_path.exists() {
            let timestamp = chrono::Utc::now().timestamp_millis();
            let mut name_str = file_name.to_string_lossy().to_string();
            name_str.push_str(&format!("_{}", timestamp));
            dest_path = trash_dir.join(name_str);
        }

        // Try standard rename, fallback to copy + remove for cross-filesystem moves
        if fs::rename(path, &dest_path).is_err() {
            if path.is_dir() {
                copy_dir_all(path, &dest_path)?;
                fs::remove_dir_all(path)?;
            } else {
                fs::copy(path, &dest_path)?;
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }
}

fn resolve_trash_directory() -> io::Result<PathBuf> {
    let os = OsPlatform::current();
    let home = dirs::home_dir().ok_or_else(|| {
        Error::new(ErrorKind::NotFound, "Could not resolve user home directory")
    })?;

    match os {
        OsPlatform::Linux => Ok(home.join(".local/share/Trash/files")),
        OsPlatform::MacOS => Ok(home.join(".Trash")),
        OsPlatform::Windows => Ok(dirs::data_dir()
            .unwrap_or_else(|| home.clone())
            .join("SWAL/Trash")),
        OsPlatform::Unknown => Ok(home.join(".local/share/Trash/files")),
    }
}

#[cfg(target_os = "linux")]
fn is_linux_pseudo_fs(fstype: &str, device: &str, mount_point: &str) -> bool {
    if mount_point == "/" {
        return false;
    }
    const PSEUDO_TYPES: &[&str] = &[
        "proc", "sysfs", "devtmpfs", "devpts", "tmpfs", "cgroup", "cgroup2",
        "pstore", "bpf", "autofs", "securityfs", "configfs", "tracefs",
        "hugetlbfs", "mqueue", "ramfs", "nsfs", "rpc_pipefs", "binfmt_misc",
    ];
    if PSEUDO_TYPES.contains(&fstype) {
        return true;
    }
    if mount_point.starts_with("/proc")
        || mount_point.starts_with("/sys")
        || mount_point.starts_with("/dev")
    {
        return true;
    }
    if device == "none" || device.starts_with("systemd") {
        return true;
    }
    false
}

fn get_posix_statvfs<P: AsRef<Path>>(path: P) -> Option<(u64, u64)> {
    use std::ffi::CString;
    let c_path = CString::new(path.as_ref().to_str()?).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let res = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if res == 0 {
        let block_size = stat.f_frsize as u64;
        let total = stat.f_blocks as u64 * block_size;
        let available = stat.f_bavail as u64 * block_size;
        Some((total, available))
    } else {
        None
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_os_detection() {
        let os = PlatformAbstraction::detect_os();
        assert_eq!(os, OsPlatform::current());
        #[cfg(target_os = "linux")]
        assert_eq!(os, OsPlatform::Linux);
        #[cfg(target_os = "windows")]
        assert_eq!(os, OsPlatform::Windows);
        #[cfg(target_os = "macos")]
        assert_eq!(os, OsPlatform::MacOS);
    }

    #[test]
    fn test_system_folders() {
        let home = PlatformAbstraction::get_system_folder(SystemFolder::Home);
        assert!(home.is_some(), "Home directory should be resolvable");

        let docs = PlatformAbstraction::get_system_folder(SystemFolder::Documents);
        let downloads = PlatformAbstraction::get_system_folder(SystemFolder::Downloads);
        let pictures = PlatformAbstraction::get_system_folder(SystemFolder::Pictures);
        let music = PlatformAbstraction::get_system_folder(SystemFolder::Music);
        let videos = PlatformAbstraction::get_system_folder(SystemFolder::Videos);
        let desktop = PlatformAbstraction::get_system_folder(SystemFolder::Desktop);
        let config = PlatformAbstraction::get_system_folder(SystemFolder::Config);

        // Print debug info for resolved folders
        println!("Resolved folders: home={:?}, docs={:?}, downloads={:?}, pictures={:?}, music={:?}, videos={:?}, desktop={:?}, config={:?}",
            home, docs, downloads, pictures, music, videos, desktop, config);
    }

    #[test]
    fn test_list_system_drives() {
        let drives = PlatformAbstraction::list_system_drives();
        assert!(!drives.is_empty(), "Should return at least one system drive");
        let first = &drives[0];
        assert!(!first.mount_point.is_empty());
        assert!(!first.drive_type.is_empty());
    }

    #[test]
    fn test_drive_info_new() {
        let info = DriveInfo::new("/mnt/usb", 1000, 500, "vfat", true);
        assert_eq!(info.mount_point, "/mnt/usb");
        assert_eq!(info.total_bytes, 1000);
        assert_eq!(info.available_bytes, 500);
        assert_eq!(info.drive_type, "vfat");
        assert!(info.is_removable);
    }

    #[test]
    fn test_normalize_path_unix() {
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("/usr/local/../bin/foo")),
            PathBuf::from("/usr/bin/foo")
        );
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("/")),
            PathBuf::from("/")
        );
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("")),
            PathBuf::from(".")
        );
    }

    #[test]
    fn test_normalize_path_windows() {
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("C:\\Users\\Admin\\..\\Bela\\Documents")),
            PathBuf::from("C:\\Users\\Bela\\Documents")
        );
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("c:/users/bela/./downloads")),
            PathBuf::from("C:\\users\\bela\\downloads")
        );
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("D:\\")),
            PathBuf::from("D:\\")
        );
    }

    #[test]
    fn test_normalize_path_unc() {
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("\\\\server\\share\\folder\\..\\file.txt")),
            PathBuf::from("\\\\server\\share\\file.txt")
        );
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("//nas/data/docs")),
            PathBuf::from("\\\\nas\\data\\docs")
        );
    }

    #[test]
    fn test_normalize_path_relative() {
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("foo/bar/../baz")),
            PathBuf::from("foo/baz")
        );
        assert_eq!(
            PlatformAbstraction::normalize_path(Path::new("a/./b/../../c")),
            PathBuf::from("c")
        );
    }

    #[test]
    fn test_open_with_default_app_non_existent() {
        let invalid_path = Path::new("/non_existent_swal_path_12345.xyz");
        let result = PlatformAbstraction::open_with_default_app(invalid_path);
        assert!(result.is_err(), "Opening non-existent path should return Error");
        assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn test_trash_operations_lifecycle() {
        let dir = tempdir().expect("Failed to create tempdir");
        let test_file = dir.path().join("trash_test_file.txt");
        fs::write(&test_file, "SWAL trash test data").expect("Failed to write test file");

        assert!(test_file.exists());
        let result = PlatformAbstraction::move_to_trash(&test_file);
        assert!(result.is_ok(), "move_to_trash should succeed for valid file: {:?}", result);
        assert!(!test_file.exists(), "Original file should no longer exist after moving to trash");

        // Non-existent trash attempt
        let ghost_file = dir.path().join("ghost.txt");
        let ghost_res = PlatformAbstraction::move_to_trash(&ghost_file);
        assert!(ghost_res.is_err());
    }

    #[test]
    fn test_trash_directory_move_lifecycle() {
        let temp = tempdir().expect("Failed to create tempdir");
        let sub_dir = temp.path().join("trash_folder");
        fs::create_dir_all(&sub_dir).expect("Failed to create sub_dir");
        let inner_file = sub_dir.join("inside.txt");
        fs::write(&inner_file, "inside content").expect("Failed to write inner file");

        assert!(sub_dir.exists());
        let res = PlatformAbstraction::move_to_trash(&sub_dir);
        assert!(res.is_ok(), "Moving directory to trash should succeed");
        assert!(!sub_dir.exists(), "Original directory should be moved");
    }
}
