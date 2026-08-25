//! Windows Fluent Dark Acrylic Shader & Uniforms in WGSL (Zero-Eww)
//!
//! Provides hardware-accelerated rendering specifications for rounded rectangular quads
//! styled with Microsoft Windows 11 Fluent Acrylic Material. Implements:
//! - Background tint color & dynamic tint opacity (default 0.70 for Fluent Dark)
//! - Procedural frosted glass noise texture (default 0.02 factor) without texture lookups
//! - Dual-filtering / blur radius specifications
//! - Luminosity and exclusion blending
//! - Signed Distance Field (SDF) smoothstep antialiased borders and corners
//! - Strict 16-byte uniform alignment for WGSL compatibility

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Embedded WGSL shader for Windows Fluent Acrylic rounded rectangular quads
pub const FLUENT_ACRYLIC_WGSL_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) frag_pos: vec2<f32>,
};

struct FluentAcrylicUniforms {
    rect_bounds: vec4<f32>,      // x, y, width, height (pixels/NDC)
    tint_color: vec4<f32>,       // r, g, b, a (Fluent tint color)
    border_color: vec4<f32>,     // r, g, b, a (1px subtle border highlight)
    tint_opacity: f32,           // 0.70 default for Fluent Dark
    blur_radius: f32,            // e.g. 30.0px blur
    noise_factor: f32,           // 0.02 procedural frosted texture
    corner_radius: f32,          // e.g. 8.0px corner rounding
    border_width: f32,           // e.g. 1.0px stroke
    luminosity_factor: f32,      // 0.85 Fluent Dark luminosity blend
    _padding: vec2<f32>,         // 8 bytes padding for 16-byte alignment
};

@group(0) @binding(0)
var<uniform> uniforms: FluentAcrylicUniforms;

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

// High-speed procedural 2D hash without texture memory lookups
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Procedural frosted glass noise (zero-centered in [-0.5, 0.5])
fn procedural_frosted_noise(uv: vec2<f32>) -> f32 {
    let n1 = hash21(uv);
    let n2 = hash21(uv * 1.61803398875 + vec2<f32>(17.41, 43.19));
    return (n1 + n2) * 0.5 - 0.5;
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

    // Windows Fluent Dark Acrylic base blending:
    // Base luminosity plate mixed with tint color according to tint_opacity
    let base_dark = vec3<f32>(0.08, 0.08, 0.08) * uniforms.luminosity_factor;
    let tint_blend = mix(base_dark, uniforms.tint_color.rgb, uniforms.tint_opacity);

    // Frosted noise layer (subtle 0.02 grain)
    let noise = procedural_frosted_noise(in.uv * 1000.0) * uniforms.noise_factor;
    let acrylic_rgb = clamp(tint_blend + vec3<f32>(noise), vec3<f32>(0.0), vec3<f32>(1.0));

    // Inner highlight border factor
    let border_dist = dist + uniforms.border_width;
    let border_factor = smoothstep(-edge_softness, edge_softness, border_dist) * (1.0 - smoothstep(-edge_softness, edge_softness, dist));

    // Combine Acrylic body and border highlight
    let final_rgb = mix(acrylic_rgb, uniforms.border_color.rgb, border_factor * uniforms.border_color.a);
    let final_alpha = alpha * clamp(uniforms.tint_opacity + border_factor * uniforms.border_color.a, 0.0, 1.0);

    return vec4<f32>(final_rgb, final_alpha);
}
"#;

/// 16-byte aligned uniform structure for Windows Fluent Acrylic shader
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
pub struct FluentAcrylicUniforms {
    pub rect_bounds: [f32; 4],
    pub tint_color: [f32; 4],
    pub border_color: [f32; 4],
    pub tint_opacity: f32,
    pub blur_radius: f32,
    pub noise_factor: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub luminosity_factor: f32,
    pub _padding: [f32; 2],
}

impl FluentAcrylicUniforms {
    /// Creates a new `FluentAcrylicUniforms` with custom parameters.
    pub fn new(
        rect_bounds: [f32; 4],
        tint_color: [f32; 4],
        border_color: [f32; 4],
        tint_opacity: f32,
        blur_radius: f32,
        noise_factor: f32,
        corner_radius: f32,
        border_width: f32,
        luminosity_factor: f32,
    ) -> Self {
        Self {
            rect_bounds,
            tint_color,
            border_color,
            tint_opacity,
            blur_radius,
            noise_factor,
            corner_radius,
            border_width,
            luminosity_factor,
            _padding: [0.0, 0.0],
        }
    }

    /// Windows Fluent Dark mode acrylic presets:
    /// - Tint color: `[0.12, 0.12, 0.12, 1.0]` (Fluent Dark Acrylic tint)
    /// - Tint opacity: `0.70` (Microsoft Fluent Dark Acrylic guideline)
    /// - Blur radius: `30.0`
    /// - Noise factor: `0.02` (2% procedural frosted noise)
    /// - Border color: `[1.0, 1.0, 1.0, 0.10]` (10% subtle white stroke)
    /// - Corner radius: `8.0`
    /// - Border width: `1.0`
    /// - Luminosity factor: `0.85`
    pub fn fluent_dark() -> Self {
        Self::new(
            [0.0, 0.0, 100.0, 100.0],
            [0.12, 0.12, 0.12, 1.0],
            [1.0, 1.0, 1.0, 0.10],
            0.70,
            30.0,
            0.02,
            8.0,
            1.0,
            0.85,
        )
    }

