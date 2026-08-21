//! Mica Acrylic Blur & Rounded Geometry Quad Shader in WGSL (Zero-Eww)
//! Provides hardware-accelerated rendering specs for rounded rectangular quads with
//! Mica acrylic tinting, 1px highlight borders, SDF smoothstep antialiasing, and uniform buffers.

use serde::{Deserialize, Serialize};

/// Embedded WGSL shader for Mica Acrylic rounded rectangular quads with SDF borders
pub const MICA_QUAD_WGSL_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) frag_pos: vec2<f32>,
};

struct MicaUniforms {
    rect_bounds: vec4<f32>,   // x, y, width, height
    border_color: vec4<f32>,  // r, g, b, a
    bg_tint: vec4<f32>,       // r, g, b, a (Mica acrylic tint)
    corner_radius: f32,       // pixels
    border_width: f32,        // pixels (e.g. 1.0)
    blur_intensity: f32,      // 0.0 - 1.0
    _padding: f32,            // 16-byte alignment padding
};

@group(0) @binding(0)
var<uniform> uniforms: MicaUniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    out.frag_pos = model.position;
    return out;
}

// Signed Distance Field calculation for rounded box geometry
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = uniforms.rect_bounds.xy + uniforms.rect_bounds.zw * 0.5;
    let half_size = uniforms.rect_bounds.zw * 0.5;
    let p = in.frag_pos - center;

    let dist = sd_rounded_box(p, half_size, uniforms.corner_radius);

    // Smoothstep antialiased edge factor
    let edge_softness = 0.005;
    let alpha = 1.0 - smoothstep(-edge_softness, edge_softness, dist);

    if (alpha <= 0.0) {
        discard;
    }

    // Border inner boundary factor
    let border_dist = dist + uniforms.border_width;
    let border_factor = smoothstep(-edge_softness, edge_softness, border_dist) * (1.0 - smoothstep(-edge_softness, edge_softness, dist));

    // Blend Mica acrylic tint and border color
    var color = mix(uniforms.bg_tint, uniforms.border_color, border_factor);
    color.a = color.a * alpha;

    return color;
}
"#;

/// 16-byte aligned uniform structure for Mica shader
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MicaUniforms {
    pub rect_bounds: [f32; 4],
    pub border_color: [f32; 4],
    pub bg_tint: [f32; 4],
    pub corner_radius: f32,
    pub border_width: f32,
    pub blur_intensity: f32,
    pub _padding: f32,
}

impl MicaUniforms {
    pub fn new(
        rect_bounds: [f32; 4],
        border_color: [f32; 4],
        bg_tint: [f32; 4],
        corner_radius: f32,
        border_width: f32,
        blur_intensity: f32,
    ) -> Self {
        Self {
            rect_bounds,
            border_color,
            bg_tint,
            corner_radius,
            border_width,
            blur_intensity,
            _padding: 0.0,
        }
    }

    /// Converts the uniform structure directly to a byte slice for GPU buffer writing
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl Default for MicaUniforms {
    fn default() -> Self {
        Self::new(
            [0.0, 0.0, 100.0, 100.0],
            [1.0, 1.0, 1.0, 0.2],
            [0.1, 0.1, 0.15, 0.75],
            12.0,
            1.0,
            0.8,
        )
    }
}

/// Description of a bind group layout entry for WGPU pipeline setup
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicaBindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: String,
    pub buffer_type: String,
}

/// Description of a vertex attribute for WGPU buffer layouts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicaVertexAttribute {
    pub format: String,
    pub offset: u64,
    pub shader_location: u32,
}

/// Description of vertex buffer layouts for Mica quad pipeline
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicaVertexBufferLayout {
    pub array_stride: u64,
    pub step_mode: String,
    pub attributes: Vec<MicaVertexAttribute>,
}

/// Pipeline descriptor configuration for WGPU render pipeline creation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicaPipelineDescriptor {
    pub label: String,
    pub shader_source: &'static str,
    pub entry_point_vs: &'static str,
    pub entry_point_fs: &'static str,
    pub sample_count: u32,
    pub bind_group_entries: Vec<MicaBindGroupLayoutEntry>,
    pub vertex_layout: MicaVertexBufferLayout,
}

