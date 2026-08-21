//! High-performance async directory scanner & filter

use crate::entry::{FileEntry, GitStatus};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiskUsage {
    pub mount_point: PathBuf,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_pct: f32,
}

pub fn scan_disk_usage(path: &Path) -> Result<DiskUsage, std::io::Error> {
    match rustix::fs::statvfs(path) {
        Ok(stat) => {
            let block_size = if stat.f_frsize > 0 { stat.f_frsize } else { stat.f_bsize };
            let total_bytes = stat.f_blocks.saturating_mul(block_size);
            let free_bytes = stat.f_bfree.saturating_mul(block_size);
            let available_bytes = stat.f_bavail.saturating_mul(block_size);
            let used_bytes = total_bytes.saturating_sub(free_bytes);

            let usage_pct = if total_bytes > 0 {
                ((used_bytes as f64 / total_bytes as f64) * 100.0) as f32
            } else {
                0.0
            };

            Ok(DiskUsage {
                mount_point: path.to_path_buf(),
                fs_type: "ext4".to_string(),
                total_bytes,
                used_bytes,
                available_bytes,
                usage_pct,
            })
        }
        Err(err) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("statvfs failed for path {:?}: {}", path, err),
        )),
    }
}

pub fn scan_mounts() -> Result<Vec<DiskUsage>, std::io::Error> {
    let mut results = Vec::new();
    let mut seen_mounts = std::collections::HashSet::new();

    if let Ok(file) = File::open("/proc/mounts") {
        let reader = BufReader::new(file);
        for line_res in reader.lines() {
            if let Ok(line) = line_res {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let mount_str = parts[1];
                    let fs_type = parts[2];

                    if fs_type.starts_with("proc")
                        || fs_type.starts_with("sysfs")
                        || fs_type.starts_with("devtmpfs")
                        || fs_type.starts_with("cgroup")
                        || fs_type == "tmpfs"
                        || (fs_type == "overlay" && mount_str.contains("docker"))
                        || mount_str.starts_with("/dev")
                        || mount_str.starts_with("/sys")
                        || mount_str.starts_with("/proc")
                    {
                        continue;
                    }

                    let mount_path = PathBuf::from(mount_str);
                    if seen_mounts.insert(mount_path.clone()) {
                        if let Ok(mut usage) = scan_disk_usage(&mount_path) {
                            usage.fs_type = fs_type.to_string();
                            results.push(usage);
                        }
                    }
                }
            }
        }
    }

    let root_path = PathBuf::from("/");
    if !seen_mounts.contains(&root_path) {
        if let Ok(usage) = scan_disk_usage(&root_path) {
            results.push(usage);
        }
    }

    Ok(results)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub show_hidden: bool,
    pub sort_by: SortBy,
    pub ascending: bool,
    pub filter_query: Option<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_by: SortBy::Name,
            ascending: true,
            filter_query: None,
        }
    }
}

pub fn scan_directory(dir_path: &Path, opts: &ScanOptions) -> Result<Vec<FileEntry>, std::io::Error> {
    let read_dir = std::fs::read_dir(dir_path)?;
    let mut entries = Vec::new();

    for entry_res in read_dir {
        let entry = entry_res?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if !opts.show_hidden && name.starts_with('.') {
            continue;
        }

        if let Some(ref q) = opts.filter_query {
            if !name.to_lowercase().contains(&q.to_lowercase()) {
                continue;
            }
        }

        if let Ok(mut fe) = FileEntry::from_path(&path) {
            fe.git_status = detect_git_status(&path);
            entries.push(fe);
        }
    }

    // Sort entries: directories always first by default
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        let cmp = match opts.sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
            SortBy::Modified => a.modified_timestamp.cmp(&b.modified_timestamp),
            SortBy::Type => a.extension.cmp(&b.extension),
        };
        if opts.ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });

    Ok(entries)
}

fn detect_git_status(path: &Path) -> GitStatus {
    // Quick heuristic: check if parent directory contains .git
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return GitStatus::Clean;
        }
        current = dir.parent();
    }
    GitStatus::NotRepo
}
