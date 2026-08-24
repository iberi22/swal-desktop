//! Terminal UI (TUI) Mode for Remote SSH & Headless Linux in Pure Rust.
//! Designed for swal-files running without display servers.

use std::fs;
use std::path::{Path, PathBuf};

/// Layout options for the TUI interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiLayoutMode {
    SinglePane,
    DualPane,
    PreviewFocused,
    HelpModal,
}

/// Color theme options for ANSI escapes rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiColorTheme {
    SwalDark,
    CyberNeon,
    Monochrome,
}

/// Viewport configuration and navigation cursor state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiViewport {
    pub width: u16,
    pub height: u16,
    pub cursor_row: usize,
    pub scroll_offset: usize,
}

impl Default for TuiViewport {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            cursor_row: 0,
            scroll_offset: 0,
        }
    }
}

/// Cell structure for terminal rendering buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiCell {
    pub ch: char,
    pub fg_ansi: Option<u8>,
    pub bg_ansi: Option<u8>,
    pub bold: bool,
}

impl Default for TuiCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg_ansi: None,
            bg_ansi: None,
            bold: false,
        }
    }
}

/// Character cell matrix with ANSI escape sequence generation.
#[derive(Debug, Clone)]
pub struct TuiRenderBuffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Vec<TuiCell>>,
}

impl TuiRenderBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![TuiCell::default(); width as usize]; height as usize];
        Self { width, height, cells }
    }

    pub fn draw_str(&mut self, row: usize, col: usize, text: &str, fg: Option<u8>, bg: Option<u8>, bold: bool) {
        if row >= self.height as usize {
            return;
        }
        let mut curr_col = col;
        for ch in text.chars() {
            if curr_col >= self.width as usize {
                break;
            }
            self.cells[row][curr_col] = TuiCell {
                ch,
                fg_ansi: fg,
                bg_ansi: bg,
                bold,
            };
            curr_col += 1;
        }
    }

    pub fn render_ansi(&self) -> String {
        let mut out = String::with_capacity((self.width as usize * self.height as usize) * 4);
        for row in &self.cells {
            let mut current_fg: Option<u8> = None;
            let mut current_bg: Option<u8> = None;
            let mut current_bold = false;

            for cell in row {
                let fg_changed = cell.fg_ansi != current_fg;
                let bg_changed = cell.bg_ansi != current_bg;
                let bold_changed = cell.bold != current_bold;

                if fg_changed || bg_changed || bold_changed {
                    out.push_str("\x1b[0m");
                    current_fg = None;
                    current_bg = None;
                    current_bold = false;

                    if cell.bold {
                        out.push_str("\x1b[1m");
                        current_bold = true;
                    }
                    if let Some(fg) = cell.fg_ansi {
                        out.push_str(&format!("\x1b[38;5;{}m", fg));
                        current_fg = Some(fg);
                    }
                    if let Some(bg) = cell.bg_ansi {
                        out.push_str(&format!("\x1b[48;5;{}m", bg));
                        current_bg = Some(bg);
                    }
                }
                out.push(cell.ch);
            }
            out.push_str("\x1b[0m\n");
        }
        out
    }
}

/// Action response returned after key event handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiActionResponse {
    None,
    Redrawn,
    Navigated(PathBuf),
    OpenedFile(PathBuf),
    SearchUpdated(String),
    Quit,
}

/// Single file entry detail stored in TUI app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiFileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_str: String,
    pub git_flag: String,
}

/// Interactive TUI File Manager application struct.
#[derive(Debug, Clone)]
pub struct TuiFileManagerApp {
    pub current_path: PathBuf,
    pub dual_pane_path: PathBuf,
    pub active_pane_is_right: bool,
    pub layout_mode: TuiLayoutMode,
    pub theme: TuiColorTheme,
    pub items: Vec<TuiFileItem>,
    pub right_items: Vec<TuiFileItem>,
    pub selected_index: usize,
    pub right_selected_index: usize,
    pub search_query: String,
    pub status_message: String,
}

