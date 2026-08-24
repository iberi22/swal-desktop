# [Ola 1.02] feat-files: Real-Time Inotify Filesystem Watcher with notify Crate

> Ola 1 — File Manager & System Events.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-files` in `crates/swal-files`
- File: `crates/swal-files/src/lib.rs` (35 tests passing)
- Problem: File manager currently relies on a 2-second polling timer to refresh directory contents.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-files/src/watcher.rs` using `notify` crate to watch active tab directories.
- Emit asynchronous `FsChangeEvent::Modified(PathBuf)` and `FsChangeEvent::Created(PathBuf)` events without CPU polling.
- Expose `DirectoryWatcher::new(tx)` and integrate with tab session update loop.

## Web Research Required
1. search: "rust notify crate 6.1 RecommendedWatcher channel debouncing"
2. search: "non-blocking file watcher tokio mpsc channel rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all tests pass
- [ ] `grep -rn "DirectoryWatcher" crates/swal-files/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/watcher.rs` | None (New) | Implement Inotify watcher using notify crate | LOW |
| `crates/swal-files/src/lib.rs` | 420 lines | Export `pub mod watcher;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-telemetry-rs/` — assigned to Issue #3
- `crates/swal-render-pipeline/` — assigned to Issue #4

## Anti-Hallucination Guard
1. READ before write: inspect `crates/swal-files/Cargo.toml` and existing tab management code
2. Ensure channels are bounded or non-blocking to prevent memory leaks

## Merge Order
- **Merge order within wave:** 2
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
