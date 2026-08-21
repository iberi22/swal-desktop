//! macOS-Inspired Centralized System Settings Window Layout Builder
//!
//! Provides a two-column settings panel builder producing declarative A2UI (`ComponentNode`) trees.
//! Features sidebar navigation across 9 system categories and active selection highlighting.

use serde::{Deserialize, Serialize};
use swal_a2ui_engine::ComponentNode;

/// System Settings Categories matching macOS-inspired layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsCategory {
    General,
    Appearance,
    Agents,
    Display,
    Storage,
    Xavier,
    Keybinds,
    Audio,
    About,
}

impl SettingsCategory {
    /// Returns all 9 categories in canonical sidebar display order.
    pub fn all() -> [SettingsCategory; 9] {
        [
            SettingsCategory::General,
            SettingsCategory::Appearance,
            SettingsCategory::Agents,
            SettingsCategory::Display,
            SettingsCategory::Storage,
            SettingsCategory::Xavier,
            SettingsCategory::Keybinds,
            SettingsCategory::Audio,
            SettingsCategory::About,
        ]
    }

    /// Label display text for category.
    pub fn label(&self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Appearance => "Appearance",
            SettingsCategory::Agents => "AI Agents",
            SettingsCategory::Display => "Display",
            SettingsCategory::Storage => "Storage",
            SettingsCategory::Xavier => "Xavier Core",
            SettingsCategory::Keybinds => "Keybinds",
            SettingsCategory::Audio => "Audio",
            SettingsCategory::About => "About SWAL",
        }
    }

    /// Category icon symbol.
    pub fn icon(&self) -> &'static str {
        match self {
            SettingsCategory::General => "⚙️",
            SettingsCategory::Appearance => "🎨",
            SettingsCategory::Agents => "🤖",
            SettingsCategory::Display => "🖥️",
            SettingsCategory::Storage => "💾",
            SettingsCategory::Xavier => "🧠",
            SettingsCategory::Keybinds => "⌨️",
            SettingsCategory::Audio => "🔊",
            SettingsCategory::About => "ℹ️",
        }
    }

    /// Machine action ID string for switching category.
    pub fn action_id(&self) -> String {
        format!("settings.switch_category:{}", match self {
            SettingsCategory::General => "general",
            SettingsCategory::Appearance => "appearance",
            SettingsCategory::Agents => "agents",
            SettingsCategory::Display => "display",
            SettingsCategory::Storage => "storage",
            SettingsCategory::Xavier => "xavier",
            SettingsCategory::Keybinds => "keybinds",
            SettingsCategory::Audio => "audio",
            SettingsCategory::About => "about",
        })
    }
}

/// System settings configuration model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwalSystemSettings {
    pub theme_name: String,
    pub corner_radius: u32,
    pub opacity: f32,
    pub enable_ai_agents: bool,
    pub agent_auto_start: bool,
    pub storage_warning_threshold: u32,
    pub xavier_api_url: String,
    pub xavier_mcp_port: u16,
    pub default_keybinds_preset: String,
    pub audio_output_device: String,
    pub audio_volume: u32,
    pub system_version: String,
}

impl Default for SwalSystemSettings {
    fn default() -> Self {
        Self {
            theme_name: "Fluent Glass".to_string(),
            corner_radius: 12,
            opacity: 0.85,
            enable_ai_agents: true,
            agent_auto_start: true,
            storage_warning_threshold: 85,
            xavier_api_url: "http://127.0.0.1:8006".to_string(),
            xavier_mcp_port: 8100,
            default_keybinds_preset: "Hyprland / Niri Standard".to_string(),
            audio_output_device: "Default Analog Stereo".to_string(),
            audio_volume: 75,
            system_version: "SWAL Desktop 5.0.4".to_string(),
        }
    }
}

/// Layout builder for macOS-inspired centralized system settings.
pub struct SettingsWindowBuilder;

