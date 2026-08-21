use std::fs;
use std::path::{Path, PathBuf};
use swal_files::agent::{execute_local_agent_action, AgentActionRequest};
use swal_files::cli::handle_command;
use swal_files::config::FileManagerConfig;
use swal_files::entry::{FileCategory, FileEntry, GitStatus};
use swal_files::git::detect_git_status_for_dir;
use swal_files::gui::{build_gui_payload, get_breadcrumbs};
use swal_files::omnibar::{parse_omnibar_input, OmnibarIntent};
use swal_files::preview::{
    format_preview_bytes, generate_preview_for_path, load_editor_state_from_path,
    save_editor_state_to_path, PreviewState,
};
use swal_files::scanner::{
    group_entries, scan_directory, GroupBy, ScanOptions, SortBy,
};
use swal_files::session::{
    load_session_from_path, save_session_to_path, SessionState, TabState,
};
use tempfile::tempdir;


#[test]
fn test_agent_actions_coverage() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("code.rs");
    fs::write(&file, "fn test() {}").unwrap();

    // 1. Summarize file
    let req_sum_file = AgentActionRequest {
        target_path: file.clone(),
        action_type: "summarize".to_string(),
        prompt: None,
    };
    let res = execute_local_agent_action(&req_sum_file);
    assert!(res.success);
    assert!(res.output_summary.contains("code.rs"));

    // 2. Summarize directory
    let req_sum_dir = AgentActionRequest {
        target_path: dir.path().to_path_buf(),
        action_type: "summarize".to_string(),
        prompt: None,
    };
    let res_dir = execute_local_agent_action(&req_sum_dir);
    assert!(res_dir.success);

    // 3. Index memory
    let req_idx = AgentActionRequest {
        target_path: file.clone(),
        action_type: "index_memory".to_string(),
        prompt: None,
    };
    let res_idx = execute_local_agent_action(&req_idx);
    assert!(res_idx.success);
    assert!(res_idx.output_summary.contains("GraphRAG"));

    // 4. Spawn issue
    let req_iss = AgentActionRequest {
        target_path: file.clone(),
        action_type: "spawn_issue".to_string(),
        prompt: None,
    };
    let res_iss = execute_local_agent_action(&req_iss);
    assert!(res_iss.success);

    // 5. Unknown action
    let req_unk = AgentActionRequest {
        target_path: file.clone(),
        action_type: "custom_op".to_string(),
        prompt: None,
    };
    let res_unk = execute_local_agent_action(&req_unk);
    assert!(res_unk.success);

    // 6. Non-existent path
    let req_non = AgentActionRequest {
        target_path: dir.path().join("does_not_exist.txt"),
        action_type: "summarize".to_string(),
        prompt: None,
    };
    let res_non = execute_local_agent_action(&req_non);
    assert!(!res_non.success);
}

#[test]
fn test_git_status_detection_branches_and_non_repo() {
    let dir = tempdir().unwrap();
    let non_repo = detect_git_status_for_dir(dir.path());
    assert!(!non_repo.is_git_repo);
    assert_eq!(non_repo.summary, "Sin Git");

    // Existing repository check on current swal-desktop
    let repo_path = Path::new("/home/belal/proyectosSWAL/periferia/swal-desktop");
    if repo_path.exists() {
        let repo_status = detect_git_status_for_dir(repo_path);
        assert!(repo_status.is_git_repo);
        assert!(!repo_status.branch.is_empty());
    }
}

