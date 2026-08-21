use swal_files::dual_pane::{DualPaneController, PaneSide};
use swal_files::storage::{DiskUsageScanner, DriveInfo};
use swal_files::tabs_extended::{ExtendedTabInfo, ExtendedTabManager};
use tempfile::tempdir;

#[test]
fn test_dual_pane_controller_split_and_focus() {
    let dir = tempdir().unwrap();
    let left = dir.path().join("left");
    let right = dir.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();

    let mut controller = DualPaneController::new(left.clone(), right.clone());
    assert!(!controller.enabled);
    assert_eq!(controller.state.active_pane, PaneSide::Left);

    // Toggle on
    assert!(controller.toggle_dual_pane());
    assert!(controller.enabled);

    // Switch focus
    controller.focus_right();
    assert_eq!(controller.state.active_pane, PaneSide::Right);

    controller.focus_left();
    assert_eq!(controller.state.active_pane, PaneSide::Left);

    // Sync panes
    let sub_left = left.join("sub");
    std::fs::create_dir(&sub_left).unwrap();
    controller.state.left_path = sub_left.clone();
    controller.sync_panes();
    assert_eq!(controller.state.right_path, sub_left);

    // Swap panes
    controller.state.right_path = right.clone();
    controller.swap_panes();
    assert_eq!(controller.state.left_path, right);
    assert_eq!(controller.state.right_path, sub_left);

    // Toggle off
    assert!(!controller.toggle_dual_pane());
}

#[test]
fn test_storage_scanner_and_drive_info() {
    let drive = DriveInfo::new(
        "/".to_string(),
        "ext4".to_string(),
        1_000_000_000,
        400_000_000,
        false,
    );

    assert_eq!(drive.mount_point, "/");
    assert!((drive.used_percentage - 60.0).abs() < 0.1);
    assert!(!drive.formatted_total().is_empty());
    assert!(!drive.formatted_available().is_empty());
    assert!(!drive.formatted_used().is_empty());

    // Scan mounted drives on current Linux system
    let scanner = DiskUsageScanner::new();
    let drives = scanner.scan_mounted_drives();
    assert!(!drives.is_empty(), "Expected at least one mounted drive");
}

#[test]
fn test_extended_tab_manager_lifecycle() {
    let dir = tempdir().unwrap();
    let p1 = dir.path().join("p1");
    let p2 = dir.path().join("p2");
    std::fs::create_dir(&p1).unwrap();
    std::fs::create_dir(&p2).unwrap();

    let mut manager = ExtendedTabManager::new();
    let t1 = ExtendedTabInfo::new(1, "Tab 1", p1.clone());
    let t2 = ExtendedTabInfo::new(2, "Tab 2", p2.clone());

    let id1 = manager.add_tab(t1);
    let id2 = manager.add_tab(t2);

    assert_eq!(manager.tabs.len(), 2);
    assert_eq!(manager.active_tab_id, Some(id1));

    // Duplicate tab
    let dup = manager.duplicate_tab(id1).expect("Duplication failed");
    assert_eq!(manager.tabs.len(), 3);
    assert_eq!(dup.path, p1);

    // Reorder tabs
    assert!(manager.move_tab(0, 2));

    // Tooltip metadata
    let tab = &manager.tabs[0];
    assert!(tab.tooltip().contains("Path:"));

    // Close other tabs
    manager.close_other_tabs(id2);
    assert_eq!(manager.tabs.len(), 1);
    assert_eq!(manager.active_tab_id, Some(id2));
}