impl TuiFileManagerApp {
    /// Create a new TuiFileManagerApp instance initialized at initial_path.
    pub fn new(initial_path: &Path) -> Self {
        let abs_path = fs::canonicalize(initial_path).unwrap_or_else(|_| initial_path.to_path_buf());
        let mut app = Self {
            current_path: abs_path.clone(),
            dual_pane_path: abs_path.clone(),
            active_pane_is_right: false,
            layout_mode: TuiLayoutMode::SinglePane,
            theme: TuiColorTheme::SwalDark,
            items: Vec::new(),
            right_items: Vec::new(),
            selected_index: 0,
            right_selected_index: 0,
            search_query: String::new(),
            status_message: "Ready".to_string(),
        };
        app.reload_current_directory();
        app
    }

    /// Read entries from filesystem for active pane path.
    pub fn reload_current_directory(&mut self) {
        self.items = Self::scan_path(&self.current_path, &self.search_query);
        if self.selected_index >= self.items.len() && !self.items.is_empty() {
            self.selected_index = self.items.len() - 1;
        }

        if self.layout_mode == TuiLayoutMode::DualPane {
            self.right_items = Self::scan_path(&self.dual_pane_path, &self.search_query);
            if self.right_selected_index >= self.right_items.len() && !self.right_items.is_empty() {
                self.right_selected_index = self.right_items.len() - 1;
            }
        }
    }

