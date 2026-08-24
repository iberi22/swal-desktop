# [Ola 6.03] feat-swal-53 — Decentralized P2P & Remote Storage Provider Engine in Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-files/src/storage.rs` scans local disk drives (`/`, `/home`, etc.).
- There is no unified interface for virtual remote filesystems, cloud sync, WebDAV, or SWAL Edge-Mesh P2P network directories.
- Standalone users cannot access remote SWAL Node storage shares or cloud buckets directly from the file manager.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/cloud_sync.rs`:
  - Structs & Enums:
    - `StorageProviderKind`: `LocalDisk`, `EdgeMeshP2P`, `WebDAV`, `S3Compatible`, `SftpShare`.
    - `RemoteStorageAccount`: `id: String`, `name: String`, `kind: StorageProviderKind`, `endpoint_url: String`, `base_path: String`, `is_online: bool`, `sync_status: SyncStatus`.
    - `SyncStatus`: `Idle`, `Syncing { files_remaining: usize, bytes_transferred: u64 }`, `Error(String)`, `Paused`.
    - `RemoteFileEntry`: `name: String`, `remote_path: String`, `size: u64`, `is_dir: bool`, `modified_timestamp: u64`, `etag: Option<String>`.
    - `StorageProviderManager`: Manager struct with methods:
      - `new() -> Self`
      - `register_account(&mut self, account: RemoteStorageAccount) -> Result<(), String>`
      - `list_accounts(&self) -> Vec<RemoteStorageAccount>`
      - `list_remote_entries(&self, account_id: &str, path: &str) -> Result<Vec<RemoteFileEntry>, String>`
      - `trigger_sync(&mut self, account_id: &str) -> Result<SyncStatus, String>`
      - `generate_share_link(&self, account_id: &str, file_path: &str) -> Result<String, String>`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing account registration, provider serialization, mock directory listings, sync state transitions, and URL link generation with 100% test coverage.
- **File Target**: `crates/swal-files/src/cloud_sync.rs`

## Web Research Required
1. search: "rust virtual filesystem vfs abstraction trait design"
2. search: "rust remote file sync state machine"
3. search: "p2p file sharing metadata rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "StorageProviderManager" crates/swal-files/src/cloud_sync.rs` >= 1 match
- [ ] `grep -rn "RemoteStorageAccount" crates/swal-files/src/cloud_sync.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/cloud_sync.rs` | Non-existent | [NEW] Remote storage and P2P provider engine with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/storage.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 3
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
