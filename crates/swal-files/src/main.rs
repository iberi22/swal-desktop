//! SWAL Files & Minimalist QuickLook Viewer — 100% Pure Rust.
//! Inspired by files-community/Files, Sublime Text & Yazi.
//!
//! Zero-Eww runtime modes:
//! - `swal-files --gui`  → full-screen TUI file manager (crossterm raw mode)
//!                         inside a floating terminal window. Real keys
//!                         (j/k/enter/backspace/tab/p/ctrl+t), live resize.
//!                         Registers its PID so `swal-files` toggles it closed.
//! - `swal-files <cmd>`  → headless CLI (view-json, nav, open-item, ...)
//! - `swal-files`        → toggles the GUI window open/closed

use std::env;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};

use swal_files::cli::run_cli;
use swal_files::session::load_session;
use swal_files::tui::{TuiFileManagerApp, TuiViewport};

const PID_FILE: &str = "/tmp/swal-files.pid";
const VISIBLE_FLAG: &str = "/tmp/swal_files_visible.flag";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--gui" {
        run_gui_mode();
        return;
    }

    run_cli(&args);
}

/// Native GUI mode: crossterm raw-mode TUI file manager.
/// Writes the PID file so the toggle logic (`swal-files`, `close-files`) can
/// find and terminate us. Removes it on any exit path (SIGTERM handler + guard).
fn run_gui_mode() {
    let pid = std::process::id();
    let _ = std::fs::write(PID_FILE, pid.to_string());
    let _ = std::fs::write(VISIBLE_FLAG, "1");

    // SIGTERM (from `swal-files` toggle / close-files) → clean exit.
    unsafe {
        libc::signal(libc::SIGTERM, on_sigterm as extern "C" fn(i32) as usize);
    }

    // Recover last-visited path from session state
    let session = load_session();
    let initial_path: PathBuf = session
        .tabs
        .iter()
        .find(|t| t.id == session.active_tab_id)
        .map(|t| PathBuf::from(&t.path))
        .filter(|p| p.exists())
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

    let mut app = TuiFileManagerApp::new(&initial_path);

    if enable_raw_mode().is_err() {
        eprintln!("⚠ Raw mode unavailable — run inside a terminal (ghostty/kitty).");
        cleanup_files();
        return;
    }
    let mut out = stdout();
    let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));

    'main_loop: loop {
        // Render frame sized to actual terminal (guard against 0x0 reports
        // from exotic PTYs — fall back to a sane default viewport).
        let viewport = match crossterm::terminal::size() {
            Ok((w, h)) if w >= 20 && h >= 6 => TuiViewport {
                width: w,
                height: h,
                ..Default::default()
            },
            _ => TuiViewport {
                width: 100,
                height: 30,
                ..Default::default()
            },
        };
        let frame = app.render_to_buffer(&viewport);
        let _ = execute!(out, MoveTo(0, 0), Clear(ClearType::All), Print(frame));
        let _ = out.flush();

        if !event::poll(std::time::Duration::from_millis(250)).unwrap_or(false) {
            continue;
        }
        let ev = match event::read() {
            Ok(e) => e,
            Err(_) => continue,
        };
        if let Event::Key(k) = ev {
            let key = match k.code {
                KeyCode::Char('q') => "q".to_string(),
                KeyCode::Char('?') => "?".to_string(),
                KeyCode::Char('j') => "j".to_string(),
                KeyCode::Char('k') => "k".to_string(),
                KeyCode::Char('h') => "h".to_string(),
                KeyCode::Char('l') => "l".to_string(),
                KeyCode::Char('p') => "p".to_string(),
                KeyCode::Down => "j".to_string(),
                KeyCode::Up => "k".to_string(),
                KeyCode::Left => "h".to_string(),
                KeyCode::Right => "l".to_string(),
                KeyCode::Tab => "tab".to_string(),
                KeyCode::Enter => "enter".to_string(),
                KeyCode::Backspace => "backspace".to_string(),
                KeyCode::Esc => "q".to_string(),
                KeyCode::Char('t') if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                    "ctrl-t".to_string()
                }
                _ => continue,
            };

            let action = if key == "ctrl-t" {
                app.handle_key_event("t", true, false)
            } else {
                app.handle_key_event(&key, false, false)
            };
            match action {
                swal_files::tui::TuiActionResponse::Quit => break 'main_loop,
                swal_files::tui::TuiActionResponse::OpenedFile(path) => {
                    // Open file with default handler (xdg-open); stay in manager
                    let _ = Command::new("xdg-open").arg(path).spawn();
                }
                _ => {}
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0));
    cleanup_files();
}

extern "C" fn on_sigterm(_sig: i32) {
    let _ = disable_raw_mode();
    cleanup_files();
    std::process::exit(0);
}

fn cleanup_files() {
    let _ = std::fs::remove_file(PID_FILE);
    let _ = std::fs::remove_file(VISIBLE_FLAG);
}