//! Partition Storage Scanner and statvfs Disk Metrics for SWAL Telemetry

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem::MaybeUninit;
use std::path::Path;

/// Information about a mounted disk partition and its storage utilization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_pct: f32,
}

/// Known pseudo/virtual filesystems that should not be reported as physical storage.
const PSEUDO_FS_TYPES: &[&str] = &[
    "proc",
    "sysfs",
    "devpts",
    "devtmpfs",
    "securityfs",
    "cgroup",
    "cgroup2",
    "pstore",
    "bpf",
    "autofs",
    "debugfs",
    "tracefs",
    "fusectl",
    "hugetlbfs",
    "mqueue",
    "ramfs",
    "binfmt_misc",
    "efivarfs",
    "configfs",
    "nsfs",
    "rpc_pipefs",
    "fuse.portal",
    "portal",
    "overlay",
    "squashfs",
];

/// Unescapes octal sequences found in `/proc/mounts` (e.g. `\040` for space, `\011` for tab).
pub fn unescape_octal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            let mut octal = String::new();
            for _ in 0..3 {
                if let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() && next_c <= '7' {
                        octal.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            if octal.len() == 3 {
                if let Ok(byte) = u8::from_str_radix(&octal, 8) {
                    out.push(byte as char);
                    continue;
                }
            }
            out.push('\\');
            out.push_str(&octal);
        } else {
            out.push(c);
        }
    }

    out
}

/// Checks if a filesystem type and mount point should be considered a real storage partition.
pub fn is_real_partition(spec: &str, mount_point: &str, fs_type: &str) -> bool {
    // Check if filesystem is in pseudo list
    if PSEUDO_FS_TYPES.contains(&fs_type) || (fs_type.starts_with("fuse.") && fs_type != "fuseblk") {
        return false;
    }

    // Ignore loop devices (e.g. snap mounts)
    if spec.starts_with("/dev/loop") {
        return false;
    }

    // Filter tmpfs unless it is mounted on root "/" (e.g. NixOS / impermanence setups)
    if fs_type == "tmpfs" {
        return mount_point == "/";
    }

    // Filter paths under pseudo dirs
    if mount_point.starts_with("/proc")
        || mount_point.starts_with("/sys")
        || mount_point.starts_with("/dev")
    {
        return false;
    }

    // Real device, network fs, or supported storage fs
    spec.starts_with("/dev/")
        || mount_point == "/"
        || matches!(
            fs_type,
            "btrfs"
                | "zfs"
                | "xfs"
                | "ext4"
                | "ext3"
                | "ext2"
                | "f2fs"
                | "vfat"
                | "fat"
                | "ntfs"
                | "ntfs3"
                | "exfat"
                | "nfs"
                | "nfs4"
                | "cifs"
                | "smbfs"
                | "bcachefs"
                | "fuseblk"
        )
}

/// Queries filesystem metrics for a given mount point using `libc::statvfs`.
pub fn get_disk_info(mount_point: &str) -> Option<DiskInfo> {
    let c_path = CString::new(mount_point).ok()?;
    unsafe {
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let stat = stat.assume_init();
            let fragment_size = if stat.f_frsize > 0 {
                stat.f_frsize as u64
            } else {
                stat.f_bsize as u64
            };

            if fragment_size == 0 || stat.f_blocks == 0 {
                return None;
            }

            let total_bytes = (stat.f_blocks as u64).saturating_mul(fragment_size);
            let free_bytes = (stat.f_bavail as u64).saturating_mul(fragment_size);
            let used_bytes = total_bytes.saturating_sub(free_bytes);

            let used_pct = if total_bytes > 0 {
                ((used_bytes as f64 / total_bytes as f64) * 100.0) as f32
            } else {
                0.0
            };

            Some(DiskInfo {
                mount_point: mount_point.to_string(),
                total_bytes,
                free_bytes,
                used_pct: used_pct.clamp(0.0, 100.0),
            })
        } else {
            None
        }
    }
}

/// Parses mount entries from any reader implementing `BufRead`.
pub fn parse_mounts<R: BufRead>(reader: R) -> Vec<DiskInfo> {
    let mut disks = Vec::new();
    let mut seen_mounts = HashSet::new();

    for line in reader.lines().flatten() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let spec = parts[0];
            let mount_point_raw = parts[1];
            let fs_type = parts[2];

            let mount_point = unescape_octal(mount_point_raw);

            if !seen_mounts.contains(&mount_point) && is_real_partition(spec, &mount_point, fs_type) {
                if let Some(info) = get_disk_info(&mount_point) {
                    seen_mounts.insert(mount_point);
                    disks.push(info);
                }
            }
        }
    }

    disks
}

