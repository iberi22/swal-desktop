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
    Conflicted,
    NotRepo,
}

impl GitStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GitStatus::Clean => "clean",
            GitStatus::Modified => "modified",
            GitStatus::Untracked => "untracked",
            GitStatus::Staged => "staged",
            GitStatus::Ignored => "ignored",
            GitStatus::Conflicted => "conflicted",
            GitStatus::NotRepo => "not_repo",
        }
    }

    pub fn badge_icon(&self) -> &'static str {
        match self {
            GitStatus::Clean => "✓",
            GitStatus::Modified => "●",
            GitStatus::Untracked => "…",
            GitStatus::Staged => "+",
            GitStatus::Ignored => "◌",
            GitStatus::Conflicted => "!",
            GitStatus::NotRepo => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileCategory {
    Folder,
    Code,
    Document,
    Image,
    Audio,
    Video,
    Archive,
    Config,
    Binary,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub formatted_size: String,
    pub modified_timestamp: u64,
    pub formatted_date: String,
    pub extension: String,
    pub mime_category: String,
    pub category: FileCategory,
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

        let (modified_timestamp, formatted_date) = meta.modified()
            .map(|sys_time| {
                let dt: chrono::DateTime<chrono::Local> = sys_time.into();
                (
                    sys_time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
                    dt.format("%Y-%m-%d %H:%M").to_string(),
                )
            })
            .unwrap_or_else(|_| (0, "--".to_string()));

        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let (category, mime_category, icon) = resolve_category_and_icon(&name, &extension, is_dir);

        Ok(Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            size_bytes,
            formatted_size,
            modified_timestamp,
            formatted_date,
            extension,
            mime_category,
            category,
            icon,
            git_status: GitStatus::Clean,
            tags: Vec::new(),
            agent_summary: None,
        })
    }

    pub fn matches_filter(&self, filter: &str) -> bool {
        match filter.to_lowercase().as_str() {
            "all" | "" => true,
            "folders" | "folder" | "dirs" => self.is_dir,
            "code" => self.category == FileCategory::Code,
            "documents" | "docs" | "document" => self.category == FileCategory::Document,
            "images" | "image" | "img" => self.category == FileCategory::Image,
            "media" | "audio" | "video" => matches!(self.category, FileCategory::Audio | FileCategory::Video),
            "archives" | "archive" | "zip" => self.category == FileCategory::Archive,
            _ => true,
        }
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

fn resolve_category_and_icon(name: &str, ext: &str, is_dir: bool) -> (FileCategory, String, String) {
    if is_dir {
        if name.starts_with('.') {
            return (FileCategory::Folder, "directory-hidden".to_string(), "📁".to_string());
        }
        return (FileCategory::Folder, "directory".to_string(), "📂".to_string());
    }

    match ext {
        "rs" => (FileCategory::Code, "code-rust".to_string(), "🦀".to_string()),
        "py" => (FileCategory::Code, "code-python".to_string(), "🐍".to_string()),
        "ts" | "js" | "tsx" | "jsx" => (FileCategory::Code, "code-web".to_string(), "📜".to_string()),
        "nix" => (FileCategory::Code, "code-nix".to_string(), "❄️".to_string()),
        "sh" | "bash" | "zsh" => (FileCategory::Code, "code-shell".to_string(), "🐚".to_string()),
        "c" | "cpp" | "h" | "hpp" => (FileCategory::Code, "code-c".to_string(), "⚙️".to_string()),
        "html" | "css" | "scss" => (FileCategory::Code, "code-style".to_string(), "🎨".to_string()),
        "json" | "toml" | "yaml" | "yml" => (FileCategory::Config, "config-data".to_string(), "⚙️".to_string()),
        "md" | "txt" | "pdf" | "doc" | "docx" | "csv" => (FileCategory::Document, "document".to_string(), "📝".to_string()),
        "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif" | "bmp" | "ico" => (FileCategory::Image, "image".to_string(), "🖼️".to_string()),
        "mp3" | "wav" | "flac" | "ogg" => (FileCategory::Audio, "audio".to_string(), "🎵".to_string()),
        "mp4" | "mkv" | "webm" | "mov" | "avi" => (FileCategory::Video, "video".to_string(), "🎬".to_string()),
        "tar" | "gz" | "zip" | "7z" | "xz" | "rar" => (FileCategory::Archive, "archive".to_string(), "📦".to_string()),
        "bin" | "so" | "dll" | "exe" | "o" => (FileCategory::Binary, "binary".to_string(), "💽".to_string()),
        _ => (FileCategory::Other, "file".to_string(), "📄".to_string()),
    }
}