#[test]
fn test_preview_generators_edge_cases_and_persistence() {
    let dir = tempdir().unwrap();

    // 1. Long text file (> 150 lines)
    let long_file = dir.path().join("long.rs");
    let lines: Vec<String> = (1..=200).map(|i| format!("// line {}", i)).collect();
    fs::write(&long_file, lines.join("\n")).unwrap();

    let prev_long = generate_preview_for_path(&long_file);
    assert!(prev_long.is_text);
    assert!(prev_long.content.contains("Truncado"));
    assert_eq!(prev_long.file_type, "Código fuente Rust");

    // 2. Formats: python, shell, nix, html, css, yaml
    let py = dir.path().join("a.py");
    fs::write(&py, "print(1)").unwrap();
    assert_eq!(generate_preview_for_path(&py).file_type, "Script Python");

    let sh = dir.path().join("a.sh");
    fs::write(&sh, "echo 1").unwrap();
    assert_eq!(generate_preview_for_path(&sh).file_type, "Shell Script");

    let nix = dir.path().join("a.nix");
    fs::write(&nix, "{ pkgs }: {}").unwrap();
    assert_eq!(generate_preview_for_path(&nix).file_type, "Módulo NixOS");

    let css = dir.path().join("a.css");
    fs::write(&css, "body {}").unwrap();
    assert_eq!(generate_preview_for_path(&css).file_type, "Hoja de estilo CSS");

    let html = dir.path().join("a.html");
    fs::write(&html, "<html></html>").unwrap();
    assert_eq!(generate_preview_for_path(&html).file_type, "Documento HTML");

    let yml = dir.path().join("a.yaml");
    fs::write(&yml, "key: value").unwrap();
    assert_eq!(generate_preview_for_path(&yml).file_type, "Configuración / Datos");

    // 3. Image preview
    let img = dir.path().join("logo.png");
    fs::write(&img, b"\x89PNG\r\n\x1a\n").unwrap();
    let prev_img = generate_preview_for_path(&img);
    assert!(prev_img.is_image);
    assert!(!prev_img.is_text);

    // 4. Directory preview
    let prev_dir = generate_preview_for_path(dir.path());
    assert!(prev_dir.is_dir);
    assert!(prev_dir.content.contains("Directorio:"));

    // 5. Bytes formatting
    assert_eq!(format_preview_bytes(500), "500 B");
    assert_eq!(format_preview_bytes(2048), "2 KB");
    assert_eq!(format_preview_bytes(5 * 1024 * 1024), "5.0 MB");
    assert_eq!(format_preview_bytes(3 * 1024 * 1024 * 1024), "3.0 GB");

    // 6. Editor state load/save
    let state_file = dir.path().join("editor.json");
    let state = PreviewState::default();
    save_editor_state_to_path(&state, &state_file);
    let loaded = load_editor_state_from_path(&state_file);
    assert_eq!(loaded.file_name, "README.md");
}

#[test]
fn test_omnibar_advanced_parsing() {
    let dir = tempdir().unwrap();
    let current = dir.path();

    // 1. Agent prompt '@'
    let res_agent = parse_omnibar_input("@summarize directory", current);
    assert_eq!(res_agent, OmnibarIntent::AgentPrompt("summarize directory".to_string()));

    // 2. Command '>'
    let res_cmd = parse_omnibar_input(">view", current);
    assert_eq!(res_cmd, OmnibarIntent::Command("view".to_string()));

    // 3. Search '?'
    let res_search = parse_omnibar_input("?test", current);
    assert_eq!(res_search, OmnibarIntent::SearchQuery("test".to_string()));

    // 4. Absolute path
    let res_root = parse_omnibar_input("/", current);
    assert_eq!(res_root, OmnibarIntent::Navigate(PathBuf::from("/")));

    // 5. Tilde expansion '~'
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));
    let res_home = parse_omnibar_input("~", current);
    assert_eq!(res_home, OmnibarIntent::Navigate(home));

    // 6. Non-existent path fallback to search query
    let res_fallback = parse_omnibar_input("some_non_existing_query", current);
    assert_eq!(res_fallback, OmnibarIntent::SearchQuery("some_non_existing_query".to_string()));
}

