//! CLI command dispatcher and interactive controller for SWAL Files

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::config::FileManagerConfig;
use crate::gui::{build_gui_payload, notify_eww_update};
use crate::omnibar::{parse_omnibar_input, OmnibarIntent};
use crate::preview::{generate_preview_for_path, load_editor_state, save_editor_state};
use crate::session::{load_session, save_session, SessionState, TabState};

const PID_FILE: &str = "/tmp/swal-files.pid";

/// Portable home-directory fallback for string contexts (never a personal path).
fn home_path_string() -> String {
    dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// Remove PID file on clean exit
fn remove_pid_file() {
    let _ = fs::remove_file(PID_FILE);
}

const VISIBLE_FLAG: &str = "/tmp/swal_files_visible.flag";

pub fn is_window_open() -> bool {
    // Zero-Eww: a window is "open" iff a live swal-files --gui process owns it.
    // The PID file is the single source of truth (the --gui process writes it on
    // startup and removes it on exit). No flag files, no eww, no hyprctl scraping.
    if let Ok(content) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = content.trim().parse::<i32>() {
            return unsafe { libc::kill(pid, 0) == 0 };
        }
    }
    false
}

pub fn close_gui() {
    // Zero-Eww: signal the live --gui process to exit; it cleans its own files.
    if let Ok(content) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = content.trim().parse::<i32>() {
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
        }
    }
    fs::remove_file(VISIBLE_FLAG).ok();
    remove_pid_file();
}

pub fn open_gui(target_path: Option<&str>) {
    // EWW-based toggle: open or close the swal_files EWW overlay.
    // The native Wayland renderer (wl_shm) is still WIP — EWW is the stable path.

    // If a target path is given, navigate there first then open
    if let Some(target) = target_path {
        let p = PathBuf::from(target);
        if p.exists() {
            let path_str = p.to_string_lossy().to_string();
            let mut session = load_session();
            let found = session.tabs.iter_mut().any(|t| {
                if t.path == path_str { true } else { false }
            });
            if !found {
                let next_id = session.tabs.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                let title = p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/".to_string());
                session.tabs.push(TabState { id: next_id, title, path: path_str, active: true });
                session.active_tab_id = next_id;
            }
            save_session(&session);
        }
    }

    launch_gui_window();
}

/// Toggles the EWW swal_files overlay open/closed.
/// EWW is the stable render path while the native Wayland renderer is WIP.
fn launch_gui_window() {
    let _ = Command::new("eww")
        .args(["open", "--toggle", "swal_files"])
        .spawn();
}


/// True if the live --gui process has NO terminal window attached
/// (ghostty closed and reparented it to init, PPID == 1).
#[allow(dead_code)]
fn gui_process_is_orphaned() -> bool {
    let pid = fs::read_to_string(PID_FILE)
        .ok()
        .and_then(|c| c.trim().parse::<i32>().ok());
    let Some(pid) = pid else {
        return false;
    };
    // Parse PPID from /proc/<pid>/stat (field 4, after comm in parens)
    let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).unwrap_or_default();
    let rest = stat.rsplit(')').next().unwrap_or("");
    let fields: Vec<&str> = rest.split_whitespace().collect();
    fields
        .get(1) // after ")": state is 0, ppid is 1
        .and_then(|ppid| ppid.parse::<i32>().ok())
        .map(|ppid| ppid == 1)
        .unwrap_or(false)
}


