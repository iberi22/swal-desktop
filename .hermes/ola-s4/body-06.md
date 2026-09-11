# [Ola S4.06] feat(native-renderer): SIGUSR1 session sync + multi-tab keyboard + toggle fix

> Ola S4 — Native Rust File Manager Renderer. Depends on S4.01 + S4.04 + S4.05 merged.

## Context

The native renderer runs as a standalone process but doesn't communicate with the CLI subsystem. When `swal-files nav /path` is called from outside while `--gui-force` is running, nothing happens. This issue connects the native window to the session ecosystem.

## Current State (MEASURABLE)

- `run_native_window()`: writes PID to `/tmp/swal-files.pid` (line 105) but installs no signal handler for SIGUSR1
- No tab keyboard shortcuts (`t`, `w`, `[1-9]`, `Tab`)
- `swal-files --gui-force` called while already running: creates a second window instead of focusing existing
- `save_current_path()` (line 361): only called on quit — path not persisted during navigation

## Desired State (DELTA)

**1. SIGUSR1 handler** — Install signal handler on startup to reload session:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
static RELOAD_SESSION: AtomicBool = AtomicBool::new(false);

// Before event loop in run_native_window():
unsafe {
    libc::signal(libc::SIGUSR1, handle_sigusr1 as libc::sighandler_t);
}

extern "C" fn handle_sigusr1(_: libc::c_int) {
    RELOAD_SESSION.store(true, Ordering::Relaxed);
}

// In event loop, before draw_frame:
if RELOAD_SESSION.swap(false, Ordering::Relaxed) {
    let session = load_session();
    // Update current path from active tab
    if let Some(tab) = session.tabs.iter().find(|t| t.id == session.active_tab_id) {
        let p = PathBuf::from(&tab.path);
        if p.exists() && p != app.current_path {
            app.current_path = p;
            app.reload_dir();
        }
    }
    app.tabs = session.tabs.clone();
    app.redraw = true;
}
```

**2. Multi-tab keyboard shortcuts**:
In the keyboard char handler, add:
```
't' → create new tab: load_session(), add TabState with current_path, save_session(), reload tabs field, redraw
'w' → close current tab: if tabs.len() > 1, remove active tab from session, switch to previous, nav there
Tab → switch to next tab (cycle): session.active_tab_id = next_tab.id, nav to its path
'1'..'9' → switch to tab by index (1-based)
```

**3. Multi-tab display** — Update tab strip drawing (from J-2) to use `self.tabs: Vec<TabState>` and `self.active_tab_id: u32` fields (add to struct).

**4. Toggle fix for `--gui-force`** in `main.rs`:
Update the `--gui-force` branch:
```rust
if args.len() > 1 && args[1] == "--gui-force" {
    // Check if already running
    if let Ok(pid_str) = std::fs::read_to_string("/tmp/swal-files.pid") {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check process is alive
            if unsafe { libc::kill(pid, 0) } == 0 {
                // Send SIGUSR1 to reload session (focus/bring to front)
                unsafe { libc::kill(pid, libc::SIGUSR1); }
                return;
            }
        }
    }
    swal_files::native_window_app::run_native_window();
    return;
}
```

**5. Persist path on navigation** — Call `save_current_path()` every time `current_path` changes (in `open_selected()` and `go_up()` after `reload_dir()`).

**6. Update `cli.rs` `open_gui()`** — When `open_gui(Some(path))` is called and there's a running `--gui-force` process (pid file exists and process alive), send SIGUSR1 instead of opening EWW:
```rust
// In open_gui(), at the top:
if let Ok(pid_str) = std::fs::read_to_string("/tmp/swal-files.pid") {
    if let Ok(pid) = pid_str.trim().parse::<i32>() {
        if unsafe { libc::kill(pid, 0) } == 0 {
            // Native window is running — navigate it there
            if let Some(target) = target_path {
                // Update session, then signal
                // ... (update session code from existing open_gui)
                let _ = save_session(&session);
            }
            unsafe { libc::kill(pid, libc::SIGUSR1); }
            return;
        }
    }
}
// Fall through to EWW toggle
```

## Web Research Required

1. search: "Rust libc SIGUSR1 signal handler AtomicBool safe async signal"
2. search: "smithay-client-toolkit event loop check atomic flag between dispatches"

## Acceptance Criteria (VERIFIABLE BY COMMAND)

- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -n "SIGUSR1\|RELOAD_SESSION" crates/swal-files/src/native_window_app.rs` — signal handler present
- [ ] `grep -n "active_tab_id\|self.tabs" crates/swal-files/src/native_window_app.rs` — tab fields in struct
- [ ] `grep -n "save_current_path" crates/swal-files/src/native_window_app.rs` — called in open_selected AND go_up
- [ ] `grep -n "libc::kill.*SIGUSR1\|SIGUSR1.*kill" crates/swal-files/src/main.rs` — toggle fix in main.rs

## Files to Modify

| File | Lines | Change | Risk |
|------|-------|--------|------|
| `crates/swal-files/src/native_window_app.rs` | 660 | Signal handler, tab struct fields, keyboard shortcuts | MEDIUM |
| `crates/swal-files/src/main.rs` | 52 | Toggle check before launching native window | LOW |
| `crates/swal-files/src/cli.rs` | 517 | SIGUSR1 path in open_gui() when native process running | LOW |

## DO NOT touch

- `crates/swal-files/src/session.rs` — read only
- Other crates

## Anti-Hallucination Guard

1. Use `libc` crate for signal handling — it is already in Cargo.toml
2. The AtomicBool signal handler is the safest approach — avoid mutexes in signal handlers
3. READ `crates/swal-files/src/session.rs` to find `TabState` struct fields before using

## Merge Order

- **LAST in wave** — integrates work from J-1, J-2, J-4, J-5
- **Expected effort:** Medium (60-90m)
