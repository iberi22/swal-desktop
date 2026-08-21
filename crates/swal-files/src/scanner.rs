//! High-performance async directory scanner, filter & group engine

use crate::entry::{FileCategory, FileEntry, GitStatus};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Name
    }
}

impl SortBy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "size" | "tamano" => SortBy::Size,
            "modified" | "date" | "fecha" => SortBy::Modified,
            "type" | "tipo" => SortBy::Type,
            _ => SortBy::Name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupBy {
    None,
    Type,
    Date,
    Size,
    Alphabetical,
}

impl Default for GroupBy {
    fn default() -> Self {
        GroupBy::None
    }
}

impl GroupBy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "type" | "tipo" => GroupBy::Type,
            "date" | "modified" | "fecha" => GroupBy::Date,
            "size" | "tamano" => GroupBy::Size,
            "alpha" | "alphabetical" | "alfabetico" => GroupBy::Alphabetical,
            _ => GroupBy::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSection {
    pub title: String,
    pub count: usize,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub show_hidden: bool,
    pub sort_by: SortBy,
    pub ascending: bool,
    pub filter_query: Option<String>,
    pub filter_category: String, // "all", "code", "documents", "images", "media", "archives", "folders"
    pub group_by: GroupBy,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_by: SortBy::Name,
            ascending: true,
            filter_query: None,
            filter_category: "all".to_string(),
            group_by: GroupBy::None,
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
            if !fe.matches_filter(&opts.filter_category) {
                continue;
            }
            fe.git_status = detect_git_status(&path);
            entries.push(fe);
        }
    }

    sort_entries(&mut entries, opts.sort_by, opts.ascending);

    Ok(entries)
}

pub fn sort_entries(entries: &mut [FileEntry], sort_by: SortBy, ascending: bool) {
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir); // Directories first
        }
        let cmp = match sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
            SortBy::Modified => a.modified_timestamp.cmp(&b.modified_timestamp),
            SortBy::Type => a.mime_category.cmp(&b.mime_category),
        };
        if ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });
}

pub fn group_entries(entries: &[FileEntry], group_by: GroupBy) -> Vec<GroupSection> {
    if group_by == GroupBy::None {
        return vec![GroupSection {
            title: "Todos los elementos".to_string(),
            count: entries.len(),
            entries: entries.to_vec(),
        }];
    }

    let mut sections: Vec<(String, Vec<FileEntry>)> = Vec::new();

    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for entry in entries {
        let key = match group_by {
            GroupBy::None => "Todos".to_string(),
            GroupBy::Type => {
                if entry.is_dir {
                    "📁 Carpetas".to_string()
                } else {
                    match entry.category {
                        FileCategory::Code => "🦀 Código Fuente".to_string(),
                        FileCategory::Document => "📝 Documentos".to_string(),
                        FileCategory::Image => "🖼️ Imágenes".to_string(),
                        FileCategory::Audio | FileCategory::Video => "🎬 Multimedia".to_string(),
                        FileCategory::Archive => "📦 Comprimidos".to_string(),
                        FileCategory::Config => "⚙️ Configuración / Datos".to_string(),
                        FileCategory::Binary => "💽 Binarios".to_string(),
                        _ => "📄 Otros Archivos".to_string(),
                    }
                }
            }
            GroupBy::Date => {
                if entry.is_dir {
                    "📁 Carpetas".to_string()
                } else {
                    let age = now_secs.saturating_sub(entry.modified_timestamp);
                    if age < 86400 {
                        "📅 Hoy".to_string()
                    } else if age < 86400 * 2 {
                        "📅 Ayer".to_string()
                    } else if age < 86400 * 7 {
                        "📅 Esta Semana".to_string()
                    } else if age < 86400 * 30 {
                        "📅 Este Mes".to_string()
                    } else {
                        "📅 Más Antiguo".to_string()
                    }
                }
            }
            GroupBy::Size => {
                if entry.is_dir {
                    "📁 Carpetas".to_string()
                } else {
                    let sz = entry.size_bytes;
                    if sz < 10 * 1024 {
                        "🔹 Diminuto (< 10 KB)".to_string()
                    } else if sz < 1024 * 1024 {
                        "🔹 Pequeño (10 KB - 1 MB)".to_string()
                    } else if sz < 100 * 1024 * 1024 {
                        "🔹 Mediano (1 MB - 100 MB)".to_string()
                    } else {
                        "🔹 Grande (> 100 MB)".to_string()
                    }
                }
            }
            GroupBy::Alphabetical => {
                if entry.is_dir {
                    "📁 Carpetas".to_string()
                } else {
                    let first_char = entry.name.chars().next().unwrap_or('#').to_ascii_uppercase();
                    match first_char {
                        'A'..='D' => "🔤 A - D".to_string(),
                        'E'..='H' => "🔤 E - H".to_string(),
                        'I'..='L' => "🔤 I - L".to_string(),
                        'M'..='P' => "🔤 M - P".to_string(),
                        'Q'..='T' => "🔤 Q - T".to_string(),
                        'U'..='Z' => "🔤 U - Z".to_string(),
                        _ => "🔤 0 - 9 / Símbolos".to_string(),
                    }
                }
            }
        };

        if let Some(pos) = sections.iter().position(|(t, _)| t == &key) {
            sections[pos].1.push(entry.clone());
        } else {
            sections.push((key, vec![entry.clone()]));
        }
    }

    sections
        .into_iter()
        .map(|(title, items)| GroupSection {
            count: items.len(),
            title,
            entries: items,
        })
        .collect()
}

pub fn detect_git_status(path: &Path) -> GitStatus {
    let mut current = if path.is_file() { path.parent() } else { Some(path) };
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return GitStatus::Clean;
        }
        current = dir.parent();
    }
    GitStatus::NotRepo
}

