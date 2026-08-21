//! SWAL Files & Minimalist QuickLook Viewer — 100% Pure Rust.
//! Inspired by files-community/Files, Sublime Text & Yazi.

use std::env;
use swal_files::cli::run_cli;

fn main() {
    let args: Vec<String> = env::args().collect();
    run_cli(&args);
}
