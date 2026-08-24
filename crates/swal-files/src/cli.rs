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
const EWWSOCK: &str = "/tmp/eww.sock";

/// Check if an existing swal-files instance is running via PID file
fn is_instance_running() -> bool {
    if let Ok(content) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            // Check if process is still alive (kill signal 0)
            let alive = unsafe { libc::kill(pid as i32, 0) == 0 };
            if alive {
                return true;
            }
            // Stale PID file — clean it up
            let _ = fs::remove_file(PID_FILE);
        }
    }
    false
}

/// Write our PID to the lock file
fn write_pid_file() {
    let _ = fs::write(PID_FILE, std::process::id().to_string());
}

/// Remove PID file on clean exit
fn remove_pid_file() {
    let _ = fs::remove_file(PID_FILE);
}

/// Check if EWW daemon is responsive via IPC ping
fn eww_daemon_alive() -> bool {
    let ping = Command::new("eww").arg("ping").output();
    match ping {
        Ok(out) if out.status.success() => true,
        _ => {
            eprintln!("⚠ EWW daemon not responding, starting...");
            start_eww_daemon();
            true
        }
    }
}

/// Start EWW daemon in background
fn start_eww_daemon() {
    let _ = Command::new("eww").arg("daemon").spawn();
    // Give it a moment to initialize
    std::thread::sleep(std::time::Duration::from_millis(400));
}

/// Send notification (works on Linux with notify-send, silent on other platforms)
fn send_notification(summary: &str, body: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send")
            .args([
                "--icon=folder-open",
                "--app-name=SWAL Files",
                "-t",
                "2000",
                summary,
                body,
            ])
            .status();
    }
}

/// Clean up dead layer shells if their PID no longer exists in /proc
fn kill_orphan_layer_shells() {
    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = Command::new("hyprctl").arg("layers").output() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("gtk-layer-shell") && line.contains("pid:") {
                    if let Some(pid_str) = line.split("pid:").nth(1) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            let proc_path = format!("/proc/{}", pid);
                            if !Path::new(&proc_path).exists() {
                                eprintln!("🧹 Layer shell PID {} is dead, cleaning up", pid);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Kill orphaned EWW window processes that the daemon lost track of
fn cleanup_orphan_windows() {
    #[cfg(target_os = "linux")]
    {
        let patterns = ["eww open swal_files", "eww open swal_editor"];
        for pattern in &patterns {
            if let Ok(out) = Command::new("pgrep").args(["-f", pattern]).output() {
                let pids = String::from_utf8_lossy(&out.stdout);
                for pid_str in pids.lines() {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        eprintln!("🧹 Killing orphaned EWW window PID {} ({})", pid, pattern);
                        unsafe { libc::kill(pid as i32, libc::SIGKILL); }
                    }
                }
            }
        }
        // Clean stale PID file
        if let Ok(content) = fs::read_to_string(PID_FILE) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if unsafe { libc::kill(pid as i32, 0) } != 0 {
                    let _ = fs::remove_file(PID_FILE);
                }
            }
        }
    }
}

const VISIBLE_FLAG: &str = "/tmp/swal_files_visible.flag";

pub fn is_window_open() -> bool {
    if Path::new(VISIBLE_FLAG).exists() {
        return true;
    }
    if let Ok(out) = Command::new("hyprctl").arg("layers").output() {
        let text = String::from_utf8_lossy(&out.stdout);
        text.contains("1080 660") || text.contains("swal_files")
    } else {
        false
    }
}

pub fn close_gui() {
    let _ = Command::new("eww").args(["close", "swal_files"]).status();
    let _ = Command::new("eww").args(["close", "swal_files_maximized"]).status();
    let _ = fs::remove_file(VISIBLE_FLAG);
    remove_pid_file();
}

