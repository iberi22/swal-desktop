//! GPU Typography & Glyph Rasterizer Engine for SWAL Render Pipeline
//!
//! Provides font family resolution, font metrics calculation,
//! text layout formatting (wrapping, ellipsis truncation),
//! and vertex quad generation for WGPU text rendering.

use serde::{Deserialize, Serialize};

/// Supported primary and fallback font families.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontFamily {
    JetBrainsMono,
    Inter,
    SymbolsNerdFontMono,
    Custom(String),
}

impl FontFamily {
    /// Returns the canonical display name of the font family.
    pub fn name(&self) -> &str {
        match self {
            FontFamily::JetBrainsMono => "JetBrains Mono",
            FontFamily::Inter => "Inter",
            FontFamily::SymbolsNerdFontMono => "Symbols Nerd Font Mono",
            FontFamily::Custom(ref name) => name.as_str(),
        }
    }

    /// Resolves a string font name to a `FontFamily`.
    pub fn resolve(name: &str) -> Self {
        let trimmed = name.trim();
        let lower = trimmed.to_lowercase();
        if lower.contains("jetbrains") || (lower.contains("mono") && lower.contains("jetbrains")) {
            FontFamily::JetBrainsMono
        } else if lower.contains("inter") {
            FontFamily::Inter
        } else if lower.contains("nerd") || lower.contains("symbols") {
            FontFamily::SymbolsNerdFontMono
        } else {
            FontFamily::Custom(trimmed.to_string())
        }
    }

    /// Calculates the character width multiplier relative to `font_size`.
    ///
    /// Monospace fonts return fixed ratios (~0.60). Proportional fonts evaluate
    /// per-character width heuristics (e.g. wide vs narrow characters).
    pub fn char_width(&self, ch: char, font_size: f32) -> f32 {
        match self {
            FontFamily::JetBrainsMono | FontFamily::SymbolsNerdFontMono => font_size * 0.60,
            FontFamily::Inter => match ch {
                'W' | 'M' | '@' | '%' | 'Q' => font_size * 0.85,
                'i' | 'l' | 'f' | 't' | 'j' | 'r' | ' ' | '.' | ',' | ';' | ':' | '!' | '\'' => {
                    font_size * 0.35
                }
                'I' | '1' | '|' | '(' | ')' | '[' | ']' => font_size * 0.40,
                c if c.is_ascii_uppercase() => font_size * 0.65,
                _ => font_size * 0.55,
            },
            FontFamily::Custom(_) => font_size * 0.60,
        }
    }
}

/// Representing RGBA colors as [r, g, b, a] in normalized 0.0..1.0 float ranges.
pub type Color = [f32; 4];

/// Vertex layout for WGPU text rendering quads.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: Color,
}

/// Axis-aligned texture quad representing a single rasterized character or text glyph box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextQuad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Texture UV coordinate bounding box [u_min, v_min, u_max, v_max].
    pub uv_bounds: [f32; 4],
    pub color: Color,
}

/// GPU Typography layout and glyph rasterizer engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphRenderer {
    pub font_family: FontFamily,
    pub fallback_font: FontFamily,
    pub line_height_factor: f32,
}

impl Default for GlyphRenderer {
    fn default() -> Self {
        Self {
            font_family: FontFamily::JetBrainsMono,
            fallback_font: FontFamily::SymbolsNerdFontMono,
            line_height_factor: 1.2,
        }
    }
}

impl GlyphRenderer {
    /// Creates a new `GlyphRenderer` with the specified primary font family.
    pub fn new(font_family: FontFamily) -> Self {
        Self {
            font_family,
            ..Default::default()
        }
    }

    /// Sets the fallback font family.
    pub fn with_fallback(mut self, fallback_font: FontFamily) -> Self {
        self.fallback_font = fallback_font;
        self
    }

    /// Sets the line height multiplier factor.
    pub fn with_line_height_factor(mut self, factor: f32) -> Self {
        self.line_height_factor = factor;
        self
    }

    /// Resolves font family name string to a `FontFamily`.
    pub fn resolve_font(&self, font_name: &str) -> FontFamily {
        FontFamily::resolve(font_name)
    }

