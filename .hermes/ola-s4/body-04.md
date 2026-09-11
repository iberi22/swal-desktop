# [Ola S4.04] feat(native-renderer): Mouse click/hover/scroll + pool resize fix

> Ola S4 — Native Rust File Manager Renderer. Depends on S4.01 merged.

## Context

The file manager is keyboard-only right now. Without mouse support it cannot be used as a daily driver. This issue implements full pointer interaction and fixes the pool resize crash.

## Current State (MEASURABLE)

- `pointer_frame()` lines 590-~640: handler exists but body is empty (all events ignored)
- `SwalFilesApp` struct: no `hover_index`, no `last_click_time`, no `pointer_x/y` fields
- Pool resize: creating new pool per frame is not done — if window grows, buffer too small

## Desired State (DELTA)

**1. Add fields to `SwalFilesApp`**:
```rust
hover_index: Option<usize>,
pointer_x: f64,
pointer_y: f64,
last_click_ms: u64,      // for double-click detection
// Layout geometry cache (set in draw_frame, read in pointer_frame)
layout_sidebar_w: usize,
layout_list_top: usize,
layout_row_h: usize,
layout_content_x: usize,
layout_tab_strip_h: usize,  // height of tab strip (from J-2)
layout_toolbar_h: usize,    // height of toolbar (from J-2)
```

**2. Implement `pointer_frame()`**:
```rust
fn pointer_frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>,
                 _pointer: &WlPointer, events: &[PointerEvent]) {
    for event in events {
        match event.kind {
            PointerEventKind::Motion { time: _ } => {
                self.pointer_x = event.position.0;
                self.pointer_y = event.position.1;
                // Calculate hover row
                let y = event.position.1 as usize;
                let x = event.position.0 as usize;
                if x > self.layout_content_x && y > self.layout_list_top {
                    let row = (y - self.layout_list_top) / self.layout_row_h.max(1);
                    let new_hover = if row < self.items.len() { Some(row) } else { None };
                    if new_hover != self.hover_index {
                        self.hover_index = new_hover;
                        self.redraw = true;
                    }
                } else {
                    if self.hover_index.is_some() {
                        self.hover_index = None;
                        self.redraw = true;
                    }
                }
            }
            PointerEventKind::Press { button, .. } => {
                if button == 272 { // BTN_LEFT
                    let y = event.position.1 as usize;
                    let x = event.position.0 as usize;
                    // Click on file list
                    if x > self.layout_content_x && y > self.layout_list_top {
                        let row = (y - self.layout_list_top) / self.layout_row_h.max(1);
                        if row < self.items.len() {
                            let now_ms = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_millis() as u64;
                            let double_click = self.selected_index == row
                                && now_ms.saturating_sub(self.last_click_ms) < 500;
                            self.selected_index = row;
                            self.last_click_ms = now_ms;
                            if double_click {
                                self.open_selected();
                            }
                            self.redraw = true;
                        }
                    }
                    // Click on sidebar (x < layout_sidebar_w)
                    // TODO: sidebar item click — add in follow-up
                }
            }
            PointerEventKind::Axis { axis, value, .. } => {
                // Vertical scroll
                use smithay_client_toolkit::seat::pointer::AxisSource;
                // value > 0 = scroll down, < 0 = scroll up
                let delta = if value > 0.0 { 3 } else { 0usize.wrapping_sub(3) };
                // Use saturating arithmetic
                if value > 0.0 {
                    self.scroll_offset = self.scroll_offset.saturating_add(3)
                        .min(self.items.len().saturating_sub(1));
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_sub(3);
                }
                self.redraw = true;
            }
            _ => {}
        }
    }
}
```

**3. In `draw_frame()`** — update layout geometry cache fields after computing layout:
```rust
self.layout_sidebar_w = sidebar_w;
self.layout_list_top = list_top;
self.layout_row_h = row_h;
self.layout_content_x = content_x;
```

**4. Hover highlight** — in file list draw loop:
```rust
let is_hovered = Some(i) == self.hover_index && !is_sel;
if is_hovered {
    fill_rect(&mut buf, w, h, content_x, y - 2, content_w, row_h, HOVER_BG);
}
```
Add constant `const HOVER_BG: u32 = 0xFF162235;`

**5. Pool resize fix** (already specified in J-1, but if J-1 didn't include it, add here):
In `draw_frame()` before pool usage:
```rust
let needed = w * h * 4;
if self.pool.as_ref().map(|p| p.len() < needed).unwrap_or(false) {
    self.pool = None;
}
```

## Web Research Required

1. search: "smithay-client-toolkit 0.19 PointerEventKind Motion Press Axis button BTN_LEFT"
2. search: "wayland wl_pointer button event code 272 BTN_LEFT linux input"
3. search: "smithay-client-toolkit pointer AxisSource discrete continuous scroll"

## Acceptance Criteria (VERIFIABLE BY COMMAND)

- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -n "hover_index" crates/swal-files/src/native_window_app.rs` — field + usage
- [ ] `grep -n "PointerEventKind::Motion" crates/swal-files/src/native_window_app.rs` — handler
- [ ] `grep -n "PointerEventKind::Press" crates/swal-files/src/native_window_app.rs` — click handler
- [ ] `grep -n "PointerEventKind::Axis" crates/swal-files/src/native_window_app.rs` — scroll handler
- [ ] `grep -n "layout_sidebar_w\|layout_list_top\|layout_row_h" crates/swal-files/src/native_window_app.rs` — geometry cache fields

## Files to Modify

| File | Lines | Change | Risk |
|------|-------|--------|------|
| `crates/swal-files/src/native_window_app.rs` | 660 | Add struct fields, implement pointer_frame, update draw_frame | MEDIUM |

## DO NOT touch

- Any other file in the crate
- `Cargo.toml` — no new deps needed

## Anti-Hallucination Guard

1. READ the existing `pointer_frame` signature — must match the trait exactly
2. FIND `PointerEventKind` variants in smithay-client-toolkit 0.19 — check the actual enum variants
3. The `items` field may change to `Vec<FileListEntry>` in J-3 — coordinate: if J-3 merges first, update callers accordingly

## Merge Order

- **After J-1** (needs working window)
- **Parallel with J-2** — coordinate on new struct fields to avoid conflicts
- **Before J-6** (J-6 needs the interaction model)
- **Expected effort:** Medium (60-90m)
