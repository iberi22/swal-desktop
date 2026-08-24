//! Integration tests for ArchiveInspector and BatchTagManager in swal-files

use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use swal_files::archive::{ArchiveError, ArchiveInspector, ArchiveKind, BatchTagManager};
use tempfile::tempdir;
use zip::write::FileOptions;

#[test]
fn test_archive_inspector_zip_full_flow() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("full_test.zip");

    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.add_directory("assets/", opts).unwrap();
        zip.start_file("assets/logo.svg", opts).unwrap();
        zip.write_all(b"<svg><circle r='10'/></svg>").unwrap();

        zip.start_file("assets/notes.txt", opts).unwrap();
        zip.write_all(b"SWAL Desktop Files Archive Subsystem\nBatchTagManager Ready\nLine3").unwrap();

        zip.add_directory("deep/", opts).unwrap();
        zip.add_directory("deep/nested/folder/", opts).unwrap();
        zip.start_file("deep/nested/folder/file.rs", opts).unwrap();
        zip.write_all(b"fn main() {}").unwrap();

        zip.set_comment("Test archive comment");
        zip.finish().unwrap();
    }

    let meta = ArchiveInspector::inspect(&zip_path).unwrap();
    assert_eq!(meta.kind, ArchiveKind::Zip);
    assert_eq!(meta.comment, Some("Test archive comment".to_string()));
    assert_eq!(meta.total_entries, 6);
    assert_eq!(meta.file_count, 3);
    assert_eq!(meta.dir_count, 3);
    assert!(meta.uncompressed_size_bytes > 0);
    assert!(meta.formatted_uncompressed_size.contains('B'));

    // Test directory listing within archive
    let root_items = ArchiveInspector::list_directory_entries(&zip_path, "").unwrap();
    assert_eq!(root_items.len(), 2); // assets and deep

    let asset_items = ArchiveInspector::list_directory_entries(&zip_path, "assets").unwrap();
    assert_eq!(asset_items.len(), 2); // logo.svg and notes.txt

    // Test text preview
    let preview = ArchiveInspector::preview_text_entry(&zip_path, "assets/notes.txt", 2).unwrap();
    assert_eq!(preview, "SWAL Desktop Files Archive Subsystem\nBatchTagManager Ready");

    // Test single entry extraction to memory
    let bytes = ArchiveInspector::extract_single_entry_to_memory(&zip_path, "deep/nested/folder/file.rs").unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap(), "fn main() {}");

    // Test entry not found
    let err = ArchiveInspector::extract_single_entry_to_memory(&zip_path, "nonexistent.txt");
    assert!(matches!(err, Err(ArchiveError::EntryNotFound(_))));
}

#[test]
fn test_archive_inspector_tar_uncompressed() {
    let dir = tempdir().unwrap();
    let tar_path = dir.path().join("plain.tar");

    {
        let file = File::create(&tar_path).unwrap();
        let mut tar_builder = tar::Builder::new(file);

        let data = b"Sample plain tar data stream";
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar_builder.append_data(&mut header, "config/settings.toml", &data[..]).unwrap();
        tar_builder.finish().unwrap();
    }

    let meta = ArchiveInspector::inspect(&tar_path).unwrap();
    assert_eq!(meta.kind, ArchiveKind::Tar);
    assert_eq!(meta.total_entries, 1);
    assert_eq!(meta.entries[0].name, "settings.toml");
    assert_eq!(meta.entries[0].path, "config/settings.toml");

    let extracted = ArchiveInspector::extract_single_entry_to_memory(&tar_path, "config/settings.toml").unwrap();
    assert_eq!(String::from_utf8(extracted).unwrap(), "Sample plain tar data stream");
}

#[test]
fn test_archive_inspector_tar_gz_flow() {
    let dir = tempdir().unwrap();
    let tgz_path = dir.path().join("bundle.tgz");

    {
        let file = File::create(&tgz_path).unwrap();
        let gz = GzEncoder::new(file, Compression::default());
        let mut tar_builder = tar::Builder::new(gz);

        let data1 = b"Alpha beta gamma";
        let mut h1 = tar::Header::new_gnu();
        h1.set_size(data1.len() as u64);
        h1.set_mode(0o644);
        h1.set_cksum();
        tar_builder.append_data(&mut h1, "alpha.txt", &data1[..]).unwrap();

        let data2 = b"Delta epsilon";
        let mut h2 = tar::Header::new_gnu();
        h2.set_size(data2.len() as u64);
        h2.set_mode(0o644);
        h2.set_cksum();
        tar_builder.append_data(&mut h2, "beta.txt", &data2[..]).unwrap();

        tar_builder.finish().unwrap();
    }

    let meta = ArchiveInspector::inspect(&tgz_path).unwrap();
    assert_eq!(meta.kind, ArchiveKind::TarGz);
    assert_eq!(meta.total_entries, 2);

    let search = ArchiveInspector::search_entries(&tgz_path, "alpha").unwrap();
    assert_eq!(search.len(), 1);
    assert_eq!(search[0].name, "alpha.txt");
}

#[test]
fn test_archive_inspector_error_handling() {
    let dir = tempdir().unwrap();
    let invalid_path = dir.path().join("does_not_exist.zip");

    let res = ArchiveInspector::inspect(&invalid_path);
    assert!(matches!(res, Err(ArchiveError::Io(_))));

    // Test non-archive file
    let plain_file = dir.path().join("plain.txt");
    std::fs::write(&plain_file, b"Not an archive").unwrap();
    let res = ArchiveInspector::inspect(&plain_file);
    assert!(matches!(res, Err(ArchiveError::UnsupportedFormat(_))));
}

#[test]
fn test_batch_tag_manager_comprehensive_matrix() {
    let mut manager = BatchTagManager::new();

    let p1 = PathBuf::from("/home/user/docs/report.pdf");
    let p2 = PathBuf::from("/home/user/docs/summary.pdf");
    let p3 = PathBuf::from("/home/user/src/main.rs");
    let p4 = PathBuf::from("/home/user/src/lib.rs");

    // Add tags in batch
    manager.add_tags_batch(&[p1.clone(), p2.clone()], &["pdf", "docs", "q3"]);
    manager.add_tags_batch(&[p3.clone(), p4.clone()], &["rust", "code", "core"]);

    assert_eq!(manager.total_tagged_paths(), 4);
    assert_eq!(manager.total_unique_tags(), 6);

    // Test set_tags
    manager.set_tags(&p1, &["pdf", "reviewed"]);
    assert_eq!(manager.get_tags(&p1), vec!["pdf".to_string(), "reviewed".to_string()]);
    assert!(!manager.has_tag(&p1, "docs"));
    assert!(manager.has_tag(&p1, "reviewed"));

    // Clear specific path
    assert!(manager.clear_tags(&p1));
    assert_eq!(manager.get_tags(&p1).len(), 0);
    assert_eq!(manager.total_tagged_paths(), 3);

    // Rename tag across all
    let affected = manager.rename_tag("code", "programming");
    assert_eq!(affected, 2);
    assert_eq!(manager.find_paths_by_tag("programming").len(), 2);
    assert_eq!(manager.find_paths_by_tag("code").len(), 0);

    // Delete tag across all
    let deleted = manager.delete_tag("programming");
    assert_eq!(deleted, 2);
    assert_eq!(manager.find_paths_by_tag("programming").len(), 0);

    // Clear all
    manager.clear_all();
    assert_eq!(manager.total_tagged_paths(), 0);
    assert_eq!(manager.total_unique_tags(), 0);
}
