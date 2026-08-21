//! Extended tab metadata, tab tooltips, drag/reorder metadata, and tab management operations
//! inspired by files-community/Files and Windows 11 Files App.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Extended metadata for tab hover tooltips and drag/reorder operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtendedTabInfo {
    pub id: usize,
    pub title: String,
    pub path: PathBuf,
    pub item_count: usize,
    pub formatted_path: String,
    pub is_loading: bool,
    pub color_tag: Option<String>,
    pub preview_thumbnail: Option<String>,
    pub is_pinned: bool,
}

impl ExtendedTabInfo {
    /// Creates a new `ExtendedTabInfo` with standard default metadata.
    pub fn new(id: usize, title: impl Into<String>, path: PathBuf) -> Self {
        let formatted_path = path.to_string_lossy().to_string();
        Self {
            id,
            title: title.into(),
            path,
            item_count: 0,
            formatted_path,
            is_loading: false,
            color_tag: None,
            preview_thumbnail: None,
            is_pinned: false,
        }
    }

    /// Formats tooltip metadata text for tab hover display.
    pub fn tooltip(&self) -> String {
        let tag_str = self
            .color_tag
            .as_ref()
            .map(|t| format!(" [{}]", t))
            .unwrap_or_default();
        let loading_str = if self.is_loading { " (Loading...)" } else { "" };
        let count_str = if self.is_loading {
            "Scanning...".to_string()
        } else {
            format!("{} items", self.item_count)
        };
        format!(
            "{}{}\nPath: {}\nSize / Count: {}{}",
            self.title, tag_str, self.formatted_path, count_str, loading_str
        )
    }

    /// Updates item count and updates formatted path string.
    pub fn update_metadata(&mut self, item_count: usize, path: PathBuf) {
        self.item_count = item_count;
        self.formatted_path = path.to_string_lossy().to_string();
        self.path = path;
    }
}

/// Extended tab manager handling drag-and-drop reordering, duplication, and bulk tab closures.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtendedTabManager {
    pub tabs: Vec<ExtendedTabInfo>,
    pub active_tab_id: Option<usize>,
    next_id: usize,
}

impl ExtendedTabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            next_id: 1,
        }
    }

    pub fn add_tab(&mut self, mut tab: ExtendedTabInfo) -> usize {
        if tab.id == 0 {
            tab.id = self.next_id;
            self.next_id += 1;
        } else if tab.id >= self.next_id {
            self.next_id = tab.id + 1;
        }
        let id = tab.id;
        if self.active_tab_id.is_none() {
            self.active_tab_id = Some(id);
        }
        self.tabs.push(tab);
        id
    }

    /// Reorders a tab by moving it from `from_idx` to `to_idx`.
    pub fn move_tab(&mut self, from_idx: usize, to_idx: usize) -> bool {
        if from_idx >= self.tabs.len() || to_idx >= self.tabs.len() {
            return false;
        }
        let tab = self.tabs.remove(from_idx);
        self.tabs.insert(to_idx, tab);
        true
    }

    /// Duplicates an existing tab by tab ID and places the clone adjacent to it.
    pub fn duplicate_tab(&mut self, id: usize) -> Option<ExtendedTabInfo> {
        let index = self.tabs.iter().position(|t| t.id == id)?;
        let mut dup = self.tabs[index].clone();
        dup.id = self.next_id;
        self.next_id += 1;
        dup.title = format!("{} (Copy)", dup.title);
        self.tabs.insert(index + 1, dup.clone());
        Some(dup)
    }

    /// Closes all tabs except the tab matching `id`.
    pub fn close_other_tabs(&mut self, id: usize) -> bool {
        if !self.tabs.iter().any(|t| t.id == id) {
            return false;
        }
        self.tabs.retain(|t| t.id == id);
        self.active_tab_id = Some(id);
        true
    }

    /// Returns reference to the active tab if present.
    pub fn active_tab(&self) -> Option<&ExtendedTabInfo> {
        let active_id = self.active_tab_id?;
        self.tabs.iter().find(|t| t.id == active_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_tab_info_creation_and_tooltip() {
        let p = PathBuf::from("/home/user/documents");
        let mut tab = ExtendedTabInfo::new(1, "Documents", p.clone());
        tab.item_count = 42;
        tab.color_tag = Some("blue".to_string());

        assert_eq!(tab.id, 1);
        assert_eq!(tab.item_count, 42);
        assert_eq!(tab.formatted_path, "/home/user/documents");
        assert_eq!(tab.color_tag, Some("blue".to_string()));

        let tooltip = tab.tooltip();
        assert!(tooltip.contains("Documents [blue]"));
        assert!(tooltip.contains("Path: /home/user/documents"));
        assert!(tooltip.contains("Size / Count: 42 items"));
    }

    #[test]
    fn test_tab_manager_move_tab() {
        let mut mgr = ExtendedTabManager::new();
        mgr.add_tab(ExtendedTabInfo::new(1, "Tab 1", PathBuf::from("/1")));
        mgr.add_tab(ExtendedTabInfo::new(2, "Tab 2", PathBuf::from("/2")));
        mgr.add_tab(ExtendedTabInfo::new(3, "Tab 3", PathBuf::from("/3")));

        assert_eq!(mgr.tabs[0].id, 1);
        assert_eq!(mgr.tabs[1].id, 2);
        assert_eq!(mgr.tabs[2].id, 3);

        // Move Tab 1 (index 0) to index 2
        assert!(mgr.move_tab(0, 2));
        assert_eq!(mgr.tabs[0].id, 2);
        assert_eq!(mgr.tabs[1].id, 3);
        assert_eq!(mgr.tabs[2].id, 1);

        // Move with invalid index
        assert!(!mgr.move_tab(0, 10));
    }

    #[test]
    fn test_tab_manager_duplicate_tab() {
        let mut mgr = ExtendedTabManager::new();
        let id1 = mgr.add_tab(ExtendedTabInfo::new(1, "Tab 1", PathBuf::from("/1")));

        let dup = mgr.duplicate_tab(id1).expect("Failed to duplicate tab");
        assert_eq!(mgr.tabs.len(), 2);
        assert_eq!(dup.title, "Tab 1 (Copy)");
        assert_eq!(mgr.tabs[1].title, "Tab 1 (Copy)");
        assert_ne!(dup.id, id1);
    }

    #[test]
    fn test_tab_manager_close_other_tabs() {
        let mut mgr = ExtendedTabManager::new();
        let _id1 = mgr.add_tab(ExtendedTabInfo::new(1, "Tab 1", PathBuf::from("/1")));
        let id2 = mgr.add_tab(ExtendedTabInfo::new(2, "Tab 2", PathBuf::from("/2")));
        mgr.add_tab(ExtendedTabInfo::new(3, "Tab 3", PathBuf::from("/3")));

        assert_eq!(mgr.tabs.len(), 3);

        assert!(mgr.close_other_tabs(id2));
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.tabs[0].id, id2);
        assert_eq!(mgr.active_tab_id, Some(id2));

        // Invalid tab ID returns false
        assert!(!mgr.close_other_tabs(999));
    }
}