/// Scans mounted partitions by parsing `/proc/mounts` and querying `statvfs`.
pub fn scan_mounted_partitions() -> Vec<DiskInfo> {
    scan_mounted_partitions_from_path("/proc/mounts")
}

/// Scans mounted partitions from a custom mounts file path.
pub fn scan_mounted_partitions_from_path<P: AsRef<Path>>(path: P) -> Vec<DiskInfo> {
    if let Ok(file) = File::open(path) {
        parse_mounts(BufReader::new(file))
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_unescape_octal() {
        assert_eq!(unescape_octal("/home/user/My\\040Documents"), "/home/user/My Documents");
        assert_eq!(unescape_octal("/plain/path"), "/plain/path");
        assert_eq!(unescape_octal("/tab\\011here"), "/tab\there");
    }

    #[test]
    fn test_is_real_partition_filtering() {
        // Virtual filesystems
        assert!(!is_real_partition("proc", "/proc", "proc"));
        assert!(!is_real_partition("sysfs", "/sys", "sysfs"));
        assert!(!is_real_partition("devpts", "/dev/pts", "devpts"));
        assert!(!is_real_partition("cgroup2", "/sys/fs/cgroup", "cgroup2"));
        assert!(!is_real_partition("portal", "/run/user/1000/doc", "fuse.portal"));
        assert!(!is_real_partition("/dev/loop0", "/snap/core/123", "squashfs"));

        // tmpfs
        assert!(is_real_partition("tmpfs", "/", "tmpfs")); // root on tmpfs allowed
        assert!(!is_real_partition("tmpfs", "/run", "tmpfs"));
        assert!(!is_real_partition("tmpfs", "/dev/shm", "tmpfs"));

        // Real partitions
        assert!(is_real_partition("/dev/nvme0n1p2", "/", "ext4"));
        assert!(is_real_partition("/dev/sda1", "/home", "btrfs"));
        assert!(is_real_partition("/dev/nvme1n1p5", "/mnt/ssd-2tb", "ntfs3"));
        assert!(is_real_partition("/dev/disk/by-uuid/xyz", "/nix", "ext4"));
        assert!(is_real_partition("rpool/ROOT/nixos", "/", "zfs"));
    }

    #[test]
    fn test_get_disk_info_root() {
        let root_info = get_disk_info("/");
        assert!(root_info.is_some());
        let info = root_info.unwrap();
        assert_eq!(info.mount_point, "/");
        assert!(info.total_bytes > 0);
        assert!(info.used_pct >= 0.0 && info.used_pct <= 100.0);
    }

    #[test]
    fn test_scan_mounted_partitions_live() {
        let disks = scan_mounted_partitions();
        assert!(!disks.is_empty(), "Expected at least one mounted partition");
        for disk in &disks {
            assert!(disk.total_bytes > 0);
            assert!(disk.used_pct >= 0.0 && disk.used_pct <= 100.0);
        }
    }

    #[test]
    fn test_parse_mounts_mock_data() {
        let mock_mounts = r#"
tmpfs / tmpfs rw,nosuid,nodev,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
/dev/nvme0n1p1 /boot vfat rw,nosuid,nodev,relatime 0 0
/dev/loop0 /snap/core/1 squashfs ro 0 0
tmpfs /run tmpfs rw,nosuid,nodev 0 0
"#;
        let reader = Cursor::new(mock_mounts);
        let disks = parse_mounts(reader);
        // Should parse "/" and "/boot" (if /boot exists and is queryable)
        assert!(!disks.is_empty());
        assert!(disks.iter().any(|d| d.mount_point == "/"));
        assert!(!disks.iter().any(|d| d.mount_point == "/proc"));
        assert!(!disks.iter().any(|d| d.mount_point == "/sys"));
        assert!(!disks.iter().any(|d| d.mount_point == "/run"));
    }

    #[test]
    fn test_disk_info_serde() {
        let disk = DiskInfo {
            mount_point: "/mnt/storage".to_string(),
            total_bytes: 1_000_000_000,
            free_bytes: 400_000_000,
            used_pct: 60.0,
        };
        let json = serde_json::to_string(&disk).unwrap();
        let decoded: DiskInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(disk, decoded);
    }
}
