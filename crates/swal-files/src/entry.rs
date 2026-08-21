//! File entry representation and metadata extraction

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GitStatus {
    Clean,
    Modified,
    Untracked,
    Staged,
    Ignored,
    NotRepo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub formatted_size: String,
    pub modified_timestamp: u64,
    pub extension: String,
    pub mime_category: String,
    pub icon: String,
    pub git_status: GitStatus,
    pub tags: Vec<String>,
    pub agent_summary: Option<String>,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let meta = std::fs::symlink_metadata(path)?;
        let is_dir = meta.is_dir();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());

        let size_bytes = if is_dir { 0 } else { meta.len() };
        let formatted_size = format_size(size_bytes, is_dir);

        let modified_timestamp = meta.modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let (mime_category, icon) = resolve_category_and_icon(&name, &extension, is_dir);

        Ok(Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            size_bytes,
            formatted_size,
            modified_timestamp,
            extension,
            mime_category,
            icon,
            git_status: GitStatus::Clean,
            tags: Vec::new(),
            agent_summary: None,
        })
    }
}

pub fn format_size(bytes: u64, is_dir: bool) -> String {
    if is_dir {
        return "--".to_string();
    }
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

fn resolve_category_and_icon(name: &str, ext: &str, is_dir: bool) -> (String, String) {
    if is_dir {
        if name.starts_with('.') {
            return ("directory-hidden".to_string(), "📁".to_string());
        }
        return ("directory".to_string(), "📂".to_string());
    }

    match ext {
        "rs" => ("code-rust".to_string(), "🦀".to_string()),
        "py" => ("code-python".to_string(), "🐍".to_string()),
        "ts" | "js" | "tsx" | "jsx" => ("code-web".to_string(), "📜".to_string()),
        "nix" => ("code-nix".to_string(), "❄️".to_string()),
        "json" | "toml" | "yaml" | "yml" => ("data".to_string(), "⚙️".to_string()),
        "md" | "txt" => ("document".to_string(), "📝".to_string()),
        "png" | "jpg" | "jpeg" | "webp" | "svg" => ("image".to_string(), "🖼️".to_string()),
        "mp3" | "wav" | "flac" | "ogg" => ("audio".to_string(), "🎵".to_string()),
        "mp4" | "mkv" | "webm" => ("video".to_string(), "🎬".to_string()),
        "tar" | "gz" | "zip" | "7z" | "xz" => ("archive".to_string(), "📦".to_string()),
        _ => ("file".to_string(), "📄".to_string()),
    }
}
