//! Storage Drive & Disk Space Usage Visualizer Engine

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Information about a mounted drive/filesystem
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriveInfo {
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_percentage: f32,
    pub is_removable: bool,
}

impl DriveInfo {
    pub fn new(
        mount_point: String,
        filesystem: String,
        total_bytes: u64,
        available_bytes: u64,
        is_removable: bool,
    ) -> Self {
        let used_percentage = if total_bytes > 0 {
            let used = total_bytes.saturating_sub(available_bytes);
            ((used as f64 / total_bytes as f64) * 100.0) as f32
        } else {
            0.0
        };

        Self {
            mount_point,
            filesystem,
            total_bytes,
            available_bytes,
            used_percentage,
            is_removable,
        }
    }

    pub fn formatted_total(&self) -> String {
        format_bytes(self.total_bytes)
    }

    pub fn formatted_available(&self) -> String {
        format_bytes(self.available_bytes)
    }

    pub fn formatted_used(&self) -> String {
        let used = self.total_bytes.saturating_sub(self.available_bytes);
        format_bytes(used)
    }
}

/// Scanner engine for probing mounted drives and filesystem statistics
#[derive(Debug, Default)]
pub struct DiskUsageScanner;

impl DiskUsageScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_mounted_drives(&self) -> Vec<DriveInfo> {
        scan_mounted_drives()
    }
}

/// Scans mounted drives on Linux reading `/proc/mounts` and using POSIX `statvfs`
pub fn scan_mounted_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    let mut seen_mounts = HashSet::new();

    if let Ok(file) = File::open("/proc/mounts") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let device = parts[0];
            let mount_point = parts[1];
            let fstype = parts[2];

            if is_pseudo_filesystem(fstype, device, mount_point) {
                continue;
            }

            if !seen_mounts.insert(mount_point.to_string()) {
                continue;
            }

            if let Some((total_bytes, available_bytes)) = get_statvfs(mount_point) {
                let is_removable = is_removable_drive(mount_point, device);
                drives.push(DriveInfo::new(
                    mount_point.to_string(),
                    fstype.to_string(),
                    total_bytes,
                    available_bytes,
                    is_removable,
                ));
            }
        }
    }

    // Fallback if /proc/mounts was unavailable or yielded no valid mounts
    if drives.is_empty() {
        if let Some((total_bytes, available_bytes)) = get_statvfs("/") {
            drives.push(DriveInfo::new(
                "/".to_string(),
                "ext4".to_string(),
                total_bytes,
                available_bytes,
                false,
            ));
        }
    }

    drives
}

fn is_pseudo_filesystem(fstype: &str, device: &str, mount_point: &str) -> bool {
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

fn is_removable_drive(mount_point: &str, device: &str) -> bool {
    if mount_point.starts_with("/media")
        || mount_point.starts_with("/run/media")
        || mount_point.starts_with("/mnt")
    {
        return true;
    }

    if device.contains("usb") || device.contains("mmc") {
        return true;
    }

    false
}

fn get_statvfs<P: AsRef<Path>>(path: P) -> Option<(u64, u64)> {
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

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_info_formatting() {
        let total = 500 * 1024 * 1024 * 1024; // 500 GB
        let avail = 200 * 1024 * 1024 * 1024; // 200 GB
        let info = DriveInfo::new(
            "/".to_string(),
            "ext4".to_string(),
            total,
            avail,
            false,
        );

        assert_eq!(info.formatted_total(), "500.0 GB");
        assert_eq!(info.formatted_available(), "200.0 GB");
        assert_eq!(info.formatted_used(), "300.0 GB");
        assert!((info.used_percentage - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_scan_mounted_drives() {
        let drives = scan_mounted_drives();
        assert!(!drives.is_empty(), "Should scan at least one drive (e.g. root /)");
        let root = drives.iter().find(|d| d.mount_point == "/");
        assert!(root.is_some(), "Root filesystem should be detected");
    }

    #[test]
    fn test_disk_usage_scanner_struct() {
        let scanner = DiskUsageScanner::new();
        let drives = scanner.scan_mounted_drives();
        assert!(!drives.is_empty());
    }
}
