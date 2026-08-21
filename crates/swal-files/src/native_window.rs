//! Native SWAL Files GPU Window Layout Builder (Zero-Eww)
//!
//! Transforms the active file manager session state, storage metrics, and quicklook previews
//! directly into a declarative A2UI component tree (`ComponentNode`) for high-refresh GPU rendering.

use std::path::PathBuf;
use swal_a2ui_engine::{ComponentNode, TabItem};
use crate::config::FileManagerConfig;
use crate::gui::get_breadcrumbs;
use crate::preview::{generate_preview_for_path, sanitize_preview_text};

use crate::scanner::{scan_directory, ScanOptions, SortBy, GroupBy};
use crate::session::SessionState;
use crate::storage::DiskUsageScanner;

pub struct NativeFilesWindowBuilder;

impl NativeFilesWindowBuilder {
    /// Builds a full native A2UI component tree for the current file manager session.
    pub fn build_native_a2ui_tree(session: &SessionState) -> ComponentNode {
        let active_tab = session
            .tabs
            .iter()
            .find(|t| t.id == session.active_tab_id)
            .unwrap_or(&session.tabs[0]);

        let current_path = PathBuf::from(&active_tab.path);
        let cfg = FileManagerConfig::load();

        // 1. Header: Tab Strip
        let mut tab_items = Vec::new();
        for t in &session.tabs {
            let is_active = t.id == session.active_tab_id;
            tab_items.push(TabItem {
                id: t.id.to_string(),
                label: format!("{} {}", if is_active { "📂" } else { "📁" }, t.title),
                content: vec![
                    ComponentNode::Button {
                        label: format!("Abrir {}", t.title),
                        action: format!("switch_tab:{}", t.id),
                        variant: Some(if is_active { "primary".to_string() } else { "subtle".to_string() }),
                    }
                ],
            });
        }

        let tab_bar = ComponentNode::Tabs {
            tabs: tab_items,
        };

        // 2. Toolbar & Segmented Breadcrumbs
        let breadcrumbs = get_breadcrumbs(&current_path);
        let mut toolbar_children = Vec::new();
        toolbar_children.push(ComponentNode::Button {
            label: "⮜ Atrás".to_string(),
            action: "nav_back".to_string(),
            variant: Some("subtle".to_string()),
        });
        toolbar_children.push(ComponentNode::Button {
            label: "⮝ Subir".to_string(),
            action: "nav_up".to_string(),
            variant: Some("subtle".to_string()),
        });

        for b in breadcrumbs {
            toolbar_children.push(ComponentNode::Button {
                label: format!("{} ❯", b.name),
                action: format!("nav:{}", b.path),
                variant: Some("subtle".to_string()),
            });
        }

        let toolbar = ComponentNode::Grid {
            columns: toolbar_children.len().max(1),
            gap: 4,
            children: toolbar_children,
        };

        // 3. Sidebar: Favorites & Storage Metrics
        let mut sidebar_nodes = Vec::new();
        for pin in &cfg.pinned_locations {
            sidebar_nodes.push(ComponentNode::Button {
                label: format!("{} {}", pin.icon, pin.name),
                action: format!("nav:{}", pin.path.display()),
                variant: Some("ghost".to_string()),
            });
        }

        let scanner = DiskUsageScanner::new();
        for drive in scanner.scan_mounted_drives().iter().take(3) {
            sidebar_nodes.push(ComponentNode::MetricPill {
                label: format!("🖴 {}", drive.mount_point),
                value: format!("{:.0}% ({})", drive.used_percentage, drive.formatted_used()),
                unit: None,
                trend: None,
                color: Some(if drive.used_percentage > 90.0 { "#ef4444".to_string() } else { "#60cdff".to_string() }),
            });
        }

        let sidebar = ComponentNode::Card {
            title: Some("Navegación & Unidades".to_string()),
            elevation: Some("elevated".to_string()),
            children: sidebar_nodes,
        };

        // 4. Content Area: Scanned directory entries
        let scan_opts = ScanOptions {
            show_hidden: session.show_hidden,
            sort_by: SortBy::from_str(&session.sort_by),
            ascending: session.sort_order.to_lowercase() != "desc",
            filter_query: if session.search_query.is_empty() { None } else { Some(session.search_query.clone()) },
            filter_category: session.filter_type.clone(),
            group_by: GroupBy::None,
        };

        let scanned_entries = scan_directory(&current_path, &scan_opts).unwrap_or_default();
        let mut entry_nodes = Vec::new();

        for e in scanned_entries.iter().take(30) {
            entry_nodes.push(ComponentNode::Button {
                label: format!("{} {} ({})", e.icon, sanitize_preview_text(&e.name), e.formatted_size),
                action: format!("open:{}", e.path.display()),
                variant: Some(if e.is_dir { "primary".to_string() } else { "subtle".to_string() }),
            });
        }

        let content_panel = ComponentNode::Card {
            title: Some(format!("Archivos en {} ({})", current_path.display(), scanned_entries.len())),
            elevation: Some("flat".to_string()),
            children: entry_nodes,
        };

        // 5. Right-Hand Preview Panel
        let preview_target = if let Some(ref sel) = session.selected_path {
            PathBuf::from(sel)
        } else {
            current_path.clone()
        };
        let preview = generate_preview_for_path(&preview_target);

        let preview_lines = preview.content.lines().map(|s| s.to_string()).take(25).collect();
        let preview_panel = ComponentNode::Card {
            title: Some(format!("Vista Previa: {}", preview.file_name)),
            elevation: Some("elevated".to_string()),
            children: vec![
                ComponentNode::LogViewer {
                    source: preview.file_name,
                    height: 300,
                    lines: preview_lines,
                }
            ],
        };

        // Combine into root Card container
        ComponentNode::Card {
            title: Some("SWAL Files (Native GPU Surface)".to_string()),
            elevation: Some("mica".to_string()),
            children: vec![
                tab_bar,
                toolbar,
                ComponentNode::Grid {
                    columns: 3,
                    gap: 12,
                    children: vec![sidebar, content_panel, preview_panel],
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_native_files_window_builder_tree() {
        let dir = tempdir().unwrap();
        let session = SessionState {
            active_tab_id: 1,
            tabs: vec![crate::session::TabState {
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

        let tree = NativeFilesWindowBuilder::build_native_a2ui_tree(&session);
        match tree {
            ComponentNode::Card { title, children, .. } => {
                assert!(title.unwrap().contains("SWAL Files"));
                assert_eq!(children.len(), 3); // tab_bar, toolbar, 3-column grid
            }
            _ => panic!("Expected root Card container"),
        }
    }
}
