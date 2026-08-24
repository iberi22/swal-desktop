#[path = "../src/tui.rs"]
mod tui;

#[test]
fn test_tui_module_integration() {
    use tempfile::tempdir;
    use std::fs;
    use tui::*;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sample.txt");
    fs::write(&file_path, "sample content").unwrap();

    let mut app = TuiFileManagerApp::new(dir.path());
    assert_eq!(app.layout_mode, TuiLayoutMode::SinglePane);
    assert_eq!(app.theme, TuiColorTheme::SwalDark);

    let viewport = TuiViewport::default();
    let rendered = app.render_to_buffer(&viewport);
    assert!(rendered.contains("SWAL Files ::"));

    let preview = app.get_preview_text(5);
    assert!(!preview.is_empty());

    app.toggle_dual_pane();
    assert_eq!(app.layout_mode, TuiLayoutMode::DualPane);

    app.search_filter("sample");
    assert_eq!(app.items.len(), 1);
}
