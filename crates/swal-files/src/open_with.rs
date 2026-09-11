//! Open With, Desktop Entry Parser, and Terminal Integration for SWAL Files

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::session::SessionState;

/// Desktop Application Entry metadata parsed from .desktop files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: String,
}

/// Cleans Freedesktop Exec field codes (%f, %F, %u, %U, %i, %c, %k, %m, etc.) from an Exec string
pub fn clean_exec(exec_str: &str) -> String {
    let mut cleaned_tokens = Vec::new();
    for token in exec_str.split_whitespace() {
        let unquoted = token.trim_matches('"').trim_matches('\'');
        if unquoted.starts_with('%')
            && unquoted.len() == 2
            && unquoted.chars().nth(1).map_or(false, |c| c.is_ascii_alphabetic())
        {
            continue;
        }

        let mut t_res = String::new();
        let mut chars = token.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_alphabetic() {
                        chars.next();
                        continue;
                    }
                }
            }
            t_res.push(c);
        }

        let t_trimmed = t_res
            .trim_matches('"')
            .trim_matches('\'')
            .trim_end_matches('=')
            .trim();
        if !t_trimmed.is_empty() {
            cleaned_tokens.push(t_trimmed.to_string());
        }
    }
    cleaned_tokens.join(" ")
}

/// Parses the content of a `.desktop` INI-style entry file
pub fn parse_desktop_file_content(content: &str, file_id: &str) -> Option<AppEntry> {
    let mut in_desktop_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut no_display = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Name" if name.is_none() => name = Some(value.to_string()),
                "Exec" if exec.is_none() => exec = Some(value.to_string()),
                "Icon" if icon.is_none() => icon = Some(value.to_string()),
                "NoDisplay" => {
                    if value.eq_ignore_ascii_case("true") || value == "1" {
                        no_display = true;
                    }
                }
                _ => {}
            }
        }
    }

    if no_display {
        return None;
    }

    let raw_exec = exec?;
    let cleaned_exec = clean_exec(&raw_exec);
    if cleaned_exec.is_empty() {
        return None;
    }

    let app_name = name.unwrap_or_else(|| file_id.to_string());
    let app_icon = icon.unwrap_or_default();

    Some(AppEntry {
        id: file_id.to_string(),
        name: app_name,
        exec: cleaned_exec,
        icon: app_icon,
    })
}

fn get_desktop_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_home) = dirs::data_dir() {
        dirs.push(data_home.join("applications"));
    } else if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }

    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for path_str in xdg_data_dirs.split(':') {
            if !path_str.trim().is_empty() {
                let p = PathBuf::from(path_str.trim()).join("applications");
                if !dirs.contains(&p) {
                    dirs.push(p);
                }
            }
        }
    }

    let fallbacks = vec![
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];

    for fb in fallbacks {
        if !dirs.contains(&fb) {
            dirs.push(fb);
        }
    }

    dirs
}

/// Lists all available desktop applications by scanning `$XDG_DATA_DIRS/applications`
pub fn list_applications() -> Vec<AppEntry> {
    let search_dirs = get_desktop_search_dirs();
    let mut apps = Vec::new();
    let mut seen_ids = HashSet::new();

    for dir in search_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let id = stem.to_string();
                        if seen_ids.contains(&id) {
                            continue;
                        }
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(app) = parse_desktop_file_content(&content, &id) {
                                seen_ids.insert(id.clone());
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }
    }

    apps
}

/// Launches the specified application for a target file path
pub fn open_with(path: &Path, app_id: &str) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path not found: {}", path.display()),
        ));
    }

    let target_id = app_id.strip_suffix(".desktop").unwrap_or(app_id);
    let apps = list_applications();
    let app = apps
        .iter()
        .find(|a| a.id == target_id || a.id == app_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Application ID not found: {}", app_id),
            )
        })?;

    let parts: Vec<&str> = app.exec.split_whitespace().collect();
    if parts.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Application has empty exec field",
        ));
    }

    let bin = parts[0];
    let args = &parts[1..];

    let mut cmd = Command::new(bin);
    cmd.args(args);
    cmd.arg(path);

    match cmd.spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to spawn {}: {}", bin, e),
        )),
    }
}