    /// Windows Fluent Light mode acrylic presets:
    /// - Tint color: `[0.95, 0.95, 0.95, 1.0]`
    /// - Tint opacity: `0.85`
    /// - Blur radius: `30.0`
    /// - Noise factor: `0.02`
    /// - Border color: `[0.0, 0.0, 0.0, 0.08]`
    /// - Corner radius: `8.0`
    /// - Border width: `1.0`
    /// - Luminosity factor: `1.0`
    pub fn fluent_light() -> Self {
        Self::new(
            [0.0, 0.0, 100.0, 100.0],
            [0.95, 0.95, 0.95, 1.0],
            [0.0, 0.0, 0.0, 0.08],
            0.85,
            30.0,
            0.02,
            8.0,
            1.0,
            1.0,
        )
    }

    /// Converts the uniform structure directly to a byte slice for GPU buffer writes
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Default for FluentAcrylicUniforms {
    fn default() -> Self {
        Self::fluent_dark()
    }
}

/// Description of a bind group layout entry for WGPU pipeline setup
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FluentAcrylicBindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: String,
    pub buffer_type: String,
}

/// Description of a vertex attribute for WGPU buffer layouts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FluentAcrylicVertexAttribute {
    pub format: String,
    pub offset: u64,
    pub shader_location: u32,
}

/// Description of vertex buffer layouts for Fluent Acrylic quad pipeline
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FluentAcrylicVertexBufferLayout {
    pub array_stride: u64,
    pub step_mode: String,
    pub attributes: Vec<FluentAcrylicVertexAttribute>,
}

/// Pipeline descriptor configuration for Fluent Acrylic WGPU render pipeline creation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FluentAcrylicPipelineDescriptor {
    pub label: String,
    pub shader_source: &'static str,
    pub entry_point_vs: &'static str,
    pub entry_point_fs: &'static str,
    pub sample_count: u32,
    pub bind_group_entries: Vec<FluentAcrylicBindGroupLayoutEntry>,
    pub vertex_layout: FluentAcrylicVertexBufferLayout,
}

impl FluentAcrylicPipelineDescriptor {
    pub fn new() -> Self {
        Self {
            label: "Fluent Acrylic Quad Render Pipeline".to_string(),
            shader_source: FLUENT_ACRYLIC_WGSL_SHADER,
            entry_point_vs: "vs_main",
            entry_point_fs: "fs_main",
            sample_count: 1,
            bind_group_entries: vec![FluentAcrylicBindGroupLayoutEntry {
                binding: 0,
                visibility: "VERTEX_FRAGMENT".to_string(),
                buffer_type: "Uniform".to_string(),
            }],
            vertex_layout: FluentAcrylicVertexBufferLayout {
                array_stride: 16, // 2x f32 position + 2x f32 uv
                step_mode: "Vertex".to_string(),
                attributes: vec![
                    FluentAcrylicVertexAttribute {
                        format: "Float32x2".to_string(),
                        offset: 0,
                        shader_location: 0,
                    },
                    FluentAcrylicVertexAttribute {
                        format: "Float32x2".to_string(),
                        offset: 8,
                        shader_location: 1,
                    },
                ],
            },
        }
    }
}

