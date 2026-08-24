//! GPU Rasterizer Extension for Interactive Settings Controls
//!
//! Provides `ControlsRenderer` to generate high-refresh GPU draw commands
//! for interactive UI widgets including toggle switches, sliders, and color swatches.

use swal_a2ui_engine::native_render::{GpuDrawCommand, LayoutRect};

/// GPU rasterizer for interactive settings UI controls.
pub struct ControlsRenderer;

impl ControlsRenderer {
    /// Generates GPU draw commands for an interactive toggle switch track and thumb.
    pub fn generate_toggle_draw_commands(
        bounds: LayoutRect,
        enabled: bool,
        thumb_pos: f32,
        accent_color: [f32; 4],
    ) -> Vec<GpuDrawCommand> {
        let track_color = if enabled {
            [accent_color[0], accent_color[1], accent_color[2], 0.85]
        } else {
            [0.2, 0.22, 0.28, 0.6]
        };

        let track_cmd = GpuDrawCommand::DrawMicaCard {
            bounds,
            bg_color: track_color,
            border_color: [1.0, 1.0, 1.0, 0.12],
            radius: bounds.height / 2.0,
        };

        let thumb_bounds = Self::calculate_toggle_thumb_bounds(bounds, thumb_pos);
        let thumb_cmd = GpuDrawCommand::DrawMicaCard {
            bounds: thumb_bounds,
            bg_color: [1.0, 1.0, 1.0, 1.0],
            border_color: [1.0, 1.0, 1.0, 0.25],
            radius: thumb_bounds.height / 2.0,
        };

        vec![track_cmd, thumb_cmd]
    }

    /// Generates GPU draw commands for a slider track, progress fill, and thumb.
    pub fn generate_slider_draw_commands(
        bounds: LayoutRect,
        progress: f32,
        accent_color: [f32; 4],
    ) -> Vec<GpuDrawCommand> {
        let norm_progress = progress.clamp(0.0, 1.0);

        let track_cmd = GpuDrawCommand::DrawMicaCard {
            bounds,
            bg_color: [0.15, 0.18, 0.24, 0.8],
            border_color: [1.0, 1.0, 1.0, 0.08],
            radius: bounds.height / 2.0,
        };

        let fill_width = (bounds.width * norm_progress).max(0.0);
        let fill_bounds = LayoutRect::new(bounds.x, bounds.y, fill_width, bounds.height);
        let fill_cmd = GpuDrawCommand::DrawProgressBar {
            bounds: fill_bounds,
            progress: norm_progress,
            color: accent_color,
        };

        let thumb_bounds = Self::calculate_slider_thumb_bounds(bounds, norm_progress);
        let thumb_cmd = GpuDrawCommand::DrawMicaCard {
            bounds: thumb_bounds,
            bg_color: [1.0, 1.0, 1.0, 1.0],
            border_color: accent_color,
            radius: thumb_bounds.height / 2.0,
        };

        vec![track_cmd, fill_cmd, thumb_cmd]
    }

    /// Generates GPU draw commands for a color palette swatch widget.
    pub fn generate_swatch_draw_commands(
        bounds: LayoutRect,
        color_hex: &str,
        is_selected: bool,
    ) -> Vec<GpuDrawCommand> {
        let parsed_color = Self::hex_to_rgba(color_hex);
        let border_color = if is_selected {
            [1.0, 1.0, 1.0, 0.95]
        } else {
            [1.0, 1.0, 1.0, 0.15]
        };

        let swatch_cmd = GpuDrawCommand::DrawMicaCard {
            bounds,
            bg_color: parsed_color,
            border_color,
            radius: bounds.height / 2.0,
        };

        if is_selected {
            // Include an inner selection highlight ring command
            let inner_margin = 3.0;
            let inner_bounds = LayoutRect::new(
                bounds.x + inner_margin,
                bounds.y + inner_margin,
                (bounds.width - inner_margin * 2.0).max(1.0),
                (bounds.height - inner_margin * 2.0).max(1.0),
            );
            let highlight_cmd = GpuDrawCommand::DrawMicaCard {
                bounds: inner_bounds,
                bg_color: parsed_color,
                border_color: [1.0, 1.0, 1.0, 0.8],
                radius: inner_bounds.height / 2.0,
            };
            vec![swatch_cmd, highlight_cmd]
        } else {
            vec![swatch_cmd]
        }
    }

