use std::fs;
use swal_files::config::FileManagerConfig;
use swal_files::entry::{FileCategory, FileEntry};
use swal_files::scanner::{group_entries, scan_directory, GroupBy, ScanOptions, SortBy};
use tempfile::tempdir;

#[test]
fn test_code_format_detection_rust_python_ts() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let rs_file = p.join("main.rs");
    fs::write(&rs_file, "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();

    let py_file = p.join("script.py");
    fs::write(&py_file, "def hello():\n    print('Hello')\n").unwrap();

    let ts_file = p.join("index.ts");
    fs::write(&ts_file, "export const val: number = 42;\n").unwrap();

    let e_rs = FileEntry::from_path(&rs_file).unwrap();
    assert_eq!(e_rs.category, FileCategory::Code);
    assert_eq!(e_rs.icon, "🦀");
    assert!(e_rs.matches_filter("code"));
    assert!(!e_rs.matches_filter("images"));

    let e_py = FileEntry::from_path(&py_file).unwrap();
    assert_eq!(e_py.category, FileCategory::Code);
    assert_eq!(e_py.icon, "🐍");
    assert!(e_py.matches_filter("code"));

    let e_ts = FileEntry::from_path(&ts_file).unwrap();
    assert_eq!(e_ts.category, FileCategory::Code);
    assert_eq!(e_ts.icon, "📜");
}

#[test]
fn test_document_and_config_formats() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let md_file = p.join("README.md");
    fs::write(&md_file, "# Project Overview\nDetails here.").unwrap();

    let json_file = p.join("config.json");
    fs::write(&json_file, "{\"theme\": \"hive-dark\"}").unwrap();

    let toml_file = p.join("Cargo.toml");
    fs::write(&toml_file, "[package]\nname = \"swal\"").unwrap();

    let e_md = FileEntry::from_path(&md_file).unwrap();
    assert_eq!(e_md.category, FileCategory::Document);
    assert_eq!(e_md.icon, "📝");
    assert!(e_md.matches_filter("documents"));

    let e_json = FileEntry::from_path(&json_file).unwrap();
    assert_eq!(e_json.category, FileCategory::Config);
    assert_eq!(e_json.icon, "⚙️");

    let e_toml = FileEntry::from_path(&toml_file).unwrap();
    assert_eq!(e_toml.category, FileCategory::Config);
}

#[test]
fn test_image_and_multimedia_formats() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let png_file = p.join("photo.png");
    fs::write(&png_file, b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR").unwrap();

    let mp3_file = p.join("sound.mp3");
    fs::write(&mp3_file, b"ID3\x03\x00\x00\x00\x00").unwrap();

    let mp4_file = p.join("video.mp4");
    fs::write(&mp4_file, b"\x00\x00\x00\x18ftypmp42").unwrap();

    let e_png = FileEntry::from_path(&png_file).unwrap();
    assert_eq!(e_png.category, FileCategory::Image);
    assert_eq!(e_png.icon, "🖼️");
    assert!(e_png.matches_filter("images"));

    let e_mp3 = FileEntry::from_path(&mp3_file).unwrap();
    assert_eq!(e_mp3.category, FileCategory::Audio);
    assert_eq!(e_mp3.icon, "🎵");
    assert!(e_mp3.matches_filter("media"));

    let e_mp4 = FileEntry::from_path(&mp4_file).unwrap();
    assert_eq!(e_mp4.category, FileCategory::Video);
    assert_eq!(e_mp4.icon, "🎬");
    assert!(e_mp4.matches_filter("media"));
}

#[test]
fn test_binary_hex_dump_formatting() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    let bin_file = p.join("program.bin");
    let bytes: Vec<u8> = (0..64).collect();
    fs::write(&bin_file, &bytes).unwrap();

    let e_bin = FileEntry::from_path(&bin_file).unwrap();
    assert_eq!(e_bin.category, FileCategory::Binary);
    assert_eq!(e_bin.icon, "💽");
    assert_eq!(e_bin.size_bytes, 64);
}

#[test]
fn test_filtering_and_grouping_engine() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    // Create mixed entries
    fs::create_dir(p.join("src")).unwrap();
    fs::create_dir(p.join("docs")).unwrap();
    fs::write(p.join("app.rs"), "fn app() {}").unwrap();
    fs::write(p.join("main.py"), "print('hi')").unwrap();
    fs::write(p.join("notes.txt"), "some notes").unwrap();
    fs::write(p.join("banner.png"), b"fake png").unwrap();
    fs::write(p.join("archive.zip"), b"fake zip").unwrap();

    // 1. Filter folders
    let opts_folders = ScanOptions {
        show_hidden: false,
        sort_by: SortBy::Name,
        ascending: true,
        filter_query: None,
        filter_category: "folders".to_string(),
        group_by: GroupBy::None,
    };
    let folders = scan_directory(p, &opts_folders).unwrap();
    assert_eq!(folders.len(), 2);
    assert!(folders.iter().all(|f| f.is_dir));

    // 2. Filter code
    let opts_code = ScanOptions {
        show_hidden: false,
        sort_by: SortBy::Name,
        ascending: true,
        filter_query: None,
        filter_category: "code".to_string(),
        group_by: GroupBy::None,
    };
    let code_files = scan_directory(p, &opts_code).unwrap();
    assert_eq!(code_files.len(), 2); // app.rs and main.py

    // 3. Group by Type
    let opts_all = ScanOptions::default();
    let all = scan_directory(p, &opts_all).unwrap();
    let grouped_by_type = group_entries(&all, GroupBy::Type);

    assert!(grouped_by_type.iter().any(|g| g.title.contains("Carpetas")));
    assert!(grouped_by_type.iter().any(|g| g.title.contains("Código Fuente")));
    assert!(grouped_by_type.iter().any(|g| g.title.contains("Documentos")));
    assert!(grouped_by_type.iter().any(|g| g.title.contains("Imágenes")));

    // 4. Group by Alphabetical
    let grouped_by_alpha = group_entries(&all, GroupBy::Alphabetical);
    assert!(!grouped_by_alpha.is_empty());
}

