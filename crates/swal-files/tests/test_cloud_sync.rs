#[path = "../src/cloud_sync.rs"]
mod cloud_sync;

pub use cloud_sync::*;

#[test]
fn test_cloud_sync_integration() {
    let mut manager = StorageProviderManager::new();

    let account = RemoteStorageAccount::new(
        "edge-p2p-1",
        "Edge Mesh Node 1",
        StorageProviderKind::EdgeMeshP2P,
        "https://mesh.swal.internal:8006",
        "/shared",
    );

    assert!(manager.register_account(account).is_ok());
    let accounts = manager.list_accounts();
    assert_eq!(accounts.len(), 1);

    let entries = manager.list_remote_entries("edge-p2p-1", "/").unwrap();
    assert!(!entries.is_empty());

    let status = manager.trigger_sync("edge-p2p-1").unwrap();
    match status {
        SyncStatus::Syncing { .. } => {}
        _ => panic!("Expected syncing status"),
    }

    let share_link = manager.generate_share_link("edge-p2p-1", "file.dat").unwrap();
    assert_eq!(share_link, "swal-p2p://edge-p2p-1/file.dat");
}
