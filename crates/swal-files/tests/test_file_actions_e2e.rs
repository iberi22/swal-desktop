//! End-to-End Test Suite for SWAL Files Context Menu & File Actions (Wave FM.10)

use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
#[ignore = "wave FM.01 pendiente"]
fn test_menu_json_actions_and_paste_state() {
    let tmp = tempdir().expect("failed to create tempdir");
    let dir_path = tmp.path().to_str().expect("valid utf-8 path");

    let bin = env!("CARGO_BIN_EXE_swal-files");
    let output = Command::new(bin)
        .args(["menu-json", dir_path])
        .output()
        .expect("failed to execute swal-files menu-json");

    assert!(output.status.success(), "menu-json command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid json output from menu-json");

    let actions = json.get("actions").and_then(|v| v.as_array()).expect("actions array in menu-json");
    assert!(actions.len() >= 13, "menu-json should report at least 13 actions, got {}", actions.len());

    let paste_enabled = json.get("paste_enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    assert!(!paste_enabled, "paste action should be disabled when clipboard manifest is empty");
}

#[test]
#[ignore = "wave FM.02 pendiente"]
fn test_clip_copy_and_clip_paste_collision() {
    let tmp = tempdir().expect("failed to create tempdir");
    let src_file = tmp.path().join("source.txt");
    fs::write(&src_file, "Content A").expect("write source file");

    let dest_dir = tmp.path().join("dest");
    fs::create_dir(&dest_dir).expect("create dest dir");
    let existing_dest_file = dest_dir.join("source.txt");
    fs::write(&existing_dest_file, "Content B").expect("write existing dest file");

    let bin = env!("CARGO_BIN_EXE_swal-files");

    let copy_out = Command::new(bin)
        .args(["clip-copy", src_file.to_str().unwrap()])
        .output()
        .expect("failed to clip-copy");
    assert!(copy_out.status.success(), "clip-copy failed");

    let paste_out = Command::new(bin)
        .args(["clip-paste", dest_dir.to_str().unwrap()])
        .output()
        .expect("failed to clip-paste");
    assert!(paste_out.status.success(), "clip-paste failed");

    // Existing file should not be overwritten
    let existing_content = fs::read_to_string(&existing_dest_file).expect("read existing file");
    assert_eq!(existing_content, "Content B", "existing file was overwritten on collision");

    // Copied file should exist with suffix
    let entries: Vec<String> = fs::read_dir(&dest_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
        .collect();

    assert!(entries.len() >= 2, "expected copied file with suffix in destination");
    assert!(
        entries.iter().any(|n| n != "source.txt" && n.contains("source")),
        "collision suffix file not found in destination"
    );
}

#[test]
#[ignore = "wave FM.03 pendiente"]
fn test_clip_cut_and_clip_paste_move() {
    let tmp = tempdir().expect("failed to create tempdir");
    let src_file = tmp.path().join("move_me.txt");
    fs::write(&src_file, "Move content").expect("write move_me.txt");

    let dest_dir = tmp.path().join("target");
    fs::create_dir(&dest_dir).expect("create target dir");

    let bin = env!("CARGO_BIN_EXE_swal-files");

    let cut_out = Command::new(bin)
        .args(["clip-cut", src_file.to_str().unwrap()])
        .output()
        .expect("failed to clip-cut");
    assert!(cut_out.status.success(), "clip-cut failed");

    let paste_out = Command::new(bin)
        .args(["clip-paste", dest_dir.to_str().unwrap()])
        .output()
        .expect("failed to clip-paste");
    assert!(paste_out.status.success(), "clip-paste failed");

    assert!(!src_file.exists(), "original file should be moved/deleted from source after cut-paste");
    let moved_file = dest_dir.join("move_me.txt");
    assert!(moved_file.exists(), "moved file should exist in destination directory");

    // Cut manifest should be cleared after paste
    let second_paste = Command::new(bin)
        .args(["clip-paste", dest_dir.to_str().unwrap()])
        .output()
        .expect("failed second clip-paste");

    // Second paste should fail or be a no-op since cut manifest was cleared
    assert!(
        !second_paste.status.success() || fs::read_dir(&dest_dir).unwrap().count() == 1,
        "cut manifest was not cleared after paste"
    );
}

#[test]
#[ignore = "wave FM.04 pendiente"]
fn test_rename_item_invalid_name_fails() {
    let tmp = tempdir().expect("failed to create tempdir");
    let target_file = tmp.path().join("valid_name.txt");
    fs::write(&target_file, "data").expect("write valid_name.txt");

    let bin = env!("CARGO_BIN_EXE_swal-files");

    let rename_out = Command::new(bin)
        .args(["rename-item", target_file.to_str().unwrap(), "a/b"])
        .output()
        .expect("failed to execute rename-item");

    assert!(!rename_out.status.success(), "rename-item with invalid path separator 'a/b' should fail");
    assert!(target_file.exists(), "original file must remain untouched when rename fails");
    assert!(!tmp.path().join("a").exists(), "no side effect directory should be created on rename failure");
}

#[test]
#[ignore = "wave FM.05 pendiente"]
fn test_delete_item_requires_confirmation() {
    let tmp = tempdir().expect("failed to create tempdir");
    let target_file = tmp.path().join("protected_file.txt");
    fs::write(&target_file, "do not delete").expect("write protected_file.txt");

    let bin = env!("CARGO_BIN_EXE_swal-files");

    let delete_unconfirmed = Command::new(bin)
        .args(["delete-item", target_file.to_str().unwrap()])
        .output()
        .expect("failed to execute delete-item");

    assert!(target_file.exists(), "delete-item without --confirm=BORRAR must not delete the target file");
    assert!(!delete_unconfirmed.status.success() || target_file.exists());
}

#[test]
#[ignore = "wave FM.06 pendiente"]
fn test_properties_json_file_metadata() {
    let tmp = tempdir().expect("failed to create tempdir");
    let file_path = tmp.path().join("sample.bin");
    let mut file = File::create(&file_path).expect("create sample.bin");
    let sample_bytes = vec![0u8; 256];
    file.write_all(&sample_bytes).expect("write 256 bytes");
    drop(file);

    let bin = env!("CARGO_BIN_EXE_swal-files");

    let output = Command::new(bin)
        .args(["properties-json", file_path.to_str().unwrap()])
        .output()
        .expect("failed to execute properties-json");

    assert!(output.status.success(), "properties-json command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid json from properties-json");

    let size_bytes = json.get("size_bytes").and_then(|v| v.as_u64()).expect("size_bytes field in properties-json");
    assert_eq!(size_bytes, 256, "properties-json size_bytes mismatch");

    let mode_octal = json.get("mode_octal").and_then(|v| v.as_str()).expect("mode_octal field in properties-json");
    assert!(!mode_octal.is_empty(), "mode_octal should non-empty octal string");
}