    /// Calculates the precise `LayoutRect` thumb bounds for a toggle switch track.
    pub fn calculate_toggle_thumb_bounds(bounds: LayoutRect, thumb_pos: f32) -> LayoutRect {
        let margin = 2.0;
        let thumb_diameter = (bounds.height - margin * 2.0).max(1.0);
        let min_x = bounds.x + margin;
        let max_x = bounds.x + bounds.width - thumb_diameter - margin;
        let thumb_x = min_x + (max_x - min_x) * thumb_pos.clamp(0.0, 1.0);

        LayoutRect::new(thumb_x, bounds.y + margin, thumb_diameter, thumb_diameter)
    }

    /// Calculates the precise `LayoutRect` thumb bounds for a slider track.
    pub fn calculate_slider_thumb_bounds(bounds: LayoutRect, progress: f32) -> LayoutRect {
        let thumb_diameter = bounds.height;
        let min_x = bounds.x;
        let max_x = (bounds.x + bounds.width - thumb_diameter).max(bounds.x);
        let thumb_x = min_x + (max_x - min_x) * progress.clamp(0.0, 1.0);

        LayoutRect::new(thumb_x, bounds.y, thumb_diameter, thumb_diameter)
    }

    /// Interpolates thumb position linearly toward target frame state.
    pub fn interpolate_thumb_pos(current: f32, target: f32, factor: f32) -> f32 {
        let alpha = factor.clamp(0.0, 1.0);
        current + (target - current) * alpha
    }

    /// Tests if spatial coordinates `(px, py)` fall within the hitbox `bounds`.
    pub fn is_point_inside(bounds: LayoutRect, px: f32, py: f32) -> bool {
        px >= bounds.x && px <= bounds.x + bounds.width && py >= bounds.y && py <= bounds.y + bounds.height
    }

