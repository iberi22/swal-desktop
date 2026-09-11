# [Ola S4.01] fix(native-renderer): Fix black screen — configure flag + pixel order + robust font

> Ola S4 — Native Rust File Manager Renderer (Zero-EWW path).

## Context

`swal-files --gui-force` opens a native Wayland window using `wl_shm + ab_glyph` (smithay-client-toolkit 0.19). The window renders **completely black**. This is the wave blocker.

**Repo**: `https://github.com/iberi22/swal-desktop`  
**Primary file**: `crates/swal-files/src/native_window_app.rs` (660 lines, already committed)

## Current State (MEASURABLE)

- `run_native_window()` lines 55-121: calls `draw_frame()` on line 109 BEFORE `blocking_dispatch()` — but `configured=false` so `draw_frame()` returns immediately (line 223-224 guard)
- `WindowHandler::configure()` fires AFTER first `blocking_dispatch()` — it must set `configured=true` AND `redraw=true` AND update `width`/`height`
- Event loop lines 111-116: calls `draw_frame()` then sets `redraw=false`, THEN calls `blocking_dispatch()` — if no more events arrive, the process blocks with black window
- Font: `load_font()` hardcodes `/nix/store/ang6yzsv32vnkdq7bqr41dgna2knkz8w-dejavu-fonts-minimal-2.37/...` — panics if this hash changes
- Pool: `SlotPool::new(w*h*4)` created once — if window resizes to larger, pool too small → corruption

## Desired State (DELTA)

**Fix 1 — Event loop order** (lines 108-117):
```rust
// AFTER this fix:
event_queue.blocking_dispatch(&mut app).unwrap(); // receives configure → sets configured=true
app.draw_frame(); // NOW configured=true, draws first real frame
while !app.exit {
    if app.redraw {
        app.draw_frame();
        app.redraw = false;
    }
    event_queue.blocking_dispatch(&mut app).unwrap();
}
```

**Fix 2 — configure() handler** (around line 460, inside `impl WindowHandler for SwalFilesApp`):
```rust
fn configure(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>,
             _window: &Window, configure: WindowConfigure, _serial: u32) {
    if let Some((w, h)) = configure.new_size {
        let new_w = w.get();
        let new_h = h.get();
        if new_w != self.width || new_h != self.height {
            self.pool = None; // force pool resize
        }
        self.width = new_w;
        self.height = new_h;
    }
    self.configured = true;
    self.redraw = true;
}
```

**Fix 3 — Font loading** (lines 127-147): Replace hardcoded path with dynamic search:
```rust
fn load_font() -> FontArc {
    // Priority: system fonts via /run/current-system (NixOS), then common paths
    let candidates: Vec<std::path::PathBuf> = {
        let mut v = vec![];
        // NixOS: scan /run/current-system/sw/share/fonts recursively (small dir)
        if let Ok(entries) = std::fs::read_dir("/run/current-system/sw/share/fonts") {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().map(|x| x == "ttf").unwrap_or(false) {
                    v.push(p);
                }
            }
        }
        // Common fallbacks
        v.push("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into());
        v.push("/usr/share/fonts/TTF/DejaVuSans.ttf".into());
        v
    };
    for c in &candidates {
        if let Ok(data) = std::fs::read(c) {
            if let Ok(font) = FontArc::try_from_vec(data) {
                if font.glyph_id('H').0 != 0 { return font; }
            }
        }
    }
    // Last resort: embed a minimal fallback font or panic with useful message
    panic!("No valid TTF font found. Searched: {:?}", candidates);
}
```

**Fix 4 — Pool size guard** in `draw_frame()` before the pool block (around line 339):
```rust
let needed = w * h * 4;
if self.pool.as_ref().map(|p| p.len() < needed).unwrap_or(false) {
    self.pool = None;
}
```

## Web Research Required

1. search: "smithay-client-toolkit 0.19 WindowConfigure new_size NonZeroU32 configure handler"
2. search: "wayland xdg_toplevel black window first frame configure serial wl_shm"
3. search: "SlotPool len capacity smithay-client-toolkit shm slot"

## Acceptance Criteria (VERIFIABLE BY COMMAND)

- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo build -p swal-files` — Finished without error
- [ ] `grep -n "self.configured = true" crates/swal-files/src/native_window_app.rs` — match inside configure() handler
- [ ] `grep -n "self.pool = None" crates/swal-files/src/native_window_app.rs` — >= 1 match for resize guard
- [ ] `grep -n "blocking_dispatch" crates/swal-files/src/native_window_app.rs` — the draw_frame() call appears AFTER the first blocking_dispatch, not before

## Files to Modify

| File | Lines | Change | Risk |
|------|-------|--------|------|
| `crates/swal-files/src/native_window_app.rs` | 660 | Fix event loop order, configure handler, load_font, pool guard | MEDIUM |

## DO NOT touch

- `crates/swal-files/src/cli.rs` — updated in commit 1635ab8
- `crates/swal-files/src/main.rs` — updated in commit 1635ab8  
- `crates/swal-files/src/gui.rs` — EWW path, separate concern
- Any other crate

## Anti-Hallucination Guard

1. READ the full `native_window_app.rs` before writing — it is 660 lines
2. Find the existing `configure()` impl inside `impl WindowHandler for SwalFilesApp` — do NOT add a duplicate
3. Check `WindowConfigure` fields in smithay-client-toolkit 0.19 docs before using

## Merge Order

- **Merge FIRST** — blocker for all other S4 issues
- **Expected effort:** Medium (45-90m)
- **Parallel with:** NONE