#[test]
fn test_cli_command_handling_state_transitions() {
    let dir = tempdir().unwrap();
    let mut session = SessionState {
        active_tab_id: 1,
        tabs: vec![TabState {
            id: 1,
            title: "Test".to_string(),
            path: dir.path().to_string_lossy().to_string(),
            active: true,
        }],
        view_mode: "details".to_string(),
        show_hidden: false,
        dual_pane: false,
        search_query: String::new(),
        is_maximized: false,
        sort_by: "name".to_string(),
        sort_order: "asc".to_string(),
        group_by: "none".to_string(),
        filter_type: "all".to_string(),
        preview_mode: "sidebar".to_string(),
        selected_path: None,
    };

    // 1. view-json
    let res_json = handle_command(&mut session, &[
        "swal-files".to_string(),
        "view-json".to_string(),
    ]).unwrap();
    assert!(res_json.is_some());
    assert!(res_json.unwrap().contains("current_path"));

    // 2. editor-json
    let res_ed = handle_command(&mut session, &[
        "swal-files".to_string(),
        "editor-json".to_string(),
    ]).unwrap();
    assert!(res_ed.is_some());

    // 3. set-group
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "set-group".to_string(),
        "type".to_string(),
    ]).unwrap();
    assert_eq!(session.group_by, "type");

    // 4. set-filter
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "set-filter".to_string(),
        "code".to_string(),
    ]).unwrap();
    assert_eq!(session.filter_type, "code");

    // 5. set-sort
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "set-sort".to_string(),
        "date".to_string(),
        "desc".to_string(),
    ]).unwrap();
    assert_eq!(session.sort_by, "date");
    assert_eq!(session.sort_order, "desc");

    // 6. toggle-preview-mode
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "toggle-preview-mode".to_string(),
    ]).unwrap();
    assert_eq!(session.preview_mode, "none");

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "toggle-preview-mode".to_string(),
    ]).unwrap();
    assert_eq!(session.preview_mode, "sidebar");

    // 7. toggle-view
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "toggle-view".to_string(),
    ]).unwrap();
    assert_eq!(session.view_mode, "grid");

    // 8. toggle-hidden
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "toggle-hidden".to_string(),
    ]).unwrap();
    assert!(session.show_hidden);

    // 9. tab management (new, switch, close)
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "tab-new".to_string(),
    ]).unwrap();
    assert_eq!(session.tabs.len(), 2);
    let new_id = session.active_tab_id;

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "tab-switch".to_string(),
        "1".to_string(),
    ]).unwrap();
    assert_eq!(session.active_tab_id, 1);

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "tab-close".to_string(),
        new_id.to_string(),
    ]).unwrap();
    assert_eq!(session.tabs.len(), 1);

    // 10. select-item and nav
    let sub = dir.path().join("nested_folder");
    fs::create_dir(&sub).unwrap();
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "nav".to_string(),
        sub.to_string_lossy().to_string(),
    ]).unwrap();
    assert_eq!(session.tabs[0].path, sub.to_string_lossy().to_string());

    let file = sub.join("test.txt");
    fs::write(&file, "hello").unwrap();
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "select-item".to_string(),
        file.to_string_lossy().to_string(),
    ]).unwrap();
    assert_eq!(session.selected_path, Some(file.to_string_lossy().to_string()));
}

#[test]
fn test_scanner_sorting_and_grouping_matrix() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("alpha.txt"), "a").unwrap();
    fs::write(p.join("beta.txt"), "bbbb").unwrap();
    fs::write(p.join("gamma.rs"), "cccccc").unwrap();
    fs::create_dir(p.join("sub")).unwrap();

    // 1. Sort by size desc
    let opts_size_desc = ScanOptions {
        show_hidden: true,
        sort_by: SortBy::Size,
        ascending: false,
        filter_query: None,
        filter_category: "all".to_string(),
        group_by: GroupBy::None,
    };
    let entries = scan_directory(p, &opts_size_desc).unwrap();
    assert_eq!(entries.len(), 4);

    // 2. Sort by Date
    let opts_date = ScanOptions {
        show_hidden: false,
        sort_by: SortBy::Modified,
        ascending: true,
        filter_query: None,
        filter_category: "all".to_string(),
        group_by: GroupBy::None,
    };

    let entries_date = scan_directory(p, &opts_date).unwrap();
    assert_eq!(entries_date.len(), 4);

    // 3. Group by Date
    let grouped_date = group_entries(&entries, GroupBy::Date);
    assert!(!grouped_date.is_empty());

    // 4. Group by Size
    let grouped_size = group_entries(&entries, GroupBy::Size);
    assert!(!grouped_size.is_empty());

    // 5. Group by None
    let grouped_none = group_entries(&entries, GroupBy::None);
    assert_eq!(grouped_none.len(), 1);
    assert_eq!(grouped_none[0].title, "Todos los elementos");
}

