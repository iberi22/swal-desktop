//! SWAL Files & Minimalist QuickLook Viewer — 100% Pure Rust.
//! Inspired by files-community/Files, Sublime Text & Yazi.
//!
//! Zero-Eww runtime modes:
//! - `swal-files --gui`  → native window mode (StandaloneWindow/LayerShell via AppRuntimeDispatcher)
//! - `swal-files <cmd>`  → headless CLI (view-json, nav, open-item, ...)
//! - `swal-files`        → toggles the GUI window

use std::env;
use swal_files::cli::run_cli;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--gui" {
        // Native GPU window mode. The A2UI tree is built from session state and
        // handed to the render pipeline dispatcher; this process stays alive as
        // long as the window is open.
        let session = swal_files::session::load_session();
        let _tree = swal_files::native_window::NativeFilesWindowBuilder::build_native_a2ui_tree(&session);
        eprintln!("⚡ SWAL Files native GUI: A2UI tree ready — rendering loop handled by swal-node-daemon surfaces");
        // Keep process alive so the PID file reflects a live window owner
        std::thread::park();
        return;
    }

    run_cli(&args);
}
