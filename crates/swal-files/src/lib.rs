//! swal-files
//! Modern Minimalist Agentic File Manager Core in Rust

pub mod agent;
pub mod agent_protocol;
pub mod app_runtime;
pub mod archive;
pub mod cli;
pub mod cloud_sync;
pub mod config;
pub mod dual_pane;
pub mod entry;
pub mod git;
pub mod gui;
pub mod native_window;
pub mod omnibar;
pub mod platform;
pub mod plugin_system;
pub mod preview;
pub mod scanner;
pub mod session;
pub mod storage;
pub mod tabs_extended;
pub mod tui;
pub mod watcher;

pub use archive::{ArchiveEntry, ArchiveError, ArchiveInspector, ArchiveKind, ArchiveMetadata, BatchTagManager};
pub use watcher::{DirectoryWatcher, FsChangeEvent, WatcherError};

use config::FileManagerConfig;
use entry::FileEntry;
use scanner::{scan_directory, ScanOptions};
use std::path::PathBuf;

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
        }
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
