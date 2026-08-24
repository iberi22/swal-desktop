//! swal-a2ui-engine
//! Declarative A2UI schema compiler and validator

pub mod agent_action_card;
pub mod calendar;
pub mod hermes_streamer;
pub mod native_render;
pub mod schema;
pub mod settings_components;
pub mod standalone_window;

pub use calendar::{AgendaEvent, AgendaList, CalendarGrid};
use serde::{Deserialize, Serialize};
pub use schema::{compile_widget, validate_widget_json, SchemaValidationError, ThemePalette};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetDefinition {
    pub schema: String,
    pub title: String,
    pub root: ComponentNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub content: Vec<ComponentNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ComponentNode {
    Card {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        elevation: Option<String>,
        #[serde(default)]
        children: Vec<ComponentNode>,
    },
    Grid {
        columns: usize,
        gap: usize,
        #[serde(default)]
        children: Vec<ComponentNode>,
    },
    StatusBadge {
        status: String,
        label: String,
        #[serde(default)]
        color: Option<String>,
    },
    MetricPill {
        label: String,
        value: String,
        #[serde(default)]
        unit: Option<String>,
        #[serde(default)]
        trend: Option<String>,
        #[serde(default)]
        color: Option<String>,
    },
    Button {
        label: String,
        action: String,
        #[serde(default)]
        variant: Option<String>,
    },
    LogViewer {
        source: String,
        height: u32,
        #[serde(default)]
        lines: Vec<String>,
    },
    Terminal {
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        output: Option<Vec<String>>,
        #[serde(default)]
        height: Option<u32>,
    },
    Tabs {
        #[serde(default)]
        tabs: Vec<TabItem>,
    },
    #[serde(alias = "Range")]
    Slider {
        label: String,
        min: f32,
        max: f32,
        value: f32,
        action: String,
    },
    ProcessTable {
        limit: usize,
        sort_by: String,
    },
    CalendarGrid {
        year: u32,
        month: u32,
        #[serde(default)]
        highlighted_days: Vec<u32>,
    },
    AgendaList {
        #[serde(default)]
        events: Vec<AgendaEvent>,
    },
}

impl ComponentNode {
    /// Recursively resolve color tokens on this component node and its children.
    pub fn resolve_tokens(&mut self, palette: &ThemePalette) {
        match self {
            ComponentNode::Card { children, .. } => {
                for child in children {
                    child.resolve_tokens(palette);
                }
            }
            ComponentNode::Grid { children, .. } => {
                for child in children {
                    child.resolve_tokens(palette);
                }
            }
            ComponentNode::StatusBadge { status, color, .. } => {
                if let Some(c) = color {
                    *c = palette.resolve_token(c);
                } else {
                    *color = Some(palette.resolve_token(status));
                }
            }
            ComponentNode::MetricPill { color, .. } => {
                if let Some(c) = color {
                    *c = palette.resolve_token(c);
                }
            }
            ComponentNode::Button { variant, .. } => {
                if let Some(v) = variant {
                    *v = palette.resolve_token(v);
                }
            }
            ComponentNode::LogViewer { .. } => {}
            ComponentNode::Terminal { .. } => {}
            ComponentNode::Tabs { tabs } => {
                for tab in tabs {
                    for child in &mut tab.content {
                        child.resolve_tokens(palette);
                    }
                }
            }
            ComponentNode::Slider { .. } => {}
            ComponentNode::ProcessTable { .. } => {}
            ComponentNode::CalendarGrid { .. } => {}
            ComponentNode::AgendaList { events } => {
                for event in events {
                    event.tag = palette.resolve_token(&event.tag);
                }
            }
        }
    }
}

pub fn parse_widget_json(raw_json: &str) -> Result<WidgetDefinition, serde_json::Error> {
    serde_json::from_str(raw_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_widget() {
        let json_data = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "Xavier Telemetry",
            "root": {
                "type": "Card",
                "children": [
                    { "type": "StatusBadge", "status": "healthy", "label": "Xavier Online" }
                ]
            }
        }"#;

        let widget = parse_widget_json(json_data).expect("Must parse valid widget");
        assert_eq!(widget.title, "Xavier Telemetry");
    }

    #[test]
    fn test_parse_all_catalog_components() {
        let json_data = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "Full Component Catalog Test",
            "root": {
                "type": "Card",
                "title": "Main Dashboard",
                "elevation": "elevated",
                "children": [
                    {
                        "type": "Grid",
                        "columns": 2,
                        "gap": 8,
                        "children": [
                            {
                                "type": "StatusBadge",
                                "status": "healthy",
                                "label": "Node Active",
                                "color": "$success"
                            },
                            {
                                "type": "MetricPill",
                                "label": "CPU Usage",
                                "value": "12.4",
                                "unit": "%",
                                "trend": "+1.2%",
                                "color": "$accent_primary"
                            },
                            {
                                "type": "Button",
                                "label": "Restart Node",
                                "action": "node.restart",
                                "variant": "primary"
                            },
                            {
                                "type": "LogViewer",
                                "source": "/var/log/swal.log",
                                "height": 200,
                                "lines": ["System booted", "Mesh initialized"]
                            },
                            {
                                "type": "Terminal",
                                "command": "swal-doctor --check",
                                "output": ["Checking system...", "OK"],
                                "height": 150
                            },
                            {
                                "type": "Slider",
                                "label": "Master Volume",
                                "min": 0.0,
                                "max": 100.0,
                                "value": 75.0,
                                "action": "audio.set_volume"
                            },
                            {
                                "type": "Range",
                                "label": "Brightness",
                                "min": 0.0,
                                "max": 1.0,
                                "value": 0.85,
                                "action": "display.set_brightness"
                            },
                            {
                                "type": "ProcessTable",
                                "limit": 10,
                                "sort_by": "memory"
                            }
                        ]
                    },
                    {
                        "type": "Tabs",
                        "tabs": [
                            {
                                "id": "tab-1",
                                "label": "Overview",
                                "content": [
                                    {
                                        "type": "StatusBadge",
                                        "status": "ok",
                                        "label": "All Systems Go"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }"#;

        let widget = parse_widget_json(json_data).expect("Must parse full component tree");
        assert_eq!(widget.title, "Full Component Catalog Test");

        if let ComponentNode::Card { children, .. } = widget.root {
            assert_eq!(children.len(), 2);
            if let ComponentNode::Grid { columns, children: grid_children, .. } = &children[0] {
                assert_eq!(*columns, 2);
                assert_eq!(grid_children.len(), 8);
                assert!(matches!(&grid_children[5], ComponentNode::Slider { label, min, max, value, action } if label == "Master Volume" && *min == 0.0 && *max == 100.0 && *value == 75.0 && action == "audio.set_volume"));
                assert!(matches!(&grid_children[6], ComponentNode::Slider { label, min, max, value, action } if label == "Brightness" && *min == 0.0 && *max == 1.0 && *value == 0.85 && action == "display.set_brightness"));
                assert!(matches!(&grid_children[7], ComponentNode::ProcessTable { limit, sort_by } if *limit == 10 && sort_by == "memory"));
            } else {
                panic!("Expected Grid component");
            }

            if let ComponentNode::Tabs { tabs } = &children[1] {
                assert_eq!(tabs.len(), 1);
                assert_eq!(tabs[0].label, "Overview");
            } else {
                panic!("Expected Tabs component");
            }
        } else {
            panic!("Expected Card root node");
        }
    }

    #[test]
    fn test_parse_slider_and_range() {
        let slider_json = r#"{
            "type": "Slider",
            "label": "Audio Output",
            "min": 0.0,
            "max": 100.0,
            "value": 45.5,
            "action": "pipewire.volume"
        }"#;
        let node: ComponentNode = serde_json::from_str(slider_json).expect("Must parse Slider");
        if let ComponentNode::Slider { label, min, max, value, action } = node {
            assert_eq!(label, "Audio Output");
            assert_eq!(min, 0.0);
            assert_eq!(max, 100.0);
            assert_eq!(value, 45.5);
            assert_eq!(action, "pipewire.volume");
        } else {
            panic!("Expected Slider variant");
        }

        let range_json = r#"{
            "type": "Range",
            "label": "Backlight",
            "min": 10.0,
            "max": 100.0,
            "value": 80.0,
            "action": "brightnessctl.set"
        }"#;
        let range_node: ComponentNode = serde_json::from_str(range_json).expect("Must parse Range alias");
        if let ComponentNode::Slider { label, min, max, value, action } = range_node {
            assert_eq!(label, "Backlight");
            assert_eq!(min, 10.0);
            assert_eq!(max, 100.0);
            assert_eq!(value, 80.0);
            assert_eq!(action, "brightnessctl.set");
        } else {
            panic!("Expected Slider variant for Range");
        }
    }

    #[test]
    fn test_parse_process_table() {
        let json = r#"{
            "type": "ProcessTable",
            "limit": 15,
            "sort_by": "cpu"
        }"#;
        let node: ComponentNode = serde_json::from_str(json).expect("Must parse ProcessTable");
        if let ComponentNode::ProcessTable { limit, sort_by } = node {
            assert_eq!(limit, 15);
            assert_eq!(sort_by, "cpu");
        } else {
            panic!("Expected ProcessTable variant");
        }
    }

    #[test]
    fn test_compile_widget_hive_dark_tokens() {
        let json_data = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "Hive Dark Token Resolution Test",
            "root": {
                "type": "Grid",
                "columns": 2,
                "gap": 4,
                "children": [
                    {
                        "type": "StatusBadge",
                        "status": "healthy",
                        "label": "Online"
                    },
                    {
                        "type": "MetricPill",
                        "label": "RAM",
                        "value": "4.2",
                        "unit": "GB",
                        "color": "$accent_primary"
                    }
                ]
            }
        }"#;

        let compiled = compile_widget(json_data, "hive-dark").expect("Must compile widget");
        if let ComponentNode::Grid { children, .. } = compiled.root {
            if let ComponentNode::StatusBadge { color, .. } = &children[0] {
                assert_eq!(color.as_deref(), Some("#10b981"));
            } else {
                panic!("Expected StatusBadge");
            }

            if let ComponentNode::MetricPill { color, .. } = &children[1] {
                assert_eq!(color.as_deref(), Some("#06b6d4"));
            } else {
                panic!("Expected MetricPill");
            }
        }
    }

    #[test]
    fn test_compile_widget_cyber_neon_tokens() {
        let json_data = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "Cyber Neon Token Resolution Test",
            "root": {
                "type": "MetricPill",
                "label": "VRAM",
                "value": "8.0",
                "unit": "GB",
                "color": "accent_primary"
            }
        }"#;

        let compiled = compile_widget(json_data, "cyber-neon").expect("Must compile widget");
        if let ComponentNode::MetricPill { color, .. } = compiled.root {
            assert_eq!(color.as_deref(), Some("#00ff88"));
        } else {
            panic!("Expected MetricPill");
        }
    }

    #[test]
    fn test_schema_validation_errors() {
        let empty_title = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "",
            "root": { "type": "StatusBadge", "status": "ok", "label": "Test" }
        }"#;
        assert!(matches!(validate_widget_json(empty_title), Err(SchemaValidationError::EmptyTitle)));

        let missing_schema = r#"{
            "schema": "",
            "title": "Test Title",
            "root": { "type": "StatusBadge", "status": "ok", "label": "Test" }
        }"#;
        assert!(matches!(validate_widget_json(missing_schema), Err(SchemaValidationError::MissingField(_))));

        let invalid_json = r#"{ invalid json"#;
        assert!(matches!(validate_widget_json(invalid_json), Err(SchemaValidationError::InvalidJson(_))));
    }

    #[test]
    fn test_parse_calendar_and_agenda_nodes() {
        let calendar_json = r#"{
            "type": "CalendarGrid",
            "year": 2026,
            "month": 8,
            "highlighted_days": [10, 15, 23]
        }"#;

        let cal_node: ComponentNode = serde_json::from_str(calendar_json).expect("Must parse CalendarGrid");
        if let ComponentNode::CalendarGrid { year, month, highlighted_days } = cal_node {
            assert_eq!(year, 2026);
            assert_eq!(month, 8);
            assert_eq!(highlighted_days, vec![10, 15, 23]);
        } else {
            panic!("Expected CalendarGrid component variant");
        }

        let agenda_json = r#"{
            "type": "AgendaList",
            "events": [
                {
                    "title": "Release 1.01",
                    "time": "09:00 AM",
                    "tag": "$accent_primary"
                },
                {
                    "title": "Standup",
                    "time": "10:30 AM",
                    "tag": "work"
                }
            ]
        }"#;

        let agenda_node: ComponentNode = serde_json::from_str(agenda_json).expect("Must parse AgendaList");
        if let ComponentNode::AgendaList { events } = agenda_node {
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].title, "Release 1.01");
            assert_eq!(events[0].tag, "$accent_primary");
        } else {
            panic!("Expected AgendaList component variant");
        }
    }

    #[test]
    fn test_compile_widget_agenda_token_resolution() {
        let json_data = r#"{
            "schema": "https://swal.dev/schemas/a2ui.v1.json",
            "title": "Calendar & Agenda Widget",
            "root": {
                "type": "Card",
                "title": "Daily Schedule",
                "children": [
                    {
                        "type": "CalendarGrid",
                        "year": 2026,
                        "month": 8,
                        "highlighted_days": [23]
                    },
                    {
                        "type": "AgendaList",
                        "events": [
                            {
                                "title": "Deploy SWAL",
                                "time": "12:00 PM",
                                "tag": "$accent_primary"
                            },
                            {
                                "title": "Node Maintenance",
                                "time": "03:00 PM",
                                "tag": "$danger"
                            }
                        ]
                    }
                ]
            }
        }"#;

        let compiled = compile_widget(json_data, "hive-dark").expect("Must compile widget");
        if let ComponentNode::Card { children, .. } = compiled.root {
            assert_eq!(children.len(), 2);
            if let ComponentNode::CalendarGrid { year, month, highlighted_days } = &children[0] {
                assert_eq!(*year, 2026);
                assert_eq!(*month, 8);
                assert_eq!(highlighted_days, &[23]);
            } else {
                panic!("Expected CalendarGrid");
            }

            if let ComponentNode::AgendaList { events } = &children[1] {
                assert_eq!(events.len(), 2);
                assert_eq!(events[0].tag, "#06b6d4"); // $accent_primary for hive-dark
                assert_eq!(events[1].tag, "#ef4444"); // $danger for hive-dark
            } else {
                panic!("Expected AgendaList");
            }
        } else {
            panic!("Expected Card root");
        }
    }
}