#[test]
fn test_breadcrumbs_and_gui_payload_building() {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/belal"));

    // 1. Breadcrumbs for Home
    let bc_home = get_breadcrumbs(&home);
    assert_eq!(bc_home.len(), 1);
    assert_eq!(bc_home[0].name, "Home");

    // 2. Breadcrumbs for subfolder of Home
    let sub = home.join("proyectosSWAL").join("periferia");
    let bc_sub = get_breadcrumbs(&sub);
    assert_eq!(bc_sub[0].name, "Home");
    assert_eq!(bc_sub[1].name, "proyectosSWAL");
    assert_eq!(bc_sub[2].name, "periferia");

    // 3. Breadcrumbs for Root
    let bc_root = get_breadcrumbs(Path::new("/"));
    assert_eq!(bc_root[0].name, "Root (/)");

    // 4. GUI Payload generation
    let session = SessionState::default();
    let payload = build_gui_payload(&session);
    assert!(!payload.current_path.is_empty());
    assert!(!payload.favorites.is_empty());
    assert!(!payload.tabs.is_empty());
}

#[test]
fn test_lib_session_and_tab_operations() {
    let dir = tempdir().unwrap();
    let p1 = dir.path().join("p1");
    let p2 = dir.path().join("p2");
    fs::create_dir(&p1).unwrap();
    fs::create_dir(&p2).unwrap();

    let mut tab = swal_files::FileTab::new(1, p1.clone());
    assert_eq!(tab.id, 1);
    assert_eq!(tab.current_path, p1);

    tab.navigate_to(p2.clone());
    assert_eq!(tab.current_path, p2);

    let went_back = tab.go_back();
    assert!(went_back);
    assert_eq!(tab.current_path, p1);
    assert!(!tab.go_back()); // No more back history

    // Session
    let cfg = FileManagerConfig::default();
    let mut session = swal_files::FileManagerSession::new(p1.clone(), cfg);
    assert_eq!(session.active_tab().current_path, p1);

    let t2_idx = session.new_tab(p2.clone());
    assert_eq!(t2_idx, 1);
    assert_eq!(session.active_tab().current_path, p2);

    let closed = session.close_tab(1);
    assert!(closed);
    assert_eq!(session.active_tab().current_path, p1);

    let opts = ScanOptions::default();
    let entries = session.current_entries(&opts).unwrap();
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_cli_extended_commands_pin_unpin_and_views() {
    let dir = tempdir().unwrap();
    let mut session = SessionState::default();

    // 1. open-item with directory vs file
    let test_dir = dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    let test_file = test_dir.join("sample.rs");
    fs::write(&test_file, "pub fn sample() {}").unwrap();

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "open-item".to_string(),
        test_dir.to_string_lossy().to_string(),
    ]).unwrap();
    assert_eq!(session.tabs[0].path, test_dir.to_string_lossy().to_string());

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "open-item".to_string(),
        test_file.to_string_lossy().to_string(),
    ]).unwrap();
    assert_eq!(session.selected_path, Some(test_file.to_string_lossy().to_string()));

    // 2. open-editor
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "open-editor".to_string(),
        test_file.to_string_lossy().to_string(),
    ]).unwrap();

    // 3. toggle-maximize
    let init_max = session.is_maximized;
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "toggle-maximize".to_string(),
    ]).unwrap();
    assert_ne!(session.is_maximized, init_max);

    // 4. set-preview-mode
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "set-preview-mode".to_string(),
        "none".to_string(),
    ]).unwrap();
    assert_eq!(session.preview_mode, "none");
}

