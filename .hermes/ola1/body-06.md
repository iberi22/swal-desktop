# [Ola 1.06] feat-vault: Widget Inotify Auto-Reload and Semantic Pinning Engine

> Ola 1 — Widget Vault & Hot-Reload.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-vault` in `crates/swal-widget-vault`
- File: `crates/swal-widget-vault/src/lib.rs` (2 tests passing, CRUD/export/import)
- Problem: The vault stores and pins widgets, but requires manual method calls to detect newly written JSON files from `~/.config/swal/widgets/`.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-widget-vault/src/hot_reload.rs` with `WidgetVaultWatcher` monitoring the widget directory.
- On file create/modify, validate schema using `swal-a2ui-engine::validate_widget_json` and update vault in-memory cache.
- Emit `VaultEvent::WidgetUpdated(String)`.

## Web Research Required
1. search: "rust inotify directory watcher debounce with mpsc channel"
2. search: "hot reloading configuration files rust tokio"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-widget-vault` — 0 errors
- [ ] `cargo test -p swal-widget-vault` — all tests pass
- [ ] `grep -rn "WidgetVaultWatcher" crates/swal-widget-vault/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-widget-vault/src/hot_reload.rs` | None (New) | Implement inotify watcher & hot reload bridge | LOW |
| `crates/swal-widget-vault/src/lib.rs` | 130 lines | Export `pub mod hot_reload;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-node-daemon/` — assigned to Issue #5

## Anti-Hallucination Guard
1. Filter only `.json` files
2. Discard malformed JSON files gracefully without panicking

## Merge Order
- **Merge order within wave:** 6
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
