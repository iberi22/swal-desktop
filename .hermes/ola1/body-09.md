# [Ola 1.09] feat-files: Multi-File Batch Tagging and Archive Extractor

> Ola 1 — File Manager & Batch Operations.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-files` in `crates/swal-files`
- File: `crates/swal-files/src/lib.rs` (35 tests passing)
- Problem: File operations currently perform single-file operations, lacking native `.tar.gz` and `.zip` archive previews and batch metadata tagging.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-files/src/archive.rs` containing `ArchiveInspector` to list and extract entries from tar/zip archives without spawning external unzip binaries.
- Implement `BatchTagManager` to attach semantic tags (`"ai-project"`, `"audio-sample"`, `"document"`) to file metadata.

## Web Research Required
1. search: "rust tar crate iterate entries without full extraction"
2. search: "rust zip crate read central directory fast"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all tests pass
- [ ] `grep -rn "ArchiveInspector" crates/swal-files/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/archive.rs` | None (New) | Implement archive inspector and batch tagging | LOW |
| `crates/swal-files/src/lib.rs` | 420 lines | Export `pub mod archive;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-telemetry-rs/` — assigned to Issue #3
- `crates/swal-ambient-orb/` — assigned to Issue #7

## Anti-Hallucination Guard
1. Use stream readers to prevent loading multi-gigabyte archives into RAM
2. Return typed `Result<Vec<ArchiveEntry>, ArchiveError>`

## Merge Order
- **Merge order within wave:** 9
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
