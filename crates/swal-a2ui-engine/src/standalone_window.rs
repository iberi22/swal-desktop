use serde::{Deserialize, Serialize};
use crate::ComponentNode;

/// Titlebar decoration styling variants for standalone windows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TitlebarStyle {
    FluentMica,
    MacOSTrafficLights,
    MinimalistFrameless,
    CustomSkin(String),
}

/// Caption and window control action button kinds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowButtonKind {
    Close,
    Minimize,
    Maximize,
    Restore,
    PinToTop,
    Settings,
}

/// Standalone Window Frame AST model holding window state, titlebar style, and action controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandaloneWindowFrame {
    pub title: String,
    pub app_icon: String,
    pub style: TitlebarStyle,
    pub is_maximized: bool,
    pub is_focused: bool,
    pub show_breadcrumbs: bool,
    pub custom_actions: Vec<ComponentNode>,
}

impl StandaloneWindowFrame {
    /// Creates a new `StandaloneWindowFrame` with default attributes.
    pub fn new(title: &str, style: TitlebarStyle) -> Self {
        Self {
            title: title.to_string(),
            app_icon: "swal-desktop".to_string(),
            style,
            is_maximized: false,
            is_focused: true,
            show_breadcrumbs: true,
            custom_actions: Vec::new(),
        }
    }

    /// Appends a custom action `ComponentNode` (such as a toolbar button) to the frame.
    pub fn with_custom_action(mut self, node: ComponentNode) -> Self {
        self.custom_actions.push(node);
        self
    }

    /// Sets the app icon identifier string.
    pub fn with_app_icon(mut self, icon: impl Into<String>) -> Self {
        self.app_icon = icon.into();
        self
    }

    /// Sets the window maximized state.
    pub fn with_maximized(mut self, is_maximized: bool) -> Self {
        self.is_maximized = is_maximized;
        self
    }