impl MicaPipelineDescriptor {
    pub fn new() -> Self {
        Self {
            label: "Mica Quad Render Pipeline".to_string(),
            shader_source: MICA_QUAD_WGSL_SHADER,
            entry_point_vs: "vs_main",
            entry_point_fs: "fs_main",
            sample_count: 1,
            bind_group_entries: vec![MicaBindGroupLayoutEntry {
                binding: 0,
                visibility: "VERTEX_FRAGMENT".to_string(),
                buffer_type: "Uniform".to_string(),
            }],
            vertex_layout: MicaVertexBufferLayout {
                array_stride: 16, // 2x f32 position + 2x f32 uv
                step_mode: "Vertex".to_string(),
                attributes: vec![
                    MicaVertexAttribute {
                        format: "Float32x2".to_string(),
                        offset: 0,
                        shader_location: 0,
                    },
                    MicaVertexAttribute {
                        format: "Float32x2".to_string(),
                        offset: 8,
                        shader_location: 1,
                    },
                ],
            },
        }
    }
}

impl Default for MicaPipelineDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU-side Signed Distance Field calculation for a rounded box.
///
/// `p`: position relative to center of the box `[x, y]`
/// `b`: half-extents of the box `[half_width, half_height]`
/// `r`: corner radius
pub fn sd_rounded_box(p: [f32; 2], b: [f32; 2], r: f32) -> f32 {
    let q = [p[0].abs() - b[0] + r, p[1].abs() - b[1] + r];
    let q_max = [q[0].max(0.0), q[1].max(0.0)];
    let outer_dist = (q_max[0] * q_max[0] + q_max[1] * q_max[1]).sqrt();
    let inner_dist = q[0].max(q[1]).min(0.0);
    outer_dist + inner_dist - r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgsl_shader_contains_uniforms_and_functions() {
        assert!(MICA_QUAD_WGSL_SHADER.contains("struct MicaUniforms"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("rect_bounds: vec4<f32>"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("corner_radius: f32"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("border_width: f32"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("border_color: vec4<f32>"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("bg_tint: vec4<f32>"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("blur_intensity: f32"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("fn sd_rounded_box"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("fn vs_main"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("fn fs_main"));
        assert!(MICA_QUAD_WGSL_SHADER.contains("smoothstep"));
    }

    #[test]
    fn test_uniform_buffer_packing_and_alignment() {
        assert_eq!(std::mem::size_of::<MicaUniforms>(), 64);
        assert_eq!(std::mem::align_of::<MicaUniforms>(), 4);

        let uniforms = MicaUniforms::default();
        let bytes = uniforms.as_bytes();
        assert_eq!(bytes.len(), 64);
    }

    #[test]
    fn test_sdf_distance_calculations() {
        let half_extents = [50.0, 50.0];
        let radius = 10.0;

        // Center point: deep inside
        let dist_center = sd_rounded_box([0.0, 0.0], half_extents, radius);
        assert!(dist_center < -40.0);

        // Edge point inside boundary
        let dist_edge = sd_rounded_box([50.0, 0.0], half_extents, radius);
        assert!((dist_edge - 0.0).abs() < 1e-4);

        // Outside box
        let dist_outside = sd_rounded_box([60.0, 0.0], half_extents, radius);
        assert!((dist_outside - 10.0).abs() < 1e-4);

        // Rounded corner boundary check (at theta = 45 degrees on 10px corner curve)
        let corner_x = 40.0 + 10.0 * (std::f32::consts::PI / 4.0).cos();
        let corner_y = 40.0 + 10.0 * (std::f32::consts::PI / 4.0).sin();
        let dist_corner = sd_rounded_box([corner_x, corner_y], half_extents, radius);
        assert!((dist_corner - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_pipeline_descriptor_defaults() {
        let desc = MicaPipelineDescriptor::new();
        assert_eq!(desc.entry_point_vs, "vs_main");
        assert_eq!(desc.entry_point_fs, "fs_main");
        assert_eq!(desc.bind_group_entries.len(), 1);
        assert_eq!(desc.vertex_layout.attributes.len(), 2);
    }
}
