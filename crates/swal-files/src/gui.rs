//! GUI payload formatting and aggregation for Eww integration

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::config::FileManagerConfig;
use crate::git::{detect_git_status_for_dir, GitRepoSummary};
use crate::preview::{generate_preview_for_path, sanitize_preview_text, PreviewState};
use crate::scanner::{group_entries, scan_directory, GroupBy, ScanOptions, SortBy};
use crate::session::SessionState;
use crate::storage::scan_mounted_drives;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRowItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: String,
    pub r#type: String,
    pub date_modified: String,
    pub icon: String,
    pub git_status: String,
    pub git_badge: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiGroupSection {
    pub title: String,
    pub count: usize,
    pub entries: Vec<FileRowItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarPinPayload {
    pub name: String,
    pub path: String,
    pub icon: String,
    pub section: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStatePayload {
    pub id: usize,
    pub title: String,
    pub path: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilterPresetPayload {
    pub name: String,
    pub filter_type: String,
    pub sort_by: String,
    pub sort_order: String,
    pub group_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterButtonPayload {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiPayload {
    pub current_path: String,
    pub parent_path: String,
    pub total_items: usize,
    pub active_tab_id: usize,
    pub tabs: Vec<TabStatePayload>,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub view_mode: String,
    pub show_hidden: bool,
    pub dual_pane: bool,
    pub sort_by: String,
    pub sort_order: String,
    pub group_by: String,
    pub filter_type: String,
    pub preview_mode: String,
    pub is_maximized: bool,
    pub is_current_pinned: bool,
    pub theme_id: String,
    pub favorites: Vec<SidebarPinPayload>,
    pub workspaces: Vec<SidebarPinPayload>,
    pub entries: Vec<FileRowItem>,
    pub groups: Vec<GuiGroupSection>,
    pub git_status: GitRepoSummary,
    pub preview: PreviewState,
    /// Rendered filter button bar for the toolbar
    pub filter_buttons: Vec<FilterButtonPayload>,
    /// User-saved filter presets for the quick-access preset menu
    pub saved_filter_presets: Vec<SavedFilterPresetPayload>,
    /// Live disk usage for sidebar (populated from /proc/mounts + statvfs)
    pub disks: Vec<DiskPayload>,
}

/// Compact disk info for EWW sidebar rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPayload {
    pub mount: String,
    pub label: String,
    pub icon: String,
    pub used: String,
    pub total: String,
    pub used_pct: f32,
    pub is_removable: bool,
}


pub fn get_breadcrumbs(current_path: &Path) -> Vec<BreadcrumbItem> {
    let home = dirs::home_dir().unwrap_or_default();
    let mut parts = Vec::new();

    if current_path == home {
        parts.push(BreadcrumbItem {
            name: "Home".to_string(),
            path: home.to_string_lossy().to_string(),
        });
        return parts;
    }

    if let Ok(rel) = current_path.strip_prefix(&home) {
        parts.push(BreadcrumbItem {
            name: "Home".to_string(),
            path: home.to_string_lossy().to_string(),
        });
        let mut accum = home.clone();
        for segment in rel.iter() {
            accum.push(segment);
            parts.push(BreadcrumbItem {
                name: sanitize_preview_text(&segment.to_string_lossy()),
                path: accum.to_string_lossy().to_string(),
            });
        }
        return parts;
    }

    let mut accum = PathBuf::from("/");
    parts.push(BreadcrumbItem {
        name: "Root (/)".to_string(),
        path: "/".to_string(),
    });

    for segment in current_path.iter() {
        let segment_str = segment.to_string_lossy().to_string();
        if segment_str == "/" {
            continue;
        }
        accum.push(segment);
        parts.push(BreadcrumbItem {
            name: sanitize_preview_text(&segment_str),
            path: accum.to_string_lossy().to_string(),
        });
    }

    parts
}

pub fn build_gui_payload(session: &SessionState) -> GuiPayload {
    let home = dirs::home_dir().unwrap_or_default();
    let active_tab = session
        .tabs
        .iter()
        .find(|t| t.id == session.active_tab_id)
        .unwrap_or(&session.tabs[0]);

    let mut current_path = PathBuf::from(&active_tab.path);
    if !current_path.is_dir() {
        current_path = home.clone();
    }

    let sort_by = SortBy::from_str(&session.sort_by);
    let group_by = GroupBy::from_str(&session.group_by);
    let ascending = session.sort_order.to_lowercase() != "desc";

    let scan_opts = ScanOptions {
        show_hidden: session.show_hidden,
        sort_by,
        ascending,
        filter_query: if session.search_query.is_empty() { None } else { Some(session.search_query.clone()) },
        filter_category: session.filter_type.clone(),
        group_by,
    };

    let mut scanned_entries = scan_directory(&current_path, &scan_opts).unwrap_or_default();

    // Defensive fallback: if a non-"all" filter yields 0 results on a non-empty dir,
    // the session state is stale/corrupted. Auto-reset to "all" to avoid black panels.
    let effective_filter = if scanned_entries.is_empty()
        && session.filter_type != "all"
        && current_path.is_dir()
    {
        let fallback_opts = ScanOptions {
            show_hidden: session.show_hidden,
            sort_by,
            ascending,
            filter_query: None,
            filter_category: "all".to_string(),
            group_by,
        };
        scanned_entries = scan_directory(&current_path, &fallback_opts).unwrap_or_default();
        "all".to_string()
    } else {
        session.filter_type.clone()
    };

    let grouped = group_entries(&scanned_entries, group_by);

    let selected_path_str = session.selected_path.as_deref().unwrap_or("");

    let mut all_row_items: Vec<FileRowItem> = Vec::new();
    let mut gui_groups: Vec<GuiGroupSection> = Vec::new();

    for g in grouped {
        let mut group_items = Vec::new();
        for e in g.entries {
            let is_sel = e.path.to_string_lossy() == selected_path_str;
            let row = FileRowItem {
                name: sanitize_preview_text(&e.name),
                path: e.path.to_string_lossy().to_string(),
                is_dir: e.is_dir,
                size: e.formatted_size,
                r#type: e.mime_category,
                date_modified: e.formatted_date,
                icon: e.icon,
                git_status: e.git_status.as_str().to_string(),
                git_badge: e.git_status.badge_icon().to_string(),
                is_selected: is_sel,
            };
            group_items.push(row.clone());
            all_row_items.push(row);
        }
        gui_groups.push(GuiGroupSection {
            title: g.title,
            count: g.count,
            entries: group_items,
        });
    }

    let breadcrumbs = get_breadcrumbs(&current_path);
    let parent_path = current_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    let tabs: Vec<TabStatePayload> = session
        .tabs
        .iter()
        .map(|t| TabStatePayload {
            id: t.id,
            title: sanitize_preview_text(&t.title),
            path: t.path.clone(),
            active: t.id == session.active_tab_id,
        })
        .collect();

    let git_status = detect_git_status_for_dir(&current_path);

    let preview_target = if let Some(ref sel) = session.selected_path {
        PathBuf::from(sel)
    } else if let Some(first) = all_row_items.first() {
        PathBuf::from(&first.path)
    } else {
        current_path.clone()
    };

    let preview = generate_preview_for_path(&preview_target);

    let cfg = FileManagerConfig::load();
    let is_current_pinned = cfg.is_pinned(&current_path);

    let mut favorites = Vec::new();
    let mut workspaces = Vec::new();

    for pin in &cfg.pinned_locations {
        let is_active = pin.path == current_path;
        let payload_pin = SidebarPinPayload {
            name: sanitize_preview_text(&pin.name),
            path: pin.path.to_string_lossy().to_string(),
            icon: pin.icon.clone(),
            section: pin.section.clone(),
            is_active,
        };
        if pin.section == "workspaces" {
            workspaces.push(payload_pin);
        } else {
            favorites.push(payload_pin);
        }
    }

    // Build filter toolbar buttons — active state matches current filter_type
    let filter_definitions: &[(&str, &str, &str)] = &[
        ("all",       "Todo",       "📁"),
        ("folders",   "Carpetas",   "📂"),
        ("images",    "Imágenes",   "🖼"),
        ("documents", "Documentos", "📄"),
        ("code",      "Código",     "⌨"),
        ("media",     "Media",      "🎬"),
        ("archives",  "Archivos",   "📦"),
    ];
    let filter_buttons: Vec<FilterButtonPayload> = filter_definitions.iter().map(|(id, label, icon)| {
        FilterButtonPayload {
            id: id.to_string(),
            label: label.to_string(),
            icon: icon.to_string(),
            active: session.filter_type == *id,
        }
    }).collect();

    let saved_filter_presets_payload: Vec<SavedFilterPresetPayload> = session.saved_filter_presets.iter().map(|p| {
        SavedFilterPresetPayload {
            name: p.name.clone(),
            filter_type: p.filter_type.clone(),
            sort_by: p.sort_by.clone(),
            sort_order: p.sort_order.clone(),
            group_by: p.group_by.clone(),
        }
    }).collect();

    // Build disk payload for sidebar
    let disks: Vec<DiskPayload> = scan_mounted_drives()
        .into_iter()
        .take(6)
        .map(|d| {
            let icon = if d.is_removable { "󱊲".to_string() } else { "󰋊".to_string() };
            let label = if d.mount_point == "/" {
                "Root (/)".to_string()
            } else {
                d.mount_point
                    .rsplit('/')
                    .next()
                    .unwrap_or(&d.mount_point)
                    .to_string()
            };
            DiskPayload {
                mount: d.mount_point.clone(),
                label,
                icon,
                used: d.formatted_used(),
                total: d.formatted_total(),
                used_pct: d.used_percentage,
                is_removable: d.is_removable,
            }
        })
        .collect();

    GuiPayload {
        current_path: current_path.to_string_lossy().to_string(),
        parent_path,
        total_items: all_row_items.len(),
        active_tab_id: session.active_tab_id,
        tabs,
        breadcrumbs: breadcrumbs.into_iter().rev().take(5).rev().collect(),
        view_mode: session.view_mode.clone(),
        show_hidden: session.show_hidden,
        dual_pane: session.dual_pane,
        sort_by: session.sort_by.clone(),
        sort_order: session.sort_order.clone(),
        group_by: session.group_by.clone(),
        filter_type: effective_filter,
        preview_mode: session.preview_mode.clone(),
        is_maximized: session.is_maximized,
        is_current_pinned,
        theme_id: cfg.theme_id,
        favorites,
        workspaces,
        entries: all_row_items.into_iter().take(50).collect(),
        groups: gui_groups,
        git_status,
        preview,
        filter_buttons,
        saved_filter_presets: saved_filter_presets_payload,
        disks,
    }
}

pub fn notify_eww_update(payload: &GuiPayload) {
    if let Ok(json_str) = serde_json::to_string(payload) {
        // Use spawn() so we don't block the CLI process waiting for EWW IPC
        let _ = Command::new("eww")
            .args(["update", &format!("swal_files_data={}", json_str)])
            .spawn();
    }
}