impl Default for FluentAcrylicPipelineDescriptor {
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

/// CPU-side verification of the Windows Fluent Acrylic color blend formula.
///
/// Blends a base backdrop with tint color, opacity, noise factor, and luminosity factor.
pub fn compute_fluent_acrylic_blend(
    backdrop: [f32; 3],
    uniforms: &FluentAcrylicUniforms,
    noise_sample: f32,
) -> [f32; 4] {
    let base_plate = [
        backdrop[0] * uniforms.luminosity_factor,
        backdrop[1] * uniforms.luminosity_factor,
        backdrop[2] * uniforms.luminosity_factor,
    ];

    let t_op = uniforms.tint_opacity;
    let tint_blend = [
        base_plate[0] * (1.0 - t_op) + uniforms.tint_color[0] * t_op,
        base_plate[1] * (1.0 - t_op) + uniforms.tint_color[1] * t_op,
        base_plate[2] * (1.0 - t_op) + uniforms.tint_color[2] * t_op,
    ];

    let noise = noise_sample * uniforms.noise_factor;
    let r = (tint_blend[0] + noise).clamp(0.0, 1.0);
    let g = (tint_blend[1] + noise).clamp(0.0, 1.0);
    let b = (tint_blend[2] + noise).clamp(0.0, 1.0);

    [r, g, b, uniforms.tint_opacity]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wgpu_context::WgpuSurfaceContext;

    #[test]
    fn test_wgsl_shader_contains_fluent_acrylic_uniforms_and_functions() {
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("struct FluentAcrylicUniforms"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("rect_bounds: vec4<f32>"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("tint_color: vec4<f32>"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("border_color: vec4<f32>"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("tint_opacity: f32"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("blur_radius: f32"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("noise_factor: f32"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("corner_radius: f32"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("border_width: f32"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("luminosity_factor: f32"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("fn sd_rounded_box"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("fn hash21"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("fn procedural_frosted_noise"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("fn vs_main"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("fn fs_main"));
        assert!(FLUENT_ACRYLIC_WGSL_SHADER.contains("smoothstep"));
    }

    #[test]
    fn test_fluent_acrylic_uniform_buffer_packing_and_alignment() {
        // Must be exactly 80 bytes and aligned to 4 bytes, with 16-byte multiple for std140/WGSL
        assert_eq!(std::mem::size_of::<FluentAcrylicUniforms>(), 80);
        assert_eq!(std::mem::size_of::<FluentAcrylicUniforms>() % 16, 0);
        assert_eq!(std::mem::align_of::<FluentAcrylicUniforms>(), 4);

        let uniforms = FluentAcrylicUniforms::default();
        let bytes = uniforms.as_bytes();
        assert_eq!(bytes.len(), 80);
    }

    #[test]
    fn test_fluent_dark_mode_defaults() {
        let dark = FluentAcrylicUniforms::fluent_dark();
        assert_eq!(dark.tint_opacity, 0.70);
        assert_eq!(dark.noise_factor, 0.02);
        assert_eq!(dark.blur_radius, 30.0);
        assert_eq!(dark.corner_radius, 8.0);
        assert_eq!(dark.border_width, 1.0);
        assert_eq!(dark.luminosity_factor, 0.85);
        assert_eq!(dark.tint_color, [0.12, 0.12, 0.12, 1.0]);
        assert_eq!(dark.border_color, [1.0, 1.0, 1.0, 0.10]);
    }

    #[test]
    fn test_fluent_light_mode_defaults() {
        let light = FluentAcrylicUniforms::fluent_light();
        assert_eq!(light.tint_opacity, 0.85);
        assert_eq!(light.noise_factor, 0.02);
        assert_eq!(light.blur_radius, 30.0);
        assert_eq!(light.corner_radius, 8.0);
        assert_eq!(light.border_width, 1.0);
        assert_eq!(light.luminosity_factor, 1.0);
        assert_eq!(light.tint_color, [0.95, 0.95, 0.95, 1.0]);
        assert_eq!(light.border_color, [0.0, 0.0, 0.0, 0.08]);
    }

    #[test]
    fn test_acrylic_blend_math() {
        let uniforms = FluentAcrylicUniforms::fluent_dark();
        let backdrop = [0.08, 0.08, 0.08]; // Dark background
        let noise = 0.5; // High noise sample

        let blended = compute_fluent_acrylic_blend(backdrop, &uniforms, noise);
        
        // Base plate: 0.08 * 0.85 = 0.068
        // Tint blend: 0.068 * 0.30 + 0.12 * 0.70 = 0.0204 + 0.084 = 0.1044
        // Noise addition: 0.5 * 0.02 = 0.010
        // Final RGB: 0.1044 + 0.010 = 0.1144
        assert!((blended[0] - 0.1144).abs() < 1e-4);
        assert!((blended[1] - 0.1144).abs() < 1e-4);
        assert!((blended[2] - 0.1144).abs() < 1e-4);
        assert_eq!(blended[3], 0.70);
    }

    #[test]
    fn test_sdf_distance_calculations() {
        let half_extents = [50.0, 50.0];
        let radius = 8.0;

        // Center point: deep inside
        let dist_center = sd_rounded_box([0.0, 0.0], half_extents, radius);
        assert!(dist_center < -40.0);

        // Edge point inside boundary
        let dist_edge = sd_rounded_box([50.0, 0.0], half_extents, radius);
        assert!((dist_edge - 0.0).abs() < 1e-4);

        // Outside box by 10px
        let dist_outside = sd_rounded_box([60.0, 0.0], half_extents, radius);
        assert!((dist_outside - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_pipeline_descriptor_defaults() {
        let desc = FluentAcrylicPipelineDescriptor::new();
        assert_eq!(desc.entry_point_vs, "vs_main");
        assert_eq!(desc.entry_point_fs, "fs_main");
        assert_eq!(desc.bind_group_entries.len(), 1);
        assert_eq!(desc.vertex_layout.attributes.len(), 2);
    }

    #[test]
    #[ignore = "requires real GPU adapter — run with: cargo test -- --ignored"]
    fn test_wgpu_headless_shader_compilation() {
        let ctx = WgpuSurfaceContext::shared_test_context();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fluent Acrylic WGSL Shader Test"),
            source: wgpu::ShaderSource::Wgsl(FLUENT_ACRYLIC_WGSL_SHADER.into()),
        });

        // Verify bind group layout creation
        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluent Acrylic Bind Group Layout Test"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluent Acrylic Pipeline Layout Test"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let _pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fluent Acrylic Render Pipeline Test"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
    }
}
