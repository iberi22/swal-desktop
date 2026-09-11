# [Ola S4.02] feat(native-renderer): Cyber-neon palette + complete 3-column layout

> Ola S4 — Native Rust File Manager Renderer. Depends on S4.01 merged.

## Context

After J-1 fixes the black screen, this issue upgrades the visual quality of `native_window_app.rs`:
- Replace the wrong/placeholder color palette with the actual cyber-neon SWAL theme tokens
- Draw a complete tab strip, toolbar with breadcrumbs, and proper sidebar from session data
- Add drawing helpers: `draw_line_h`, `draw_separator_v`, `draw_badge`

## Current State (MEASURABLE)

- Color constants (lines ~47-53): `BG=0xFF020617`, `ELEVATED=0xFF0f172a`, `ACCENT=0xFF06b6d4` (wrong: EWW uses green `#00FF88` as primary accent, cyan as secondary)
- Sidebar (lines 238-264): hardcoded labels `["📌 Home", "📁 Descargas", "📁 Documentos", "📂 Proyectos SWAL", "🖴 /"]` — ignores `session.favorites` and `session.workspaces`
- No tab strip drawn — only file list header text
- No toolbar / breadcrumb strip drawn
- Footer (lines 327-334): minimal hint text

## Desired State (DELTA)

**1. Color palette** — replace constants:
```rust
const BG: u32       = 0xFF0A0F1D;  // rgba(10, 15, 29, 1) — main bg
const ELEVATED: u32 = 0xFF111827;  // sidebar/card surface
const ACCENT: u32   = 0xFF00FF88;  // cyber-neon green (primary)
const ACCENT2: u32  = 0xFF00CCFF;  // cyan (secondary)
const DANGER: u32   = 0xFFFF4444;
const WARNING: u32  = 0xFFFFBB00;
const SUCCESS: u32  = 0xFF00FF88;  // same as ACCENT
const TEXT: u32     = 0xFFE2E8F0;
const TEXT_DIM: u32 = 0xFF94A3B8;
const SELECTED: u32 = 0xFF1A3D2B;  // semi-dark green highlight (no real alpha in wl_shm)
const BORDER: u32   = 0xFF1E2A3A;  // subtle border
```

**2. New drawing helpers** to add before `draw_text`:
```rust
fn draw_line_h(buf: &mut [u32], w: usize, _h: usize, x: usize, y: usize, len: usize, color: u32)
fn draw_separator_v(buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, len: usize, color: u32)
fn draw_badge_dot(buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, color: u32) // 4×4 dot
```

**3. Tab strip** (draw at top, height=32px, above toolbar):
- Read `session.tabs` from `load_session()`
- Store tabs in `SwalFilesApp` struct as `tabs: Vec<crate::session::TabState>`
- Draw each tab: background=ELEVATED, border-bottom=BORDER, active tab: border-bottom=ACCENT (2px line)
- Tab text: truncated to 16 chars, color=TEXT (active) or TEXT_DIM (inactive)
- `+` button at right for new tab (display only, click handled in J-4)

**4. Toolbar / breadcrumbs** (draw below tab strip, height=28px):
- Buttons: `⮜` (back/up), `⮝` (up), `⟳` (reload) — left side
- Breadcrumb: split `current_path` into parts, draw with `›` separators, last part in ACCENT
- Right side: filter indicator `[all]` or `[images]` etc from session

**5. Sidebar from session**:
- Load `session.favorites` and `session.workspaces` instead of hardcoded list
- Draw each: icon + name, with ACCENT left-bar if is_active
- Section headers: "FAVORITOS", "ESPACIOS", "UNIDADES" in TEXT_DIM 10px bold
- Disk section: use `DiskUsageScanner::scan_mounted_drives()`, draw progress bar for each

**6. Status bar** (bottom 20px):
- Left: `{total_items} elementos`
- Center: git branch if `session.git_status.is_git_repo` (from session)
- Right: `SWAL Files ⚡` in ACCENT

## Acceptance Criteria (VERIFIABLE BY COMMAND)

- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -n "0xFF00FF88" crates/swal-files/src/native_window_app.rs` — cyber-neon green present
- [ ] `grep -n "session.favorites" crates/swal-files/src/native_window_app.rs` — sidebar reads real favorites
- [ ] `grep -n "draw_line_h" crates/swal-files/src/native_window_app.rs` — helper exists
- [ ] `grep -n "breadcrumb\|Breadcrumb\|BREADCRUMB" crates/swal-files/src/native_window_app.rs` — breadcrumbs drawn

## Files to Modify

| File | Lines | Change | Risk |
|------|-------|--------|------|
| `crates/swal-files/src/native_window_app.rs` | 660 | Replace palette, add helpers, rewrite draw_frame sections | MEDIUM |

## DO NOT touch

- `crates/swal-files/src/session.rs` — read-only access via `load_session()`
- Any other crate
- `Cargo.toml` — no new deps

## Anti-Hallucination Guard

1. READ `crates/swal-files/src/session.rs` to understand `SessionState`, `TabState`, `SidebarPin` fields
2. READ `crates/swal-files/src/storage.rs` to understand `DriveInfo` fields
3. Do NOT remove existing `SwalFilesApp` fields — only add new ones

## Merge Order

- **After J-1** (depends on black screen fix)
- **Parallel with J-4** (disjoint changes within same file — coordinate on struct fields)
- **Expected effort:** Medium (60-90m)
