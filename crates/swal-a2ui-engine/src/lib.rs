//! swal-a2ui-engine
//! Declarative A2UI schema compiler and validator

pub mod hermes_streamer;
pub mod native_render;
pub mod schema;


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
                assert_eq!(grid_children.len(), 5);
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
}