pub fn handle_command(session: &mut SessionState, args: &[String]) -> Result<Option<String>, String> {
    if args.len() < 2 {
        open_gui(None);
        return Ok(None);
    }

    let cmd = args[1].as_str();

    // Pre-dispatch to island CLI handlers (WAVE-FM.02 .. WAVE-FM.07)
    if let Some(res) = crate::clipboard_ops::handle_cli(args, session) {
        return Ok(Some(res));
    }
    if let Some(res) = crate::file_ops::handle_cli(args, session) {
        return Ok(Some(res));
    }
    if let Some(res) = crate::trash_ops::handle_cli(args, session) {
        return Ok(Some(res));
    }
    if let Some(res) = crate::properties::handle_cli(args, session) {
        return Ok(Some(res));
    }
    if let Some(res) = crate::text_viewer::handle_cli(args, session) {
        return Ok(Some(res));
    }
    if let Some(res) = crate::open_with::handle_cli(args, session) {
        return Ok(Some(res));
    }

    // Direct path argument
    if cmd.starts_with('/') || cmd.starts_with('~') || Path::new(cmd).exists() {
        open_gui(Some(cmd));
        return Ok(None);
    }

    // Try dispatching clipboard commands first
    if let Some(res) = crate::clipboard_ops::handle_cli(args, session) {
        return Ok(Some(res));
    }

    let mut state_changed = false;

    match cmd {
        "menu-json" | "menu_json" => {
            let target = args.get(2).map(Path::new);
            let json_str = crate::context_menu::menu_json(target);
            return Ok(Some(json_str));
        }
        "context-open" | "context_open" => {
            if args.len() > 2 {
                let target = PathBuf::from(&args[2]);
                session.selected_path = Some(target.to_string_lossy().to_string());
                state_changed = true;
            }
            let _ = Command::new("eww").args(["open", "files_ctx_menu"]).status();
        }
        "context-close" | "context_close" => {
            let _ = Command::new("eww").args(["close", "files_ctx_menu"]).status();
        }
        "menu-run" | "menu_run" => {
            let action_id = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if action_id.is_empty() {
                eprintln!("Error: Target action-id required for menu-run");
                return Ok(Some("Error: Target action-id required".to_string()));
            }

            let target_str = args
                .get(3)
                .cloned()
                .or_else(|| session.selected_path.clone())
                .or_else(|| {
                    session
                        .tabs
                        .iter()
                        .find(|t| t.id == session.active_tab_id)
                        .map(|t| t.path.clone())
                });

            let Some(target_path_str) = target_str else {
                eprintln!("Error: Target path missing for action '{}'", action_id);
                return Ok(Some(format!("Error: Target path missing for action '{}'", action_id)));
            };

            let target_path = Path::new(&target_path_str);

            let result_msg = match action_id {
                "copy-path" => {
                    if target_path_str.is_empty() {
                        eprintln!("Error: Invalid target path");
                        "Error: Invalid target path".to_string()
                    } else {
                        let _ = Command::new("wl-copy").arg(&target_path_str).status();
                        format!("✓ Copied path: {}", target_path_str)
                    }
                }
                "copy-name" => {
                    if let Some(filename) = target_path.file_name().and_then(|n| n.to_str()) {
                        let _ = Command::new("wl-copy").arg(filename).status();
                        format!("✓ Copied name: {}", filename)
                    } else {
                        eprintln!("Error: Invalid target filename for copy-name");
                        "Error: Invalid target filename".to_string()
                    }
                }
                "open" => {
                    if target_path.is_dir() {
                        for t in session.tabs.iter_mut() {
                            if t.id == session.active_tab_id {
                                t.path = target_path_str.clone();
                                t.title = target_path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "/".to_string());
                            }
                        }
                        session.selected_path = None;
                        state_changed = true;
                        format!("✓ Navigated to {}", target_path_str)
                    } else if target_path.is_file() {
                        let _ = crate::platform::PlatformAbstraction::open_with_default_app(target_path);
                        format!("✓ Opened file {}", target_path_str)
                    } else {
                        eprintln!("Error: Target path does not exist: {}", target_path_str);
                        format!("Error: Target path does not exist: {}", target_path_str)
                    }
                }
                "trash" => {
                    if target_path.exists() {
                        if let Err(e) = crate::platform::PlatformAbstraction::move_to_trash(target_path) {
                            eprintln!("Error moving to trash: {}", e);
                            format!("Error moving to trash: {}", e)
                        } else {
                            session.selected_path = None;
                            state_changed = true;
                            format!("✓ Moved to trash: {}", target_path_str)
                        }
                    } else {
                        eprintln!("Error: Target path does not exist for trash: {}", target_path_str);
                        format!("Error: Target path does not exist: {}", target_path_str)
                    }
                }
                _ => {
                    if !target_path.exists() && action_id != "new-folder" && action_id != "paste" {
                        eprintln!("Error: Target path does not exist for action '{}': {}", action_id, target_path_str);
                        format!("Error: Target path does not exist for action '{}': {}", action_id, target_path_str)
                    } else {
                        format!("✓ Executed context action '{}' on {}", action_id, target_path_str)
                    }
                }
            };

            if state_changed {
                save_session(session);
                let payload = build_gui_payload(session);
                notify_eww_update(&payload);
            }

            return Ok(Some(result_msg));
        }
        "view-json" | "view_json" | "json" => {
            let payload = build_gui_payload(session);
            return Ok(Some(serde_json::to_string(&payload).map_err(|e| e.to_string())?));
        }
        "editor-json" | "editor_json" => {
            let editor = load_editor_state();
            return Ok(Some(serde_json::to_string(&editor).map_err(|e| e.to_string())?));
        }
        "properties-json" | "properties_json" | "properties" => {
            if let Some(json_out) = crate::properties::handle_cli(args, session) {
                return Ok(Some(json_out));
            } else {
                return Err("Failed to compute item properties".to_string());
            }
        }
        "nav" => {
            if args.len() > 2 {
                let target = PathBuf::from(&args[2]);
                let target_dir = if target.is_dir() {
                    Some(target)
                } else if let Ok(canon) = fs::canonicalize(&target) {
                    if canon.is_dir() { Some(canon) } else { None }
                } else {
                    None
                };

                if let Some(dir) = target_dir {
                    for t in session.tabs.iter_mut() {
                        if t.id == session.active_tab_id {
                            t.path = dir.to_string_lossy().to_string();
                            t.title = dir
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "/".to_string());
                        }
                    }
                    session.selected_path = None;
                    state_changed = true;
                }
            }
        }
        "select-item" | "select_item" | "select" => {
            if args.len() > 2 {
                let target = PathBuf::from(&args[2]);
                let target_str = target.to_string_lossy().to_string();
                session.selected_path = Some(target_str.clone());
                session.selected_paths = vec![target_str];
                state_changed = true;
            }
        }
        "select-toggle" | "select_toggle" => {
            if args.len() > 2 {
                let target = PathBuf::from(&args[2]);
                let target_str = target.to_string_lossy().to_string();
                if let Some(pos) = session.selected_paths.iter().position(|p| p == &target_str) {
                    session.selected_paths.remove(pos);
                    if session.selected_path.as_deref() == Some(&target_str) {
                        session.selected_path = session.selected_paths.last().cloned();
                    }
                } else {
                    session.selected_paths.push(target_str.clone());
                    session.selected_path = Some(target_str);
                }
                state_changed = true;
            }
        }
        "select-range" | "select_range" => {
            if args.len() > 2 {
                let target_str = PathBuf::from(&args[2]).to_string_lossy().to_string();
                let payload = build_gui_payload(session);
                let visible_paths: Vec<String> = payload.entries.into_iter().map(|e| e.path).collect();

                let anchor_str = session.selected_path.clone().unwrap_or_else(|| target_str.clone());
                let start_idx = visible_paths.iter().position(|p| p == &anchor_str);
                let end_idx = visible_paths.iter().position(|p| p == &target_str);

                if let (Some(s), Some(e)) = (start_idx, end_idx) {
                    let (from, to) = if s <= e { (s, e) } else { (e, s) };
                    for p in &visible_paths[from..=to] {
                        if !session.selected_paths.contains(p) {
                            session.selected_paths.push(p.clone());
                        }
                    }
                    session.selected_path = Some(target_str);
                    state_changed = true;
                } else {
                    // Fallback if not found in visible list
                    if !session.selected_paths.contains(&target_str) {
                        session.selected_paths.push(target_str.clone());
                    }
                    session.selected_path = Some(target_str);
                    state_changed = true;
                }
            }
        }
        "select-all" | "select_all" => {
            let payload = build_gui_payload(session);
            let visible_paths: Vec<String> = payload.entries.into_iter().map(|e| e.path).collect();
            if !visible_paths.is_empty() {
                session.selected_paths = visible_paths;
                session.selected_path = session.selected_paths.last().cloned();
                state_changed = true;
            }
        }
        "select-clear" | "select_clear" => {
            session.selected_paths.clear();
            session.selected_path = None;
            state_changed = true;
        }
        "menu-run" | "menu_run" => {
            if args.len() > 2 {
                let action = &args[2];
                let target_path = args.get(3).cloned();

                let batch_paths = if let Some(ref path) = target_path {
                    if session.selected_paths.contains(path) {
                        session.selected_paths.clone()
                    } else {
                        vec![path.clone()]
                    }
                } else if !session.selected_paths.is_empty() {
                    session.selected_paths.clone()
                } else if let Some(ref sel) = session.selected_path {
                    vec![sel.clone()]
                } else {
                    Vec::new()
                };

                eprintln!("✓ Menú contextual ejecutado: '{}' sobre {:?} items", action, batch_paths.len());
                state_changed = true;
            }
        }
        "open-item" | "open_item" => {
            if args.len() > 2 {
                let target = PathBuf::from(&args[2]);
                let is_dir = target.is_dir() || fs::canonicalize(&target).map(|c| c.is_dir()).unwrap_or(false);
                let is_file = target.is_file() || fs::canonicalize(&target).map(|c| c.is_file()).unwrap_or(false);

                if is_dir {
                    let dir = if target.is_dir() { target } else { fs::canonicalize(&target).unwrap() };
                    for t in session.tabs.iter_mut() {
                        if t.id == session.active_tab_id {
                            t.path = dir.to_string_lossy().to_string();
                            t.title = dir
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "/".to_string());
                        }
                    }
                    session.selected_path = None;
                    state_changed = true;
                } else if is_file {
                    session.selected_path = Some(target.to_string_lossy().to_string());
                    state_changed = true;

                    // Open in floating editor on double-click without disabling sidebar preview
                    let preview = generate_preview_for_path(&target);
                    save_editor_state(&preview);
                    let _ = Command::new("eww").args(["open", "swal_editor"]).status();
                }
            }
        }
        "pin" | "add-pin" | "pin-add" => {
            let mut cfg = FileManagerConfig::load();
            let active_path = session.tabs.iter().find(|t| t.id == session.active_tab_id).map(|t| t.path.clone()).unwrap_or_else(home_path_string);
            let target_str = if args.len() > 2 {
                &args[2]
            } else {
                &active_path
            };
            let target = PathBuf::from(target_str);
            let name = args.get(3).cloned();
            let icon = args.get(4).cloned();
            cfg.add_pin(target, name, icon, Some("pinned".to_string()));
            let _ = cfg.save();
            state_changed = true;
        }
        "unpin" | "remove-pin" | "pin-remove" => {
            let mut cfg = FileManagerConfig::load();
            let active_path = session.tabs.iter().find(|t| t.id == session.active_tab_id).map(|t| t.path.clone()).unwrap_or_else(home_path_string);
            let target_str = if args.len() > 2 {
                &args[2]
            } else {
                &active_path
            };
            let target = PathBuf::from(target_str);
            cfg.remove_pin(&target);
            let _ = cfg.save();
            state_changed = true;
        }
        "pin-current" | "toggle-pin-current" | "toggle-pin" => {
            let mut cfg = FileManagerConfig::load();
            let active_path = session.tabs.iter().find(|t| t.id == session.active_tab_id).map(|t| t.path.clone()).unwrap_or_else(home_path_string);
            let target_str = if args.len() > 2 {
                &args[2]
            } else {
                &active_path
            };
            let target = PathBuf::from(target_str);
            cfg.toggle_pin(target);
            let _ = cfg.save();
            state_changed = true;
        }
        "open-editor" | "open_editor" | "editor" => {
            if args.len() > 2 {
                let target = PathBuf::from(&args[2]);
                if let Ok(canon) = fs::canonicalize(&target) {
                    let preview = generate_preview_for_path(&canon);
                    save_editor_state(&preview);
                    let _ = Command::new("eww").args(["open", "swal_editor"]).status();
                }
            }
            return Ok(None);
        }
        "set-group" | "set_group" | "group" => {
            if args.len() > 2 {
                session.group_by = args[2].to_lowercase();
                state_changed = true;
            }
        }
        "set-filter" | "set_filter" | "filter" => {
            if args.len() > 2 {
                let new_filter = args[2].to_lowercase();
                session.filter_type = new_filter.clone();
                // Remember per-path: what filter was last used in the active directory
                let active_path = session.tabs.iter()
                    .find(|t| t.id == session.active_tab_id)
                    .map(|t| t.path.clone())
                    .unwrap_or_default();
                if !active_path.is_empty() {
                    session.path_filter_memory.insert(active_path, new_filter);
                }
                state_changed = true;
            }
        }
        "clear-filter" | "clear_filter" | "filter-all" => {
            session.filter_type = "all".to_string();
            state_changed = true;
        }
        // Cycle through filters: all → folders → images → documents → code → media → archives → all
        "cycle-filter" | "cycle_filter" | "next-filter" => {
            session.filter_type = match session.filter_type.as_str() {
                "all"       => "folders".to_string(),
                "folders"   => "images".to_string(),
                "images"    => "documents".to_string(),
                "documents" => "code".to_string(),
                "code"      => "media".to_string(),
                "media"     => "archives".to_string(),
                _           => "all".to_string(),
            };
            state_changed = true;
        }
        // Save current filter+sort+group as a named preset
        "save-filter" | "save_filter" | "filter-save" => {
            if args.len() > 2 {
                use crate::config::SavedFilterPreset;
                let preset_name = args[2].clone();
                // Remove existing preset with same name
                session.saved_filter_presets.retain(|p| p.name != preset_name);
                session.saved_filter_presets.push(SavedFilterPreset {
                    name: preset_name.clone(),
                    filter_type: session.filter_type.clone(),
                    sort_by: session.sort_by.clone(),
                    sort_order: session.sort_order.clone(),
                    group_by: session.group_by.clone(),
                });
                eprintln!("✓ Filtro guardado como preset: \"{}\"", preset_name);
                state_changed = true;
            }
        }
        // Load a named preset
        "load-filter" | "load_filter" | "filter-load" => {
            if args.len() > 2 {
                let preset_name = &args[2];
                let preset = session.saved_filter_presets.iter().find(|p| &p.name == preset_name).cloned();
                if let Some(p) = preset {
                    session.filter_type = p.filter_type;
                    session.sort_by = p.sort_by;
                    session.sort_order = p.sort_order;
                    session.group_by = p.group_by;
                    eprintln!("✓ Preset de filtro cargado: \"{}\"", preset_name);
                    state_changed = true;
                } else {
                    eprintln!("⚠ Preset no encontrado: \"{}\"", preset_name);
                }
            }
        }
        // Delete a saved preset
        "delete-filter" | "delete_filter" | "filter-delete" => {
            if args.len() > 2 {
                let preset_name = &args[2];
                let before = session.saved_filter_presets.len();
                session.saved_filter_presets.retain(|p| &p.name != preset_name);
                if session.saved_filter_presets.len() < before {
                    eprintln!("✓ Preset eliminado: \"{}\"", preset_name);
                    state_changed = true;
                }
            }
        }
        "set-sort" | "set_sort" | "sort" => {
            if args.len() > 2 {
                session.sort_by = args[2].to_lowercase();
                if args.len() > 3 {
                    session.sort_order = args[3].to_lowercase();
                }
                state_changed = true;
            }
        }

        "set-preview-mode" | "set_preview_mode" => {
            if args.len() > 2 {
                session.preview_mode = args[2].to_lowercase();
                state_changed = true;
            }
        }
        "toggle-preview-mode" | "toggle_preview_mode" | "toggle-preview" | "toggle-sidebar" => {
            session.preview_mode = match session.preview_mode.as_str() {
                "sidebar" => "none".to_string(),
                _ => "sidebar".to_string(),
            };
            state_changed = true;
        }
        // Densidad de filas de la lista: compact (mas filas visibles) <-> comfortable
        "toggle-density" | "toggle_density" => {
            session.row_density = match session.row_density.as_str() {
                "compact" => "comfortable".to_string(),
                _ => "compact".to_string(),
            };
            state_changed = true;
        }
        "set-density" | "set_density" => {
            if args.len() > 2 {
                session.row_density = match args[2].to_lowercase().as_str() {
                    "compact" | "compacta" => "compact".to_string(),
                    _ => "comfortable".to_string(),
                };
                state_changed = true;
            }
        }
        // Wrap del panel de preview: ON = las lineas largas se ajustan a la ventana
        "toggle-wrap" | "toggle_wrap" | "wrap" => {
            session.preview_wrap = !session.preview_wrap;
            state_changed = true;
        }
        "set-wrap" | "set_wrap" => {
            if args.len() > 2 {
                session.preview_wrap = matches!(args[2].to_lowercase().as_str(), "on" | "true" | "1");
                state_changed = true;
            }
        }
        // Ancho de columna de la lista (en caracteres), ajustable con scroll en la cabecera.
        // Uso: swal-files col-width <name|date|type|size> <up|down>
        "col-width" | "col_width" | "column-width" => {
            if args.len() > 3 {
                let col = args[2].to_lowercase();
                let dir = args[3].to_lowercase();
                let (min, max) = match col.as_str() {
                    "name" => (10i64, 60i64),
                    "date" => (6, 16),
                    "type" => (4, 14),
                    "size" => (4, 12),
                    _ => (4, 60),
                };
                let cur = session.col_chars.get(&col).copied().unwrap_or(20);
                let next = match dir.as_str() {
                    "up" | "+" | "grow" | "more" => (cur + 2).min(max),
                    "down" | "-" | "shrink" | "less" => (cur - 2).max(min),
                    _ => cur,
                };
                session.col_chars.insert(col.clone(), next);
                eprintln!("✓ columna {} -> {} chars (rango {}-{})", col, next, min, max);
                state_changed = true;
            }
        }
        "col-reset" | "col_reset" | "columns-reset" => {
            session.col_chars = crate::session::default_col_chars();
            eprintln!("✓ anchos de columna restaurados");
            state_changed = true;
        }
        "toggle-maximize" | "toggle_maximize" | "maximize" => {
            session.is_maximized = !session.is_maximized;
            state_changed = true;
            // Zero-Eww: the live --gui process re-reads session on SIGUSR1
            // (window state is persisted by save_session below). No eww calls.
        }
        // Fija el flag sin ambiguedad (lo usa swal_files_maximize.sh, que decide por la
        // ventana realmente abierta y luego sincroniza el flag).
        "set-maximize" | "set_maximize" => {
            if args.len() > 2 {
                session.is_maximized =
                    matches!(args[2].to_lowercase().as_str(), "on" | "true" | "1");
                state_changed = true;
            }
        }
        "tab-new" | "tab_new" => {
            let home = home_path_string();
            let next_id = session.tabs.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            session.tabs.push(TabState {
                id: next_id,
                title: "Home".to_string(),
                path: home,
                active: true,
            });
            session.active_tab_id = next_id;
            state_changed = true;
        }
        "tab-close" | "tab_close" => {
            if args.len() > 2 {
                if let Ok(id) = args[2].parse::<usize>() {
                    if session.tabs.len() > 1 {
                        session.tabs.retain(|t| t.id != id);
                        if session.active_tab_id == id {
                            session.active_tab_id = session.tabs[0].id;
                        }
                        state_changed = true;
                    }
                }
            }
        }
        "tab-switch" | "tab_switch" => {
            if args.len() > 2 {
                if let Ok(id) = args[2].parse::<usize>() {
                    if session.tabs.iter().any(|t| t.id == id) {
                        session.active_tab_id = id;
                        state_changed = true;
                    }
                }
            }
        }
        "toggle-hidden" | "toggle_hidden" => {
            session.show_hidden = !session.show_hidden;
            state_changed = true;
        }
        "toggle-view" | "toggle_view" => {
            session.view_mode = match session.view_mode.as_str() {
                "details" => "grid".to_string(),
                _ => "details".to_string(),
            };
            state_changed = true;
        }
        "omnibar" => {
            if args.len() > 2 {
                let input = &args[2];
                let current = PathBuf::from(
                    &session
                        .tabs
                        .iter()
                        .find(|t| t.id == session.active_tab_id)
                        .map(|t| t.path.clone())
                        .unwrap_or_else(home_path_string),
                );

                let parsed = parse_omnibar_input(input, &current);
                match parsed {
                    OmnibarIntent::Navigate(target) => {
                        if target.exists() && target.is_dir() {
                            for t in session.tabs.iter_mut() {
                                if t.id == session.active_tab_id {
                                    t.path = target.to_string_lossy().to_string();
                                    t.title = target
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| "/".to_string());
                                }
                            }
                            session.search_query.clear();
                            state_changed = true;
                        }
                    }
                    OmnibarIntent::SearchQuery(query) => {
                        session.search_query = query;
                        state_changed = true;
                    }
                    OmnibarIntent::AgentPrompt(prompt) => {
                        let _ = Command::new("notify-send")
                            .args(["SWAL Agent Prompt", &prompt])
                            .status();
                    }
                    OmnibarIntent::Command(cmd_name) => match cmd_name.as_str() {
                        "hidden" => {
                            session.show_hidden = !session.show_hidden;
                            state_changed = true;
                        }
                        "view" => {
                            session.view_mode = match session.view_mode.as_str() {
                                "details" => "grid".to_string(),
                                _ => "details".to_string(),
                            };
                            state_changed = true;
                        }
                        "quit" | "q" => {
                            // Zero-Eww: signal the live --gui process to exit.
                            if let Ok(content) = fs::read_to_string(PID_FILE) {
                                if let Ok(pid) = content.trim().parse::<i32>() {
                                    unsafe { libc::kill(pid, libc::SIGTERM); }
                                }
                            }
                            // Clean up PID file to prevent zombie detection
                            remove_pid_file();
                            return Ok(None);
                        }
                        _ => {}
                    },
                }
            }
        }
        "reset-session" | "reset" => {
            // Emergency reset: navigate home, clear all filters/groups/search
            let home = home_path_string();
            let home_title = "Home".to_string();
            for t in session.tabs.iter_mut() {
                if t.id == session.active_tab_id {
                    t.path = home.clone();
                    t.title = home_title.clone();
                }
            }
            session.filter_type = "all".to_string();
            session.group_by = "none".to_string();
            session.search_query.clear();
            session.selected_path = None;
            session.path_filter_memory.clear();
            eprintln!("✓ Sesión reseteada → Home, filtro: all, grupo: none");
            state_changed = true;
        }
        "rename-item" | "rename_item" | "rename"
        | "new-folder" | "new_folder" | "mkdir"
        | "duplicate-item" | "duplicate_item" | "duplicate" => {
            if let Some(msg) = crate::file_ops::handle_cli(args, session) {
                save_session(session);
                let payload = build_gui_payload(session);
                notify_eww_update(&payload);
                return Ok(Some(msg));
            }
        }
        _ => {
            open_gui(Some(cmd));
        }
    }

    if state_changed {
        save_session(session);
        let payload = build_gui_payload(session);
        notify_eww_update(&payload);
    }

    Ok(None)
}

pub fn run_cli(args: &[String]) {
    let mut session = load_session();
    match handle_command(&mut session, args) {
        Ok(Some(output)) if output.starts_with("Error") => {
            eprintln!("{}", output);
            std::process::exit(1);
        }
        Ok(Some(output)) => println!("{}", output),
        Ok(None) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