impl SettingsWindowBuilder {
    /// Builds a declarative A2UI `ComponentNode` tree for the settings window.
    pub fn build_settings_layout(
        active_category: SettingsCategory,
        settings: &SwalSystemSettings,
    ) -> ComponentNode {
        // 1. Build Left Column Sidebar Navigation
        let mut sidebar_nodes = Vec::new();
        for cat in SettingsCategory::all() {
            let is_selected = cat == active_category;
            let label = if is_selected {
                format!("▶ {} {}", cat.icon(), cat.label())
            } else {
                format!("  {} {}", cat.icon(), cat.label())
            };

            sidebar_nodes.push(ComponentNode::Button {
                label,
                action: cat.action_id(),
                variant: Some(if is_selected {
                    "primary".to_string()
                } else {
                    "ghost".to_string()
                }),
            });
        }

        let sidebar = ComponentNode::Card {
            title: Some("System Settings".to_string()),
            elevation: Some("elevated".to_string()),
            children: sidebar_nodes,
        };

        // 2. Build Right Column Category Content Panel
        let content_panel = match active_category {
            SettingsCategory::General => Self::build_general_panel(settings),
            SettingsCategory::Appearance => Self::build_appearance_panel(settings),
            SettingsCategory::Agents => Self::build_agents_panel(settings),
            SettingsCategory::Display => Self::build_display_panel(settings),
            SettingsCategory::Storage => Self::build_storage_panel(settings),
            SettingsCategory::Xavier => Self::build_xavier_panel(settings),
            SettingsCategory::Keybinds => Self::build_keybinds_panel(settings),
            SettingsCategory::Audio => Self::build_audio_panel(settings),
            SettingsCategory::About => Self::build_about_panel(settings),
        };

        // 3. Combine into two-column root container
        ComponentNode::Card {
            title: Some(format!(
                "Centralized System Preferences — {}",
                active_category.label()
            )),
            elevation: Some("mica".to_string()),
            children: vec![ComponentNode::Grid {
                columns: 2,
                gap: 16,
                children: vec![sidebar, content_panel],
            }],
        }
    }

