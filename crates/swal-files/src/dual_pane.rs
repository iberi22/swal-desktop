use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents which pane is currently active in dual-pane view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneSide {
    Left,
    Right,
}

impl PaneSide {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Holds state for dual-pane file management.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DualPaneState {
    pub left_path: PathBuf,
    pub right_path: PathBuf,
    pub active_pane: PaneSide,
    pub split_ratio: f32,
}

impl Default for DualPaneState {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self {
            left_path: home.clone(),
            right_path: home,
            active_pane: PaneSide::Left,
            split_ratio: 0.5,
        }
    }
}

/// Controller for synchronized dual-pane layout and navigation state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DualPaneController {
    pub enabled: bool,
    pub state: DualPaneState,
}

impl Default for DualPaneController {
    fn default() -> Self {
        Self::new(
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        )
    }
}

impl DualPaneController {
    /// Create a new DualPaneController with given left and right paths.
    pub fn new(left_path: PathBuf, right_path: PathBuf) -> Self {
        Self {
            enabled: false,
            state: DualPaneState {
                left_path,
                right_path,
                active_pane: PaneSide::Left,
                split_ratio: 0.5,
            },
        }
    }

    /// Toggle dual-pane view on or off. Returns the new enabled status.
    pub fn toggle_dual_pane(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    /// Focus the left pane.
    pub fn focus_left(&mut self) {
        self.state.active_pane = PaneSide::Left;
    }

    /// Focus the right pane.
    pub fn focus_right(&mut self) {
        self.state.active_pane = PaneSide::Right;
    }

    /// Synchronize non-active pane path to match active pane path.
    pub fn sync_panes(&mut self) {
        match self.state.active_pane {
            PaneSide::Left => {
                self.state.right_path = self.state.left_path.clone();
            }
            PaneSide::Right => {
                self.state.left_path = self.state.right_path.clone();
            }
        }
    }

    /// Swap left and right paths between panes.
    pub fn swap_panes(&mut self) {
        std::mem::swap(&mut self.state.left_path, &mut self.state.right_path);
    }

    /// Get reference to active pane's current path.
    pub fn active_path(&self) -> &PathBuf {
        match self.state.active_pane {
            PaneSide::Left => &self.state.left_path,
            PaneSide::Right => &self.state.right_path,
        }
    }

    /// Get mutable reference to active pane's current path.
    pub fn active_path_mut(&mut self) -> &mut PathBuf {
        match self.state.active_pane {
            PaneSide::Left => &mut self.state.left_path,
            PaneSide::Right => &mut self.state.right_path,
        }
    }

    /// Set active pane's path.
    pub fn set_active_path(&mut self, path: PathBuf) {
        match self.state.active_pane {
            PaneSide::Left => self.state.left_path = path,
            PaneSide::Right => self.state.right_path = path,
        }
    }

    /// Update split ratio bounded between 0.1 and 0.9.
    pub fn set_split_ratio(&mut self, ratio: f32) {
        self.state.split_ratio = ratio.clamp(0.1, 0.9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_dual_pane() {
        let mut controller = DualPaneController::new(PathBuf::from("/left"), PathBuf::from("/right"));
        assert!(!controller.enabled);

        assert!(controller.toggle_dual_pane());
        assert!(controller.enabled);

        assert!(!controller.toggle_dual_pane());
        assert!(!controller.enabled);
    }

    #[test]
    fn test_focus_and_paths() {
        let mut controller = DualPaneController::new(PathBuf::from("/left"), PathBuf::from("/right"));
        assert_eq!(controller.state.active_pane, PaneSide::Left);
        assert_eq!(controller.active_path(), &PathBuf::from("/left"));

        controller.focus_right();
        assert_eq!(controller.state.active_pane, PaneSide::Right);
        assert_eq!(controller.active_path(), &PathBuf::from("/right"));

        controller.focus_left();
        assert_eq!(controller.state.active_pane, PaneSide::Left);
    }

    #[test]
    fn test_sync_panes() {
        let mut controller = DualPaneController::new(PathBuf::from("/left"), PathBuf::from("/right"));
        controller.focus_left();
        controller.sync_panes();
        assert_eq!(controller.state.right_path, PathBuf::from("/left"));

        controller.set_active_path(PathBuf::from("/new_left"));
        controller.focus_right();
        controller.set_active_path(PathBuf::from("/new_right"));
        controller.sync_panes();
        assert_eq!(controller.state.left_path, PathBuf::from("/new_right"));
    }

    #[test]
    fn test_swap_panes() {
        let mut controller = DualPaneController::new(PathBuf::from("/left"), PathBuf::from("/right"));
        controller.swap_panes();
        assert_eq!(controller.state.left_path, PathBuf::from("/right"));
        assert_eq!(controller.state.right_path, PathBuf::from("/left"));
    }

    #[test]
    fn test_serde_serialization() {
        let controller = DualPaneController::new(PathBuf::from("/left"), PathBuf::from("/right"));
        let json = serde_json::to_string(&controller).unwrap();
        let deserialized: DualPaneController = serde_json::from_str(&json).unwrap();
        assert_eq!(controller, deserialized);
    }
}
