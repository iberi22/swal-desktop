//! Direct A2UI AST GPU Node Rasterizer
//!
//! Converts ComponentNode trees directly into GPU draw commands and layout bounding boxes.

use crate::ComponentNode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuDrawCommand {
    DrawMicaCard {
        bounds: LayoutRect,
        bg_color: [f32; 4],
        border_color: [f32; 4],
        radius: f32,
    },
    DrawText {
        text: String,
        x: f32,
        y: f32,
        font_size: f32,
        color: [f32; 4],
    },
    DrawButton {
        bounds: LayoutRect,
        label: String,
        is_hovered: bool,
        action_id: String,
    },
    DrawProgressBar {
        bounds: LayoutRect,
        progress: f32,
        color: [f32; 4],
    },
}

/// Evaluates a `ComponentNode` tree and returns a flattened list of GPU draw commands.
pub fn evaluate_ast_to_gpu_commands(root: &ComponentNode, viewport: LayoutRect) -> Vec<GpuDrawCommand> {
    let mut commands = Vec::new();
    evaluate_node(root, viewport, &mut commands);
    commands
}

fn evaluate_node(node: &ComponentNode, bounds: LayoutRect, commands: &mut Vec<GpuDrawCommand>) {
    match node {
        ComponentNode::Card { title, children, .. } => {
            // Draw card background
            commands.push(GpuDrawCommand::DrawMicaCard {
                bounds,
                bg_color: [0.06, 0.09, 0.16, 0.85],
                border_color: [1.0, 1.0, 1.0, 0.08],
                radius: 8.0,
            });

            let padding = 12.0;
            let mut current_y = bounds.y + padding;
            let inner_width = (bounds.width - padding * 2.0).max(1.0);

            if let Some(ref t) = title {
                commands.push(GpuDrawCommand::DrawText {
                    text: t.clone(),
                    x: bounds.x + padding,
                    y: current_y,
                    font_size: 16.0,
                    color: [0.95, 0.96, 0.98, 1.0],
                });
                current_y += 24.0;
            }

            let item_height = 40.0;
            let gap = 8.0;

            for child in children {
                let child_bounds = LayoutRect::new(bounds.x + padding, current_y, inner_width, item_height);
                evaluate_node(child, child_bounds, commands);
                current_y += item_height + gap;
            }
        }
        ComponentNode::Grid { columns, gap, children } => {
            let cols = (*columns).max(1) as f32;
            let gap_f = *gap as f32;
            let total_gaps = (cols - 1.0) * gap_f;
            let cell_width = ((bounds.width - total_gaps) / cols).max(1.0);
            let cell_height = 36.0;

            for (idx, child) in children.iter().enumerate() {
                let col = (idx % *columns) as f32;
                let row = (idx / *columns) as f32;

                let cell_x = bounds.x + col * (cell_width + gap_f);
                let cell_y = bounds.y + row * (cell_height + gap_f);
                let cell_bounds = LayoutRect::new(cell_x, cell_y, cell_width, cell_height);

                evaluate_node(child, cell_bounds, commands);
            }
        }
        ComponentNode::Button { label, action, .. } => {
            commands.push(GpuDrawCommand::DrawButton {
                bounds,
                label: label.clone(),
                is_hovered: false,
                action_id: action.clone(),
            });
        }
        ComponentNode::StatusBadge { status, label, color } => {
            let badge_bg = match status.as_str() {
                "healthy" | "ok" | "completed" => [0.06, 0.72, 0.51, 0.90],
                "warn" | "warning" => [0.96, 0.62, 0.04, 0.90],
                "danger" | "error" | "critical" => [0.94, 0.27, 0.27, 0.90],
                _ => [0.02, 0.71, 0.83, 0.90],
            };

            commands.push(GpuDrawCommand::DrawMicaCard {
                bounds,
                bg_color: badge_bg,
                border_color: [1.0, 1.0, 1.0, 0.12],
                radius: 4.0,
            });

            commands.push(GpuDrawCommand::DrawText {
                text: format!("{}: {}", status.to_uppercase(), label),
                x: bounds.x + 8.0,
                y: bounds.y + (bounds.height / 2.0) - 6.0,
                font_size: 12.0,
                color: [1.0, 1.0, 1.0, 1.0],
            });

            if let Some(c) = color {
                if c.contains("progress") || c.starts_with('#') {
                    // Draw optional color accent progress indicator
                    commands.push(GpuDrawCommand::DrawProgressBar {
                        bounds: LayoutRect::new(bounds.x, bounds.y + bounds.height - 3.0, bounds.width, 3.0),
                        progress: 1.0,
                        color: badge_bg,
                    });
                }
            }
        }
        ComponentNode::MetricPill { label, value, unit, color, .. } => {
            commands.push(GpuDrawCommand::DrawMicaCard {
                bounds,
                bg_color: [0.1, 0.15, 0.25, 0.75],
                border_color: [0.06, 0.71, 0.83, 0.30],
                radius: 6.0,
            });

            let val_str = match unit {
                Some(u) => format!("{} {}", value, u),
                None => value.clone(),
            };

            commands.push(GpuDrawCommand::DrawText {
                text: format!("{}: {}", label, val_str),
                x: bounds.x + 6.0,
                y: bounds.y + 6.0,
                font_size: 13.0,
                color: [0.9, 0.9, 0.9, 1.0],
            });

            // If metric value is numeric percentage, draw progress bar
            if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
                let norm_progress = (pct / 100.0).clamp(0.0, 1.0);
                commands.push(GpuDrawCommand::DrawProgressBar {
                    bounds: LayoutRect::new(
                        bounds.x + 6.0,
                        bounds.y + bounds.height - 8.0,
                        (bounds.width - 12.0).max(1.0),
                        4.0,
                    ),
                    progress: norm_progress,
                    color: [0.02, 0.71, 0.83, 1.0],
                });
            } else if let Some(_c) = color {
                commands.push(GpuDrawCommand::DrawProgressBar {
                    bounds: LayoutRect::new(
                        bounds.x + 6.0,
                        bounds.y + bounds.height - 8.0,
                        (bounds.width - 12.0).max(1.0),
                        4.0,
                    ),
                    progress: 0.5,
                    color: [0.06, 0.72, 0.51, 1.0],
                });
            }
        }
        ComponentNode::LogViewer { source, height, lines } => {
            let container_bounds = LayoutRect::new(bounds.x, bounds.y, bounds.width, *height as f32);
            commands.push(GpuDrawCommand::DrawMicaCard {
                bounds: container_bounds,
                bg_color: [0.02, 0.04, 0.08, 0.95],
                border_color: [0.2, 0.25, 0.3, 0.5],
                radius: 4.0,
            });

            commands.push(GpuDrawCommand::DrawText {
                text: format!("LOG: {}", source),
                x: bounds.x + 8.0,
                y: bounds.y + 6.0,
                font_size: 11.0,
                color: [0.6, 0.7, 0.8, 1.0],
            });

            let mut line_y = bounds.y + 22.0;
            for line in lines {
                commands.push(GpuDrawCommand::DrawText {
                    text: line.clone(),
                    x: bounds.x + 8.0,
                    y: line_y,
                    font_size: 10.0,
                    color: [0.8, 0.85, 0.9, 1.0],
                });
                line_y += 14.0;
            }
        }
        ComponentNode::Terminal { command, output, height } => {
            let term_height = height.unwrap_or(120) as f32;
            let term_bounds = LayoutRect::new(bounds.x, bounds.y, bounds.width, term_height);

            commands.push(GpuDrawCommand::DrawMicaCard {
                bounds: term_bounds,
                bg_color: [0.0, 0.0, 0.0, 0.9],
                border_color: [0.0, 0.9, 0.4, 0.4],
                radius: 4.0,
            });

            if let Some(cmd) = command {
                commands.push(GpuDrawCommand::DrawText {
                    text: format!("$ {}", cmd),
                    x: bounds.x + 8.0,
                    y: bounds.y + 8.0,
                    font_size: 12.0,
                    color: [0.0, 1.0, 0.5, 1.0],
                });
            }

            if let Some(out_lines) = output {
                let mut out_y = bounds.y + 26.0;
                for line in out_lines {
                    commands.push(GpuDrawCommand::DrawText {
                        text: line.clone(),
                        x: bounds.x + 8.0,
                        y: out_y,
                        font_size: 11.0,
                        color: [0.8, 0.8, 0.8, 1.0],
                    });
                    out_y += 14.0;
                }
            }
        }
        ComponentNode::Tabs { tabs } => {
            let tab_header_height = 28.0;
            let mut tab_x = bounds.x;

            for tab in tabs {
                let tab_btn_width = (tab.label.len() as f32 * 8.0 + 16.0).max(60.0);
                let tab_bounds = LayoutRect::new(tab_x, bounds.y, tab_btn_width, tab_header_height);

                commands.push(GpuDrawCommand::DrawButton {
                    bounds: tab_bounds,
                    label: tab.label.clone(),
                    is_hovered: false,
                    action_id: format!("tab_select:{}", tab.id),
                });

                tab_x += tab_btn_width + 4.0;
            }

            let content_y = bounds.y + tab_header_height + 4.0;
            let _content_height = (bounds.height - tab_header_height - 4.0).max(1.0);

            // Render first tab content
            if let Some(first_tab) = tabs.first() {
                let mut child_y = content_y;
                for child in &first_tab.content {
                    let child_bounds = LayoutRect::new(bounds.x, child_y, bounds.width, 36.0);
                    evaluate_node(child, child_bounds, commands);
                    child_y += 40.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TabItem;

    #[test]
    fn test_evaluate_card_and_button_ast() {
        let root = ComponentNode::Card {
            title: Some("System Controls".to_string()),
            elevation: Some("elevated".to_string()),
            children: vec![
                ComponentNode::Button {
                    label: "Reboot Node".to_string(),
                    action: "node.reboot".to_string(),
                    variant: None,
                },
            ],
        };

        let viewport = LayoutRect::new(0.0, 0.0, 400.0, 300.0);
        let commands = evaluate_ast_to_gpu_commands(&root, viewport);

        assert!(!commands.is_empty());
        assert!(matches!(commands[0], GpuDrawCommand::DrawMicaCard { .. }));

        if let GpuDrawCommand::DrawText { ref text, .. } = commands[1] {
            assert_eq!(text, "System Controls");
        } else {
            panic!("Expected DrawText for card title");
        }

        let has_button = commands.iter().any(|cmd| match cmd {
            GpuDrawCommand::DrawButton { label, action_id, .. } => {
                label == "Reboot Node" && action_id == "node.reboot"
            }
            _ => false,
        });

        assert!(has_button, "Commands list must contain DrawButton command");
    }

    #[test]
    fn test_evaluate_grid_and_progressbar_ast() {
        let root = ComponentNode::Grid {
            columns: 2,
            gap: 8,
            children: vec![
                ComponentNode::MetricPill {
                    label: "CPU Usage".to_string(),
                    value: "75.0%".to_string(),
                    unit: None,
                    trend: None,
                    color: Some("#00ff88".to_string()),
                },
                ComponentNode::StatusBadge {
                    status: "healthy".to_string(),
                    label: "Active".to_string(),
                    color: None,
                },
            ],
        };

        let viewport = LayoutRect::new(10.0, 10.0, 500.0, 200.0);
        let commands = evaluate_ast_to_gpu_commands(&root, viewport);

        let has_progressbar = commands.iter().any(|cmd| match cmd {
            GpuDrawCommand::DrawProgressBar { progress, .. } => (*progress - 0.75).abs() < 0.01,
            _ => false,
        });

        assert!(has_progressbar, "Expected DrawProgressBar with 75% progress from MetricPill");
    }

    #[test]
    fn test_evaluate_tabs_and_terminal_ast() {
        let root = ComponentNode::Tabs {
            tabs: vec![TabItem {
                id: "tab-1".to_string(),
                label: "Console".to_string(),
                content: vec![ComponentNode::Terminal {
                    command: Some("swal --status".to_string()),
                    output: Some(vec!["Status OK".to_string()]),
                    height: Some(100),
                }],
            }],
        };

        let viewport = LayoutRect::new(0.0, 0.0, 600.0, 400.0);
        let commands = evaluate_ast_to_gpu_commands(&root, viewport);

        let has_tab_button = commands.iter().any(|cmd| match cmd {
            GpuDrawCommand::DrawButton { action_id, label, .. } => {
                label == "Console" && action_id == "tab_select:tab-1"
            }
            _ => false,
        });
        assert!(has_tab_button, "Expected tab button GPU command");

        let has_terminal_cmd = commands.iter().any(|cmd| match cmd {
            GpuDrawCommand::DrawText { text, .. } => text.contains("swal --status"),
            _ => false,
        });
        assert!(has_terminal_cmd, "Expected terminal command text GPU command");
    }
}
