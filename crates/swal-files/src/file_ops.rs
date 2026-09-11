//! Basic file operations for SWAL Files: rename, create directory, and duplicate.

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use crate::session::SessionState;

/// Renames a file or directory item to `new_name`.
///
/// Validations:
/// - `new_name` trimmed must not be empty.
/// - `new_name` must not contain path separators (`/` or `\`).
/// - `new_name` must not be reserved names `.` or `..`.
/// - Rejects target if `new_name` already exists in parent directory.
pub fn rename_item(path: &Path, new_name: &str) -> io::Result<PathBuf> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "New name cannot be empty"));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "New name cannot contain path separators",
        ));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(Error::new(ErrorKind::InvalidInput, "Reserved name cannot be used"));
    }

    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "Source path does not exist"));
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let dest_path = parent.join(trimmed);

    if dest_path.exists() && dest_path != path {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("Target path '{}' already exists", dest_path.display()),
        ));
    }

    if fs::rename(path, &dest_path).is_err() {
        // Fallback for cross-device links or rename restrictions
        if path.is_dir() {
            copy_dir_recursive(path, &dest_path)?;
            fs::remove_dir_all(path)?;
        } else {
            fs::copy(path, &dest_path)?;
            fs::remove_file(path)?;
        }
    }

    Ok(dest_path)
}

/// Creates a new directory inside `parent`.
///
/// If `name` is empty or whitespace-only, defaults to `"Nueva carpeta"`.
/// Validates that name does not contain `/`, `\`, `.`, or `..`.
/// If a directory/file with the same name exists, automatically appends a numerical suffix.
pub fn create_dir(parent: &Path, name: &str) -> io::Result<PathBuf> {
    let trimmed = name.trim();
    let base_name = if trimmed.is_empty() {
        "Nueva carpeta"
    } else {
        trimmed
    };

    if base_name.contains('/') || base_name.contains('\\') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Folder name cannot contain path separators",
        ));
    }
    if base_name == "." || base_name == ".." {
        return Err(Error::new(ErrorKind::InvalidInput, "Reserved folder name"));
    }

    let available_name = suggest_available_name(parent, base_name);
    let dest_path = parent.join(&available_name);

    fs::create_dir(&dest_path)?;
    Ok(dest_path)
}

/// Duplicates a file or directory recursively without overwriting existing files.
pub fn duplicate_item(path: &Path) -> io::Result<PathBuf> {
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "Target path does not exist"));
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let base_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Invalid filename"))?;

    let available_name = suggest_available_name(parent, &base_name);
    let dest_path = parent.join(&available_name);

    if path.is_dir() {
        copy_dir_recursive(path, &dest_path)?;
    } else {
        fs::copy(path, &dest_path)?;
    }

    Ok(dest_path)
}

/// Suggests an available filename/foldername in `dir` based on `base`.
/// If `base` collides with an existing file or directory, appends an incrementing numerical suffix.
pub fn suggest_available_name(dir: &Path, base: &str) -> String {
    let trimmed = base.trim();
    let effective_base = if trimmed.is_empty() {
        "Nueva carpeta"
    } else {
        trimmed
    };

    if !dir.join(effective_base).exists() {
        return effective_base.to_string();
    }

    let (stem, ext) = split_stem_ext(effective_base);
    let (base_stem, mut count) = parse_trailing_number(&stem);

    loop {
        count += 1;
        let candidate = if base_stem.is_empty() {
            format!("{}{}", count, ext)
        } else {
            format!("{} {}{}", base_stem, count, ext)
        };
        if !dir.join(&candidate).exists() {
            return candidate;
        }
    }
}

