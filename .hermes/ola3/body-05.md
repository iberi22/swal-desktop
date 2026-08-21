# [Ola 3.05] feat-swal-20 — Windows Files Tab Tooltips & Pin Drag/Reorder Metadata

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-files/src/session.rs` contains basic `TabState` struct.
- Lacks extended metadata for tab hover tooltips (item count, disk location, preview thumbnail) and drag-and-drop reorder indices.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/tabs_extended.rs`:
  - Struct `ExtendedTabInfo` with `item_count: usize`, `formatted_path: String`, `is_loading: bool`, `color_tag: Option<String>`.
  - Methods for reordering tabs (`move_tab(from_idx, to_idx)`), duplicating tabs (`duplicate_tab(id)`), and closing other tabs (`close_other_tabs(id)`).
- **File Target**: `crates/swal-files/src/tabs_extended.rs`

## Web Research Required
1. search: "files-community/Files tab reorder duplicate tab close others"
2. search: "windows 11 files app tab metadata preview tooltip"
3. search: "rust vector reorder element move tab index"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -rn "ExtendedTabInfo" crates/swal-files/src/tabs_extended.rs` >= 1 match
- [ ] `grep -rn "move_tab" crates/swal-files/src/tabs_extended.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/tabs_extended.rs` | Non-existent | [NEW] Extended tab metadata, tooltip generators and tab reordering | LOW |

## DO NOT touch
- `crates/swal-files/src/session.rs` — core session state
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/session.rs` and `crates/swal-files/src/lib.rs` first.
2. Include comprehensive unit tests inside `tabs_extended.rs`.

## Merge Order
- **Merge order within wave:** 5
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