pub fn open_gui(target_path: Option<&str>) {
    // If opening without arguments and window is already open, toggle close it
    if target_path.is_none() && is_window_open() {
        close_gui();
        return;
    }

    let mut session = load_session();

    // Handle target path — add as new tab or switch to existing tab
    if let Some(target) = target_path {
        let p = PathBuf::from(target);
        if let Ok(canon) = fs::canonicalize(&p) {
            let path_str = canon.to_string_lossy().to_string();
            let title = canon
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());

            // Check if this path already has a tab
            let mut found = false;
            for t in session.tabs.iter_mut() {
                if t.path == path_str {
                    session.active_tab_id = t.id;
                    found = true;
                    break;
                }
            }

            if !found {
                let next_id = session.tabs.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                session.tabs.push(TabState {
                    id: next_id,
                    title,
                    path: path_str,
                    active: true,
                });
                session.active_tab_id = next_id;
            }
        }
    }

    save_session(&session);

    // Kill any orphan layer shells from previous daemon crashes
    kill_orphan_layer_shells();

    // Ensure EWW daemon is alive before doing anything
    eww_daemon_alive();

    // Check if another swal-files instance is already managing the window
    let already_running = is_instance_running();

    if already_running && is_window_open() {
        // Another instance is handling it — just update data
        if let Some(target) = target_path {
            let display = Path::new(target)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| target.to_string());
            send_notification("📂 SWAL Files", &format!("Abriendo en nueva pestaña: {}", display));
        }
        let payload = build_gui_payload(&session);
        notify_eww_update(&payload);
    } else {
        // We are the primary instance — claim the PID file and open the window
        write_pid_file();
        let _ = fs::write(VISIBLE_FLAG, "1");

        // Clean up any orphaned EWW window processes before opening
        cleanup_orphan_windows();

        let target_win = if session.is_maximized {
            "swal_files_maximized"
        } else {
            "swal_files"
        };
        let _ = Command::new("eww").args(["open", target_win]).status();
    }
}

pub fn handle_command(session: &mut SessionState, args: &[String]) -> Result<Option<String>, String> {
    if args.len() < 2 {
        open_gui(None);
        return Ok(None);
    }

    let cmd = args[1].as_str();

    // Direct path argument
    if cmd.starts_with('/') || cmd.starts_with('~') || Path::new(cmd).exists() {
        open_gui(Some(cmd));
        return Ok(None);
    }

    let mut state_changed = false;

    match cmd {
        "view-json" | "view_json" | "json" => {
            let payload = build_gui_payload(session);
            return Ok(Some(serde_json::to_string(&payload).map_err(|e| e.to_string())?));
        }
        "editor-json" | "editor_json" => {
            let editor = load_editor_state();
            return Ok(Some(serde_json::to_string(&editor).map_err(|e| e.to_string())?));
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
                session.selected_path = Some(target.to_string_lossy().to_string());
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
            let active_path = session.tabs.iter().find(|t| t.id == session.active_tab_id).map(|t| t.path.clone()).unwrap_or_else(|| "/home/belal".to_string());
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
            let active_path = session.tabs.iter().find(|t| t.id == session.active_tab_id).map(|t| t.path.clone()).unwrap_or_else(|| "/home/belal".to_string());
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
            let active_path = session.tabs.iter().find(|t| t.id == session.active_tab_id).map(|t| t.path.clone()).unwrap_or_else(|| "/home/belal".to_string());
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
                session.filter_type = args[2].to_lowercase();
                state_changed = true;
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
        "toggle-maximize" | "toggle_maximize" | "maximize" => {
            session.is_maximized = !session.is_maximized;
            state_changed = true;

            let active_win = Command::new("eww").arg("active-windows").output();
            let is_open = if let Ok(out) = active_win {
                let text = String::from_utf8_lossy(&out.stdout);
                text.contains("swal_files")
            } else {
                false
            };

            if is_open {
                let _ = Command::new("eww").args(["close", "swal_files"]).status();
                let _ = Command::new("eww").args(["close", "swal_files_maximized"]).status();
                let next_win = if session.is_maximized {
                    "swal_files_maximized"
                } else {
                    "swal_files"
                };
                let _ = Command::new("eww").args(["open", next_win]).status();
            }
        }
        "tab-new" | "tab_new" => {
            let home = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/home/belal"))
                .to_string_lossy()
                .to_string();
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
                        .unwrap_or_else(|| "/home/belal".to_string()),
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
                            let _ = Command::new("eww").args(["close", "swal_files"]).status();
                            let _ = Command::new("eww").args(["close", "swal_files_maximized"]).status();
                            // Clean up PID file to prevent zombie detection
                            remove_pid_file();
                            return Ok(None);
                        }
                        _ => {}
                    },
                }
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
        Ok(Some(output)) => println!("{}", output),
        Ok(None) => {},
        Err(e) => eprintln!("Error: {}", e),
    }
}
