//! Generative AUI Agent Action Card & Dynamic Response Streamer
//!
//! Pairs agent cognitive thoughts with dynamic actionable UI controls,
//! including metric impact previews, diff views, and action callbacks.

use serde::{Deserialize, Serialize};

/// Status of the agent action card execution cycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentActionStatus {
    Pending,
    Ready,
    Executed,
    RolledBack,
    Failed,
}

impl Default for AgentActionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Visual metric impact comparison (before and after state).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetricImpact {
    pub label: String,
    pub before: String,
    pub after: String,
}

impl MetricImpact {
    pub fn new(label: impl Into<String>, before: impl Into<String>, after: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            before: before.into(),
            after: after.into(),
        }
    }
}

/// Style variant for action buttons.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionButtonVariant {
    Primary,
    Destructive,
    Subtle,
}

impl Default for ActionButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

/// Actionable UI button control with structured callback payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionButton {
    pub id: String,
    pub label: String,
    pub action_payload: String,
    pub variant: ActionButtonVariant,
}

impl ActionButton {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        action_payload: impl Into<String>,
        variant: ActionButtonVariant,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            action_payload: action_payload.into(),
            variant,
        }
    }

    /// Parse the action payload as structured JSON into the requested type `T`.
    pub fn parse_payload_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.action_payload)
    }
}

/// Actionable card combining agent thoughts with dynamic UI controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AgentActionCard {
    pub agent_id: String,
    pub task_id: String,
    pub thought_summary: String,
    pub status: AgentActionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_impact: Option<MetricImpact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_buttons: Vec<ActionButton>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_preview: Option<String>,
}

impl AgentActionCard {
    /// Create a new AgentActionCard given an agent ID and cognitive thought summary.
    pub fn new(agent_id: impl Into<String>, thought: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            thought_summary: thought.into(),
            ..Default::default()
        }
    }

    /// Set the task ID associated with this action card.
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = task_id.into();
        self
    }

    /// Set the current execution status of the card.
    pub fn with_status(mut self, status: AgentActionStatus) -> Self {
        self.status = status;
        self
    }

    /// Attach a metric impact comparison (e.g. RAM released before vs after).
    pub fn with_metric_impact(
        mut self,
        label: impl Into<String>,
        before: impl Into<String>,
        after: impl Into<String>,
    ) -> Self {
        self.metric_impact = Some(MetricImpact::new(label, before, after));
        self
    }

    /// Add an actionable button control to the card.
    pub fn add_action_button(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        action_payload: impl Into<String>,
        variant: ActionButtonVariant,
    ) -> Self {
        self.action_buttons
            .push(ActionButton::new(id, label, action_payload, variant));
        self
    }

    /// Attach a code or text diff preview string.
    pub fn with_diff_preview(mut self, diff: impl Into<String>) -> Self {
        self.diff_preview = Some(diff.into());
        self
    }

    /// Serialize this action card to a formatted JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Deserialize an AgentActionCard from a JSON string slice.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_builder() {
        let card = AgentActionCard::new("hermes-01", "Optimizing system memory allocation")
            .with_task_id("task-102")
            .with_status(AgentActionStatus::Ready)
            .with_metric_impact("RAM Liberada", "8.2 GB", "4.1 GB")
            .with_diff_preview("--- cache.old\n+++ cache.new\n- reserved: 8GB\n+ reserved: 4GB")
            .add_action_button(
                "btn-exec",
                "Liberar Memoria",
                r#"{"cmd":"sys.flush_cache","force":true}"#,
                ActionButtonVariant::Primary,
            )
            .add_action_button(
                "btn-rollback",
                "Revertir",
                r#"{"cmd":"sys.rollback","task_id":"task-102"}"#,
                ActionButtonVariant::Subtle,
            );

        assert_eq!(card.agent_id, "hermes-01");
        assert_eq!(card.task_id, "task-102");
        assert_eq!(card.thought_summary, "Optimizing system memory allocation");
        assert_eq!(card.status, AgentActionStatus::Ready);

        let metric = card.metric_impact.as_ref().unwrap();
        assert_eq!(metric.label, "RAM Liberada");
        assert_eq!(metric.before, "8.2 GB");
        assert_eq!(metric.after, "4.1 GB");

        assert_eq!(card.action_buttons.len(), 2);
        assert_eq!(card.action_buttons[0].variant, ActionButtonVariant::Primary);
        assert_eq!(card.action_buttons[1].variant, ActionButtonVariant::Subtle);

        assert!(card.diff_preview.as_ref().unwrap().contains("reserved: 4GB"));
    }

    #[test]
    fn test_json_roundtrip() {
        let card = AgentActionCard::new("xavier-core", "Pruning dead background processes")
            .with_task_id("task-500")
            .with_status(AgentActionStatus::Executed)
            .with_metric_impact("Procesos Activos", "142", "88")
            .add_action_button(
                "btn-undo",
                "Undo Prune",
                r#"{"action":"undo"}"#,
                ActionButtonVariant::Destructive,
            );

        let json_str = card.to_json();
        assert!(!json_str.is_empty());

        let deserialized = AgentActionCard::from_json(&json_str).expect("Must parse serialized card JSON");
        assert_eq!(card, deserialized);
    }

    #[test]
    fn test_action_button_payload_parsing() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct CmdPayload {
            cmd: String,
            force: bool,
            count: u32,
        }

        let button = ActionButton::new(
            "btn-clean",
            "Clean Cache",
            r#"{"cmd":"purge","force":true,"count":5}"#,
            ActionButtonVariant::Primary,
        );

        let parsed: CmdPayload = button
            .parse_payload_json()
            .expect("Must parse button payload JSON");

        assert_eq!(
            parsed,
            CmdPayload {
                cmd: "purge".to_string(),
                force: true,
                count: 5,
            }
        );
    }
}
