# [Ola 1.03] feat-telemetry: Partition Storage Scanner and statvfs Disk Metrics

> Ola 1 — Telemetry & Storage.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-telemetry` in `crates/swal-telemetry-rs`
- File: `crates/swal-telemetry-rs/src/lib.rs` (7 tests passing, CPU/RAM/GPU/Temps/Net covered)
- Problem: Disk partitions and drive capacity metrics are not yet exposed natively in `SystemMetrics`.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-telemetry-rs/src/storage.rs` to parse `/proc/mounts` and call `nix::sys::statvfs` or libc `statvfs64` for mount points (`/`, `/boot`, `/home`, external drives).
- Expose `DiskInfo { mount_point: String, total_bytes: u64, free_bytes: u64, used_pct: f32 }`.
- Add `pub disks: Vec<DiskInfo>` to `SystemMetrics`.

## Web Research Required
1. search: "rust libc statvfs disk usage zero allocation"
2. search: "reading /proc/mounts filtering virtual filesystems rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-telemetry-rs` — 0 errors
- [ ] `cargo test -p swal-telemetry-rs` — all tests pass
- [ ] `grep -rn "scan_mounted_partitions" crates/swal-telemetry-rs/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-telemetry-rs/src/storage.rs` | None (New) | Implement partition scanner via statvfs | LOW |
| `crates/swal-telemetry-rs/src/lib.rs` | 320 lines | Export `pub mod storage;` and embed in metrics | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-render-pipeline/` — assigned to Issue #4

## Anti-Hallucination Guard
1. Filter out pseudo-filesystems (`proc`, `sysfs`, `devpts`, `tmpfs` unless root) to only report real block devices.
2. Follow strict zero-panic error handling.

## Merge Order
- **Merge order within wave:** 3
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
