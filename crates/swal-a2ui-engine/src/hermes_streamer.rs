//! Hermes Direct A2UI Component Streamer
//!
//! Provides high-level streamer builder to emit incremental A2UI fragments
//! (Cards, ProgressSteps, StatBadges, ActionButtons) as well as Eww Yuck snippets.

use crate::{ComponentNode, WidgetDefinition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentStep {
    pub label: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ActionButton {
    pub label: String,
    pub callback_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentMetric {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HermesA2uiStreamer {
    pub title: String,
    pub summary: String,
    pub steps: Vec<AgentStep>,
    pub action_buttons: Vec<ActionButton>,
    pub metrics: Vec<AgentMetric>,
}

impl HermesA2uiStreamer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_agent_card(title: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            summary: summary.into(),
            ..Default::default()
        }
    }

    pub fn add_step(mut self, label: impl Into<String>, status: impl Into<String>) -> Self {
        self.steps.push(AgentStep {
            label: label.into(),
            status: status.into(),
        });
        self
    }

    pub fn push_step(&mut self, label: impl Into<String>, status: impl Into<String>) -> &mut Self {
        self.steps.push(AgentStep {
            label: label.into(),
            status: status.into(),
        });
        self
    }

    pub fn add_action_button(
        mut self,
        label: impl Into<String>,
        callback_cmd: impl Into<String>,
    ) -> Self {
        self.action_buttons.push(ActionButton {
            label: label.into(),
            callback_cmd: callback_cmd.into(),
        });
        self
    }

    pub fn push_action_button(
        &mut self,
        label: impl Into<String>,
        callback_cmd: impl Into<String>,
    ) -> &mut Self {
        self.action_buttons.push(ActionButton {
            label: label.into(),
            callback_cmd: callback_cmd.into(),
        });
        self
    }

    pub fn add_metric(
        mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        let u_str = unit.into();
        let unit_opt = if u_str.trim().is_empty() {
            None
        } else {
            Some(u_str)
        };
        self.metrics.push(AgentMetric {
            label: label.into(),
            value: value.into(),
            unit: unit_opt,
        });
        self
    }

    pub fn push_metric(
        &mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        unit: impl Into<String>,
    ) -> &mut Self {
        let u_str = unit.into();
        let unit_opt = if u_str.trim().is_empty() {
            None
        } else {
            Some(u_str)
        };
        self.metrics.push(AgentMetric {
            label: label.into(),
            value: value.into(),
            unit: unit_opt,
        });
        self
    }

    pub fn to_widget_definition(&self) -> WidgetDefinition {
        let mut children = Vec::new();

        if !self.summary.is_empty() {
            children.push(ComponentNode::StatusBadge {
                status: "info".to_string(),
                label: self.summary.clone(),
                color: None,
            });
        }

        for step in &self.steps {
            children.push(ComponentNode::StatusBadge {
                status: step.status.clone(),
                label: step.label.clone(),
                color: None,
            });
        }

        if !self.metrics.is_empty() {
            let metric_nodes: Vec<ComponentNode> = self
                .metrics
                .iter()
                .map(|m| ComponentNode::MetricPill {
                    label: m.label.clone(),
                    value: m.value.clone(),
                    unit: m.unit.clone(),
                    trend: None,
                    color: None,
                })
                .collect();

            children.push(ComponentNode::Grid {
                columns: 2,
                gap: 4,
                children: metric_nodes,
            });
        }

        for btn in &self.action_buttons {
            children.push(ComponentNode::Button {
                label: btn.label.clone(),
                action: btn.callback_cmd.clone(),
                variant: None,
            });
        }

        let root = ComponentNode::Card {
            title: if self.title.is_empty() {
                None
            } else {
                Some(self.title.clone())
            },
            elevation: Some("elevated".to_string()),
            children,
        };

        WidgetDefinition {
            schema: "https://swal.dev/schemas/a2ui.v1.json".to_string(),
            title: if self.title.is_empty() {
                "Hermes Fragment".to_string()
            } else {
                self.title.clone()
            },
            root,
        }
    }

    pub fn to_json_fragment(&self) -> String {
        let widget = self.to_widget_definition();
        serde_json::to_string_pretty(&widget).unwrap_or_default()
    }

    pub fn to_eww_yuck_snippet(&self) -> String {
        let mut yuck = String::new();
        yuck.push_str("(box :class \"hermes-agent-card\" :orientation \"v\" :spacing 4 :space-evenly false\n");

        if !self.title.is_empty() || !self.summary.is_empty() {
            yuck.push_str("  (box :class \"hermes-card-header\" :orientation \"h\" :space-evenly false\n");
            if !self.title.is_empty() {
                yuck.push_str(&format!(
                    "    (label :class \"hermes-card-title\" :text \"{}\" :xalign 0)\n",
                    self.title
                ));
            }
            if !self.summary.is_empty() {
                yuck.push_str(&format!(
                    "    (label :class \"hermes-card-summary\" :text \"{}\" :xalign 1)\n",
                    self.summary
                ));
            }
            yuck.push_str("  )\n");
        }

        if !self.steps.is_empty() {
            yuck.push_str("  (box :class \"hermes-steps\" :orientation \"v\" :spacing 2 :space-evenly false\n");
            for step in &self.steps {
                yuck.push_str(&format!(
                    "    (box :class \"hermes-step-item\" :orientation \"h\" :space-evenly false\n      (label :class \"hermes-step-label\" :text \"{}\")\n      (label :class \"hermes-step-status badge {}\" :text \"{}\"))\n",
                    step.label, step.status, step.status
                ));
            }
            yuck.push_str("  )\n");
        }

        if !self.metrics.is_empty() {
            yuck.push_str("  (box :class \"hermes-metrics\" :orientation \"h\" :spacing 4 :space-evenly false\n");
            for m in &self.metrics {
                let val_str = match &m.unit {
                    Some(u) if !u.is_empty() => format!("{} {}", m.value, u),
                    _ => m.value.clone(),
                };
                yuck.push_str(&format!(
                    "    (box :class \"hermes-metric-pill\" :orientation \"h\" :spacing 2\n      (label :class \"metric-label\" :text \"{}:\")\n      (label :class \"metric-value\" :text \"{}\"))\n",
                    m.label, val_str
                ));
            }
            yuck.push_str("  )\n");
        }

        if !self.action_buttons.is_empty() {
            yuck.push_str("  (box :class \"hermes-actions\" :orientation \"h\" :spacing 4 :space-evenly false\n");
            for btn in &self.action_buttons {
                yuck.push_str(&format!(
                    "    (button :class \"act-btn\" :onclick \"{}\"\n      (label :text \"{}\"))\n",
                    btn.callback_cmd, btn.label
                ));
            }
            yuck.push_str("  )\n");
        }

        yuck.push_str(")");
        yuck
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_widget_json;

    #[test]
    fn test_hermes_streamer_builder() {
        let streamer = HermesA2uiStreamer::create_agent_card("Hermes Orb Streamer", "Processing file tree")
            .add_step("Initialize Agent Core", "completed")
            .add_step("Parse A2UI AST", "running")
            .add_metric("CPU Load", "14.2", "%")
            .add_metric("RAM Usage", "3.8", "GB")
            .add_action_button("Cancel Operation", "hermes.cancel");

        assert_eq!(streamer.title, "Hermes Orb Streamer");
        assert_eq!(streamer.summary, "Processing file tree");
        assert_eq!(streamer.steps.len(), 2);
        assert_eq!(streamer.metrics.len(), 2);
        assert_eq!(streamer.action_buttons.len(), 1);
    }

    #[test]
    fn test_to_json_fragment_validates() {
        let streamer = HermesA2uiStreamer::create_agent_card("Hermes Task", "Executing wave 3")
            .add_step("Step 1", "ok")
            .add_metric("Progress", "50", "%")
            .add_action_button("Pause", "hermes.pause");

        let json = streamer.to_json_fragment();
        assert!(!json.is_empty());

        let validated = validate_widget_json(&json);
        assert!(validated.is_ok(), "JSON fragment must validate against A2UI schema");
    }

    #[test]
    fn test_to_eww_yuck_snippet() {
        let streamer = HermesA2uiStreamer::create_agent_card("Eww Snippet Test", "Building yuck UI")
            .add_step("Generate Yuck", "active")
            .add_metric("VRAM", "2.1", "GB")
            .add_action_button("Refresh", "eww reload");

        let yuck = streamer.to_eww_yuck_snippet();
        assert!(yuck.contains("(box :class \"hermes-agent-card\""));
        assert!(yuck.contains("Eww Snippet Test"));
        assert!(yuck.contains("Generate Yuck"));
        assert!(yuck.contains("2.1 GB"));
        assert!(yuck.contains("eww reload"));
    }
}
