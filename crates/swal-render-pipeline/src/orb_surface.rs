//! Hardware-Accelerated Hermes Ambient Orb Render Surface (Zero-Eww)
//!
//! Provides WGPU-based direct GPU rendering for the Hermes Ambient Orb,
//! compiling WGSL ports of the Hermes Cognition Vortex and A2UI Stream shaders.

use bytemuck::{Pod, Zeroable};
use swal_ambient_orb::hermes::{HermesAgentState, HermesOrbPacket};
use wgpu::util::DeviceExt;

/// WGSL Port of HERMES_COGNITION_VORTEX_SHADER (3D Volumetric Raymarching + Optical Translucency)
pub const HERMES_COGNITION_VORTEX_WGSL: &str = r#"
struct OrbUniforms {
    u_time: f32,
    u_audio_amplitude: f32,
    u_thought_trigger: f32,
    u_state_id: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: OrbUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

// 3D Rotation matrices
fn rot_y(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c, 0.0, s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-s, 0.0, c)
    );
}

fn rot_z(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c, -s, 0.0),
        vec3<f32>(s, c, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );
}

// 3D Simplex-inspired procedural noise
fn hash3(p: vec3<f32>) -> f32 {
    let p3 = fract(p * 0.1031);
    let d = dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z + d);
}

fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash3(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash3(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash3(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash3(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash3(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash3(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash3(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash3(i + vec3<f32>(1.0, 1.0, 1.0));

    let x00 = mix(n000, n100, u.x);
    let x10 = mix(n010, n110, u.x);
    let x01 = mix(n001, n101, u.x);
    let x11 = mix(n011, n111, u.x);

    let y0 = mix(x00, x10, u.y);
    let y1 = mix(x01, x11, u.y);

    return mix(y0, y1, u.z);
}

fn fbm3d(p_in: vec3<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var p = p_in;
    for (var i = 0; i < 4; i = i + 1) {
        val += amp * noise3d(p);
        p = p * 2.02 + vec3<f32>(0.12, 0.34, 0.56);
        amp *= 0.5;
    }
    return val;
}

// 3D Sphere Distance Function
fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// Color palettes for Hermes Cognitive States
const CYAN_CORE: vec3<f32>    = vec3<f32>(0.02, 0.94, 1.0);  // #05f0ff (Listening/Idle)
const INDIGO_VORTEX: vec3<f32> = vec3<f32>(0.55, 0.25, 0.98); // #8c40fa (Deep Cognition)
const EMERALD_PULSE: vec3<f32> = vec3<f32>(0.0, 1.0, 0.53);   // #00ff88 (Active Voice)
const SOLAR_AMBER: vec3<f32>   = vec3<f32>(1.0, 0.62, 0.04);  // #fb9e0b (Thinking GraphRAG)
const CRIMSON_ALERT: vec3<f32> = vec3<f32>(0.94, 0.27, 0.27); // #ef4444 (Error Alert)

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Center UV coordinates to [-1.0, 1.0]
    let uv = (in.uv - vec2<f32>(0.5)) * 2.0;
    let dist2d = length(uv);

    // Outside bounding sphere disc + atmosphere radius -> discard to maintain 100% transparent background
    if (dist2d > 0.98) {
        discard;
    }

    let time = uniforms.u_time;
    let audio = clamp(uniforms.u_audio_amplitude, 0.0, 1.0);
    let thought = clamp(uniforms.u_thought_trigger, 0.0, 1.0);
    let state_id = uniforms.u_state_id;

    // Ray setup: Camera origin and ray direction
    let ro = vec3<f32>(0.0, 0.0, 2.0);
    let rd = normalize(vec3<f32>(uv.x, -uv.y, -1.6));

    // Dynamic rotation speed based on thought trigger & audio
    let swirl_speed = 0.8 + thought * 3.5 + audio * 1.5;
    let rot = rot_y(time * swirl_speed) * rot_z(time * 0.4);

    let sphere_radius = 0.72 + audio * 0.08 + sin(time * 2.0) * 0.015;

    // Volumetric Raymarching loop (16 steps through the orb volume)
    var density_accum = 0.0;
    var emission_accum = vec3<f32>(0.0);
    var t = 1.0;
    let max_t = 3.0;
    let step_size = 0.08;

    for (var i = 0; i < 18; i = i + 1) {
        if (t > max_t) {
            break;
        }

        let p = ro + rd * t;
        let p_rot = rot * p;

        // Base sphere SDF + organic 3D turbulence displacement
        let d = sd_sphere(p, sphere_radius);
        let turb = fbm3d(p_rot * 3.2 - vec3<f32>(0.0, time * 1.2, 0.0));
        let turb2 = fbm3d(p_rot * 6.5 + vec3<f32>(time * 0.8, 0.0, time * 0.5));

        let volume_dist = d + (turb - 0.5) * 0.35 + (turb2 - 0.5) * 0.15;

        if (volume_dist < 0.08) {
            let sample_density = smoothstep(0.08, -0.25, volume_dist);
            
            // State-based dynamic color mixing
            var plasma_col = mix(INDIGO_VORTEX, CYAN_CORE, turb);
            
            // Listening State: Boost Emerald & Cyan
            if (state_id >= 0.5 && state_id < 1.5) {
                plasma_col = mix(CYAN_CORE, EMERALD_PULSE, turb2 + audio * 0.5);
            }
            // Thinking / Decomposing Plan: Solar Gold + White hot core
            else if (state_id >= 1.5 && state_id < 2.5) {
                plasma_col = mix(SOLAR_AMBER, vec3<f32>(1.0, 0.95, 0.85), turb2 + thought * 0.4);
            }
            // Error Alert:
            else if (state_id >= 5.5) {
                plasma_col = mix(CRIMSON_ALERT, vec3<f32>(1.0, 0.4, 0.2), turb);
            }

            // Spectral Chromatic Aberration at glass boundary
            let dispersion = vec3<f32>(1.0 + turb * 0.15, 1.0, 1.0 - turb * 0.15);
            plasma_col *= dispersion;

            let step_light = plasma_col * sample_density * (1.0 + audio * 1.4);
            emission_accum += step_light * (1.0 - density_accum) * 0.24;
            density_accum += sample_density * 0.14;

            if (density_accum >= 0.95) {
                break;
            }
        }

        t += step_size;
    }

    // Optical Fresnel Rim Glow (Holographic Glass Edge)
    let normal_approx = normalize(vec3<f32>(uv, sqrt(max(0.0, 1.0 - dist2d * dist2d))));
    let fresnel = pow(1.0 - max(dot(normal_approx, -rd), 0.0), 3.2);

    var rim_color = mix(CYAN_CORE, INDIGO_VORTEX, sin(time * 2.0) * 0.5 + 0.5);
    if (state_id >= 1.5 && state_id < 2.5) {
        rim_color = mix(SOLAR_AMBER, vec3<f32>(1.0, 0.8, 0.2), fresnel);
    } else if (state_id >= 0.5 && state_id < 1.5) {
        rim_color = mix(CYAN_CORE, EMERALD_PULSE, audio);
    }

    // Soft outer atmospheric glow
    let atmosphere = smoothstep(0.92, 0.45, dist2d) * (0.3 + 0.7 * fresnel);
    let final_rgb = emission_accum + rim_color * fresnel * 0.8 + rim_color * atmosphere * 0.2;

    // Translucent core with solid rim
    let final_alpha = clamp(density_accum * 0.85 + fresnel * 0.9 + atmosphere * 0.35, 0.0, 1.0);

    return vec4<f32>(final_rgb * final_alpha, final_alpha);
}
"#;

/// WGSL Port of HERMES_A2UI_STREAM_SHADER
pub const HERMES_A2UI_STREAM_WGSL: &str = r#"
struct OrbUniforms {
    u_time: f32,
    u_audio_amplitude: f32,
    u_thought_trigger: f32,
    u_state_id: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: OrbUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

const ELECTRIC_CYAN: vec3<f32> = vec3<f32>(0.023529, 0.713725, 0.831373); // #06b6d4
const PURPLE_STREAM: vec3<f32> = vec3<f32>(0.658824, 0.356863, 0.952941); // #a85bf4

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let st = in.uv - vec2<f32>(0.5);
    let dist = length(st);

    let wave1 = sin(st.x * 20.0 + uniforms.u_time * 6.0 + sin(st.y * 10.0 + uniforms.u_time * 3.0));
    let wave2 = cos(st.y * 25.0 - uniforms.u_time * 4.0 + cos(st.x * 12.0 + uniforms.u_time * 5.0));
    let wave3 = sin((st.x + st.y) * 15.0 + uniforms.u_time * 8.0 * (1.0 + uniforms.u_audio_amplitude));

    var stream = (wave1 + wave2 + wave3) / 3.0;
    stream = smoothstep(-0.3, 0.7, stream);

    let boundary = smoothstep(0.49, 0.35, dist);
    let color = mix(ELECTRIC_CYAN, PURPLE_STREAM, stream) * (1.1 + uniforms.u_thought_trigger * 0.5);

    return vec4<f32>(color, boundary * (0.7 + stream * 0.3));
}
"#;

/// Vertex structure representing a 2D quad vertex with position and texture coordinates.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct OrbVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl OrbVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OrbVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Fullscreen Quad Vertices (2 triangles, 6 vertices)
pub const QUAD_VERTICES: &[OrbVertex] = &[
    OrbVertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
    OrbVertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
    OrbVertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
    OrbVertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
    OrbVertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
    OrbVertex { position: [-1.0, 1.0], uv: [0.0, 0.0] },
];

/// Uniform buffer layout matching shader bindings.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct OrbUniforms {
    pub time: f32,
    pub audio_amplitude: f32,
    pub thought_trigger: f32,
    pub state_id: f32,
}

impl Default for OrbUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            audio_amplitude: 0.0,
            thought_trigger: 0.0,
            state_id: 0.0,
        }
    }
}

/// Hardware-Accelerated Hermes Ambient Orb Render Surface
pub struct HermesOrbRenderSurface {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniforms: OrbUniforms,
}

impl HermesOrbRenderSurface {
    /// Helper mapping HermesAgentState to numeric state_id float
    pub fn state_id_for_state(state: HermesAgentState) -> f32 {
        match state {
            HermesAgentState::Idle => 0.0,
            HermesAgentState::ListeningVoice => 1.0,
            HermesAgentState::DecomposingPlan => 2.0,
            HermesAgentState::StreamingA2Ui => 3.0,
            HermesAgentState::ExecutingToolAction => 4.0,
            HermesAgentState::AwaitingUserFeedback => 5.0,
            HermesAgentState::ErrorAlert => 6.0,
        }
    }

    /// Creates a new `HermesOrbRenderSurface` on the given WGPU device and texture target format.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hermes Cognition Vortex Shader"),
            source: wgpu::ShaderSource::Wgsl(HERMES_COGNITION_VORTEX_WGSL.into()),
        });

        let uniforms = OrbUniforms::default();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hermes Orb Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hermes Orb Bind Group Layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hermes Orb Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hermes Orb Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hermes Orb Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[OrbVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hermes Orb Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            uniforms,
        }
    }

    /// Access underlying uniform values
    pub fn uniforms(&self) -> &OrbUniforms {
        &self.uniforms
    }

    /// Update uniforms from incoming telemetry `HermesOrbPacket`
    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, packet: &HermesOrbPacket) {
        self.uniforms.audio_amplitude = packet.audio_level.clamp(0.0, 1.0);
        self.uniforms.thought_trigger = packet.progress_pct.clamp(0.0, 1.0);
        self.uniforms.state_id = Self::state_id_for_state(packet.state);

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }

    /// Advance animation time and flush updated uniform buffer to GPU queue
    pub fn tick_animation(&mut self, queue: &wgpu::Queue, delta_time: f32) {
        self.uniforms.time += delta_time;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }

    /// Record draw commands rendering quad into target texture view
    pub fn render_to_texture<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        target_view: &'a wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hermes Orb Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });


        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..QUAD_VERTICES.len() as u32, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_id_mappings() {
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::Idle), 0.0);
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::ListeningVoice), 1.0);
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::DecomposingPlan), 2.0);
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::StreamingA2Ui), 3.0);
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::ExecutingToolAction), 4.0);
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::AwaitingUserFeedback), 5.0);
        assert_eq!(HermesOrbRenderSurface::state_id_for_state(HermesAgentState::ErrorAlert), 6.0);
    }

    #[test]
    fn test_orb_uniforms_default_and_pod() {
        let uniforms = OrbUniforms::default();
        assert_eq!(uniforms.time, 0.0);
        assert_eq!(uniforms.audio_amplitude, 0.0);
        assert_eq!(uniforms.thought_trigger, 0.0);
        assert_eq!(uniforms.state_id, 0.0);

        let bytes = bytemuck::bytes_of(&uniforms);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_quad_vertices() {
        assert_eq!(QUAD_VERTICES.len(), 6);
        let layout = OrbVertex::layout();
        assert_eq!(layout.array_stride, 16);
        assert_eq!(layout.attributes.len(), 2);
    }

    #[test]
    fn test_wgsl_shader_strings_validity() {
        assert!(!HERMES_COGNITION_VORTEX_WGSL.is_empty());
        assert!(!HERMES_A2UI_STREAM_WGSL.is_empty());
        assert!(HERMES_COGNITION_VORTEX_WGSL.contains("vs_main"));
        assert!(HERMES_COGNITION_VORTEX_WGSL.contains("fs_main"));
        assert!(HERMES_A2UI_STREAM_WGSL.contains("vs_main"));
        assert!(HERMES_A2UI_STREAM_WGSL.contains("fs_main"));
    }

    #[test]
    #[ignore = "requires real GPU adapter — run with: cargo test -- --ignored"]
    fn test_wgpu_headless_pipeline_and_uniform_updates() {
        use crate::wgpu_context::WgpuSurfaceContext;
        let ctx = WgpuSurfaceContext::shared_test_context();
        let mut surface = HermesOrbRenderSurface::new(&ctx.device, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(surface.uniforms().time, 0.0);

        // Test tick_animation
        surface.tick_animation(&ctx.queue, 0.016);
        assert!((surface.uniforms().time - 0.016).abs() < 0.001);

        // Test update_uniforms with HermesOrbPacket
        let packet = HermesOrbPacket::new(HermesAgentState::StreamingA2Ui)
            .with_audio_level(0.75)
            .with_progress(0.9);

        surface.update_uniforms(&ctx.queue, &packet);
        assert_eq!(surface.uniforms().state_id, 3.0);
        assert!((surface.uniforms().audio_amplitude - 0.75).abs() < 0.001);
        assert!((surface.uniforms().thought_trigger - 0.9).abs() < 0.001);

        // Test render_to_texture
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Test Render Target"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Test Encoder"),
        });

        surface.render_to_texture(&mut encoder, &target_view);
        ctx.queue.submit(Some(encoder.finish()));
        ctx.device.poll(wgpu::Maintain::Poll);
    }
}

