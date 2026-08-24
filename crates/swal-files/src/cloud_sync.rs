//! Decentralized P2P & Remote Storage Provider Engine in Rust
//! Provides virtual remote filesystem abstractions, cloud sync state tracking,
//! WebDAV, S3, SFTP, and SWAL Edge-Mesh P2P storage provider integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage Provider Kinds supported by SWAL Desktop
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageProviderKind {
    LocalDisk,
    EdgeMeshP2P,
    WebDAV,
    S3Compatible,
    SftpShare,
}

/// Synchronization status of a remote storage account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing {
        files_remaining: usize,
        bytes_transferred: u64,
    },
    Error(String),
    Paused,
}

/// Remote storage account configuration and state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteStorageAccount {
    pub id: String,
    pub name: String,
    pub kind: StorageProviderKind,
    pub endpoint_url: String,
    pub base_path: String,
    pub is_online: bool,
    pub sync_status: SyncStatus,
}

impl RemoteStorageAccount {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: StorageProviderKind,
        endpoint_url: impl Into<String>,
        base_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            endpoint_url: endpoint_url.into(),
            base_path: base_path.into(),
            is_online: true,
            sync_status: SyncStatus::Idle,
        }
    }
}

/// Representation of a file or directory entry on a remote storage provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteFileEntry {
    pub name: String,
    pub remote_path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified_timestamp: u64,
    pub etag: Option<String>,
}

impl RemoteFileEntry {
    pub fn new(
        name: impl Into<String>,
        remote_path: impl Into<String>,
        size: u64,
        is_dir: bool,
        modified_timestamp: u64,
        etag: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            remote_path: remote_path.into(),
            size,
            is_dir,
            modified_timestamp,
            etag,
        }
    }
}

/// Storage Provider Manager managing accounts, remote entry listing, sync state, and link sharing
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StorageProviderManager {
    accounts: HashMap<String, RemoteStorageAccount>,
}

impl StorageProviderManager {
    /// Creates a new `StorageProviderManager` instance
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Registers a new storage account.
    /// Returns an error if the account ID is empty or if an account with the same ID already exists.
    pub fn register_account(&mut self, account: RemoteStorageAccount) -> Result<(), String> {
        if account.id.trim().is_empty() {
            return Err("Account ID cannot be empty".to_string());
        }
        if self.accounts.contains_key(&account.id) {
            return Err(format!("Account ID '{}' already registered", account.id));
        }
        self.accounts.insert(account.id.clone(), account);
        Ok(())
    }

    /// Removes an account by ID. Returns true if removed, false if not found.
    pub fn unregister_account(&mut self, account_id: &str) -> bool {
        self.accounts.remove(account_id).is_some()
    }

    /// Retrieves an account by ID if registered.
    pub fn get_account(&self, account_id: &str) -> Option<&RemoteStorageAccount> {
        self.accounts.get(account_id)
    }

    /// Retrieves a mutable reference to an account by ID if registered.
    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut RemoteStorageAccount> {
        self.accounts.get_mut(account_id)
    }

    /// Returns a list of all registered remote storage accounts.
    pub fn list_accounts(&self) -> Vec<RemoteStorageAccount> {
        let mut accounts: Vec<_> = self.accounts.values().cloned().collect();
        accounts.sort_by(|a, b| a.id.cmp(&b.id));
        accounts
    }