    fn scan_path(dir: &Path, filter: &str) -> Vec<TuiFileItem> {
        let mut res = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

                if !filter.is_empty() && !name.to_lowercase().contains(&filter.to_lowercase()) {
                    continue;
                }

                let is_dir = path.is_dir();
                let size_str = if is_dir {
                    "<DIR>".to_string()
                } else {
                    entry.metadata().map(|m| format!("{} B", m.len())).unwrap_or_else(|_| "0 B".to_string())
                };

                let git_flag = if name.ends_with(".rs") || name.ends_with(".toml") {
                    "M".to_string()
                } else if name.starts_with('.') {
                    "?".to_string()
                } else {
                    " ".to_string()
                };

                res.push(TuiFileItem {
                    name,
                    path,
                    is_dir,
                    size_str,
                    git_flag,
                });
            }
        }
        res.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        res
    }

    /// Handle key events (up, down, enter, backspace, tab, etc.).
    pub fn handle_key_event(&mut self, key_code: &str, is_ctrl: bool, _is_alt: bool) -> TuiActionResponse {
        if self.layout_mode == TuiLayoutMode::HelpModal {
            if key_code == "escape" || key_code == "q" || key_code == "?" {
                self.layout_mode = TuiLayoutMode::SinglePane;
                return TuiActionResponse::Redrawn;
            }
            return TuiActionResponse::None;
        }

        match key_code {
            "q" => TuiActionResponse::Quit,
            "?" => {
                self.layout_mode = TuiLayoutMode::HelpModal;
                TuiActionResponse::Redrawn
            }
            "tab" => {
                if self.layout_mode == TuiLayoutMode::DualPane {
                    self.active_pane_is_right = !self.active_pane_is_right;
                    self.status_message = if self.active_pane_is_right {
                        "Focused Right Pane".to_string()
                    } else {
                        "Focused Left Pane".to_string()
                    };
                    TuiActionResponse::Redrawn
                } else {
                    self.toggle_dual_pane();
                    TuiActionResponse::Redrawn
                }
            }
            "up" | "k" => {
                if self.active_pane_is_right {
                    if self.right_selected_index > 0 {
                        self.right_selected_index -= 1;
                    }
                } else if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                TuiActionResponse::Redrawn
            }
            "down" | "j" => {
                if self.active_pane_is_right {
                    if !self.right_items.is_empty() && self.right_selected_index < self.right_items.len() - 1 {
                        self.right_selected_index += 1;
                    }
                } else if !self.items.is_empty() && self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                }
                TuiActionResponse::Redrawn
            }
            "backspace" | "left" | "h" => {
                let target_path = if self.active_pane_is_right {
                    self.dual_pane_path.parent().map(|p| p.to_path_buf())
                } else {
                    self.current_path.parent().map(|p| p.to_path_buf())
                };

                if let Some(parent) = target_path {
                    if self.active_pane_is_right {
                        self.dual_pane_path = parent.clone();
                        self.right_selected_index = 0;
                    } else {
                        self.current_path = parent.clone();
                        self.selected_index = 0;
                    }
                    self.reload_current_directory();
                    TuiActionResponse::Navigated(parent)
                } else {
                    TuiActionResponse::None
                }
            }
            "enter" | "right" | "l" => {
                let (active_items, active_idx) = if self.active_pane_is_right {
                    (&self.right_items, self.right_selected_index)
                } else {
                    (&self.items, self.selected_index)
                };

                if let Some(item) = active_items.get(active_idx) {
                    if item.is_dir {
                        let new_path = item.path.clone();
                        if self.active_pane_is_right {
                            self.dual_pane_path = new_path.clone();
                            self.right_selected_index = 0;
                        } else {
                            self.current_path = new_path.clone();
                            self.selected_index = 0;
                        }
                        self.reload_current_directory();
                        TuiActionResponse::Navigated(new_path)
                    } else {
                        TuiActionResponse::OpenedFile(item.path.clone())
                    }
                } else {
                    TuiActionResponse::None
                }
            }
            "p" => {
                if self.layout_mode == TuiLayoutMode::PreviewFocused {
                    self.layout_mode = TuiLayoutMode::SinglePane;
                } else {
                    self.layout_mode = TuiLayoutMode::PreviewFocused;
                }
                TuiActionResponse::Redrawn
            }
            "t" if is_ctrl => {
                self.theme = match self.theme {
                    TuiColorTheme::SwalDark => TuiColorTheme::CyberNeon,
                    TuiColorTheme::CyberNeon => TuiColorTheme::Monochrome,
                    TuiColorTheme::Monochrome => TuiColorTheme::SwalDark,
                };
                TuiActionResponse::Redrawn
            }
            _ => TuiActionResponse::None,
        }
    }

    /// Toggle between single-pane and dual-pane views.
    pub fn toggle_dual_pane(&mut self) {
        if self.layout_mode == TuiLayoutMode::DualPane {
            self.layout_mode = TuiLayoutMode::SinglePane;
            self.active_pane_is_right = false;
        } else {
            self.layout_mode = TuiLayoutMode::DualPane;
            if self.dual_pane_path == PathBuf::default() || self.dual_pane_path == self.current_path {
                self.dual_pane_path = self.current_path.clone();
            }
            self.reload_current_directory();
        }
    }

    /// Update search filter query and refresh item list.
    pub fn search_filter(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.reload_current_directory();
    }

    /// Retrieve preview text lines for currently selected item.
    pub fn get_preview_text(&self, max_lines: usize) -> Vec<String> {
        let (active_items, active_idx) = if self.active_pane_is_right {
            (&self.right_items, self.right_selected_index)
        } else {
            (&self.items, self.selected_index)
        };

        if let Some(item) = active_items.get(active_idx) {
            if item.is_dir {
                vec![format!("Directory: {}", item.name), "[Folder contents preview]".to_string()]
            } else if let Ok(content) = fs::read_to_string(&item.path) {
                content.lines().take(max_lines).map(|s| s.to_string()).collect()
            } else {
                vec!["[Binary or Unreadable File]".to_string()]
            }
        } else {
            vec!["[No item selected]".to_string()]
        }
    }

    /// Render terminal UI to string buffer containing ANSI sequences.
    pub fn render_to_buffer(&self, viewport: &TuiViewport) -> String {
        let mut buffer = TuiRenderBuffer::new(viewport.width, viewport.height);
        let (header_fg, active_bg, dir_fg, file_fg) = match self.theme {
            TuiColorTheme::SwalDark => (Some(141), Some(237), Some(39), Some(252)),
            TuiColorTheme::CyberNeon => (Some(51), Some(198), Some(226), Some(255)),
            TuiColorTheme::Monochrome => (Some(255), Some(240), Some(255), Some(250)),
        };

        if self.layout_mode == TuiLayoutMode::HelpModal {
            buffer.draw_str(1, 2, "┌──────────────────────────────────────────┐", header_fg, None, true);
            buffer.draw_str(2, 2, "│      SWAL Files TUI Quick Help           │", header_fg, None, true);
            buffer.draw_str(3, 2, "├──────────────────────────────────────────┤", header_fg, None, false);
            buffer.draw_str(4, 2, "│  j / Down   : Move selection down        │", file_fg, None, false);
            buffer.draw_str(5, 2, "│  k / Up     : Move selection up          │", file_fg, None, false);
            buffer.draw_str(6, 2, "│  Enter / l  : Open folder or file        │", file_fg, None, false);
            buffer.draw_str(7, 2, "│  Backspace/h: Navigate to parent folder │", file_fg, None, false);
            buffer.draw_str(8, 2, "│  Tab        : Toggle / Switch Dual Pane  │", file_fg, None, false);
            buffer.draw_str(9, 2, "│  p          : Toggle File Preview        │", file_fg, None, false);
            buffer.draw_str(10, 2, "│  q          : Quit TUI Mode              │", file_fg, None, false);
            buffer.draw_str(11, 2, "└──────────────────────────────────────────┘", header_fg, None, true);
            return buffer.render_ansi();
        }

        // Header Breadcrumbs
        let breadcrumbs = format!(" SWAL Files :: {} ", self.current_path.to_string_lossy());
        buffer.draw_str(0, 0, &breadcrumbs, header_fg, None, true);

        let content_height = if viewport.height > 3 { viewport.height as usize - 2 } else { 1 };

        if self.layout_mode == TuiLayoutMode::DualPane {
            let half_width = (viewport.width as usize) / 2;

            // Render Left Pane
            let max_pane_chars = half_width.saturating_sub(2);
            for (idx, item) in self.items.iter().take(content_height).enumerate() {
                let is_sel = idx == self.selected_index && !self.active_pane_is_right;
                let bg = if is_sel { active_bg } else { None };
                let fg = if item.is_dir { dir_fg } else { file_fg };
                let line = format!("{} {} [{}]", item.git_flag, item.name, item.size_str);
                let truncated: String = line.chars().take(max_pane_chars).collect();
                buffer.draw_str(1 + idx, 1, &truncated, fg, bg, is_sel);
            }

            // Render Split Line
            for r in 1..=content_height {
                buffer.draw_str(r, half_width, "│", header_fg, None, false);
            }

            // Render Right Pane
            for (idx, item) in self.right_items.iter().take(content_height).enumerate() {
                let is_sel = idx == self.right_selected_index && self.active_pane_is_right;
                let bg = if is_sel { active_bg } else { None };
                let fg = if item.is_dir { dir_fg } else { file_fg };
                let line = format!("{} {} [{}]", item.git_flag, item.name, item.size_str);
                let truncated: String = line.chars().take(max_pane_chars).collect();
                buffer.draw_str(1 + idx, half_width + 2, &truncated, fg, bg, is_sel);
            }
        } else {
            let left_width = if self.layout_mode == TuiLayoutMode::PreviewFocused {
                (viewport.width as usize) / 3
            } else {
                (viewport.width as usize) * 3 / 5
            };
            let max_left_chars = left_width.saturating_sub(1);
            let max_right_chars = (viewport.width as usize).saturating_sub(left_width).saturating_sub(3);

            // Render Single Pane File List
            for (idx, item) in self.items.iter().skip(viewport.scroll_offset).take(content_height).enumerate() {
                let actual_idx = idx + viewport.scroll_offset;
                let is_sel = actual_idx == self.selected_index;
                let bg = if is_sel { active_bg } else { None };
                let fg = if item.is_dir { dir_fg } else { file_fg };
                let line = format!("{} {} [{}]", item.git_flag, item.name, item.size_str);
                let truncated: String = line.chars().take(max_left_chars).collect();
                buffer.draw_str(1 + idx, 1, &truncated, fg, bg, is_sel);
            }

            // Render Preview Side Panel
            let preview_lines = self.get_preview_text(content_height);
            for (r, line) in preview_lines.iter().enumerate().take(content_height) {
                let truncated: String = line.chars().take(max_right_chars).collect();
                buffer.draw_str(1 + r, left_width + 2, &truncated, file_fg, None, false);
            }
        }

        // Status bar
        let status = format!(" [Filter: {}] | {} | Press ? for Help", self.search_query, self.status_message);
        buffer.draw_str(viewport.height as usize - 1, 0, &status, header_fg, None, true);

        buffer.render_ansi()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tui_app_initialization_and_navigation() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sub_folder");
        fs::create_dir_all(&sub_dir).unwrap();
        let file_path = dir.path().join("hello.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let mut app = TuiFileManagerApp::new(dir.path());
        assert!(!app.items.is_empty());

        // Test down navigation
        let resp = app.handle_key_event("down", false, false);
        assert_eq!(resp, TuiActionResponse::Redrawn);

        // Test up navigation
        let resp_up = app.handle_key_event("up", false, false);
        assert_eq!(resp_up, TuiActionResponse::Redrawn);

        // Test enter directory navigation
        let enter_resp = app.handle_key_event("enter", false, false);
        assert!(matches!(enter_resp, TuiActionResponse::Navigated(_) | TuiActionResponse::OpenedFile(_)));

        // Test backspace navigation
        let back_resp = app.handle_key_event("backspace", false, false);
        assert!(matches!(back_resp, TuiActionResponse::Navigated(_)));
    }

    #[test]
    fn test_tui_ansi_rendering_and_dual_pane() {
        let dir = tempdir().unwrap();
        let mut app = TuiFileManagerApp::new(dir.path());
        let viewport = TuiViewport {
            width: 80,
            height: 24,
            cursor_row: 0,
            scroll_offset: 0,
        };

        let output_single = app.render_to_buffer(&viewport);
        assert!(output_single.contains("SWAL Files ::"));

        // Toggle Dual Pane
        app.toggle_dual_pane();
        assert_eq!(app.layout_mode, TuiLayoutMode::DualPane);

        let output_dual = app.render_to_buffer(&viewport);
        assert!(output_dual.contains("│"));

        // Tab to switch panes
        let tab_resp = app.handle_key_event("tab", false, false);
        assert_eq!(tab_resp, TuiActionResponse::Redrawn);
        assert!(app.active_pane_is_right);
    }

    #[test]
    fn test_tui_search_filter_and_preview_clipping() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test_code.rs");
        fs::write(&test_file, "line1\nline2\nline3\nline4\nline5").unwrap();

        let mut app = TuiFileManagerApp::new(dir.path());
        app.search_filter("test_code");
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].name, "test_code.rs");

        let preview = app.get_preview_text(3);
        assert_eq!(preview.len(), 3);
        assert_eq!(preview[0], "line1");
        assert_eq!(preview[1], "line2");
        assert_eq!(preview[2], "line3");
    }

    #[test]
    fn test_tui_help_modal_and_quit() {
        let dir = tempdir().unwrap();
        let mut app = TuiFileManagerApp::new(dir.path());
        let help_resp = app.handle_key_event("?", false, false);
        assert_eq!(help_resp, TuiActionResponse::Redrawn);
        assert_eq!(app.layout_mode, TuiLayoutMode::HelpModal);

        let esc_resp = app.handle_key_event("escape", false, false);
        assert_eq!(esc_resp, TuiActionResponse::Redrawn);
        assert_eq!(app.layout_mode, TuiLayoutMode::SinglePane);

        let quit_resp = app.handle_key_event("q", false, false);
        assert_eq!(quit_resp, TuiActionResponse::Quit);
    }
}
