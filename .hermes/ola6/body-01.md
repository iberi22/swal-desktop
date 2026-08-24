# [Ola 6.01] feat-swal-51 — Cross-Platform OS & Filesystem Abstraction Layer in Pure Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-files/` currently has POSIX-centric paths and direct `/proc` / Linux-specific directory lookups.
- Running `swal-files` on Windows (e.g. `C:\`, UNC shares) or generic macOS/Linux without Hyprland/NixOS is not fully abstracted.
- No unified platform abstraction layer (`PlatformAbstraction`) exists to resolve system folders, trash bin, and default file openers cross-platform.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/platform.rs`:
  - Structs & Enums:
    - `OsPlatform`: `Linux`, `Windows`, `MacOS`, `Unknown`.
    - `SystemFolder`: `Home`, `Documents`, `Downloads`, `Pictures`, `Music`, `Videos`, `Desktop`, `Config`.
    - `DriveInfo`: `mount_point: String`, `total_bytes: u64`, `available_bytes: u64`, `drive_type: String`, `is_removable: bool`.
    - `PlatformAbstraction`: Static methods to resolve cross-platform paths and operations:
      - `detect_os() -> OsPlatform`
      - `get_system_folder(folder: SystemFolder) -> Option<PathBuf>`
      - `list_system_drives() -> Vec<DriveInfo>`
      - `normalize_path(path: &Path) -> PathBuf` (Handles Windows drive letters `C:\`, backslashes, UNC, and Unix root `/`)
      - `open_with_default_app(path: &Path) -> Result<()>` (Dispatches to `xdg-open` on Linux, `start` / `ShellExecute` on Windows, `open` on macOS)
      - `move_to_trash(path: &Path) -> Result<()>` (Cross-platform trash support)
  - **Embedded Unit Tests**: Include comprehensive unit tests testing OS detection, path normalization, Windows drive parsing, system folder lookups, and error handling with 100% test coverage.
- **File Target**: `crates/swal-files/src/platform.rs`

## Web Research Required
1. search: "rust cross platform system folder dirs crate pattern"
2. search: "rust normalize windows unc path and drive letters"
3. search: "rust open file with default os application cross platform"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "PlatformAbstraction" crates/swal-files/src/platform.rs` >= 1 match
- [ ] `grep -rn "normalize_path" crates/swal-files/src/platform.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/platform.rs` | Non-existent | [NEW] Cross-platform OS & Filesystem abstraction with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/storage.rs` and `crates/swal-files/src/config.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 1
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
