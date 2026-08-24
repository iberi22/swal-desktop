//! Integration tests for swal-files platform abstraction module

#[path = "../src/platform.rs"]
pub mod platform;

use platform::{OsPlatform, PlatformAbstraction, SystemFolder};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_integration_os_and_system_folders() {
    let os = PlatformAbstraction::detect_os();
    assert_ne!(os, OsPlatform::Unknown);

    let home = PlatformAbstraction::get_system_folder(SystemFolder::Home);
    assert!(home.is_some(), "Home folder should be resolved");

    let config = PlatformAbstraction::get_system_folder(SystemFolder::Config);
    assert!(config.is_some(), "Config folder should be resolved");
}

#[test]
fn test_integration_list_drives() {
    let drives = PlatformAbstraction::list_system_drives();
    assert!(!drives.is_empty(), "Should return at least one system drive");

    for d in &drives {
        assert!(!d.mount_point.is_empty());
    }
}

#[test]
fn test_integration_path_normalization_matrix() {
    // Unix path tests
    assert_eq!(
        PlatformAbstraction::normalize_path(Path::new("/var/log/../lib/app")),
        PathBuf::from("/var/lib/app")
    );

    // Windows path tests
    assert_eq!(
        PlatformAbstraction::normalize_path(Path::new("C:\\Program Files\\..\\Windows\\System32")),
        PathBuf::from("C:\\Windows\\System32")
    );

    // Windows UNC path tests
    assert_eq!(
        PlatformAbstraction::normalize_path(Path::new("\\\\server\\share\\docs\\..\\images")),
        PathBuf::from("\\\\server\\share\\images")
    );
}

#[test]
fn test_integration_move_to_trash_and_open_app_errors() {
    let dir = tempdir().expect("Failed to create tempdir");
    let file_path = dir.path().join("integration_trash_item.txt");
    fs::write(&file_path, "swal test content").expect("Write failed");

    assert!(file_path.exists());
    let trash_res = PlatformAbstraction::move_to_trash(&file_path);
    assert!(trash_res.is_ok(), "Moving file to trash should succeed");
    assert!(!file_path.exists(), "File should be moved to trash");

    let ghost_path = dir.path().join("non_existent_file.txt");
    let err = PlatformAbstraction::open_with_default_app(&ghost_path);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().kind(), ErrorKind::NotFound);
}
