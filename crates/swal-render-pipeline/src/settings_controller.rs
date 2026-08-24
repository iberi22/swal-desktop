//! Interactive Settings Hit-Testing & Value Mutation Controller
//! Handles click hit-testing and dragging state transitions for system settings controls
//! (toggles, sliders, option selectors, swatch color pickers).

use serde::{Deserialize, Serialize};

/// Represents a rectangular bounding box for UI layout element hit-testing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

/// The specific interactive control type associated with a hit box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlKind {
    Toggle { current: bool },
    Slider { min: f32, max: f32, current: f32 },
    OptionSelect { options: Vec<String>, selected: String },
    Swatch { hex: String },
}

/// An interactive hit box representing a settings control.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractiveControlHitBox {
    pub key: String,
    pub bounds: LayoutRect,
    pub kind: ControlKind,
}

impl InteractiveControlHitBox {
    pub fn new(key: impl Into<String>, bounds: LayoutRect, kind: ControlKind) -> Self {
        Self {
            key: key.into(),
            bounds,
            kind,
        }
    }
}

/// Events representing mutated setting values triggered by user interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingMutationEvent {
    ToggleSwitched { key: String, new_value: bool },
    SliderChanged { key: String, new_value: f32 },
    OptionSelected { key: String, selected: String },
    SwatchPicked { key: String, hex: String },
}

/// Manages interactive state transitions for settings controls,
/// including slider drag tracking and click hit-testing.
#[derive(Debug, Default, Clone)]
pub struct SettingsInteractionController {
    pub active_slider_drag: Option<(String, LayoutRect)>,
}

impl SettingsInteractionController {
    pub fn new() -> Self {
        Self {
            active_slider_drag: None,
        }
    }

