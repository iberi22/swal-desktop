# [Ola S4.03] feat(native-renderer): File icons by extension + git badges + row coloring

> Ola S4 — Native Rust File Manager Renderer. Depends on S4.01 + S4.02 merged.

## Context

Currently file entries show `[D] name  size` and `[F] name  size`. This issue implements:
- Extension-based icons (using Unicode/ASCII glyphs that ab_glyph can render)
- Row coloring by file type (dirs cyan, executables green, code files accent, etc.)
- Git status badges as colored 4×4 dot in right column
- Proper icon rendering with graceful fallback when glyph not in font

## Current State (MEASURABLE)

- `reload_dir()` lines 172-184: maps FileEntry to `"[D] {}  {}"` or `"[F] {}  {}"` format
- `draw_frame()` file list (lines 284-300): draws all items with same TEXT/DIR_COLOR — no extension-based coloring
- `raw_name()` and `is_dir_entry()`: parse from the `[D]`/`[F]` prefix — brittle
- No git badge rendering anywhere in the file

## Desired State (DELTA)

**1. Refactor FileListEntry struct** to replace the String-based item list:
```rust
struct FileListEntry {
    name: String,
    is_dir: bool,
    ext: String,           // lowercase extension without dot
    size_fmt: String,      // formatted size
    git_status: GitBadge,  // None, Modified, Staged, Untracked, Conflict
    is_executable: bool,
}

#[derive(Clone, Copy)]
enum GitBadge { None, Modified, Staged, Untracked, Conflict }
```

Change `items: Vec<String>` → `items: Vec<FileListEntry>` in `SwalFilesApp`.

**2. `fn extension_color(ext: &str, is_dir: bool) -> u32`**:
```
dir        → ACCENT2 (cyan)
rs, toml   → 0xFF FF6B35 (orange, Rust)
js, ts, json → 0xFF F7DF1E (yellow, JS)
py         → 0xFF 3776AB (blue, Python)
md, txt    → TEXT
sh, bash   → 0xFF 4EAA25 (green, shell)
png, jpg, svg, webp → 0xFF FF69B4 (pink, media)
zip, tar, gz → 0xFF FFD700 (gold, archives)
exe, bin   → DANGER (red)
nix        → 0xFF 7EBAE4 (light blue, Nix)
_default_  → TEXT
```

**3. `fn file_icon(ext: &str, is_dir: bool) -> &'static str`**:
- Use plain ASCII/Unicode that DejaVuSans supports (no Nerd Font codepoints):
```
dir     → "/"
rs      → "R"  (in orange box or just colored)
md      → "M"
sh      → "$"
json    → "{}"
py      → "Py"
js/ts   → "JS"
png/jpg → "~"
zip/tar → "Z"
_       → "."
```
Note: ab_glyph renders standard Unicode. For directories, draw a small filled square (5×5 block) in ACCENT2 color instead of text.

**4. Git badge rendering** in file list rows:
- Read git status from `crate::git::get_git_status(&current_path)` (already implemented in git.rs)
- In `reload_dir()`, for each entry: check git status, populate `FileListEntry.git_status`
- In `draw_frame()` file list: after drawing row text, draw a 4×4 dot at `x = content_x + content_w - 16`:
  - Modified → WARNING (yellow)
  - Staged → SUCCESS (green)  
  - Untracked → ACCENT2 (cyan)
  - Conflict → DANGER (red)

**5. Update `raw_name()` and `is_dir_entry()`** to use the new struct (simple field access).

## Web Research Required

1. search: "ab_glyph render unicode block characters box drawing Rust"
2. search: "DejaVuSans unicode coverage box drawing geometric shapes"

## Acceptance Criteria (VERIFIABLE BY COMMAND)

- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -n "FileListEntry" crates/swal-files/src/native_window_app.rs` — struct exists
- [ ] `grep -n "extension_color" crates/swal-files/src/native_window_app.rs` — function exists
- [ ] `grep -n "GitBadge" crates/swal-files/src/native_window_app.rs` — enum exists
- [ ] `grep -n "git_status\|get_git_status" crates/swal-files/src/native_window_app.rs` — git integration present

## Files to Modify

| File | Lines | Change | Risk |
|------|-------|--------|------|
| `crates/swal-files/src/native_window_app.rs` | 660 | New struct, new fns, update reload_dir + draw_frame | MEDIUM |

## DO NOT touch

- `crates/swal-files/src/git.rs` — read only
- `crates/swal-files/src/scanner.rs` — read only
- Any other crate

## Anti-Hallucination Guard

1. READ `crates/swal-files/src/git.rs` to find the exact function signature for git status
2. READ `crates/swal-files/src/scanner.rs` to understand `FileEntry` fields
3. The `items` field type change from `Vec<String>` to `Vec<FileListEntry>` will break `raw_name()`, `is_dir_entry()` — update ALL callers

## Merge Order

- **After J-1 and J-2** (needs palette constants from J-2)
- **Parallel with J-5** (different sections of draw_frame)
- **Expected effort:** Medium (60m)
