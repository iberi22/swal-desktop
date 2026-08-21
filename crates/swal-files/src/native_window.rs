//! Native SWAL Files GPU Window Layout Builder (Zero-Eww)
//!
//! Converts an active file manager session (`SessionState`) directly into
//! an A2UI declarative component tree (`ComponentNode`) for direct GPU rendering.

use crate::gui::{build_gui_payload, GuiPayload};
use crate::session::SessionState;
use crate::storage::{scan_mounted_drives, DriveInfo};
use swal_a2ui_engine::{ComponentNode, TabItem};

/// Native window layout builder for SWAL Files
pub struct NativeFilesWindowBuilder;

impl NativeFilesWindowBuilder {
    /// Builds the complete native A2UI component tree for GPU rendering
    pub fn build_native_a2ui_tree(session: &SessionState) -> ComponentNode {
        let gui = build_gui_payload(session);

        let header = Self::build_header(session);
        let toolbar = Self::build_toolbar(session, &gui);
        let sidebar = Self::build_sidebar(&gui);
        let content_grid = Self::build_content_grid(&gui);
        let preview_panel = Self::build_preview_panel(&gui);

        // Body grid: 3 columns (Sidebar, Content Grid/List, QuickLook Preview)
        let main_body = ComponentNode::Grid {
            columns: 3,
            gap: 12,
            children: vec![sidebar, content_grid, preview_panel],
        };

        ComponentNode::Card {
            title: Some(format!("SWAL Files — {}", gui.current_path)),
            elevation: Some("mica".to_string()),
            children: vec![header, toolbar, main_body],
        }
    }

    /// Builds header section: Tab strip with active indicator, close buttons, and new tab +
    fn build_header(session: &SessionState) -> ComponentNode {
        let tabs: Vec<TabItem> = session
            .tabs
            .iter()
            .map(|t| {
                let is_active = t.id == session.active_tab_id;
                let active_marker = if is_active { " [Active]" } else { "" };
                let label = format!("{} (ID: {}){}", t.title, t.id, active_marker);

                let tab_buttons = vec![
                    ComponentNode::Button {
                        label: t.title.clone(),
                        action: format!("tab.switch:{}", t.id),
                        variant: Some(if is_active {
                            "primary".to_string()
                        } else {
                            "subtle".to_string()
                        }),
                    },
                    ComponentNode::Button {
                        label: "✕".to_string(),
                        action: format!("tab.close:{}", t.id),
                        variant: Some("danger_icon".to_string()),
                    },
                ];

                TabItem {
                    id: format!("tab-{}", t.id),
                    label,
                    content: vec![ComponentNode::Grid {
                        columns: 2,
                        gap: 4,
                        children: tab_buttons,
                    }],
                }
            })
            .collect();

        let mut header_children = vec![ComponentNode::Tabs { tabs }];

        header_children.push(ComponentNode::Button {
            label: "+ New Tab".to_string(),
            action: "tab.new".to_string(),
            variant: Some("accent".to_string()),
        });

        ComponentNode::Grid {
            columns: 2,
            gap: 8,
            children: header_children,
        }
    }

    /// Builds toolbar section: Nav buttons (Back, Up, Refresh), segmented breadcrumb chevrons, search bar, preview toggle
    fn build_toolbar(session: &SessionState, gui: &GuiPayload) -> ComponentNode {
        let nav_buttons = ComponentNode::Grid {
            columns: 4,
            gap: 6,
            children: vec![
                ComponentNode::Button {
                    label: "◀".to_string(),
                    action: "nav.back".to_string(),
                    variant: Some("icon".to_string()),
                },
                ComponentNode::Button {
                    label: "▲".to_string(),
                    action: "nav.up".to_string(),
                    variant: Some("icon".to_string()),
                },
                ComponentNode::Button {
                    label: "↻".to_string(),
                    action: "nav.refresh".to_string(),
                    variant: Some("icon".to_string()),
                },
                ComponentNode::Button {
                    label: if session.preview_mode == "none" {
                        "👁 Preview"
                    } else {
                        "👁 Hide Preview"
                    }
                    .to_string(),
                    action: "toggle.preview".to_string(),
                    variant: Some("secondary".to_string()),
                },
            ],
        };

        // Segmented breadcrumbs
        let breadcrumb_nodes: Vec<ComponentNode> = gui
            .breadcrumbs
            .iter()
            .map(|b| ComponentNode::Button {
                label: format!("> {}", b.name),
                action: format!("nav.go:{}", b.path),
                variant: Some("breadcrumb".to_string()),
            })
            .collect();

        let breadcrumb_strip = ComponentNode::Grid {
            columns: breadcrumb_nodes.len().max(1),
            gap: 4,
            children: breadcrumb_nodes,
        };

        let search_bar = ComponentNode::StatusBadge {
            status: "info".to_string(),
            label: if session.search_query.is_empty() {
                "🔍 Search files...".to_string()
            } else {
                format!("🔍 Query: {}", session.search_query)
            },
            color: Some("$accent_primary".to_string()),
        };

        ComponentNode::Grid {
            columns: 3,
            gap: 10,
            children: vec![nav_buttons, breadcrumb_strip, search_bar],
        }
    }

