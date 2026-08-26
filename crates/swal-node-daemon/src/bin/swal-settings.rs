//! `swal-settings` — thin binary wrapper around settings_cli::cli_main().
//! Replaces the legacy eww/scripts/swal_settings.py Python backend.
use swal_node_daemon::settings_cli::cli_main;

fn main() {
    cli_main();
}
