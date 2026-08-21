//! High-performance async directory scanner & filter

use crate::entry::{FileEntry, GitStatus};
use std::path::Path;

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
