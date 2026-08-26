//! SWAL Files & Minimalist QuickLook Viewer — 100% Pure Rust.
//! Inspired by files-community/Files, Sublime Text & Yazi.
//!
//! Modes:
//! - `swal-files --gui`       → tries native Wayland window; falls back to EWW overlay
//! - `swal-files --gui-force` → native Wayland window only (debug/dev)
//! - `swal-files <cmd>`       → headless CLI (view-json, nav, open-item, ...)
//! - `swal-files`             → toggles the EWW window open/closed

use std::env;
use std::process::Command;
use swal_files::cli::run_cli;

fn main() {
    let args: Vec<String> = env::args().collect();

    // --gui-force: bypass EWW and go straight to native Wayland renderer (dev mode)
    if args.len() > 1 && args[1] == "--gui-force" {
        swal_files::native_window_app::run_native_window();
        return;
    }

    // --gui: EWW is the stable fallback while native renderer is WIP.
    // The native wl_shm + ab_glyph renderer renders black (xdg-toplevel configure
    // loop not completing frame commits reliably). EWW overlay works 100%.
    if args.len() > 1 && args[1] == "--gui" {
        open_eww_with_warning();
        return;
    }

    run_cli(&args);
}

/// Opens the EWW swal_files overlay and warns the user that the native
/// renderer is still WIP (so they know why ghostty isn't launching).
fn open_eww_with_warning() {
    // Fire notify-send in background — don't block EWW open
    let _ = Command::new("notify-send")
        .args([
            "--urgency=normal",
            "--icon=dialog-warning",
            "--app-name=SWAL Files",
            "--expire-time=5000",
            "SWAL Files — Modo EWW",
            "El renderer nativo (wl_shm) aún no está completo.\nUsando EWW overlay como fallback estable.",
        ])
        .spawn();

    // Toggle the EWW swal_files window
    let _ = Command::new("eww")
        .args(["open", "--toggle", "swal_files"])
        .spawn();
}