    /// Builds sidebar section: Favorites, Workspaces, Drive storage capacity progress bars
    fn build_sidebar(gui: &GuiPayload) -> ComponentNode {
        let mut sidebar_nodes = Vec::new();

        // Favorites
        let mut fav_children = Vec::new();
        for fav in &gui.favorites {
            fav_children.push(ComponentNode::Button {
                label: format!("{} {}", fav.icon, fav.name),
                action: format!("nav.go:{}", fav.path),
                variant: Some(if fav.is_active {
                    "sidebar_active".to_string()
                } else {
                    "sidebar_item".to_string()
                }),
            });
        }
        sidebar_nodes.push(ComponentNode::Card {
            title: Some("Favorites".to_string()),
            elevation: Some("subtle".to_string()),
            children: fav_children,
        });

        // Workspaces
        let mut ws_children = Vec::new();
        for ws in &gui.workspaces {
            ws_children.push(ComponentNode::Button {
                label: format!("{} {}", ws.icon, ws.name),
                action: format!("nav.go:{}", ws.path),
                variant: Some(if ws.is_active {
                    "sidebar_active".to_string()
                } else {
                    "sidebar_item".to_string()
                }),
            });
        }
        sidebar_nodes.push(ComponentNode::Card {
            title: Some("Workspaces".to_string()),
            elevation: Some("subtle".to_string()),
            children: ws_children,
        });

        // Drive Storage Capacity
        let drives: Vec<DriveInfo> = scan_mounted_drives();
        let mut drive_nodes = Vec::new();

        for drive in drives {
            let used_str = format!(
                "{} / {} ({:.1}%)",
                drive.formatted_used(),
                drive.formatted_total(),
                drive.used_percentage
            );

            drive_nodes.push(ComponentNode::MetricPill {
                label: format!("Drive ({})", drive.mount_point),
                value: used_str,
                unit: Some(drive.filesystem.clone()),
                trend: if drive.used_percentage > 85.0 {
                    Some("HIGH USAGE".to_string())
                } else {
                    None
                },
                color: Some(if drive.used_percentage > 90.0 {
                    "$danger".to_string()
                } else if drive.used_percentage > 75.0 {
                    "$warning".to_string()
                } else {
                    "$accent_primary".to_string()
                }),
            });
        }

        sidebar_nodes.push(ComponentNode::Card {
            title: Some("Drives & Storage".to_string()),
            elevation: Some("subtle".to_string()),
            children: drive_nodes,
        });

        ComponentNode::Card {
            title: Some("Navigation Sidebar".to_string()),
            elevation: Some("sidebar".to_string()),
            children: sidebar_nodes,
        }
    }