    /// Lists remote entries at a target path for a specified account.
    pub fn list_remote_entries(
        &self,
        account_id: &str,
        path: &str,
    ) -> Result<Vec<RemoteFileEntry>, String> {
        let account = self
            .accounts
            .get(account_id)
            .ok_ok_or_else(|| format!("Account not found: {}", account_id))?;

        if !account.is_online {
            return Err(format!("Account '{}' is offline", account_id));
        }

        let clean_path = path.trim_matches('/');
        let base_prefix = account.base_path.trim_matches('/');
        let full_path = if clean_path.is_empty() {
            base_prefix.to_string()
        } else if base_prefix.is_empty() {
            clean_path.to_string()
        } else {
            format!("{}/{}", base_prefix, clean_path)
        };

        let mock_entries = match account.kind {
            StorageProviderKind::LocalDisk => vec![
                RemoteFileEntry::new("documents", format!("{}/documents", full_path), 0, true, 1700000000, None),
                RemoteFileEntry::new("local_backup.tar.gz", format!("{}/local_backup.tar.gz", full_path), 10485760, false, 1700000100, Some("hash-local-1".to_string())),
            ],
            StorageProviderKind::EdgeMeshP2P => vec![
                RemoteFileEntry::new("shared_mesh_folder", format!("{}/shared_mesh_folder", full_path), 0, true, 1700000200, None),
                RemoteFileEntry::new("p2p_stream.bin", format!("{}/p2p_stream.bin", full_path), 52428800, false, 1700000300, Some("p2p-hash-001".to_string())),
            ],
            StorageProviderKind::WebDAV => vec![
                RemoteFileEntry::new("dav_vault", format!("{}/dav_vault", full_path), 0, true, 1700000400, None),
                RemoteFileEntry::new("notes.txt", format!("{}/notes.txt", full_path), 2048, false, 1700000500, Some("\"dav-etag-123\"".to_string())),
            ],
            StorageProviderKind::S3Compatible => vec![
                RemoteFileEntry::new("assets", format!("{}/assets", full_path), 0, true, 1700000600, None),
                RemoteFileEntry::new("dataset.json", format!("{}/dataset.json", full_path), 40960, false, 1700000700, Some("\"s3-etag-abc\"".to_string())),
            ],
            StorageProviderKind::SftpShare => vec![
                RemoteFileEntry::new("remote_builds", format!("{}/remote_builds", full_path), 0, true, 1700000800, None),
                RemoteFileEntry::new("syslog.log", format!("{}/syslog.log", full_path), 81920, false, 1700000900, Some("sftp-sha256-def".to_string())),
            ],
        };

        Ok(mock_entries)
    }

    /// Triggers synchronization for an account and updates its `sync_status`.
    pub fn trigger_sync(&mut self, account_id: &str) -> Result<SyncStatus, String> {
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_ok_or_else(|| format!("Account not found: {}", account_id))?;

        if !account.is_online {
            account.sync_status = SyncStatus::Error("Account is offline".to_string());
            return Err(format!("Account '{}' is offline", account_id));
        }

        let new_status = match &account.sync_status {
            SyncStatus::Idle | SyncStatus::Paused | SyncStatus::Error(_) => SyncStatus::Syncing {
                files_remaining: 5,
                bytes_transferred: 4096,
            },
            SyncStatus::Syncing { .. } => SyncStatus::Idle,
        };

        account.sync_status = new_status.clone();
        Ok(new_status)
    }

    /// Generates a sharable link for a file on the remote storage account.
    pub fn generate_share_link(
        &self,
        account_id: &str,
        file_path: &str,
    ) -> Result<String, String> {
        let account = self
            .accounts
            .get(account_id)
            .ok_ok_or_else(|| format!("Account not found: {}", account_id))?;

        if !account.is_online {
            return Err(format!("Account '{}' is offline", account_id));
        }

        let clean_path = file_path.trim_matches('/');
        let base_url = account.endpoint_url.trim_end_matches('/');

        let link = match account.kind {
            StorageProviderKind::EdgeMeshP2P => {
                format!("swal-p2p://{}/{}", account.id, clean_path)
            }
            StorageProviderKind::WebDAV => {
                format!("{}/dav/{}", base_url, clean_path)
            }
            StorageProviderKind::S3Compatible => {
                format!("{}/{}/{}", base_url, account.base_path.trim_matches('/'), clean_path)
            }
            StorageProviderKind::SftpShare => {
                format!("{}/sftp/{}", base_url, clean_path)
            }
            StorageProviderKind::LocalDisk => {
                format!("file://{}/{}", base_url, clean_path)
            }
        };

        Ok(link)
    }
}

