# [Ola S4.05] feat(native-renderer): Preview panel from PreviewState + inline search with /

> Ola S4 — Native Rust File Manager Renderer. Depends on S4.01 + S4.02 merged.

## Context

The current preview panel does `std::fs::read_to_string()` directly on selected files — it doesn't use the existing `crate::preview` module which has full syntax highlighting, file type detection, image metadata, etc. This issue connects the preview module to the native renderer and adds `/` search mode.

## Current State (MEASURABLE)

- Preview panel (lines 302-323 of native_window_app.rs): reads file content raw with `std::fs::read_to_string()`, displays up to 40 lines with just line numbers — no syntax coloring, no file type awareness
- `crates/swal-files/src/preview.rs`: has `PreviewState` struct and `generate_preview()` function — completely unused by native_window_app.rs
- No search functionality — `/` key does nothing in `key_char()` or `key_press()`

## Desired State (DELTA)

**1. Add preview state to struct**:
```rust
// In SwalFilesApp:
preview: Option<crate::preview::PreviewContent>,  // check actual type in preview.rs
search_mode: bool,
search_query: String,
items_filtered: Vec<usize>,  // indices into self.items when search_mode=true
```

**2. `update_preview()` method**:
```rust
fn update_preview(&mut self) {
    if let Some(name) = self.raw_name(self.selected_index) {
        let path = self.current_path.join(&name);
        self.preview = crate::preview::generate_preview(&path).ok();  // check actual API
    } else {
        self.preview = None;
    }
}
```
Call `update_preview()` in: `open_selected()` (after nav), `reload_dir()` (after loading items), whenever `selected_index` changes (in key handlers and pointer handler).

**3. Preview panel draw** — replace current raw-read with structured preview:
```rust
// In draw_frame(), preview panel section:
let preview_x = content_x + content_w;
fill_rect(&mut buf, w, h, preview_x, 0, preview_w, h, ELEVATED);

if let Some(ref prev) = self.preview {
    // File name (bold, accent)
    draw_text(&mut buf, w, h, &self.font, 13.0, preview_x + 10, 40,
              &prev.file_name, ACCENT, true);
    // File type + size
    let meta = format!("{} · {}", prev.file_type, prev.size_formatted);
    draw_text(&mut buf, w, h, &self.font, 11.0, preview_x + 10, 58, &meta, TEXT_DIM, false);
    // Separator line
    draw_line_h(&mut buf, w, h, preview_x + 10, 74, preview_w - 20, BORDER);
    // Content lines with "syntax" coloring (keywords in accent, strings in yellow, etc.)
    let mut py = 82usize;
    for (i, line) in prev.lines.iter().enumerate() {
        if py + 14 > h { break; }
        let lineno = format!("{:>3} │ ", i + 1);
        let lineno_color = TEXT_DIM;
        draw_text(&mut buf, w, h, &self.font, 11.0, preview_x + 8, py, &lineno, lineno_color, false);
        // Line content — color by content type if available
        let content_color = line_color_hint(&line.content, &prev.file_type);
        draw_text_trunc(&mut buf, w, h, &self.font, 11.0,
                        preview_x + 44, py, &line.content,
                        content_color, false, preview_w - 56);
        py += 14;
    }
}
```

Check the actual `PreviewContent` / `PreviewState` field names in `preview.rs` before writing — use the real API.

**4. `fn line_color_hint(content: &str, file_type: &str) -> u32`**:
Simple heuristic: if line starts with `//`, `#`, `--` → TEXT_DIM (comment); if contains `"` or `'` → WARNING (string); if first word is a keyword (`fn`, `pub`, `let`, `const`, `import`, `def`, `class`) → ACCENT; else → TEXT.

**5. Search mode**:
```rust
// In SwalFilesApp, add to keyboard handler:
'/' => {
    self.search_mode = true;
    self.search_query.clear();
    self.redraw = true;
}
```
When `search_mode=true` and a char key is pressed: append to `search_query`, filter `items_filtered` to indices where `name.to_lowercase().contains(&search_query)`, set `selected_index=0`, `redraw=true`.

`Escape` → `search_mode=false`, `search_query.clear()`, `items_filtered.clear()`, `redraw=true`.

In draw_frame, when `search_mode=true`:
- Draw search bar in footer area: `[/ {search_query}_]` with ACCENT color
- File list shows only `items_filtered` indices

**6. Directory preview** (when selected item is a dir):
- Show first 12 entries of the directory (use `scan_directory`)
- Format: `📁 subdir` or `· file.rs` with count at top: `{n} elementos`

## Web Research Required

1. search: "swal-files preview.rs PreviewContent PreviewState generate_preview API" (READ the actual file first)

## Acceptance Criteria (VERIFIABLE BY COMMAND)

- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -n "search_mode\|search_query" crates/swal-files/src/native_window_app.rs` — search fields exist
- [ ] `grep -n "update_preview\|generate_preview" crates/swal-files/src/native_window_app.rs` — preview integration
- [ ] `grep -n "line_color_hint" crates/swal-files/src/native_window_app.rs` — syntax coloring fn
- [ ] `grep -n "items_filtered" crates/swal-files/src/native_window_app.rs` — filtered list for search

## Files to Modify

| File | Lines | Change | Risk |
|------|-------|--------|------|
| `crates/swal-files/src/native_window_app.rs` | 660 | Add fields, update_preview(), rewrite preview panel draw, search mode | MEDIUM |

## DO NOT touch

- `crates/swal-files/src/preview.rs` — READ ONLY — use its public API
- `crates/swal-files/src/scanner.rs` — READ ONLY

## Anti-Hallucination Guard

1. READ `crates/swal-files/src/preview.rs` FULLY before writing — use its exact struct/function names
2. If `generate_preview()` returns `Result<T>`, handle the error case
3. If `preview.rs` has no public `lines` field, use whatever is available

## Merge Order

- **After J-1 + J-2** (needs working window + palette)
- **Parallel with J-3** (different sections of draw_frame — preview panel vs file list)
- **Expected effort:** Medium (60-90m)
