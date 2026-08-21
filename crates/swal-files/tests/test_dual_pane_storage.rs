use std::path::PathBuf;
use swal_files::config::FileManagerConfig;
use swal_files::scanner::{scan_disk_usage, scan_mounts};
use swal_files::{FileManagerSession, PaneFocus};
use tempfile::tempdir;

#[test]
fn test_dual_pane_split_and_focus_switching() {
    let dir = tempdir().unwrap();
    let left_path = dir.path().join("left_pane");
    let right_path = dir.path().join("right_pane");
    std::fs::create_dir(&left_path).unwrap();
    std::fs::create_dir(&right_path).unwrap();

    let config = FileManagerConfig::default();
    let mut session = FileManagerSession::new(left_path.clone(), config);

    assert!(!session.is_dual_pane());
    assert_eq!(session.active_pane(), PaneFocus::Primary);

    // Split dual pane with right_path as secondary
    session.split_dual_pane(Some(right_path.clone()));
    assert!(session.is_dual_pane());
    assert_eq!(session.dual_pane_path, Some(right_path.clone()));

    // Switch focus between Primary (left) and Secondary (right)
    session.switch_pane_focus();
    assert_eq!(session.active_pane(), PaneFocus::Secondary);

    session.switch_pane_focus();
    assert_eq!(session.active_pane(), PaneFocus::Primary);

    // Explicit set focus
    session.set_pane_focus(PaneFocus::Secondary);
    assert_eq!(session.active_pane(), PaneFocus::Secondary);

    // Close dual pane
    session.close_dual_pane();
    assert!(!session.is_dual_pane());
    assert_eq!(session.active_pane(), PaneFocus::Primary);
}

#[test]
fn test_dual_pane_path_synchronization() {
    let dir = tempdir().unwrap();
    let p1 = dir.path().join("pane1");
    let p2 = dir.path().join("pane2");
    std::fs::create_dir(&p1).unwrap();
    std::fs::create_dir(&p2).unwrap();

    let config = FileManagerConfig::default();
    let mut session = FileManagerSession::new(p1.clone(), config);
    session.split_dual_pane(Some(p2.clone()));

    // Focus Primary: sync_dual_pane_paths copies active tab path (p1) to dual_pane_path
    session.set_pane_focus(PaneFocus::Primary);

    let p3 = dir.path().join("pane3");
    std::fs::create_dir(&p3).unwrap();
    session.active_tab_mut().navigate_to(p3.clone());
    assert_eq!(session.active_tab().current_path, p3);

    session.sync_dual_pane_paths();
    assert_eq!(session.dual_pane_path, Some(p3.clone()));

    // Focus Secondary: sync_dual_pane_paths navigates active tab to dual_pane_path
    session.dual_pane_path = Some(p2.clone());
    session.set_pane_focus(PaneFocus::Secondary);
    session.sync_dual_pane_paths();
    assert_eq!(session.active_tab().current_path, p2);
}

#[test]
fn test_disk_usage_scanner_root_and_mounts() {
    let dir = tempdir().unwrap();
    let root = PathBuf::from("/");

    let root_usage = scan_disk_usage(&root).expect("Failed to scan root disk usage");
    assert_eq!(root_usage.mount_point, root);
    assert!(root_usage.total_bytes > 0);
    assert!(root_usage.available_bytes <= root_usage.total_bytes);
    assert!(root_usage.usage_pct >= 0.0 && root_usage.usage_pct <= 100.0);

    let temp_usage = scan_disk_usage(dir.path()).expect("Failed to scan tempdir disk usage");
    assert!(temp_usage.total_bytes > 0);

    let mounts = scan_mounts().expect("Failed to scan system mounts");
    assert!(!mounts.is_empty());
    assert!(mounts.iter().any(|m| m.mount_point == root));
}

#[test]
fn test_extended_tab_reordering_and_duplication() {
    let dir = tempdir().unwrap();
    let p1 = dir.path().join("dir1");
    let p2 = dir.path().join("dir2");
    let p3 = dir.path().join("dir3");
    std::fs::create_dir(&p1).unwrap();
    std::fs::create_dir(&p2).unwrap();
    std::fs::create_dir(&p3).unwrap();

    let config = FileManagerConfig::default();
    let mut session = FileManagerSession::new(p1.clone(), config);
    session.new_tab(p2.clone());
    session.new_tab(p3.clone());

    assert_eq!(session.tabs.len(), 3);
    assert_eq!(session.active_tab_idx, 2);
    assert_eq!(session.active_tab().current_path, p3);

    // Tab duplication
    let dup_idx = session.duplicate_tab(0).expect("Failed to duplicate tab 0");
    assert_eq!(dup_idx, 1);
    assert_eq!(session.tabs.len(), 4);
    assert_eq!(session.tabs[1].current_path, p1);
    assert_ne!(session.tabs[0].id, session.tabs[1].id); // ID must be unique

    // Tab reordering
    let reorder_ok = session.reorder_tab(0, 3);
    assert!(reorder_ok);
    assert_eq!(session.tabs.len(), 4);

    // Invalid index reordering
    assert!(!session.reorder_tab(10, 0));
}