    /// Converts hex color string (`#RRGGBB` or `#RRGGBBAA`) to normalized `[f32; 4]` RGBA float array.
    pub fn hex_to_rgba(hex: &str) -> [f32; 4] {
        let clean_hex = hex.trim().trim_start_matches('#');
        match clean_hex.len() {
            6 => {
                let r = u8::from_str_radix(&clean_hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
                let g = u8::from_str_radix(&clean_hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
                let b = u8::from_str_radix(&clean_hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
                [r, g, b, 1.0]
            }
            8 => {
                let r = u8::from_str_radix(&clean_hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
                let g = u8::from_str_radix(&clean_hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
                let b = u8::from_str_radix(&clean_hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
                let a = u8::from_str_radix(&clean_hex[6..8], 16).unwrap_or(255) as f32 / 255.0;
                [r, g, b, a]
            }
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_thumb_bounds_and_draw_commands() {
        let track_bounds = LayoutRect::new(10.0, 20.0, 50.0, 24.0);
        let accent_color = [0.0, 0.8, 1.0, 1.0];

        // Off state (thumb_pos = 0.0)
        let commands_off = ControlsRenderer::generate_toggle_draw_commands(
            track_bounds,
            false,
            0.0,
            accent_color,
        );
        assert_eq!(commands_off.len(), 2);

        let thumb_bounds_off = ControlsRenderer::calculate_toggle_thumb_bounds(track_bounds, 0.0);
        assert_eq!(thumb_bounds_off.x, 12.0); // 10.0 + 2.0 margin
        assert_eq!(thumb_bounds_off.y, 22.0); // 20.0 + 2.0 margin
        assert_eq!(thumb_bounds_off.width, 20.0); // 24.0 - 4.0 margin

        // On state (thumb_pos = 1.0)
        let commands_on = ControlsRenderer::generate_toggle_draw_commands(
            track_bounds,
            true,
            1.0,
            accent_color,
        );
        assert_eq!(commands_on.len(), 2);

        let thumb_bounds_on = ControlsRenderer::calculate_toggle_thumb_bounds(track_bounds, 1.0);
        assert_eq!(thumb_bounds_on.x, 38.0); // 10.0 + 50.0 - 20.0 - 2.0
        assert_eq!(thumb_bounds_on.y, 22.0);

        // Halfway animated state (thumb_pos = 0.5)
        let thumb_bounds_mid = ControlsRenderer::calculate_toggle_thumb_bounds(track_bounds, 0.5);
        assert_eq!(thumb_bounds_mid.x, 25.0);
    }

    #[test]
    fn test_slider_track_fills_and_thumb_bounds() {
        let slider_bounds = LayoutRect::new(0.0, 0.0, 200.0, 20.0);
        let accent = [0.2, 0.9, 0.3, 1.0];

        let commands = ControlsRenderer::generate_slider_draw_commands(
            slider_bounds,
            0.75,
            accent,
        );
        assert_eq!(commands.len(), 3);

        if let GpuDrawCommand::DrawProgressBar { bounds, progress, color } = &commands[1] {
            assert_eq!(*progress, 0.75);
            assert_eq!(bounds.width, 150.0); // 200.0 * 0.75
            assert_eq!(*color, accent);
        } else {
            panic!("Expected DrawProgressBar for slider fill");
        }

        let thumb_bounds = ControlsRenderer::calculate_slider_thumb_bounds(slider_bounds, 0.75);
        assert_eq!(thumb_bounds.x, 135.0); // (200 - 20) * 0.75 = 135
        assert_eq!(thumb_bounds.width, 20.0);
    }

    #[test]
    fn test_swatch_draw_commands_and_selection() {
        let swatch_bounds = LayoutRect::new(5.0, 5.0, 30.0, 30.0);

        let unselected_cmds = ControlsRenderer::generate_swatch_draw_commands(
            swatch_bounds,
            "#00FF88",
            false,
        );
        assert_eq!(unselected_cmds.len(), 1);

        let selected_cmds = ControlsRenderer::generate_swatch_draw_commands(
            swatch_bounds,
            "#00FF88",
            true,
        );
        assert_eq!(selected_cmds.len(), 2);

        if let GpuDrawCommand::DrawMicaCard { bg_color, .. } = &selected_cmds[0] {
            assert!((bg_color[0] - 0.0).abs() < 0.01);
            assert!((bg_color[1] - 1.0).abs() < 0.01);
            assert!((bg_color[2] - 0.533).abs() < 0.01);
            assert_eq!(bg_color[3], 1.0);
        } else {
            panic!("Expected DrawMicaCard for color swatch");
        }
    }

    #[test]
    fn test_thumb_interpolation() {
        let start = 0.0;
        let end = 1.0;

        assert_eq!(ControlsRenderer::interpolate_thumb_pos(start, end, 0.0), 0.0);
        assert_eq!(ControlsRenderer::interpolate_thumb_pos(start, end, 0.5), 0.5);
        assert_eq!(ControlsRenderer::interpolate_thumb_pos(start, end, 1.0), 1.0);
        assert_eq!(ControlsRenderer::interpolate_thumb_pos(start, end, 1.5), 1.0);
        assert_eq!(ControlsRenderer::interpolate_thumb_pos(start, end, -0.2), 0.0);
    }

    #[test]
    fn test_hex_to_rgba_conversion() {
        let rgb_hex = ControlsRenderer::hex_to_rgba("#FF0000");
        assert_eq!(rgb_hex, [1.0, 0.0, 0.0, 1.0]);

        let rgba_hex = ControlsRenderer::hex_to_rgba("00FF0080");
        assert!((rgba_hex[0] - 0.0).abs() < 0.01);
        assert!((rgba_hex[1] - 1.0).abs() < 0.01);
        assert!((rgba_hex[2] - 0.0).abs() < 0.01);
        assert!((rgba_hex[3] - 0.501).abs() < 0.01);

        let invalid_hex = ControlsRenderer::hex_to_rgba("invalid");
        assert_eq!(invalid_hex, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_hitbox_point_inside() {
        let box_rect = LayoutRect::new(10.0, 10.0, 50.0, 30.0);

        assert!(ControlsRenderer::is_point_inside(box_rect, 10.0, 10.0));
        assert!(ControlsRenderer::is_point_inside(box_rect, 30.0, 20.0));
        assert!(ControlsRenderer::is_point_inside(box_rect, 60.0, 40.0));

        assert!(!ControlsRenderer::is_point_inside(box_rect, 9.9, 10.0));
        assert!(!ControlsRenderer::is_point_inside(box_rect, 60.1, 20.0));
        assert!(!ControlsRenderer::is_point_inside(box_rect, 30.0, 40.1));
    }
}
