# [Ola 3.04] feat-swal-19 — Storage Drive & Disk Space Usage Visualizer Engine in Rust

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-files/` shows static `NVMe Root (/)` in sidebar.
- No dynamic disk drive probing, total/available space calculation, or percentage progress bar payload is generated.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/storage.rs` with `DiskUsageScanner`:
  - Struct `DriveInfo` with `mount_point: String`, `filesystem: String`, `total_bytes: u64`, `available_bytes: u64`, `used_percentage: f32`, `is_removable: bool`.
  - Function `scan_mounted_drives() -> Vec<DriveInfo>` reading `/proc/mounts` or `statvfs` on Linux.
  - Format helpers: `formatted_total`, `formatted_available`, `formatted_used`.
- **File Target**: `crates/swal-files/src/storage.rs`

## Web Research Required
1. search: "files-community/Files drives sidebar disk usage bar"
2. search: "rust linux get filesystem disk usage statvfs proc mounts"
3. search: "rust nix sys statvfs disk capacity percentage"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `grep -rn "scan_mounted_drives" crates/swal-files/src/storage.rs` >= 1 match
- [ ] `grep -rn "DriveInfo" crates/swal-files/src/storage.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/storage.rs` | Non-existent | [NEW] Linux drive probe and disk usage visualizer | LOW |

## DO NOT touch
- `crates/swal-files/src/config.rs` — configuration store
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/entry.rs` to maintain format consistency.
2. Use standard POSIX/Linux filesystem inspection (`rustix` or `std::fs` / `/proc/mounts`).

## Merge Order
- **Merge order within wave:** 4
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