    /// Sets the window focus state.
    pub fn with_focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        self
    }

    /// Sets whether titlebar breadcrumbs are visible.
    pub fn with_breadcrumbs(mut self, show: bool) -> Self {
        self.show_breadcrumbs = show;
        self
    }

    /// Wraps an arbitrary content A2UI `ComponentNode` tree with titlebar, controls, and status footer.
    pub fn wrap_content_tree(&self, content_root: ComponentNode) -> ComponentNode {
        let _max_label = if self.is_maximized { "Restore" } else { "Maximize" };
        let max_action = if self.is_maximized { "window.restore" } else { "window.maximize" };

        let mut header_children = Vec::new();

        match &self.style {
            TitlebarStyle::MacOSTrafficLights => {
                header_children.push(ComponentNode::Grid {
                    columns: 3,
                    gap: 6,
                    children: vec![
                        ComponentNode::Button {
                            label: "🔴".to_string(),
                            action: "window.close".to_string(),
                            variant: Some("traffic_close".to_string()),
                        },
                        ComponentNode::Button {
                            label: "🟡".to_string(),
                            action: "window.minimize".to_string(),
                            variant: Some("traffic_minimize".to_string()),
                        },
                        ComponentNode::Button {
                            label: "🟢".to_string(),
                            action: max_action.to_string(),
                            variant: Some("traffic_maximize".to_string()),
                        },
                    ],
                });

                let mut title_label = format!("{} - {}", self.app_icon, self.title);
                if self.show_breadcrumbs {
                    title_label = format!("SWAL Desktop ❯ {}", title_label);
                }

                header_children.push(ComponentNode::StatusBadge {
                    status: if self.is_focused { "active".to_string() } else { "inactive".to_string() },
                    label: title_label,
                    color: None,
                });
            }
            _ => {
                let mut title_label = format!("{} - {}", self.app_icon, self.title);
                if self.show_breadcrumbs {
                    title_label = format!("SWAL Desktop ❯ {}", title_label);
                }

                header_children.push(ComponentNode::StatusBadge {
                    status: if self.is_focused { "active".to_string() } else { "inactive".to_string() },
                    label: title_label,
                    color: None,
                });

                let mut controls = vec![
                    ComponentNode::Button {
                        label: "—".to_string(),
                        action: "window.minimize".to_string(),
                        variant: Some("caption_minimize".to_string()),
                    },
                    ComponentNode::Button {
                        label: if self.is_maximized { "🗗" } else { "🗖" }.to_string(),
                        action: max_action.to_string(),
                        variant: Some("caption_maximize".to_string()),
                    },
                    ComponentNode::Button {
                        label: "✕".to_string(),
                        action: "window.close".to_string(),
                        variant: Some("caption_close".to_string()),
                    },
                ];

                for custom in &self.custom_actions {
                    controls.insert(0, custom.clone());
                }

                header_children.push(ComponentNode::Grid {
                    columns: controls.len(),
                    gap: 4,
                    children: controls,
                });
            }
        }

        let header_grid = ComponentNode::Grid {
            columns: header_children.len(),
            gap: 12,
            children: header_children,
        };

        let footer = ComponentNode::MetricPill {
            label: "Window State".to_string(),
            value: if self.is_maximized { "Maximized".to_string() } else { "Windowed".to_string() },
            unit: None,
            trend: None,
            color: Some(if self.is_focused { "$accent_primary".to_string() } else { "$text_secondary".to_string() }),
        };

        ComponentNode::Card {
            title: Some(format!("Frame: {}", self.title)),
            elevation: Some("fluent_mica_frame".to_string()),
            children: vec![header_grid, content_root, footer],
        }
    }

    /// Spatial hit-testing logic for caption window buttons given pointer coordinates and frame width.
    /// Titlebar region assumption: 0.0 <= y <= 32.0.
    pub fn handle_caption_hit_test(&self, x: f32, y: f32, width: f32) -> Option<WindowButtonKind> {
        let titlebar_height = 32.0;
        if y < 0.0 || y > titlebar_height || x < 0.0 || x > width {
            return None;
        }

        match &self.style {
            TitlebarStyle::MacOSTrafficLights => {
                // Left-aligned macOS traffic lights: Close @ 12..24, Minimize @ 28..40, Max/Restore @ 44..56
                if x >= 12.0 && x <= 24.0 {
                    Some(WindowButtonKind::Close)
                } else if x >= 28.0 && x <= 40.0 {
                    Some(WindowButtonKind::Minimize)
                } else if x >= 44.0 && x <= 56.0 {
                    Some(if self.is_maximized { WindowButtonKind::Restore } else { WindowButtonKind::Maximize })
                } else {
                    None
                }
            }
            _ => {
                // Right-aligned window caption controls: button width = 45.0
                let btn_width = 45.0;
                let close_start = width - btn_width;
                let max_start = width - btn_width * 2.0;
                let min_start = width - btn_width * 3.0;
                let pin_start = width - btn_width * 4.0;
                let settings_start = width - btn_width * 5.0;

                if x >= close_start && x <= width {
                    Some(WindowButtonKind::Close)
                } else if x >= max_start && x < close_start {
                    Some(if self.is_maximized { WindowButtonKind::Restore } else { WindowButtonKind::Maximize })
                } else if x >= min_start && x < max_start {
                    Some(WindowButtonKind::Minimize)
                } else if x >= pin_start && x < min_start {
                    Some(WindowButtonKind::PinToTop)
                } else if x >= settings_start && x < pin_start {
                    Some(WindowButtonKind::Settings)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standalone_window_frame_new_and_builder() {
        let custom_button = ComponentNode::Button {
            label: "Pin".to_string(),
            action: "window.pin".to_string(),
            variant: None,
        };

        let frame = StandaloneWindowFrame::new("Xavier Workspace", TitlebarStyle::FluentMica)
            .with_app_icon("xavier-app")
            .with_maximized(true)
            .with_focused(false)
            .with_breadcrumbs(false)
            .with_custom_action(custom_button.clone());

        assert_eq!(frame.title, "Xavier Workspace");
        assert_eq!(frame.app_icon, "xavier-app");
        assert_eq!(frame.style, TitlebarStyle::FluentMica);
        assert!(frame.is_maximized);
        assert!(!frame.is_focused);
        assert!(!frame.show_breadcrumbs);
        assert_eq!(frame.custom_actions.len(), 1);
        assert_eq!(frame.custom_actions[0], custom_button);
    }

    #[test]
    fn test_titlebar_style_serde_serialization() {
        let styles = vec![
            TitlebarStyle::FluentMica,
            TitlebarStyle::MacOSTrafficLights,
            TitlebarStyle::MinimalistFrameless,
            TitlebarStyle::CustomSkin("cyber_neon_v2".to_string()),
        ];

        for style in styles {
            let serialized = serde_json::to_string(&style).expect("Must serialize TitlebarStyle");
            let deserialized: TitlebarStyle = serde_json::from_str(&serialized).expect("Must deserialize TitlebarStyle");
            assert_eq!(style, deserialized);
        }

        let button_kinds = vec![
            WindowButtonKind::Close,
            WindowButtonKind::Minimize,
            WindowButtonKind::Maximize,
            WindowButtonKind::Restore,
            WindowButtonKind::PinToTop,
            WindowButtonKind::Settings,
        ];

        for kind in button_kinds {
            let serialized = serde_json::to_string(&kind).expect("Must serialize WindowButtonKind");
            let deserialized: WindowButtonKind = serde_json::from_str(&serialized).expect("Must deserialize WindowButtonKind");
            assert_eq!(kind, deserialized);
        }
    }

    #[test]
    fn test_frame_serde_roundtrip() {
        let frame = StandaloneWindowFrame::new("Telemetry Dashboard", TitlebarStyle::FluentMica)
            .with_maximized(false)
            .with_focused(true);

        let json = serde_json::to_string(&frame).expect("Must serialize StandaloneWindowFrame");
        let deserialized: StandaloneWindowFrame = serde_json::from_str(&json).expect("Must deserialize StandaloneWindowFrame");
        assert_eq!(frame, deserialized);
    }

    #[test]
    fn test_wrap_content_tree_hierarchy() {
        let frame = StandaloneWindowFrame::new("SWAL Settings", TitlebarStyle::FluentMica)
            .with_breadcrumbs(true)
            .with_maximized(false);

        let content_node = ComponentNode::Card {
            title: Some("Preferences".to_string()),
            elevation: None,
            children: vec![ComponentNode::Button {
                label: "Save".to_string(),
                action: "settings.save".to_string(),
                variant: None,
            }],
        };

        let wrapped = frame.wrap_content_tree(content_node.clone());

        if let ComponentNode::Card { title, elevation, children } = wrapped {
            assert_eq!(title, Some("Frame: SWAL Settings".to_string()));
            assert_eq!(elevation, Some("fluent_mica_frame".to_string()));
            assert_eq!(children.len(), 3);
            // Header grid
            assert!(matches!(children[0], ComponentNode::Grid { .. }));
            // Inner content root
            assert_eq!(children[1], content_node);
            // Footer metric pill
            assert!(matches!(children[2], ComponentNode::MetricPill { .. }));
        } else {
            panic!("Expected Card root from wrap_content_tree");
        }
    }

    #[test]
    fn test_wrap_content_tree_macos_traffic_lights() {
        let frame = StandaloneWindowFrame::new("Files", TitlebarStyle::MacOSTrafficLights)
            .with_maximized(true);

        let content_node = ComponentNode::StatusBadge {
            status: "ok".to_string(),
            label: "Ready".to_string(),
            color: None,
        };

        let wrapped = frame.wrap_content_tree(content_node);

        if let ComponentNode::Card { children, .. } = wrapped {
            if let ComponentNode::Grid { children: header_children, .. } = &children[0] {
                assert_eq!(header_children.len(), 2);
                // First header child is traffic lights grid
                if let ComponentNode::Grid { children: traffic_btns, .. } = &header_children[0] {
                    assert_eq!(traffic_btns.len(), 3);
                    assert!(matches!(&traffic_btns[2], ComponentNode::Button { action, .. } if action == "window.restore"));
                } else {
                    panic!("Expected traffic lights grid");
                }
            } else {
                panic!("Expected Header Grid");
            }
        } else {
            panic!("Expected Card root");
        }
    }

    #[test]
    fn test_caption_hit_testing_macos_style() {
        let frame = StandaloneWindowFrame::new("Finder", TitlebarStyle::MacOSTrafficLights)
            .with_maximized(false);

        let width = 800.0;

        // Outside titlebar height
        assert_eq!(frame.handle_caption_hit_test(15.0, 40.0, width), None);
        assert_eq!(frame.handle_caption_hit_test(15.0, -5.0, width), None);

        // macOS left-aligned buttons
        assert_eq!(frame.handle_caption_hit_test(18.0, 16.0, width), Some(WindowButtonKind::Close));
        assert_eq!(frame.handle_caption_hit_test(32.0, 16.0, width), Some(WindowButtonKind::Minimize));
        assert_eq!(frame.handle_caption_hit_test(50.0, 16.0, width), Some(WindowButtonKind::Maximize));

        // When maximized, green traffic light returns Restore
        let max_frame = frame.with_maximized(true);
        assert_eq!(max_frame.handle_caption_hit_test(50.0, 16.0, width), Some(WindowButtonKind::Restore));

        // Blank area on MacOS titlebar
        assert_eq!(max_frame.handle_caption_hit_test(100.0, 16.0, width), None);
    }

    #[test]
    fn test_caption_hit_testing_fluent_style() {
        let frame = StandaloneWindowFrame::new("SWAL Terminal", TitlebarStyle::FluentMica)
            .with_maximized(false);

        let width = 1000.0;

        // Fluent right-aligned buttons (width = 1000.0, btn width = 45.0)
        // Close: 955.0 .. 1000.0
        // Maximize: 910.0 .. 955.0
        // Minimize: 865.0 .. 910.0
        // PinToTop: 820.0 .. 865.0
        // Settings: 775.0 .. 820.0

        assert_eq!(frame.handle_caption_hit_test(970.0, 15.0, width), Some(WindowButtonKind::Close));
        assert_eq!(frame.handle_caption_hit_test(930.0, 15.0, width), Some(WindowButtonKind::Maximize));
        assert_eq!(frame.handle_caption_hit_test(880.0, 15.0, width), Some(WindowButtonKind::Minimize));
        assert_eq!(frame.handle_caption_hit_test(840.0, 15.0, width), Some(WindowButtonKind::PinToTop));
        assert_eq!(frame.handle_caption_hit_test(790.0, 15.0, width), Some(WindowButtonKind::Settings));

        // Maximized restore test
        let max_frame = frame.with_maximized(true);
        assert_eq!(max_frame.handle_caption_hit_test(930.0, 15.0, width), Some(WindowButtonKind::Restore));

        // Unhit area
        assert_eq!(max_frame.handle_caption_hit_test(500.0, 15.0, width), None);
        assert_eq!(max_frame.handle_caption_hit_test(1050.0, 15.0, width), None);
    }
}