#[test]
fn test_config_persistence_and_theme_roundtrip() {
    let dir = tempdir().unwrap();
    let cfg_path = dir.path().join("config.json");

    let mut cfg = FileManagerConfig::default();
    cfg.theme_id = "cyber-neon".to_string();
    cfg.group_by = "type".to_string();
    cfg.filter_type = "code".to_string();
    cfg.preview_mode = "sidebar".to_string();

    cfg.save_to_path(&cfg_path).unwrap();

    let loaded = FileManagerConfig::load_from_path(&cfg_path);
    assert_eq!(loaded.theme_id, "cyber-neon");
    assert_eq!(loaded.group_by, "type");
    assert_eq!(loaded.filter_type, "code");
    assert_eq!(loaded.preview_mode, "sidebar");
}

#[test]
fn test_pin_and_unpin_favorites_lifecycle() {
    let dir = tempdir().unwrap();
    let cfg_path = dir.path().join("config.json");

    let mut cfg = FileManagerConfig::default();

    // Default pins contain Home, Descargas, Documentos, Proyectos SWAL
    assert!(cfg.is_pinned(&dirs::home_dir().unwrap_or_default().join("Documents")));

    // User unpins "Documentos"
    let docs_path = dirs::home_dir().unwrap_or_default().join("Documents");
    assert!(cfg.remove_pin(&docs_path));
    assert!(!cfg.is_pinned(&docs_path));

    // User pins a new custom workspace / favorite folder
    let custom_proj = dir.path().join("my-cool-project");
    fs::create_dir(&custom_proj).unwrap();

    assert!(cfg.add_pin(custom_proj.clone(), Some("Cool Project".to_string()), Some("🚀".to_string()), Some("pinned".to_string())));
    assert!(cfg.is_pinned(&custom_proj));

    // Test toggle_pin
    let toggled_off = cfg.toggle_pin(custom_proj.clone());
    assert!(!toggled_off);
    assert!(!cfg.is_pinned(&custom_proj));

    let toggled_on = cfg.toggle_pin(custom_proj.clone());
    assert!(toggled_on);
    assert!(cfg.is_pinned(&custom_proj));

    // Save and verify roundtrip
    cfg.save_to_path(&cfg_path).unwrap();
    let loaded = FileManagerConfig::load_from_path(&cfg_path);
    assert!(!loaded.is_pinned(&docs_path));
    assert!(loaded.is_pinned(&custom_proj));
}

#[test]
fn test_nested_directory_navigation_and_breadcrumbs() {
    let root = tempdir().unwrap();
    let swal_dir = root.path().join("proyectosSWAL");
    let periferia_dir = swal_dir.join("periferia");
    let swal_desktop_dir = periferia_dir.join("swal-desktop");
    let crates_dir = swal_desktop_dir.join("crates");
    let swal_files_dir = crates_dir.join("swal-files");
    let src_dir = swal_files_dir.join("src");

    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn lib() {}").unwrap();
    fs::write(swal_desktop_dir.join("Cargo.toml"), "[workspace]").unwrap();

    // 1. Scan at proyectosSWAL level
    let scan_opts = ScanOptions::default();
    let entries_swal = scan_directory(&swal_dir, &scan_opts).unwrap();
    assert_eq!(entries_swal.len(), 1);
    assert_eq!(entries_swal[0].name, "periferia");
    assert!(entries_swal[0].is_dir);

    // 2. Navigate into periferia
    let entries_periferia = scan_directory(&periferia_dir, &scan_opts).unwrap();
    assert_eq!(entries_periferia.len(), 1);
    assert_eq!(entries_periferia[0].name, "swal-desktop");
    assert!(entries_periferia[0].is_dir);

    // 3. Navigate into swal-desktop
    let entries_desktop = scan_directory(&swal_desktop_dir, &scan_opts).unwrap();
    assert!(entries_desktop.iter().any(|e| e.name == "crates" && e.is_dir));
    assert!(entries_desktop.iter().any(|e| e.name == "Cargo.toml" && !e.is_dir));

    // 4. Navigate into crates -> swal-files -> src
    let entries_src = scan_directory(&src_dir, &scan_opts).unwrap();
    assert_eq!(entries_src.len(), 2);
    assert!(entries_src.iter().any(|e| e.name == "main.rs"));
    assert!(entries_src.iter().any(|e| e.name == "lib.rs"));
}

