//! Procedural WGSL Fluid Particle Dispersion & SDF Boundary Morphing Shader for Speaking State
//!
//! Provides the WebGPU Shading Language (WGSL) shader definitions, uniform data structures,
//! CPU reference implementations for Signed Distance Field (SDF) boundary morphing and fluid
//! particle dispersion, and state lookup helpers for the SWAL Ambient Orb surface.

use crate::OrbState;
use serde::{Deserialize, Serialize};

/// Hex color code for the SWAL Emerald Accent particle core
pub const EMERALD_PARTICLE_HEX: &str = "#00ff88";

/// Hex color code for the Cyber Cyan outer glow
pub const CYAN_GLOW_HEX: &str = "#06b6d4";

/// WGSL shader source for the Speaking state:
/// Procedural fluid particle dispersion and dynamic SDF boundary morphing
/// reactive to audio amplitude, speech harmonics, and thought triggers.
pub const SPEAKING_PARTICLE_WGSL_SHADER: &str = r#"
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
    @location(1) frag_pos: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    out.frag_pos = model.position;
    return out;
}

// Color palette constants
const EMERALD_PARTICLE: vec3<f32> = vec3<f32>(0.0, 1.0, 0.5333);   // #00ff88 SWAL Emerald Accent
const CYAN_GLOW: vec3<f32>        = vec3<f32>(0.023529, 0.713725, 0.831373); // #06b6d4 Cyber Cyan
const ELECTRIC_MINT: vec3<f32>    = vec3<f32>(0.2, 1.0, 0.75);      // #33ffc0 Mint Highlight
const DEEP_CORE: vec3<f32>        = vec3<f32>(0.01, 0.12, 0.08);    // Deep background glow

// 2D Hash function for procedural pseudorandom noise
fn hash2(p: vec2<f32>) -> vec2<f32> {
    let p_dot = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(p_dot) * 43758.5453123);
}

// 2D Simplex/Value Noise with cubic Hermite interpolation
fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let d00 = dot(hash2(i + vec2<f32>(0.0, 0.0)), f - vec2<f32>(0.0, 0.0));
    let d10 = dot(hash2(i + vec2<f32>(1.0, 0.0)), f - vec2<f32>(1.0, 0.0));
    let d01 = dot(hash2(i + vec2<f32>(0.0, 1.0)), f - vec2<f32>(0.0, 1.0));
    let d11 = dot(hash2(i + vec2<f32>(1.0, 1.0)), f - vec2<f32>(1.0, 1.0));

    return mix(mix(d00, d10, u.x), mix(d01, d11, u.x), u.y);
}

// Multi-octave Fractal Brownian Motion (FBM) for fluid turbulence
fn fbm(p_in: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var p = p_in;
    let rot = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
    for (var i = 0; i < 4; i = i + 1) {
        val += amp * noise2d(p);
        p = rot * p * 2.02;
        amp *= 0.5;
    }
    return val;
}

// Signed Distance Field calculation with fluid boundary morphing and harmonic wave deformations
fn sd_morphed_boundary(p: vec2<f32>, base_radius: f32, time: f32, amplitude: f32) -> f32 {
    let angle = atan2(p.y, p.x);
    let r = length(p);

    // Dynamic angular harmonics driven by voice amplitude
    let h1 = sin(angle * 6.0 + time * 6.0) * 0.04 * (1.0 + amplitude * 2.0);
    let h2 = cos(angle * 11.0 - time * 4.0) * 0.02 * amplitude;
    let h3 = sin(angle * 18.0 + time * 8.0) * 0.012 * amplitude;

    // Organic fluid turbulent deformation
    let turbulence = fbm(p * 4.0 + vec2<f32>(time * 0.8, -time * 0.5)) * 0.04 * (1.0 + amplitude * 1.5);

    let morphed_r = base_radius + h1 + h2 + h3 + turbulence;
    return r - morphed_r;
}

