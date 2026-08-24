//! test_directory_watcher.rs
//! Integration tests for DirectoryWatcher and FsChangeEvent.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use swal_files::watcher::{DirectoryWatcher, WatcherError};
use tempfile::tempdir;

#[test]
fn test_watcher_initialization_and_metadata() {
    let dir = tempdir().expect("tempdir");
    let watcher = DirectoryWatcher::new(dir.path()).expect("create watcher");

    assert!(watcher.is_watching());
    assert!(!watcher.is_recursive());
    assert_eq!(
        watcher.watched_path(),
        fs::canonicalize(dir.path()).unwrap().as_path()
    );
}

#[test]
fn test_watcher_detects_create_modify_delete() {
    let dir = tempdir().expect("tempdir");
    let watcher = DirectoryWatcher::new(dir.path()).expect("create watcher");

    // Initially no events
    assert_eq!(watcher.try_recv(), None);
    assert!(watcher.poll_events().is_empty());

    // Create file
    let file_path = dir.path().join("alpha.txt");
    {
        let mut f = File::create(&file_path).expect("create alpha.txt");
        f.write_all(b"initial content\n").expect("write");
        f.sync_all().expect("sync");
    }

    let mut event_received = false;
    for _ in 0..25 {
        if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
            if evt.path() == file_path || evt.path().ends_with("alpha.txt") {
                event_received = true;
                break;
            }
        }
    }
    assert!(event_received, "Watcher should receive event on file creation");

    // Modify file
    {
        let mut f = fs::OpenOptions::new()
            .append(true)
            .open(&file_path)
            .expect("open alpha.txt");
        f.write_all(b"second line\n").expect("append");
        f.sync_all().expect("sync");
    }

    let mut modified_received = false;
    for _ in 0..25 {
        if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
            if evt.is_modified() && (evt.path() == file_path || evt.path().ends_with("alpha.txt")) {
                modified_received = true;
                break;
            }
        }
    }
    assert!(modified_received, "Watcher should receive modify event");

    // Delete file
    fs::remove_file(&file_path).expect("delete alpha.txt");

    let mut delete_received = false;
    for _ in 0..25 {
        if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
            if evt.is_deleted() && (evt.path() == file_path || evt.path().ends_with("alpha.txt")) {
                delete_received = true;
                break;
            }
        }
    }
    assert!(delete_received, "Watcher should receive delete event");
}

#[test]
fn test_watcher_switch_directory_and_unwatch() {
    let dir_1 = tempdir().expect("tempdir 1");
    let dir_2 = tempdir().expect("tempdir 2");

    let mut watcher = DirectoryWatcher::new(dir_1.path()).expect("create watcher");
    assert!(watcher.is_watching());

    watcher.unwatch().expect("unwatch");
    assert!(!watcher.is_watching());

    watcher.watch_path(dir_2.path(), false).expect("watch dir 2");
    assert!(watcher.is_watching());
    assert_eq!(
        watcher.watched_path(),
        fs::canonicalize(dir_2.path()).unwrap().as_path()
    );
}

#[test]
fn test_watcher_recursive_mode() {
    let root_dir = tempdir().expect("tempdir root");
    let sub_dir = root_dir.path().join("nested_folder");
    fs::create_dir(&sub_dir).expect("create sub dir");

    let watcher = DirectoryWatcher::new_recursive(root_dir.path()).expect("create recursive watcher");
    assert!(watcher.is_recursive());

    let nested_file = sub_dir.join("sub_file.txt");
    {
        let mut f = File::create(&nested_file).expect("create nested file");
        f.write_all(b"nested text\n").expect("write nested file");
        f.sync_all().expect("sync");
    }

    let mut detected = false;
    for _ in 0..25 {
        if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
            if evt.path() == nested_file || evt.path().ends_with("sub_file.txt") {
                detected = true;
                break;
            }
        }
    }
    assert!(detected, "Recursive watcher should detect nested file changes");
}

#[test]
fn test_watcher_error_display_and_source() {
    let not_found = WatcherError::PathNotFound(PathBuf::from("/nonexistent"));
    assert!(not_found.to_string().contains("Path not found"));

    let not_dir = WatcherError::NotADirectory(PathBuf::from("/file.txt"));
    assert!(not_dir.to_string().contains("Path is not a directory"));

    let not_init = WatcherError::WatcherNotInitialized;
    assert!(not_init.to_string().contains("not initialized"));
}