    /// Evaluates a click event against a list of interactive control hit boxes.
    /// Returns a `SettingMutationEvent` if an interactive element was clicked.
    pub fn handle_click(
        &mut self,
        x: f64,
        y: f64,
        hitboxes: &[InteractiveControlHitBox],
    ) -> Option<SettingMutationEvent> {
        for hitbox in hitboxes {
            if hitbox.bounds.contains(x, y) {
                match &hitbox.kind {
                    ControlKind::Toggle { current } => {
                        return Some(SettingMutationEvent::ToggleSwitched {
                            key: hitbox.key.clone(),
                            new_value: !current,
                        });
                    }
                    ControlKind::Slider { .. } => {
                        self.active_slider_drag = Some((hitbox.key.clone(), hitbox.bounds));
                        let ratio = if hitbox.bounds.width > 0.0 {
                            ((x - hitbox.bounds.x) / hitbox.bounds.width).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        return Some(SettingMutationEvent::SliderChanged {
                            key: hitbox.key.clone(),
                            new_value: ratio as f32,
                        });
                    }
                    ControlKind::OptionSelect { options, selected } => {
                        if options.is_empty() {
                            return None;
                        }
                        let next_idx = match options.iter().position(|opt| opt == selected) {
                            Some(idx) => (idx + 1) % options.len(),
                            None => 0,
                        };
                        return Some(SettingMutationEvent::OptionSelected {
                            key: hitbox.key.clone(),
                            selected: options[next_idx].clone(),
                        });
                    }
                    ControlKind::Swatch { hex } => {
                        return Some(SettingMutationEvent::SwatchPicked {
                            key: hitbox.key.clone(),
                            hex: hex.clone(),
                        });
                    }
                }
            }
        }
        None
    }

    /// Handles active mouse drag events for sliders.
    /// Returns `SettingMutationEvent::SliderChanged` with clamped percentage if dragging.
    pub fn handle_drag(&self, x: f64, _y: f64) -> Option<SettingMutationEvent> {
        if let Some((key, bounds)) = &self.active_slider_drag {
            let ratio = if bounds.width > 0.0 {
                ((x - bounds.x) / bounds.width).clamp(0.0, 1.0)
            } else {
                0.0
            };
            Some(SettingMutationEvent::SliderChanged {
                key: key.clone(),
                new_value: ratio as f32,
            })
        } else {
            None
        }
    }

    /// Terminates any active slider dragging state.
    pub fn end_drag(&mut self) {
        self.active_slider_drag = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_to_toggle() {
        let mut controller = SettingsInteractionController::new();
        let hitboxes = vec![
            InteractiveControlHitBox::new(
                "dark_mode",
                LayoutRect::new(10.0, 10.0, 40.0, 20.0),
                ControlKind::Toggle { current: false },
            ),
        ];

        let event = controller.handle_click(15.0, 15.0, &hitboxes);
        assert_eq!(
            event,
            Some(SettingMutationEvent::ToggleSwitched {
                key: "dark_mode".to_string(),
                new_value: true,
            })
        );

        // Click outside hitbox should return None
        let miss_event = controller.handle_click(100.0, 100.0, &hitboxes);
        assert_eq!(miss_event, None);
    }

    #[test]
    fn test_drag_to_slide_percentage_calculation_and_clamping() {
        let mut controller = SettingsInteractionController::new();
        let slider_rect = LayoutRect::new(100.0, 200.0, 200.0, 20.0);
        let hitboxes = vec![
            InteractiveControlHitBox::new(
                "volume",
                slider_rect,
                ControlKind::Slider { min: 0.0, max: 100.0, current: 50.0 },
            ),
        ];

        // Click at middle (x = 200.0 -> 50%)
        let click_event = controller.handle_click(200.0, 210.0, &hitboxes);
        assert_eq!(
            click_event,
            Some(SettingMutationEvent::SliderChanged {
                key: "volume".to_string(),
                new_value: 0.5,
            })
        );
        assert!(controller.active_slider_drag.is_some());

        // Drag to x = 250.0 (75%)
        let drag_event1 = controller.handle_drag(250.0, 210.0);
        assert_eq!(
            drag_event1,
            Some(SettingMutationEvent::SliderChanged {
                key: "volume".to_string(),
                new_value: 0.75,
            })
        );

        // Drag past left bound (x = 50.0 -> clamped to 0.0)
        let drag_event_left = controller.handle_drag(50.0, 210.0);
        assert_eq!(
            drag_event_left,
            Some(SettingMutationEvent::SliderChanged {
                key: "volume".to_string(),
                new_value: 0.0,
            })
        );

        // Drag past right bound (x = 350.0 -> clamped to 1.0)
        let drag_event_right = controller.handle_drag(350.0, 210.0);
        assert_eq!(
            drag_event_right,
            Some(SettingMutationEvent::SliderChanged {
                key: "volume".to_string(),
                new_value: 1.0,
            })
        );

        // End drag
        controller.end_drag();
        assert!(controller.active_slider_drag.is_none());
        assert_eq!(controller.handle_drag(250.0, 210.0), None);
    }

    #[test]
    fn test_option_select_cycle() {
        let mut controller = SettingsInteractionController::new();
        let options = vec!["System".to_string(), "Dark".to_string(), "Light".to_string()];
        let hitboxes = vec![
            InteractiveControlHitBox::new(
                "theme_mode",
                LayoutRect::new(50.0, 50.0, 120.0, 30.0),
                ControlKind::OptionSelect {
                    options: options.clone(),
                    selected: "Dark".to_string(),
                },
            ),
        ];

        let event = controller.handle_click(60.0, 60.0, &hitboxes);
        assert_eq!(
            event,
            Some(SettingMutationEvent::OptionSelected {
                key: "theme_mode".to_string(),
                selected: "Light".to_string(),
            })
        );
    }

    #[test]
    fn test_swatch_picked() {
        let mut controller = SettingsInteractionController::new();
        let hitboxes = vec![
            InteractiveControlHitBox::new(
                "accent_color",
                LayoutRect::new(10.0, 10.0, 30.0, 30.0),
                ControlKind::Swatch { hex: "#007ACC".to_string() },
            ),
        ];

        let event = controller.handle_click(15.0, 15.0, &hitboxes);
        assert_eq!(
            event,
            Some(SettingMutationEvent::SwatchPicked {
                key: "accent_color".to_string(),
                hex: "#007ACC".to_string(),
            })
        );
    }
}