// Procedural fluid particle dispersion simulation
fn particle_dispersion(p: vec2<f32>, time: f32, amplitude: f32) -> f32 {
    let angle = atan2(p.y, p.x);
    let dist = length(p);

    // Radial wave expansion and outward burst
    let radial_wave = sin(dist * 40.0 - time * 12.0 + amplitude * 6.0) * 0.5 + 0.5;
    let angular_pulse = cos(angle * 8.0 + time * 4.0) * 0.5 + 0.5;

    // Micro-scale particulate noise coordinates
    let noise_coord = vec2<f32>(angle * 3.0, dist * 10.0 - time * 3.0);
    let particulate = fbm(noise_coord * 2.5);

    let dispersion = smoothstep(-0.2, 0.8, radial_wave * 0.4 + angular_pulse * 0.3 + particulate * 0.5);
    return dispersion;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv - vec2<f32>(0.5);
    let dist = length(p);

    // Dynamic base radius expanding slightly with voice amplitude
    let base_radius = 0.38 + uniforms.u_audio_amplitude * 0.05;
    let sdf = sd_morphed_boundary(p, base_radius, uniforms.u_time, uniforms.u_audio_amplitude);

    // Antialiased edge transition with acoustic softness
    let edge_softness = 0.015 + uniforms.u_audio_amplitude * 0.02;
    let core_mask = 1.0 - smoothstep(-edge_softness, edge_softness, sdf);

    // Outer luminous bloom / aura
    let outer_glow = exp(-max(sdf, 0.0) * 16.0) * (0.35 + uniforms.u_audio_amplitude * 0.65);

    // Fluid particle dispersion calculation
    let dispersion = particle_dispersion(p, uniforms.u_time, uniforms.u_audio_amplitude);

    // Multi-tone spectral blend between Emerald (#00ff88), Cyan (#06b6d4), and Mint
    let color_mix = mix(EMERALD_PARTICLE, CYAN_GLOW, dispersion * 0.6 + uniforms.u_thought_trigger * 0.3);
    let highlight = mix(color_mix, ELECTRIC_MINT, smoothstep(0.6, 1.0, dispersion) * 0.4);

    // Rim energy boundary highlight
    let rim_accent = smoothstep(0.06, -0.01, abs(sdf)) * 0.6 * (1.0 + uniforms.u_audio_amplitude);

    var final_color = (highlight + vec3<f32>(rim_accent)) * (1.1 + uniforms.u_audio_amplitude * 0.8);
    let alpha = clamp(core_mask * 0.92 + outer_glow * 0.58, 0.0, 1.0);

    return vec4<f32>(final_color, alpha);
}
"#;

/// Alias for `SPEAKING_PARTICLE_WGSL_SHADER`
pub const SPEAKING_WGSL_SHADER: &str = SPEAKING_PARTICLE_WGSL_SHADER;

/// 16-byte aligned particle shader uniform parameters
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParticleShaderUniforms {
    pub time: f32,
    pub audio_amplitude: f32,
    pub thought_trigger: f32,
    pub state_id: f32,
}

impl Default for ParticleShaderUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            audio_amplitude: 0.0,
            thought_trigger: 0.0,
            state_id: 2.0, // Speaking state ID
        }
    }
}

impl ParticleShaderUniforms {
    pub fn new(time: f32, audio_amplitude: f32, thought_trigger: f32) -> Self {
        Self {
            time,
            audio_amplitude: audio_amplitude.clamp(0.0, 1.0),
            thought_trigger: thought_trigger.clamp(0.0, 1.0),
            state_id: 2.0,
        }
    }

    /// Converts uniforms directly to raw byte slice for GPU buffer writes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

/// Retrieve the WGSL shader source for the Speaking particle dispersion orb state
pub fn get_speaking_wgsl_shader() -> &'static str {
    SPEAKING_PARTICLE_WGSL_SHADER
}

/// Retrieve WGSL shader if available for the given `OrbState`
pub fn get_wgsl_shader_for_state(state: &OrbState) -> Option<&'static str> {
    match state {
        OrbState::Speaking => Some(SPEAKING_PARTICLE_WGSL_SHADER),
        _ => None,
    }
}

/// CPU-side 2D distance to circle boundary: `length(p) - r`
pub fn cpu_sd_circle(p: [f32; 2], r: f32) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt() - r
}

/// CPU-side reference calculation of morphed SDF boundary for the Speaking state
pub fn cpu_sd_morphed_boundary(p: [f32; 2], base_radius: f32, time: f32, amplitude: f32) -> f32 {
    let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
    let angle = p[1].atan2(p[0]);

    let h1 = (angle * 6.0 + time * 6.0).sin() * 0.04 * (1.0 + amplitude * 2.0);
    let h2 = (angle * 11.0 - time * 4.0).cos() * 0.02 * amplitude;
    let h3 = (angle * 18.0 + time * 8.0).sin() * 0.012 * amplitude;

    let morphed_r = base_radius + h1 + h2 + h3;
    r - morphed_r
}

