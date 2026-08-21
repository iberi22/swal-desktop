use crate::{parse_widget_json, WidgetDefinition};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemePalette {
    pub id: String,
    pub bg: String,
    pub elevated: String,
    pub elevated_850: String,
    pub void: String,
    pub accent_primary: String,
    pub accent_secondary: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub border_active: String,
    pub border_subtle: String,
}

impl ThemePalette {
    pub fn hive_dark() -> Self {
        Self {
            id: "hive-dark".to_string(),
            bg: "rgba(2, 6, 23, 0.97)".to_string(),
            elevated: "rgba(15, 23, 42, 0.85)".to_string(),
            elevated_850: "rgba(21, 30, 46, 0.90)".to_string(),
            void: "#000000".to_string(),
            accent_primary: "#06b6d4".to_string(),
            accent_secondary: "#f97316".to_string(),
            text_primary: "#f1f5f9".to_string(),
            text_secondary: "#94a3b8".to_string(),
            success: "#10b981".to_string(),
            warning: "#f59e0b".to_string(),
            danger: "#ef4444".to_string(),
            border_active: "rgba(6, 182, 212, 0.40)".to_string(),
            border_subtle: "rgba(255, 255, 255, 0.08)".to_string(),
        }
    }

    pub fn cyber_neon() -> Self {
        Self {
            id: "cyber-neon".to_string(),
            bg: "rgba(13, 17, 23, 0.97)".to_string(),
            elevated: "rgba(22, 27, 34, 0.85)".to_string(),
            elevated_850: "rgba(33, 38, 45, 0.90)".to_string(),
            void: "#0a0e12".to_string(),
            accent_primary: "#00ff88".to_string(),
            accent_secondary: "#00ccff".to_string(),
            text_primary: "#e6edf3".to_string(),
            text_secondary: "#8b949e".to_string(),
            success: "#00ff88".to_string(),
            warning: "#e3b341".to_string(),
            danger: "#f85149".to_string(),
            border_active: "rgba(0, 255, 136, 0.40)".to_string(),
            border_subtle: "rgba(255, 255, 255, 0.07)".to_string(),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "cyber-neon" | "cyber_neon" => Self::cyber_neon(),
            _ => Self::hive_dark(),
        }
    }

    pub fn resolve_token(&self, token: &str) -> String {
        let clean = token.strip_prefix('$').unwrap_or(token);
        match clean {
            "bg" => self.bg.clone(),
            "elevated" => self.elevated.clone(),
            "elevated_850" => self.elevated_850.clone(),
            "void" => self.void.clone(),
            "accent_primary" | "primary" => self.accent_primary.clone(),
            "accent_secondary" | "secondary" => self.accent_secondary.clone(),
            "text_primary" => self.text_primary.clone(),
            "text_secondary" => self.text_secondary.clone(),
            "success" | "healthy" | "ok" => self.success.clone(),
            "warning" | "warn" => self.warning.clone(),
            "danger" | "error" | "critical" => self.danger.clone(),
            "border_active" => self.border_active.clone(),
            "border_subtle" => self.border_subtle.clone(),
            _ => token.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    InvalidJson(String),
    MissingField(String),
    EmptyTitle,
    InvalidSchemaUri(String),
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaValidationError::InvalidJson(e) => write!(f, "Invalid JSON: {}", e),
            SchemaValidationError::MissingField(field) => {
                write!(f, "Missing required schema field: {}", field)
            }
            SchemaValidationError::EmptyTitle => write!(f, "Widget title cannot be empty"),
            SchemaValidationError::InvalidSchemaUri(uri) => {
                write!(f, "Invalid schema URI: {}", uri)
            }
        }
    }
}

impl std::error::Error for SchemaValidationError {}

pub fn validate_widget_json(raw_json: &str) -> Result<WidgetDefinition, SchemaValidationError> {
    let widget: WidgetDefinition =
        parse_widget_json(raw_json).map_err(|e| SchemaValidationError::InvalidJson(e.to_string()))?;

    if widget.schema.is_empty() {
        return Err(SchemaValidationError::MissingField("schema".to_string()));
    }

    if widget.title.trim().is_empty() {
        return Err(SchemaValidationError::EmptyTitle);
    }

    Ok(widget)
}

pub fn compile_widget(
    raw_json: &str,
    theme_name: &str,
) -> Result<WidgetDefinition, SchemaValidationError> {
    let mut widget = validate_widget_json(raw_json)?;
    let palette = ThemePalette::from_name(theme_name);
    widget.root.resolve_tokens(&palette);
    Ok(widget)
}
