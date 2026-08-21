use serde::{Deserialize, Serialize};
use crate::schema::ThemePalette;

/// Rich toggle switch component node for settings panels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsToggle {
    pub label: String,
    pub key: String,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

impl SettingsToggle {
    pub fn new(label: impl Into<String>, key: impl Into<String>, enabled: bool) -> Self {
        Self {
            label: label.into(),
            key: key.into(),
            enabled,
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

/// Rich slider component node for numeric preference values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsSlider {
    pub label: String,
    pub key: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub unit: String,
}

impl SettingsSlider {
    pub fn new(
        label: impl Into<String>,
        key: impl Into<String>,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        unit: impl Into<String>,
    ) -> Self {
        let mut slider = Self {
            label: label.into(),
            key: key.into(),
            value,
            min,
            max,
            step,
            unit: unit.into(),
        };
        slider.clamp_value();
        slider
    }

    pub fn clamp_value(&mut self) {
        if self.min > self.max {
            std::mem::swap(&mut self.min, &mut self.max);
        }
        self.value = self.value.clamp(self.min, self.max);
    }

    pub fn set_value(&mut self, new_val: f32) {
        self.value = new_val;
        self.clamp_value();
    }
}

/// Option dropdown / select component node for settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsSelect {
    pub label: String,
    pub key: String,
    pub selected: String,
    pub options: Vec<String>,
}

impl SettingsSelect {
    pub fn new(
        label: impl Into<String>,
        key: impl Into<String>,
        selected: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        Self {
            label: label.into(),
            key: key.into(),
            selected: selected.into(),
            options,
        }
    }
}

/// Color swatch palette picker component node for theme / accent settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsColorSwatch {
    pub label: String,
    pub key: String,
    pub selected_hex: String,
    pub swatches: Vec<String>,
}

impl SettingsColorSwatch {
    pub fn new(
        label: impl Into<String>,
        key: impl Into<String>,
        selected_hex: impl Into<String>,
        swatches: Vec<String>,
    ) -> Self {
        Self {
            label: label.into(),
            key: key.into(),
            selected_hex: selected_hex.into(),
            swatches,
        }
    }

    pub fn resolve_tokens(&mut self, palette: &ThemePalette) {
        self.selected_hex = palette.resolve_token(&self.selected_hex);
        for swatch in &mut self.swatches {
            *swatch = palette.resolve_token(swatch);
        }
    }
}

/// Enum wrapping individual settings item component variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SettingsItemNode {
    Toggle(SettingsToggle),
    Slider(SettingsSlider),
    Select(SettingsSelect),
    ColorSwatch(SettingsColorSwatch),
}

impl SettingsItemNode {
    pub fn resolve_tokens(&mut self, palette: &ThemePalette) {
        if let SettingsItemNode::ColorSwatch(swatch) = self {
            swatch.resolve_tokens(palette);
        }
    }
}

/// Group node organizing related settings component items into labeled preferences sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsGroupNode {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub items: Vec<SettingsItemNode>,
}

impl SettingsGroupNode {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            items: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_item(&mut self, item: SettingsItemNode) {
        self.items.push(item);
    }

    pub fn resolve_tokens(&mut self, palette: &ThemePalette) {
        for item in &mut self.items {
            item.resolve_tokens(palette);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_toggle_serialization_and_toggle() {
        let mut toggle = SettingsToggle::new("Dark Mode", "sys.dark_mode", true)
            .with_description("Enable dark mode UI theme");

        let json = serde_json::to_string(&toggle).expect("Must serialize toggle");
        let deserialized: SettingsToggle = serde_json::from_str(&json).expect("Must deserialize toggle");
        assert_eq!(toggle, deserialized);
        assert_eq!(deserialized.description.as_deref(), Some("Enable dark mode UI theme"));

        toggle.toggle();
        assert!(!toggle.enabled);
        toggle.toggle();
        assert!(toggle.enabled);
    }

    #[test]
    fn test_settings_slider_clamping() {
        let mut slider = SettingsSlider::new("Volume", "audio.volume", 150.0, 0.0, 100.0, 1.0, "%");
        assert_eq!(slider.value, 100.0);

        slider.set_value(-20.0);
        assert_eq!(slider.value, 0.0);

        slider.set_value(50.0);
        assert_eq!(slider.value, 50.0);

        let json = serde_json::to_string(&slider).expect("Must serialize slider");
        let deserialized: SettingsSlider = serde_json::from_str(&json).expect("Must deserialize slider");
        assert_eq!(slider, deserialized);
    }

    #[test]
    fn test_settings_select_serialization() {
        let options = vec!["Hyprland".to_string(), "Niri".to_string(), "Sway".to_string()];
        let select = SettingsSelect::new("Compositor", "sys.wm", "Niri", options);

        let json = serde_json::to_string(&select).expect("Must serialize select");
        let deserialized: SettingsSelect = serde_json::from_str(&json).expect("Must deserialize select");
        assert_eq!(select, deserialized);
        assert_eq!(deserialized.selected, "Niri");
    }

    #[test]
    fn test_settings_color_swatch_token_resolution() {
        let swatches = vec!["$accent_primary".to_string(), "$accent_secondary".to_string(), "#ff0000".to_string()];
        let mut color_swatch = SettingsColorSwatch::new("Accent Color", "theme.accent", "$accent_primary", swatches);

        let palette = ThemePalette::hive_dark();
        color_swatch.resolve_tokens(&palette);

        assert_eq!(color_swatch.selected_hex, palette.accent_primary);
        assert_eq!(color_swatch.swatches[0], palette.accent_primary);
        assert_eq!(color_swatch.swatches[1], palette.accent_secondary);
        assert_eq!(color_swatch.swatches[2], "#ff0000");
    }

    #[test]
    fn test_settings_group_node_serialization_and_token_resolution() {
        let mut group = SettingsGroupNode::new("Appearance")
            .with_description("Configure visual theme and display options");

        let toggle_item = SettingsItemNode::Toggle(
            SettingsToggle::new("Mica Effects", "theme.mica", true)
        );
        let slider_item = SettingsItemNode::Slider(
            SettingsSlider::new("Opacity", "theme.opacity", 0.85, 0.1, 1.0, 0.05, "")
        );
        let select_item = SettingsItemNode::Select(
            SettingsSelect::new("Theme", "theme.name", "hive-dark", vec!["hive-dark".into(), "cyber-neon".into()])
        );
        let swatch_item = SettingsItemNode::ColorSwatch(
            SettingsColorSwatch::new("Accent", "theme.accent", "$accent_primary", vec!["$accent_primary".into()])
        );

        group.add_item(toggle_item);
        group.add_item(slider_item);
        group.add_item(select_item);
        group.add_item(swatch_item);

        let json = serde_json::to_string(&group).expect("Must serialize group node");
        let deserialized: SettingsGroupNode = serde_json::from_str(&json).expect("Must deserialize group node");
        assert_eq!(group, deserialized);
        assert_eq!(deserialized.items.len(), 4);

        let palette = ThemePalette::hive_dark();
        group.resolve_tokens(&palette);

        if let SettingsItemNode::ColorSwatch(ref s) = group.items[3] {
            assert_eq!(s.selected_hex, palette.accent_primary);
        } else {
            panic!("Expected ColorSwatch item");
        }
    }
}
