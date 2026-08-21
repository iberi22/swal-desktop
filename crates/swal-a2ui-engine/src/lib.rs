//! swal-a2ui-engine
//! Declarative A2UI schema compiler and validator

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDefinition {
    pub schema: String,
    pub title: String,
    pub root: ComponentNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ComponentNode {
    Card { children: Vec<ComponentNode> },
    Grid { columns: usize, gap: usize, children: Vec<ComponentNode> },
    StatusBadge { status: String, label: String },
    Button { label: String, action: String },
    LogViewer { source: String, height: u32 },
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
}