#[test]
fn test_cli_runner_and_edge_command_branches() {
    let dir = tempdir().unwrap();
    let mut session = SessionState::default();

    // 1. run_cli invocations
    swal_files::cli::run_cli(&["swal-files".to_string(), "view-json".to_string()]);
    swal_files::cli::run_cli(&["swal-files".to_string(), "editor-json".to_string()]);

    // 2. omnibar command intents
    let init_hidden = session.show_hidden;
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "omnibar".to_string(),
        ">hidden".to_string(),
    ]).unwrap();
    assert_ne!(session.show_hidden, init_hidden);

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "omnibar".to_string(),
        ">view".to_string(),
    ]).unwrap();

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "omnibar".to_string(),
        "?newsearch".to_string(),
    ]).unwrap();
    assert_eq!(session.search_query, "newsearch");

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "omnibar".to_string(),
        "@summarize".to_string(),
    ]).unwrap();

    // 3. invalid / edge cases
    handle_command(&mut session, &[
        "swal-files".to_string(),
        "nav".to_string(),
        "/non_existent_path_xyz_123".to_string(),
    ]).unwrap();

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "tab-switch".to_string(),
        "99999".to_string(),
    ]).unwrap();

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "tab-close".to_string(),
        "99999".to_string(),
    ]).unwrap();

    handle_command(&mut session, &[
        "swal-files".to_string(),
        "unknown-cmd".to_string(),
    ]).unwrap();

    // 4. FileEntry methods
    let test_file = dir.path().join("entry_test.rs");
    fs::write(&test_file, "fn main() {}").unwrap();
    let entry = FileEntry::from_path(&test_file).unwrap();
    assert_eq!(entry.category, FileCategory::Code);
    assert_eq!(entry.git_status.badge_icon(), "✓");
    assert_eq!(GitStatus::Modified.badge_icon(), "●");
    assert_eq!(GitStatus::Untracked.badge_icon(), "…");
    assert_eq!(GitStatus::Staged.badge_icon(), "+");
    assert_eq!(GitStatus::Conflicted.badge_icon(), "!");
}


#[test]
fn test_session_state_io_and_cli_aliases() {
    let dir = tempdir().unwrap();
    let session_file = dir.path().join("session_test.json");

    // 1. Session load / save from path
    let mut session = SessionState::default();
    session.view_mode = "grid".to_string();
    session.sort_by = "size".to_string();
    save_session_to_path(&session, &session_file);

    let loaded = load_session_from_path(&session_file);
    assert_eq!(loaded.view_mode, "grid");
    assert_eq!(loaded.sort_by, "size");

    // 2. Test CLI command aliases
    handle_command(&mut session, &["swal-files".to_string(), "group".to_string(), "date".to_string()]).unwrap();
    assert_eq!(session.group_by, "date");

    handle_command(&mut session, &["swal-files".to_string(), "filter".to_string(), "images".to_string()]).unwrap();
    assert_eq!(session.filter_type, "images");

    handle_command(&mut session, &["swal-files".to_string(), "sort".to_string(), "name".to_string(), "desc".to_string()]).unwrap();
    assert_eq!(session.sort_by, "name");
    assert_eq!(session.sort_order, "desc");

    handle_command(&mut session, &["swal-files".to_string(), "toggle-sidebar".to_string()]).unwrap();
    handle_command(&mut session, &["swal-files".to_string(), "tab_new".to_string()]).unwrap();
    assert!(session.tabs.len() > 1);

    handle_command(&mut session, &["swal-files".to_string(), "tab_switch".to_string(), "1".to_string()]).unwrap();
    assert_eq!(session.active_tab_id, 1);

    let last_id = session.tabs.last().unwrap().id;
    handle_command(&mut session, &["swal-files".to_string(), "tab_close".to_string(), last_id.to_string()]).unwrap();

    handle_command(&mut session, &["swal-files".to_string(), "toggle_hidden".to_string()]).unwrap();
    handle_command(&mut session, &["swal-files".to_string(), "toggle_view".to_string()]).unwrap();

    // 3. Direct path handling
    let test_dir = dir.path().to_string_lossy().to_string();
    handle_command(&mut session, &["swal-files".to_string(), test_dir]).unwrap();
}
