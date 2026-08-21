//! swal-files
//! Modern Minimalist Agentic File Manager Core in Rust

pub mod agent;
pub mod config;
pub mod entry;
pub mod omnibar;
pub mod scanner;

use config::FileManagerConfig;
use entry::FileEntry;
use scanner::{scan_directory, ScanOptions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneFocus {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct FileTab {
    pub id: usize,
    pub title: String,
    pub current_path: PathBuf,
    pub history: Vec<PathBuf>,
    pub history_idx: usize,
    pub selected_entry: Option<PathBuf>,
}

impl FileTab {
    pub fn new(id: usize, path: PathBuf) -> Self {
        let title = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        Self {
            id,
            title,
            current_path: path.clone(),
            history: vec![path],
            history_idx: 0,
            selected_entry: None,
        }
    }

    pub fn navigate_to(&mut self, new_path: PathBuf) {
        if new_path.exists() && new_path.is_dir() {
            self.history.truncate(self.history_idx + 1);
            self.history.push(new_path.clone());
            self.history_idx = self.history.len() - 1;
            self.title = new_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            self.current_path = new_path;
            self.selected_entry = None;
        }
    }

    pub fn go_back(&mut self) -> bool {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            self.current_path = self.history[self.history_idx].clone();
            self.title = self.current_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            return true;
        }
        false
    }
}

pub struct FileManagerSession {
    pub config: FileManagerConfig,
    pub tabs: Vec<FileTab>,
    pub active_tab_idx: usize,
    pub dual_pane_active: bool,
    pub dual_pane_path: Option<PathBuf>,
    pub active_pane: PaneFocus,
}

impl FileManagerSession {
    pub fn new(initial_path: PathBuf, config: FileManagerConfig) -> Self {
        let tab = FileTab::new(1, initial_path);
        Self {
            config,
            tabs: vec![tab],
            active_tab_idx: 0,
            dual_pane_active: false,
            dual_pane_path: None,
            active_pane: PaneFocus::Primary,
        }
    }

    pub fn is_dual_pane(&self) -> bool {
        self.dual_pane_active
    }

    pub fn split_dual_pane(&mut self, secondary_path: Option<PathBuf>) {
        self.dual_pane_active = true;
        let path = secondary_path.unwrap_or_else(|| self.active_tab().current_path.clone());
        self.dual_pane_path = Some(path);
    }

    pub fn close_dual_pane(&mut self) {
        self.dual_pane_active = false;
        self.active_pane = PaneFocus::Primary;
    }

    pub fn switch_pane_focus(&mut self) {
        if self.dual_pane_active {
            self.active_pane = match self.active_pane {
                PaneFocus::Primary => PaneFocus::Secondary,
                PaneFocus::Secondary => PaneFocus::Primary,
            };
        }
    }

    pub fn set_pane_focus(&mut self, focus: PaneFocus) {
        self.active_pane = focus;
    }

    pub fn active_pane(&self) -> PaneFocus {
        self.active_pane
    }

    pub fn sync_dual_pane_paths(&mut self) {
        if !self.dual_pane_active {
            return;
        }
        match self.active_pane {
            PaneFocus::Primary => {
                self.dual_pane_path = Some(self.active_tab().current_path.clone());
            }
            PaneFocus::Secondary => {
                if let Some(ref sec_path) = self.dual_pane_path {
                    let sec_path_cloned = sec_path.clone();
                    self.active_tab_mut().navigate_to(sec_path_cloned);
                }
            }
        }
    }

    pub fn duplicate_tab(&mut self, idx: usize) -> Option<usize> {
        let target_idx = if idx < self.tabs.len() {
            idx
        } else {
            self.active_tab_idx
        };

        if let Some(tab_to_dup) = self.tabs.get(target_idx).cloned() {
            let next_id = self.tabs.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            let mut dup = tab_to_dup;
            dup.id = next_id;
            self.tabs.insert(target_idx + 1, dup);
            self.active_tab_idx = target_idx + 1;
            Some(self.active_tab_idx)
        } else {
            None
        }
    }

    pub fn reorder_tab(&mut self, from_idx: usize, to_idx: usize) -> bool {
        if from_idx >= self.tabs.len() || to_idx >= self.tabs.len() {
            return false;
        }
        if from_idx == to_idx {
            return true;
        }

        let tab = self.tabs.remove(from_idx);
        self.tabs.insert(to_idx, tab);

        if self.active_tab_idx == from_idx {
            self.active_tab_idx = to_idx;
        } else if from_idx < self.active_tab_idx && to_idx >= self.active_tab_idx {
            self.active_tab_idx -= 1;
        } else if from_idx > self.active_tab_idx && to_idx <= self.active_tab_idx {
            self.active_tab_idx += 1;
        }
        true
    }

    pub fn active_tab(&self) -> &FileTab {
        &self.tabs[self.active_tab_idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut FileTab {
        &mut self.tabs[self.active_tab_idx]
    }

    pub fn current_entries(&self, opts: &ScanOptions) -> Result<Vec<FileEntry>, std::io::Error> {
        let path = &self.active_tab().current_path;
        scan_directory(path, opts)
    }

    pub fn new_tab(&mut self, path: PathBuf) -> usize {
        let next_id = self.tabs.len() + 1;
        let tab = FileTab::new(next_id, path);
        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
        self.active_tab_idx
    }

    pub fn close_tab(&mut self, idx: usize) -> bool {
        if self.tabs.len() > 1 && idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_directory_scanning_and_sorting() {
        let dir = tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir(p.join("alpha_dir")).unwrap();
        std::fs::write(p.join("beta_file.rs"), "fn main() {}").unwrap();
        std::fs::write(p.join("gamma_file.json"), "{}").unwrap();

        let opts = ScanOptions::default();
        let entries = scan_directory(p, &opts).unwrap();

        assert_eq!(entries.len(), 3);
        // Directories should be first
        assert!(entries[0].is_dir);
        assert_eq!(entries[0].name, "alpha_dir");
        assert_eq!(entries[1].name, "beta_file.rs");
        assert_eq!(entries[1].icon, "🦀");
    }

    #[test]
    fn test_tab_navigation_and_history() {
        let dir = tempdir().unwrap();
        let p1 = dir.path().join("dir1");
        let p2 = dir.path().join("dir2");
        std::fs::create_dir(&p1).unwrap();
        std::fs::create_dir(&p2).unwrap();

        let mut tab = FileTab::new(1, p1.clone());
        assert_eq!(tab.current_path, p1);

        tab.navigate_to(p2.clone());
        assert_eq!(tab.current_path, p2);
        assert_eq!(tab.history.len(), 2);

        assert!(tab.go_back());
        assert_eq!(tab.current_path, p1);
    }

    #[test]
    fn test_omnibar_parsing() {
        let dir = tempdir().unwrap();
        let p = dir.path();

        let intent_agent = omnibar::parse_omnibar_input("@explain this folder", p);
        assert_eq!(intent_agent, omnibar::OmnibarIntent::AgentPrompt("explain this folder".to_string()));

        let intent_cmd = omnibar::parse_omnibar_input(">git status", p);
        assert_eq!(intent_cmd, omnibar::OmnibarIntent::Command("git status".to_string()));
    }
}