/// Opens a terminal at the directory containing the file or the directory itself
pub fn reveal_in_terminal(dir: &Path) -> io::Result<()> {
    let target_dir = if dir.is_dir() {
        dir.to_path_buf()
    } else if let Some(parent) = dir.parent() {
        parent.to_path_buf()
    } else {
        PathBuf::from(".")
    };
    terminal_here(&target_dir)
}

/// Opens Ghostty terminal focused at the specified directory
pub fn terminal_here(dir: &Path) -> io::Result<()> {
    let target_dir = if dir.is_dir() {
        dir.to_path_buf()
    } else if let Some(parent) = dir.parent() {
        parent.to_path_buf()
    } else {
        PathBuf::from(".")
    };

    let dir_str = target_dir.to_string_lossy().to_string();
    let status = Command::new("ghostty")
        .arg(format!("--working-directory={}", dir_str))
        .spawn();

    match status {
        Ok(_) => Ok(()),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to launch ghostty: {}", e),
        )),
    }
}

/// Dispatches CLI commands related to open-with and terminal actions
pub fn handle_cli(args: &[String], _session: &mut SessionState) -> Option<String> {
    if args.len() < 2 {
        return None;
    }

    let cmd = args[1].as_str();

    match cmd {
        "apps-json" | "apps_json" => {
            let apps = list_applications();
            serde_json::to_string(&apps).ok()
        }
        "open-with" | "open_with" => {
            if args.len() >= 4 {
                let path = Path::new(&args[2]);
                let app_id = &args[3];
                if let Err(e) = open_with(path, app_id) {
                    eprintln!("Error opening with {}: {}", app_id, e);
                }
            } else {
                eprintln!("Usage: swal-files open-with <path> <app-id>");
            }
            None
        }
        "reveal-terminal" | "reveal_terminal" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let path = Path::new(path_str);
            if let Err(e) = reveal_in_terminal(path) {
                eprintln!("Error revealing terminal: {}", e);
            }
            None
        }
        "terminal-here" | "terminal_here" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let path = Path::new(path_str);
            if let Err(e) = terminal_here(path) {
                eprintln!("Error opening terminal here: {}", e);
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desktop_file_fixture() {
        let fixture = r#"
[Desktop Entry]
Name=Visual Studio Code
Comment=Code Editing Redefined
Exec=code --unity-launch %F
Icon=vscode
Type=Application
StartupNotify=true
Categories=Utility;TextEditor;Development;IDE;
MimeType=text/plain;
"#;
        let entry = parse_desktop_file_content(fixture, "code").expect("Should parse desktop file");
        assert_eq!(entry.id, "code");
        assert_eq!(entry.name, "Visual Studio Code");
        assert_eq!(entry.exec, "code --unity-launch");
        assert_eq!(entry.icon, "vscode");
    }

    #[test]
    fn test_filter_no_display() {
        let fixture = r#"
[Desktop Entry]
Name=Hidden Helper
Exec=helper %u
Icon=helper-icon
NoDisplay=true
"#;
        let entry = parse_desktop_file_content(fixture, "hidden-helper");
        assert!(entry.is_none(), "NoDisplay=true should filter out app entry");
    }

    #[test]
    fn test_clean_exec_placeholders() {
        assert_eq!(clean_exec("ghostty --working-directory=%f"), "ghostty --working-directory");
        assert_eq!(clean_exec("vlc --started-from-file %U"), "vlc --started-from-file");
        assert_eq!(clean_exec("subl %F %i %c"), "subl");
        assert_eq!(clean_exec("\"/usr/bin/gimp-2.10\" %U"), "/usr/bin/gimp-2.10");
    }

    #[test]
    fn test_list_applications_does_not_panic() {
        let apps = list_applications();
        // Returns a vector of apps without panicking
        println!("Found {} apps on system", apps.len());
    }
}