    /// Builds main content grid/list: File rows with icons, names, size, git badge, and selection highlights
    fn build_content_grid(gui: &GuiPayload) -> ComponentNode {
        let mut file_rows = Vec::new();

        for entry in &gui.entries {
            let mut row_items = Vec::new();

            row_items.push(ComponentNode::StatusBadge {
                status: if entry.is_dir {
                    "dir".to_string()
                } else {
                    "file".to_string()
                },
                label: format!("{} {}", entry.icon, entry.name),
                color: Some(if entry.is_selected {
                    "$accent_primary".to_string()
                } else {
                    "$text_primary".to_string()
                }),
            });

            row_items.push(ComponentNode::MetricPill {
                label: "Size".to_string(),
                value: entry.size.clone(),
                unit: None,
                trend: None,
                color: None,
            });

            if !entry.git_badge.is_empty() {
                row_items.push(ComponentNode::StatusBadge {
                    status: entry.git_status.clone(),
                    label: format!("{} {}", entry.git_badge, entry.git_status),
                    color: Some("$success".to_string()),
                });
            }

            row_items.push(ComponentNode::Button {
                label: if entry.is_selected { "Selected" } else { "Select" }.to_string(),
                action: format!("select:{}", entry.path),
                variant: Some(if entry.is_selected {
                    "primary".to_string()
                } else {
                    "ghost".to_string()
                }),
            });

            file_rows.push(ComponentNode::Grid {
                columns: 4,
                gap: 6,
                children: row_items,
            });
        }

        ComponentNode::Card {
            title: Some(format!("Files (Total: {})", gui.total_items)),
            elevation: Some("main_content".to_string()),
            children: file_rows,
        }
    }

    /// Builds right-side panel: QuickLook code/markdown/image preview card
    fn build_preview_panel(gui: &GuiPayload) -> ComponentNode {
        let p = &gui.preview;

        let title = format!("QuickLook — {}", p.file_name);
        let preview_info = vec![
            ComponentNode::StatusBadge {
                status: p.file_type.clone(),
                label: format!("Type: {}", p.file_type),
                color: Some("$accent_secondary".to_string()),
            },
            ComponentNode::MetricPill {
                label: "File Size".to_string(),
                value: p.size_formatted.clone(),
                unit: None,
                trend: None,
                color: None,
            },
            ComponentNode::LogViewer {
                source: p.path.clone(),
                height: 300,
                lines: p.content.lines().map(|s| s.to_string()).collect(),
            },
        ];

        ComponentNode::Card {
            title: Some(title),
            elevation: Some("quicklook".to_string()),
            children: preview_info,
        }
    }
}

/// Standalone function helper for building the A2UI tree from session state
pub fn build_native_a2ui_tree(session: &SessionState) -> ComponentNode {
    NativeFilesWindowBuilder::build_native_a2ui_tree(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_files_window_builder_tree() {
        let session = SessionState::default();
        let tree = NativeFilesWindowBuilder::build_native_a2ui_tree(&session);

        if let ComponentNode::Card { title, children, .. } = tree {
            assert!(title.unwrap_or_default().contains("SWAL Files"));
            assert_eq!(children.len(), 3, "Expected Header, Toolbar, MainBody");
        } else {
            panic!("Root node must be Card");
        }
    }

    #[test]
    fn test_tab_count_consistency() {
        let session = SessionState::default();
        let tree = build_native_a2ui_tree(&session);

        if let ComponentNode::Card { children, .. } = tree {
            // Children[0] is Header Grid
            if let ComponentNode::Grid { children: header_children, .. } = &children[0] {
                if let ComponentNode::Tabs { tabs } = &header_children[0] {
                    assert_eq!(tabs.len(), session.tabs.len(), "Tab count must match session tabs");
                } else {
                    panic!("First header child must be Tabs");
                }
            } else {
                panic!("Header must be Grid");
            }
        }
    }

    #[test]
    fn test_sidebar_drive_entries() {
        let session = SessionState::default();
        let tree = NativeFilesWindowBuilder::build_native_a2ui_tree(&session);

        if let ComponentNode::Card { children, .. } = tree {
            let main_body = &children[2];
            if let ComponentNode::Grid { children: body_children, .. } = main_body {
                let sidebar = &body_children[0];
                if let ComponentNode::Card { children: sidebar_cards, .. } = sidebar {
                    // Third section in sidebar is Drives & Storage
                    let drives_card = &sidebar_cards[2];
                    if let ComponentNode::Card { title, children: drive_items, .. } = drives_card {
                        assert_eq!(title.as_deref(), Some("Drives & Storage"));
                        assert!(!drive_items.is_empty(), "Drive storage entries must not be empty");
                    } else {
                        panic!("Drives card must be Card node");
                    }
                } else {
                    panic!("Sidebar must be Card node");
                }
            } else {
                panic!("Main body must be Grid node");
            }
        }
    }
}