// Helper extension trait for std Result conversion readability
trait OptionExt<T> {
    fn ok_ok_or_else<F: FnOnce() -> String>(self, f: F) -> Result<T, String>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_ok_or_else<F: FnOnce() -> String>(self, f: F) -> Result<T, String> {
        self.ok_or_else(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_provider_kind_serde() {
        let kinds = vec![
            StorageProviderKind::LocalDisk,
            StorageProviderKind::EdgeMeshP2P,
            StorageProviderKind::WebDAV,
            StorageProviderKind::S3Compatible,
            StorageProviderKind::SftpShare,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("Serialization failed");
            let deserialized: StorageProviderKind =
                serde_json::from_str(&json).expect("Deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

    #[test]
    fn test_sync_status_serde() {
        let statuses = vec![
            SyncStatus::Idle,
            SyncStatus::Syncing {
                files_remaining: 12,
                bytes_transferred: 1048576,
            },
            SyncStatus::Error("Network timeout".to_string()),
            SyncStatus::Paused,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).expect("Serialization failed");
            let deserialized: SyncStatus =
                serde_json::from_str(&json).expect("Deserialization failed");
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_remote_storage_account_creation_and_serde() {
        let account = RemoteStorageAccount::new(
            "p2p-node-01",
            "Swal Edge Mesh Node",
            StorageProviderKind::EdgeMeshP2P,
            "https://mesh.swal.internal:8006",
            "/shares/public",
        );

        assert_eq!(account.id, "p2p-node-01");
        assert_eq!(account.name, "Swal Edge Mesh Node");
        assert_eq!(account.kind, StorageProviderKind::EdgeMeshP2P);
        assert!(account.is_online);
        assert_eq!(account.sync_status, SyncStatus::Idle);

        let json = serde_json::to_string(&account).expect("Serialization failed");
        let deserialized: RemoteStorageAccount =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(account, deserialized);
    }

    #[test]
    fn test_remote_file_entry_serde() {
        let entry = RemoteFileEntry::new(
            "document.pdf",
            "/shares/public/document.pdf",
            204800,
            false,
            1700000000,
            Some("etag-12345".to_string()),
        );

        let json = serde_json::to_string(&entry).expect("Serialization failed");
        let deserialized: RemoteFileEntry =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_account_registration_and_listing() {
        let mut manager = StorageProviderManager::new();
        assert!(manager.list_accounts().is_empty());

        let account1 = RemoteStorageAccount::new(
            "acc-1",
            "S3 Bucket",
            StorageProviderKind::S3Compatible,
            "https://s3.amazonaws.com",
            "my-bucket",
        );
        let account2 = RemoteStorageAccount::new(
            "acc-2",
            "WebDAV Server",
            StorageProviderKind::WebDAV,
            "https://nextcloud.example.com",
            "remote.php/dav",
        );

        assert!(manager.register_account(account1.clone()).is_ok());
        assert!(manager.register_account(account2.clone()).is_ok());

        // Duplicate registration error
        assert!(manager.register_account(account1.clone()).is_err());

        // Empty ID error
        let empty_account = RemoteStorageAccount::new(
            "",
            "Invalid",
            StorageProviderKind::LocalDisk,
            "",
            "",
        );
        assert!(manager.register_account(empty_account).is_err());

        let accounts = manager.list_accounts();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].id, "acc-1");
        assert_eq!(accounts[1].id, "acc-2");

        assert!(manager.get_account("acc-1").is_some());
        assert!(manager.get_account("non-existent").is_none());

        assert!(manager.unregister_account("acc-1"));
        assert_eq!(manager.list_accounts().len(), 1);
        assert!(!manager.unregister_account("acc-1"));
    }

    #[test]
    fn test_list_remote_entries_mock_providers() {
        let mut manager = StorageProviderManager::new();

        let kinds = vec![
            (StorageProviderKind::LocalDisk, "disk-1"),
            (StorageProviderKind::EdgeMeshP2P, "p2p-1"),
            (StorageProviderKind::WebDAV, "dav-1"),
            (StorageProviderKind::S3Compatible, "s3-1"),
            (StorageProviderKind::SftpShare, "sftp-1"),
        ];

        for (kind, id) in kinds {
            let account = RemoteStorageAccount::new(
                id,
                format!("Account {}", id),
                kind,
                "https://endpoint.local",
                "root",
            );
            manager.register_account(account).unwrap();

            let entries = manager.list_remote_entries(id, "folder1").unwrap();
            assert!(!entries.is_empty(), "Entries for {} should not be empty", id);
            assert!(entries.iter().any(|e| e.is_dir));
            assert!(entries.iter().any(|e| !e.is_dir));
        }

        // Test non-existent account
        assert!(manager.list_remote_entries("unknown", "path").is_err());

        // Test offline account
        let mut offline_acc = RemoteStorageAccount::new(
            "offline-1",
            "Offline Account",
            StorageProviderKind::WebDAV,
            "https://offline.local",
            "/",
        );
        offline_acc.is_online = false;
        manager.register_account(offline_acc).unwrap();
        assert!(manager.list_remote_entries("offline-1", "/").is_err());
    }

    #[test]
    fn test_trigger_sync_transitions() {
        let mut manager = StorageProviderManager::new();

        let account = RemoteStorageAccount::new(
            "sync-acc",
            "Sync Account",
            StorageProviderKind::EdgeMeshP2P,
            "https://mesh.local",
            "/sync",
        );
        manager.register_account(account).unwrap();

        // Initial state is Idle -> transitions to Syncing
        let res1 = manager.trigger_sync("sync-acc").unwrap();
        match res1 {
            SyncStatus::Syncing { files_remaining, bytes_transferred } => {
                assert_eq!(files_remaining, 5);
                assert_eq!(bytes_transferred, 4096);
            }
            _ => panic!("Expected Syncing status"),
        }

        // Second trigger transitions Syncing -> Idle
        let res2 = manager.trigger_sync("sync-acc").unwrap();
        assert_eq!(res2, SyncStatus::Idle);

        // Non-existent account
        assert!(manager.trigger_sync("non-existent").is_err());

        // Offline account trigger sync sets status to Error
        let mut offline_acc = RemoteStorageAccount::new(
            "offline-sync",
            "Offline Sync Account",
            StorageProviderKind::WebDAV,
            "https://offline.local",
            "/",
        );
        offline_acc.is_online = false;
        manager.register_account(offline_acc).unwrap();

        assert!(manager.trigger_sync("offline-sync").is_err());
        let updated_acc = manager.get_account("offline-sync").unwrap();
        match &updated_acc.sync_status {
            SyncStatus::Error(msg) => assert!(msg.contains("offline")),
            _ => panic!("Expected Error status"),
        }
    }

    #[test]
    fn test_generate_share_link_all_kinds() {
        let mut manager = StorageProviderManager::new();

        let p2p = RemoteStorageAccount::new(
            "p2p-node",
            "P2P",
            StorageProviderKind::EdgeMeshP2P,
            "https://p2p.local",
            "/",
        );
        let webdav = RemoteStorageAccount::new(
            "dav-node",
            "DAV",
            StorageProviderKind::WebDAV,
            "https://dav.local/remote",
            "/",
        );
        let s3 = RemoteStorageAccount::new(
            "s3-node",
            "S3",
            StorageProviderKind::S3Compatible,
            "https://s3.local",
            "my-bucket",
        );
        let sftp = RemoteStorageAccount::new(
            "sftp-node",
            "SFTP",
            StorageProviderKind::SftpShare,
            "https://sftp.local",
            "/",
        );
        let local = RemoteStorageAccount::new(
            "local-node",
            "Local",
            StorageProviderKind::LocalDisk,
            "https://local.path",
            "/",
        );

        manager.register_account(p2p).unwrap();
        manager.register_account(webdav).unwrap();
        manager.register_account(s3).unwrap();
        manager.register_account(sftp).unwrap();
        manager.register_account(local).unwrap();

        let link_p2p = manager.generate_share_link("p2p-node", "/docs/file.txt").unwrap();
        assert_eq!(link_p2p, "swal-p2p://p2p-node/docs/file.txt");

        let link_dav = manager.generate_share_link("dav-node", "/docs/file.txt").unwrap();
        assert_eq!(link_dav, "https://dav.local/remote/dav/docs/file.txt");

        let link_s3 = manager.generate_share_link("s3-node", "/docs/file.txt").unwrap();
        assert_eq!(link_s3, "https://s3.local/my-bucket/docs/file.txt");

        let link_sftp = manager.generate_share_link("sftp-node", "/docs/file.txt").unwrap();
        assert_eq!(link_sftp, "https://sftp.local/sftp/docs/file.txt");

        let link_local = manager.generate_share_link("local-node", "/docs/file.txt").unwrap();
        assert_eq!(link_local, "file://https://local.path/docs/file.txt");

        // Non-existent and offline checks
        assert!(manager.generate_share_link("unknown", "file.txt").is_err());

        manager.get_account_mut("p2p-node").unwrap().is_online = false;
        assert!(manager.generate_share_link("p2p-node", "file.txt").is_err());
    }
}