/// Dispatches CLI file operation subcommands.
///
/// Supported commands:
/// - `rename-item <path> <new-name>`
/// - `new-folder [parent] [name]`
/// - `duplicate-item <path>`
pub fn handle_cli(args: &[String], session: &mut SessionState) -> Option<String> {
    if args.len() < 2 {
        return None;
    }

    let cmd = args[1].as_str();
    match cmd {
        "rename-item" | "rename_item" | "rename" => {
            if args.len() > 3 {
                let target = Path::new(&args[2]);
                let new_name = &args[3];
                match rename_item(target, new_name) {
                    Ok(new_path) => {
                        let path_str = new_path.to_string_lossy().to_string();
                        session.selected_path = Some(path_str.clone());
                        return Some(format!("✓ Elemento renombrado a: {}", path_str));
                    }
                    Err(e) => {
                        eprintln!("⚠ Error renombrando elemento: {}", e);
                        return Some(format!("Error: {}", e));
                    }
                }
            } else {
                eprintln!("Uso: swal-files rename-item <path> <nuevo_nombre>");
            }
        }
        "new-folder" | "new_folder" | "mkdir" => {
            let (parent_path, folder_name) = if args.len() > 2 {
                let arg2 = Path::new(&args[2]);
                if arg2.is_dir() || args[2].starts_with('/') || args[2].starts_with('.') {
                    let name = args.get(3).cloned().unwrap_or_default();
                    (arg2.to_path_buf(), name)
                } else {
                    let active_dir = session
                        .tabs
                        .iter()
                        .find(|t| t.id == session.active_tab_id)
                        .map(|t| PathBuf::from(&t.path))
                        .unwrap_or_else(|| PathBuf::from("."));
                    (active_dir, args[2].clone())
                }
            } else {
                let active_dir = session
                    .tabs
                    .iter()
                    .find(|t| t.id == session.active_tab_id)
                    .map(|t| PathBuf::from(&t.path))
                    .unwrap_or_else(|| PathBuf::from("."));
                (active_dir, String::new())
            };

            match create_dir(&parent_path, &folder_name) {
                Ok(new_dir) => {
                    let path_str = new_dir.to_string_lossy().to_string();
                    session.selected_path = Some(path_str.clone());
                    return Some(format!("✓ Nueva carpeta creada en: {}", path_str));
                }
                Err(e) => {
                    eprintln!("⚠ Error creando carpeta: {}", e);
                    return Some(format!("Error: {}", e));
                }
            }
        }
        "duplicate-item" | "duplicate_item" | "duplicate" => {
            if args.len() > 2 {
                let target = Path::new(&args[2]);
                match duplicate_item(target) {
                    Ok(dup_path) => {
                        let path_str = dup_path.to_string_lossy().to_string();
                        session.selected_path = Some(path_str.clone());
                        return Some(format!("✓ Elemento duplicado en: {}", path_str));
                    }
                    Err(e) => {
                        eprintln!("⚠ Error duplicando elemento: {}", e);
                        return Some(format!("Error: {}", e));
                    }
                }
            } else {
                eprintln!("Uso: swal-files duplicate-item <path>");
            }
        }
        _ => {}
    }

    None
}

fn split_stem_ext(filename: &str) -> (String, String) {
    if filename.starts_with('.') && !filename[1..].contains('.') {
        return (filename.to_string(), String::new());
    }
    if let Some(dot_idx) = filename.rfind('.') {
        if dot_idx > 0 {
            return (filename[..dot_idx].to_string(), filename[dot_idx..].to_string());
        }
    }
    (filename.to_string(), String::new())
}

fn parse_trailing_number(stem: &str) -> (String, usize) {
    if let Some(space_pos) = stem.rfind(' ') {
        let num_str = &stem[space_pos + 1..];
        if !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = num_str.parse::<usize>() {
                return (stem[..space_pos].to_string(), n);
            }
        }
    }
    (stem.to_string(), 1)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rename_ok() {
        let temp = tempdir().expect("tempdir");
        let old_file = temp.path().join("old.txt");
        fs::write(&old_file, "content").expect("write");

        let result = rename_item(&old_file, "new.txt");
        assert!(result.is_ok(), "rename should succeed");
        let new_path = result.unwrap();
        assert!(new_path.exists());
        assert!(!old_file.exists());
        assert_eq!(new_path.file_name().unwrap(), "new.txt");
    }

    #[test]
    fn test_rename_invalid_name() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("file.txt");
        fs::write(&file, "content").expect("write");

        let res_slash = rename_item(&file, "invalid/name.txt");
        assert!(res_slash.is_err(), "Slash should fail rename");
        assert_eq!(res_slash.unwrap_err().kind(), ErrorKind::InvalidInput);

        let res_empty = rename_item(&file, "   ");
        assert!(res_empty.is_err(), "Empty name should fail");

        let res_dot = rename_item(&file, "..");
        assert!(res_dot.is_err(), "Reserved name .. should fail");
    }

    #[test]
    fn test_duplicate_item_no_overwrite() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("original.txt");
        fs::write(&file, "data").expect("write");

        let dup1 = duplicate_item(&file).expect("duplicate 1");
        assert!(dup1.exists());
        assert_ne!(dup1, file);
        assert_eq!(dup1.file_name().unwrap(), "original 2.txt");

        let dup2 = duplicate_item(&file).expect("duplicate 2");
        assert!(dup2.exists());
        assert_eq!(dup2.file_name().unwrap(), "original 3.txt");
    }

    #[test]
    fn test_suggest_available_name_increment() {
        let temp = tempdir().expect("tempdir");
        let base = "Nueva carpeta";

        assert_eq!(suggest_available_name(temp.path(), base), "Nueva carpeta");

        fs::create_dir(temp.path().join("Nueva carpeta")).expect("mkdir");
        assert_eq!(suggest_available_name(temp.path(), base), "Nueva carpeta 2");

        fs::create_dir(temp.path().join("Nueva carpeta 2")).expect("mkdir");
        assert_eq!(suggest_available_name(temp.path(), base), "Nueva carpeta 3");
    }
}