    /// Measures the total width and height of `text` at `font_size`.
    ///
    /// Correctly handles multi-line text separated by `\n`.
    pub fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, font_size * self.line_height_factor);
        }

        let lines: Vec<&str> = text.split('\n').collect();
        let mut max_width: f32 = 0.0;

        for line in &lines {
            let line_width: f32 = line
                .chars()
                .map(|ch| self.font_family.char_width(ch, font_size))
                .sum();
            if line_width > max_width {
                max_width = line_width;
            }
        }

        let total_height = (lines.len() as f32) * font_size * self.line_height_factor;
        (max_width, total_height)
    }

    /// Truncates `text` with ellipsis (`"..."`) if its width exceeds `max_width`.
    pub fn truncate_ellipsis(&self, text: &str, max_width: f32, font_size: f32) -> String {
        let (current_width, _) = self.measure_text(text, font_size);
        if current_width <= max_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let (ellipsis_width, _) = self.measure_text(ellipsis, font_size);

        if max_width < ellipsis_width {
            // Cannot fit even "...", trim down chars without ellipsis
            let mut acc = String::new();
            for ch in text.chars() {
                let test_str = format!("{}{}", acc, ch);
                if self.measure_text(&test_str, font_size).0 > max_width {
                    break;
                }
                acc.push(ch);
            }
            return acc;
        }

        let mut acc = String::new();
        let chars: Vec<char> = text.chars().collect();
        for ch in chars {
            let test_str = format!("{}{}{}", acc, ch, ellipsis);
            if self.measure_text(&test_str, font_size).0 > max_width {
                break;
            }
            acc.push(ch);
        }

        format!("{}{}", acc, ellipsis)
    }

    /// Wraps `text` into multiple lines so that no line exceeds `max_width`.
    pub fn wrap_text(&self, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        if text.is_empty() {
            return vec![String::new()];
        }

        let mut wrapped_lines = Vec::new();

        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                wrapped_lines.push(String::new());
                continue;
            }

            let words: Vec<&str> = paragraph.split_whitespace().collect();
            if words.is_empty() {
                wrapped_lines.push(String::new());
                continue;
            }

            let mut current_line = String::new();

            for word in words {
                if current_line.is_empty() {
                    // Check if single word exceeds max_width
                    if self.measure_text(word, font_size).0 > max_width {
                        // Word is longer than max_width, break character by character
                        for ch in word.chars() {
                            let mut test_line = current_line.clone();
                            test_line.push(ch);
                            if !current_line.is_empty()
                                && self.measure_text(&test_line, font_size).0 > max_width
                            {
                                wrapped_lines.push(current_line);
                                current_line = ch.to_string();
                            } else {
                                current_line.push(ch);
                            }
                        }
                    } else {
                        current_line.push_str(word);
                    }
                } else {
                    let test_line = format!("{} {}", current_line, word);
                    if self.measure_text(&test_line, font_size).0 <= max_width {
                        current_line = test_line;
                    } else {
                        wrapped_lines.push(current_line);
                        // Start new line with word
                        if self.measure_text(word, font_size).0 > max_width {
                            current_line = String::new();
                            for ch in word.chars() {
                                let mut test_char_line = current_line.clone();
                                test_char_line.push(ch);
                                if !current_line.is_empty()
                                    && self.measure_text(&test_char_line, font_size).0 > max_width
                                {
                                    wrapped_lines.push(current_line);
                                    current_line = ch.to_string();
                                } else {
                                    current_line.push(ch);
                                }
                            }
                        } else {
                            current_line = word.to_string();
                        }
                    }
                }
            }

            if !current_line.is_empty() {
                wrapped_lines.push(current_line);
            }
        }

        if wrapped_lines.is_empty() {
            vec![String::new()]
        } else {
            wrapped_lines
        }
    }

    /// Converts laid-out text into a series of `TextQuad` glyph boxes.
    pub fn generate_quads(
        &self,
        text: &str,
        start_x: f32,
        start_y: f32,
        font_size: f32,
        color: Color,
    ) -> Vec<TextQuad> {
        let mut quads = Vec::new();
        let line_height = font_size * self.line_height_factor;

        let mut current_y = start_y;

        for line in text.split('\n') {
            let mut current_x = start_x;
            for ch in line.chars() {
                let char_w = self.font_family.char_width(ch, font_size);
                let quad = TextQuad {
                    x: current_x,
                    y: current_y,
                    width: char_w,
                    height: font_size,
                    uv_bounds: [0.0, 0.0, 1.0, 1.0], // Normalized full glyph UV fallback
                    color,
                };
                quads.push(quad);
                current_x += char_w;
            }
            current_y += line_height;
        }

        quads
    }

    /// Converts laid-out text into vertex quads (6 vertices per glyph quad) for WGPU rendering.
    pub fn generate_vertices(
        &self,
        text: &str,
        start_x: f32,
        start_y: f32,
        font_size: f32,
        color: Color,
    ) -> Vec<TextVertex> {
        let quads = self.generate_quads(text, start_x, start_y, font_size, color);
        let mut vertices = Vec::with_capacity(quads.len() * 6);

        for q in quads {
            let x0 = q.x;
            let y0 = q.y;
            let x1 = q.x + q.width;
            let y1 = q.y + q.height;

            let u0 = q.uv_bounds[0];
            let v0 = q.uv_bounds[1];
            let u1 = q.uv_bounds[2];
            let v1 = q.uv_bounds[3];

            // Triangle 1: Top-Left, Bottom-Left, Top-Right
            vertices.push(TextVertex {
                position: [x0, y0],
                tex_coords: [u0, v0],
                color,
            });
            vertices.push(TextVertex {
                position: [x0, y1],
                tex_coords: [u0, v1],
                color,
            });
            vertices.push(TextVertex {
                position: [x1, y0],
                tex_coords: [u1, v0],
                color,
            });

            // Triangle 2: Top-Right, Bottom-Left, Bottom-Right
            vertices.push(TextVertex {
                position: [x1, y0],
                tex_coords: [u1, v0],
                color,
            });
            vertices.push(TextVertex {
                position: [x0, y1],
                tex_coords: [u0, v1],
                color,
            });
            vertices.push(TextVertex {
                position: [x1, y1],
                tex_coords: [u1, v1],
                color,
            });
        }

        vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_family_resolution() {
        assert_eq!(FontFamily::resolve("JetBrains Mono"), FontFamily::JetBrainsMono);
        assert_eq!(FontFamily::resolve("Inter Display"), FontFamily::Inter);
        assert_eq!(
            FontFamily::resolve("Symbols Nerd Font Mono"),
            FontFamily::SymbolsNerdFontMono
        );
        assert_eq!(
            FontFamily::resolve("Comic Sans MS"),
            FontFamily::Custom("Comic Sans MS".to_string())
        );

        let renderer = GlyphRenderer::default();
        assert_eq!(renderer.resolve_font("inter"), FontFamily::Inter);
    }

    #[test]
    fn test_measure_text_dimensions() {
        let renderer = GlyphRenderer::new(FontFamily::JetBrainsMono);
        let font_size = 16.0;

        // Single line: "hello" (5 chars * 16.0 * 0.60 = 48.0)
        let (width, height) = renderer.measure_text("hello", font_size);
        assert!((width - 48.0).abs() < 1e-4);
        assert!((height - 19.2).abs() < 1e-4); // 16.0 * 1.2 = 19.2

        // Multi-line: 2 lines
        let (ml_width, ml_height) = renderer.measure_text("hello\nworld!", font_size);
        // "world!" has 6 chars = 57.6
        assert!((ml_width - 57.6).abs() < 1e-4);
        assert!((ml_height - 38.4).abs() < 1e-4); // 2 * 19.2 = 38.4
    }

    #[test]
    fn test_ellipsis_truncation() {
        let renderer = GlyphRenderer::new(FontFamily::JetBrainsMono);
        let font_size = 10.0; // char_width = 6.0

        let full_text = "This is a long text line for testing truncation";
        let max_width = 100.0;

        let truncated = renderer.truncate_ellipsis(full_text, max_width, font_size);
        assert!(truncated.ends_with("..."));

        let (truncated_w, _) = renderer.measure_text(&truncated, font_size);
        assert!(
            truncated_w <= max_width,
            "Truncated width {} exceeds max_width {}",
            truncated_w,
            max_width
        );

        // Text that fits fully without truncation
        let short_text = "short";
        assert_eq!(
            renderer.truncate_ellipsis(short_text, 200.0, font_size),
            "short"
        );
    }

    #[test]
    fn test_multi_line_wrapping() {
        let renderer = GlyphRenderer::new(FontFamily::JetBrainsMono);
        let font_size = 10.0; // char_width = 6.0, space = 6.0

        // "hello world rust"
        let text = "hello world rust";
        // max_width = 60.0 allows max 10 chars per line
        let lines = renderer.wrap_text(text, 60.0, font_size);

        assert_eq!(lines, vec!["hello", "world rust"]);

        for line in &lines {
            let (w, _) = renderer.measure_text(line, font_size);
            assert!(
                w <= 60.0,
                "Line '{}' width {} exceeds max_width 60.0",
                line,
                w
            );
        }
    }

    #[test]
    fn test_vertex_and_quad_generation() {
        let renderer = GlyphRenderer::new(FontFamily::Inter);
        let font_size = 12.0;
        let color = [1.0, 1.0, 1.0, 1.0];

        let text = "SWAL";
        let quads = renderer.generate_quads(text, 0.0, 0.0, font_size, color);
        assert_eq!(quads.len(), 4);

        let vertices = renderer.generate_vertices(text, 0.0, 0.0, font_size, color);
        // 4 characters * 6 vertices per character quad = 24 vertices
        assert_eq!(vertices.len(), 24);

        for vertex in &vertices {
            assert_eq!(vertex.color, color);
        }
    }
}
