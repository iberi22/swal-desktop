//! QuickLook and sidebar file preview generator

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::git::detect_git_status_for_dir;
use crate::session::editor_state_file_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewState {
    pub path: String,
    pub file_name: String,
    pub file_type: String,
    pub size_formatted: String,
    pub date_modified: String,
    pub is_image: bool,
    pub is_text: bool,
    pub is_dir: bool,
    pub is_git_repo: bool,
    pub image_path: String,
    pub line_count: usize,
    pub content: String,
    pub gutter_lines: String,
    pub git_status_summary: String,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            path: dirs::home_dir().unwrap_or_default().to_string_lossy().to_string(),
            file_name: "README.md".to_string(),
            file_type: "Documento Markdown".to_string(),
            size_formatted: "1.2 KB".to_string(),
            date_modified: "--".to_string(),
            is_image: false,
            is_text: true,
            is_dir: false,
            is_git_repo: false,
            image_path: String::new(),
            line_count: 1,
            content: "# SWAL Files & QuickLook\nSelect a file to preview.".to_string(),
            gutter_lines: "1\n2".to_string(),
            git_status_summary: String::new(),
        }
    }
}

pub fn format_preview_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} B", bytes)
    }
}

pub fn sanitize_preview_text(text: &str) -> String {
    let clean: String = text
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace() || *c > '\u{7f}')
        .filter(|c| *c != '"' && *c != '\'' && *c != '`' && *c != '\\')
        .collect();
    if clean.trim().is_empty() {
        "item".to_string()
    } else {
        clean.trim().to_string()
    }
}

pub fn generate_preview_for_path(target_path: &Path) -> PreviewState {
    let raw_name = target_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "item".to_string());

    let ext = target_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let meta = fs::metadata(target_path).ok();
    let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
    let size_str = format_preview_bytes(size_bytes);

    let date_str = meta
        .and_then(|m| m.modified().ok())
        .map(|sys_time| {
            let dt: DateTime<Local> = sys_time.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_else(|| "--".to_string());

    if is_dir {
        let git = detect_git_status_for_dir(target_path);
        let items_count = fs::read_dir(target_path).map(|r| r.count()).unwrap_or(0);

        let mut content = format!("📂 Directorio: {}\n📊 Contenido: {} elementos\n", target_path.display(), items_count);
        if git.is_git_repo {
            content.push_str(&format!("\n🌿 Repositorio Git:\n  • Rama: {}\n  • Estado: {}\n  • Modificados: {}\n  • Staged: {}\n  • Untracked: {}\n",
                git.branch,
                if git.is_clean { "Limpio" } else { "Con cambios" },
                git.modified_count,
                git.staged_count,
                git.untracked_count
            ));
        }

        return PreviewState {
            path: target_path.to_string_lossy().to_string(),
            file_name: sanitize_preview_text(&raw_name),
            file_type: if git.is_git_repo { "Repositorio Git".to_string() } else { "Carpeta de archivos".to_string() },
            size_formatted: format!("{} items", items_count),
            date_modified: date_str,
            is_image: false,
            is_text: false,
            is_dir: true,
            is_git_repo: git.is_git_repo,
            image_path: String::new(),
            line_count: 5,
            content,
            gutter_lines: "1\n2\n3\n4\n5".to_string(),
            git_status_summary: git.summary,
        };
    }

    let is_image = matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif" | "bmp" | "ico"
    );

    let (content, line_count, is_text, file_type) = if is_image {
        (
            format!("[Vista previa de Imagen]\nArchivo: {}\nFormato: {}\nTamaño: {}", raw_name, ext.to_uppercase(), size_str),
            1,
            false,
            format!("Imagen ({})", ext.to_uppercase()),
        )
    } else {
        match fs::read_to_string(target_path) {
            Ok(text) => {
                let lines: Vec<&str> = text.lines().collect();
                let count = lines.len().max(1);
                let preview_text = if lines.len() > 150 {
                    format!("{}\n\n... [Truncado a 150 líneas de {}]", lines[..150].join("\n"), lines.len())
                } else {
                    text
                };
                let kind = match ext.as_str() {
                    "rs" => "Código fuente Rust",
                    "py" => "Script Python",
                    "ts" | "js" | "tsx" | "jsx" => "Script TypeScript/JS",
                    "json" | "toml" | "yaml" | "yml" => "Configuración / Datos",
                    "md" => "Documento Markdown",
                    "nix" => "Módulo NixOS",
                    "sh" | "bash" => "Shell Script",
                    "css" | "scss" => "Hoja de estilo CSS",
                    "html" => "Documento HTML",
                    _ => "Archivo de texto",
                };
                (preview_text, count, true, kind.to_string())
            }
            Err(_) => {
                // Binary fallback Hex dump
                let hex_view = match fs::read(target_path) {
                    Ok(bytes) => {
                        let take = bytes.len().min(512);
                        let mut hex = String::from("HEX DUMP:\n");
                        for (i, chunk) in bytes[..take].chunks(16).enumerate() {
                            let hex_bytes: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
                            let ascii: String = chunk
                                .iter()
                                .map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' })
                                .collect();
                            hex.push_str(&format!("{:06X}: {:<48} | {} |\n", i * 16, hex_bytes.join(" "), ascii));
                        }
                        hex
                    }
                    Err(e) => format!("Error al abrir archivo: {}", e),
                };
                (hex_view, 32, false, format!("Archivo binario ({})", ext.to_uppercase()))
            }
        }
    };

    let gutter: String = (1..=line_count.min(150))
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let git_summary = detect_git_status_for_dir(target_path.parent().unwrap_or(target_path)).summary;

    PreviewState {
        path: target_path.to_string_lossy().to_string(),
        file_name: sanitize_preview_text(&raw_name),
        file_type,
        size_formatted: size_str,
        date_modified: date_str,
        is_image,
        is_text,
        is_dir: false,
        is_git_repo: false,
        image_path: if is_image { target_path.to_string_lossy().to_string() } else { String::new() },
        line_count,
        content,
        gutter_lines: gutter,
        git_status_summary: git_summary,
    }
}

pub fn load_editor_state() -> PreviewState {
    load_editor_state_from_path(&editor_state_file_path())
}

pub fn load_editor_state_from_path(path: &Path) -> PreviewState {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(state) = serde_json::from_str::<PreviewState>(&content) {
            return state;
        }
    }
    PreviewState::default()
}

pub fn save_editor_state(state: &PreviewState) {
    save_editor_state_to_path(state, &editor_state_file_path());
}

pub fn save_editor_state_to_path(state: &PreviewState, path: &Path) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}