/// CPU-side reference calculation of particle dispersion intensity [0.0, 1.0]
pub fn cpu_particle_dispersion(p: [f32; 2], time: f32, amplitude: f32) -> f32 {
    let dist = (p[0] * p[0] + p[1] * p[1]).sqrt();
    let angle = p[1].atan2(p[0]);

    let radial_wave = (dist * 40.0 - time * 12.0 + amplitude * 6.0).sin() * 0.5 + 0.5;
    let angular_pulse = (angle * 8.0 + time * 4.0).cos() * 0.5 + 0.5;

    let combined = radial_wave * 0.5 + angular_pulse * 0.5;
    combined.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naga_wgsl_syntax_validation() {
        let module_result = naga::front::wgsl::parse_str(SPEAKING_PARTICLE_WGSL_SHADER);
        match module_result {
            Ok(module) => {
                // Ensure vertex and fragment entry points are present and recognized by Naga
                let mut found_vs = false;
                let mut found_fs = false;
                for ep in &module.entry_points {
                    if ep.name == "vs_main" && ep.stage == naga::ShaderStage::Vertex {
                        found_vs = true;
                    }
                    if ep.name == "fs_main" && ep.stage == naga::ShaderStage::Fragment {
                        found_fs = true;
                    }
                }
                assert!(found_vs, "vs_main vertex entry point missing in Naga module");
                assert!(found_fs, "fs_main fragment entry point missing in Naga module");
            }
            Err(e) => {
                panic!("Naga WGSL syntax validation failed: {}", e.emit_to_string(SPEAKING_PARTICLE_WGSL_SHADER));
            }
        }
    }

    #[test]
    fn test_shader_source_contents_and_brand_colors() {
        assert!(!SPEAKING_PARTICLE_WGSL_SHADER.is_empty());
        assert_eq!(SPEAKING_WGSL_SHADER, SPEAKING_PARTICLE_WGSL_SHADER);
        assert_eq!(get_speaking_wgsl_shader(), SPEAKING_PARTICLE_WGSL_SHADER);

        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("struct OrbUniforms"));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("@group(0) @binding(0)"));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("fn sd_morphed_boundary"));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("fn particle_dispersion"));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("fn fbm"));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("fn noise2d"));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains("fn hash2"));

        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains(EMERALD_PARTICLE_HEX));
        assert!(SPEAKING_PARTICLE_WGSL_SHADER.contains(CYAN_GLOW_HEX));
    }

    #[test]
    fn test_state_shader_lookup() {
        assert_eq!(get_wgsl_shader_for_state(&OrbState::Speaking), Some(SPEAKING_PARTICLE_WGSL_SHADER));
        assert_eq!(get_wgsl_shader_for_state(&OrbState::Listening), None);
        assert_eq!(get_wgsl_shader_for_state(&OrbState::Thinking), None);
    }

    #[test]
    fn test_uniforms_layout_and_methods() {
        assert_eq!(std::mem::size_of::<ParticleShaderUniforms>(), 16);
        assert_eq!(std::mem::align_of::<ParticleShaderUniforms>(), 4);

        let default_uniforms = ParticleShaderUniforms::default();
        assert_eq!(default_uniforms.time, 0.0);
        assert_eq!(default_uniforms.audio_amplitude, 0.0);
        assert_eq!(default_uniforms.thought_trigger, 0.0);
        assert_eq!(default_uniforms.state_id, 2.0);

        let custom_uniforms = ParticleShaderUniforms::new(1.5, 1.2, -0.5);
        assert_eq!(custom_uniforms.time, 1.5);
        assert_eq!(custom_uniforms.audio_amplitude, 1.0); // Clamped to 1.0
        assert_eq!(custom_uniforms.thought_trigger, 0.0); // Clamped to 0.0
        assert_eq!(custom_uniforms.state_id, 2.0);

        let bytes = custom_uniforms.as_bytes();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_cpu_sdf_and_particle_math() {
        let base_radius = 0.38;
        
        // Deep inside the orb center
        let dist_center = cpu_sd_morphed_boundary([0.0, 0.0], base_radius, 0.0, 0.0);
        assert!(dist_center < -0.30);

        // Outside the orb boundary
        let dist_outside = cpu_sd_morphed_boundary([0.8, 0.0], base_radius, 0.0, 0.0);
        assert!(dist_outside > 0.30);

        // Near boundary
        let dist_boundary = cpu_sd_morphed_boundary([0.38, 0.0], base_radius, 0.0, 0.0);
        assert!(dist_boundary.abs() < 0.05);

        // Amplitude increases harmonic distortion amplitude
        let dist_quiet = cpu_sd_morphed_boundary([0.38, 0.0], base_radius, 1.0, 0.0);
        let dist_loud = cpu_sd_morphed_boundary([0.38, 0.0], base_radius, 1.0, 1.0);
        assert_ne!(dist_quiet, dist_loud);

        // Particle dispersion values remain within valid [0.0, 1.0] bounds
        for angle_deg in (0..360).step_by(30) {
            let rad = (angle_deg as f32) * std::f32::consts::PI / 180.0;
            let p = [rad.cos() * 0.3, rad.sin() * 0.3];
            let disp = cpu_particle_dispersion(p, 0.5, 0.7);
            assert!(disp >= 0.0 && disp <= 1.0, "Dispersion {} out of range", disp);
        }

        // Circle distance test
        assert!((cpu_sd_circle([0.5, 0.0], 0.5) - 0.0).abs() < 1e-5);
    }
}