    fn build_general_panel(_settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("⚙️ General Preferences".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Host Name".to_string(),
                    value: "swal-desktop".to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#06b6d4".to_string()),
                },
                ComponentNode::Button {
                    label: "Launch SWAL on Boot: Enabled".to_string(),
                    action: "settings.toggle_boot_launch".to_string(),
                    variant: Some("subtle".to_string()),
                },
                ComponentNode::Button {
                    label: "Notifications: Allowed".to_string(),
                    action: "settings.toggle_notifications".to_string(),
                    variant: Some("subtle".to_string()),
                },
            ],
        }
    }

    fn build_appearance_panel(settings: &SwalSystemSettings) -> ComponentNode {
        let theme_buttons = vec![
            ("Fluent Glass", "theme.set:fluent-glass"),
            ("Hive Dark", "theme.set:hive-dark"),
            ("Cyber Neon", "theme.set:cyber-neon"),
            ("Nordic Frost", "theme.set:nordic-frost"),
        ]
        .into_iter()
        .map(|(t_name, action)| ComponentNode::Button {
            label: format!(
                "{} {}",
                if settings.theme_name == t_name { "✓" } else { " " },
                t_name
            ),
            action: action.to_string(),
            variant: Some(if settings.theme_name == t_name {
                "primary".to_string()
            } else {
                "subtle".to_string()
            }),
        })
        .collect();

        let theme_group = ComponentNode::Card {
            title: Some("Active Theme".to_string()),
            elevation: Some("flat".to_string()),
            children: theme_buttons,
        };

        let metrics_group = ComponentNode::Card {
            title: Some("Corner Radius & Acrylic Opacity Sliders".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Corner Radius".to_string(),
                    value: format!("{} px", settings.corner_radius),
                    unit: None,
                    trend: None,
                    color: Some("#3b82f6".to_string()),
                },
                ComponentNode::Button {
                    label: "Corner Radius - 2px".to_string(),
                    action: "settings.adjust_corner:-2".to_string(),
                    variant: Some("subtle".to_string()),
                },
                ComponentNode::Button {
                    label: "Corner Radius + 2px".to_string(),
                    action: "settings.adjust_corner:2".to_string(),
                    variant: Some("subtle".to_string()),
                },
                ComponentNode::MetricPill {
                    label: "Mica Glass Opacity".to_string(),
                    value: format!("{:.0}%", settings.opacity * 100.0),
                    unit: None,
                    trend: None,
                    color: Some("#10b981".to_string()),
                },
                ComponentNode::Button {
                    label: "Opacity - 5%".to_string(),
                    action: "settings.adjust_opacity:-0.05".to_string(),
                    variant: Some("subtle".to_string()),
                },
                ComponentNode::Button {
                    label: "Opacity + 5%".to_string(),
                    action: "settings.adjust_opacity:0.05".to_string(),
                    variant: Some("subtle".to_string()),
                },
            ],
        };

        ComponentNode::Card {
            title: Some("🎨 Appearance & Visual Styling".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![theme_group, metrics_group],
        }
    }

    fn build_agents_panel(settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("🤖 AI Agent Infrastructure".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::StatusBadge {
                    status: if settings.enable_ai_agents {
                        "active".to_string()
                    } else {
                        "disabled".to_string()
                    },
                    label: if settings.enable_ai_agents {
                        "AI Agents Active".to_string()
                    } else {
                        "AI Agents Suspended".to_string()
                    },
                    color: Some(if settings.enable_ai_agents {
                        "#10b981".to_string()
                    } else {
                        "#ef4444".to_string()
                    }),
                },
                ComponentNode::Button {
                    label: format!(
                        "Enable Autonomous AI Agents: {}",
                        if settings.enable_ai_agents { "ON" } else { "OFF" }
                    ),
                    action: "settings.toggle_ai_agents".to_string(),
                    variant: Some("primary".to_string()),
                },
                ComponentNode::Button {
                    label: format!(
                        "Auto-Start Supervisor Daemon: {}",
                        if settings.agent_auto_start { "ON" } else { "OFF" }
                    ),
                    action: "settings.toggle_agent_auto_start".to_string(),
                    variant: Some("subtle".to_string()),
                },
            ],
        }
    }

    fn build_display_panel(_settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("🖥️ Display & Refresh Rate".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Primary Monitor".to_string(),
                    value: "3840x2160 @ 240Hz".to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#8b5cf6".to_string()),
                },
                ComponentNode::MetricPill {
                    label: "HDR Mode".to_string(),
                    value: "Enabled (10-bit)".to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#10b981".to_string()),
                },
                ComponentNode::Button {
                    label: "Configure Wayland Output".to_string(),
                    action: "display.configure".to_string(),
                    variant: Some("subtle".to_string()),
                },
            ],
        }
    }

    fn build_storage_panel(settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("💾 Storage & Mount Diagnostics".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Warning Threshold".to_string(),
                    value: format!("{}%", settings.storage_warning_threshold),
                    unit: None,
                    trend: None,
                    color: Some("#f59e0b".to_string()),
                },
                ComponentNode::MetricPill {
                    label: "Root NVMe (/)" .to_string(),
                    value: "42% Used (210 GB / 500 GB)".to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#10b981".to_string()),
                },
                ComponentNode::Button {
                    label: "Run Storage Cleanup".to_string(),
                    action: "storage.cleanup".to_string(),
                    variant: Some("primary".to_string()),
                },
            ],
        }
    }

    fn build_xavier_panel(settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("🧠 Xavier Cognitive Memory Core & Doctor Diagnostic".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Xavier HTTP Endpoint".to_string(),
                    value: settings.xavier_api_url.clone(),
                    unit: None,
                    trend: None,
                    color: Some("#06b6d4".to_string()),
                },
                ComponentNode::MetricPill {
                    label: "Xavier MCP Port".to_string(),
                    value: settings.xavier_mcp_port.to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#3b82f6".to_string()),
                },
                ComponentNode::Button {
                    label: "Run Xavier Doctor Diagnostic".to_string(),
                    action: "xavier.run_doctor".to_string(),
                    variant: Some("primary".to_string()),
                },
            ],
        }
    }

    fn build_keybinds_panel(settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("⌨️ Keyboard Shortcuts".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Keybindings Preset".to_string(),
                    value: settings.default_keybinds_preset.clone(),
                    unit: None,
                    trend: None,
                    color: Some("#ec4899".to_string()),
                },
                ComponentNode::Button {
                    label: "Super + Space: Hermes Ambient Orb".to_string(),
                    action: "keybind.view:orb".to_string(),
                    variant: Some("subtle".to_string()),
                },
                ComponentNode::Button {
                    label: "Super + E: SWAL Files".to_string(),
                    action: "keybind.view:files".to_string(),
                    variant: Some("subtle".to_string()),
                },
            ],
        }
    }

    fn build_audio_panel(settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("🔊 Sound & PipeWire Settings".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "Output Device".to_string(),
                    value: settings.audio_output_device.clone(),
                    unit: None,
                    trend: None,
                    color: Some("#6366f1".to_string()),
                },
                ComponentNode::MetricPill {
                    label: "Master Volume".to_string(),
                    value: format!("{}%", settings.audio_volume),
                    unit: None,
                    trend: None,
                    color: Some("#10b981".to_string()),
                },
                ComponentNode::Button {
                    label: "Mute Output".to_string(),
                    action: "audio.mute".to_string(),
                    variant: Some("subtle".to_string()),
                },
            ],
        }
    }

    fn build_about_panel(settings: &SwalSystemSettings) -> ComponentNode {
        ComponentNode::Card {
            title: Some("ℹ️ About SWAL AI Workspace".to_string()),
            elevation: Some("flat".to_string()),
            children: vec![
                ComponentNode::MetricPill {
                    label: "System Release".to_string(),
                    value: settings.system_version.clone(),
                    unit: None,
                    trend: None,
                    color: Some("#a855f7".to_string()),
                },
                ComponentNode::MetricPill {
                    label: "Architecture".to_string(),
                    value: "x86_64-linux (NixOS + Hyprland + Niri)".to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#3b82f6".to_string()),
                },
                ComponentNode::Button {
                    label: "Check for SWAL System Updates".to_string(),
                    action: "system.check_updates".to_string(),
                    variant: Some("primary".to_string()),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_9_categories_layout_generation() {
        let settings = SwalSystemSettings::default();

        for category in SettingsCategory::all() {
            let layout = SettingsWindowBuilder::build_settings_layout(category, &settings);
            match layout {
                ComponentNode::Card {
                    ref title,
                    ref children,
                    ..
                } => {
                    assert!(
                        title.as_ref().unwrap().contains(category.label()),
                        "Title should contain category label for {:?}",
                        category
                    );
                    assert_eq!(children.len(), 1, "Expected 1 root grid child");

                    if let ComponentNode::Grid { ref children, .. } = children[0] {
                        assert_eq!(children.len(), 2, "Grid should have 2 columns (sidebar & content)");
                    } else {
                        panic!("Expected Grid component in root Card");
                    }
                }
                _ => panic!("Expected root Card component"),
            }
        }
    }

    #[test]
    fn test_active_category_highlighting() {
        let settings = SwalSystemSettings::default();
        let target_category = SettingsCategory::Appearance;

        let layout = SettingsWindowBuilder::build_settings_layout(target_category, &settings);

        if let ComponentNode::Card { ref children, .. } = layout {
            if let ComponentNode::Grid {
                ref children, ..
            } = children[0]
            {
                let sidebar = &children[0];
                if let ComponentNode::Card {
                    children: ref nav_buttons,
                    ..
                } = sidebar
                {
                    assert_eq!(nav_buttons.len(), 9, "Sidebar should have 9 category buttons");

                    for (idx, cat) in SettingsCategory::all().iter().enumerate() {
                        if let ComponentNode::Button {
                            ref label,
                            ref variant,
                            ..
                        } = nav_buttons[idx]
                        {
                            if *cat == target_category {
                                assert_eq!(
                                    variant.as_deref(),
                                    Some("primary"),
                                    "Active category button variant must be primary"
                                );
                                assert!(
                                    label.starts_with("▶"),
                                    "Active category button label must feature selection indicator ▶"
                                );
                            } else {
                                assert_eq!(
                                    variant.as_deref(),
                                    Some("ghost"),
                                    "Inactive category button variant must be ghost"
                                );
                                assert!(
                                    label.starts_with("  "),
                                    "Inactive category button label should start with spacing"
                                );
                            }
                        } else {
                            panic!("Expected Button node in sidebar");
                        }
                    }
                } else {
                    panic!("Expected sidebar Card");
                }
            }
        }
    }

    #[test]
    fn test_settings_group_content_for_each_category() {
        let settings = SwalSystemSettings::default();

        // 1. Xavier category doctor diagnostic button test
        let xavier_layout = SettingsWindowBuilder::build_settings_layout(SettingsCategory::Xavier, &settings);
        let xavier_json = serde_json::to_string(&xavier_layout).expect("Serialize xavier layout");
        assert!(
            xavier_json.contains("Run Xavier Doctor Diagnostic"),
            "Xavier category must contain doctor diagnostic button"
        );
        assert!(
            xavier_json.contains("xavier.run_doctor"),
            "Xavier category must contain xavier.run_doctor action"
        );

        // 2. Appearance theme picker test
        let app_layout = SettingsWindowBuilder::build_settings_layout(SettingsCategory::Appearance, &settings);
        let app_json = serde_json::to_string(&app_layout).expect("Serialize appearance layout");
        assert!(
            app_json.contains("Fluent Glass"),
            "Appearance category must contain Fluent Glass theme"
        );
        assert!(
            app_json.contains("Corner Radius & Acrylic Opacity Sliders"),
            "Appearance category must contain corner radius and opacity group"
        );

        // 3. Storage category test
        let storage_layout = SettingsWindowBuilder::build_settings_layout(SettingsCategory::Storage, &settings);
        let storage_json = serde_json::to_string(&storage_layout).expect("Serialize storage layout");
        assert!(
            storage_json.contains("Run Storage Cleanup"),
            "Storage category must contain cleanup button"
        );

        // 4. Agents category test
        let agents_layout = SettingsWindowBuilder::build_settings_layout(SettingsCategory::Agents, &settings);
        let agents_json = serde_json::to_string(&agents_layout).expect("Serialize agents layout");
        assert!(
            agents_json.contains("AI Agents Active"),
            "Agents category must contain agent status"
        );
    }
}
