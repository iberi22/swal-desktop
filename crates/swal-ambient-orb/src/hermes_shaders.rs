//! GLSL Fragment Shader Definitions for Hermes Thinking Particle Vortex & A2UI Reactive Waves

/// Fragment shader source for Hermes Cognition Vortex:
/// Multi-octave swirling particle vortex in deep indigo (#8b5cf6) & electric cyan (#06b6d4).
pub const HERMES_COGNITION_VORTEX_SHADER: &str = r#"#version 450
precision highp float;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(set = 0, binding = 0) uniform OrbUniforms {
    float u_time;
    float u_audio_amplitude;
    float u_thought_trigger;
    float u_padding;
};

// Color definitions
const vec3 ELECTRIC_CYAN = vec3(0.023529, 0.713725, 0.831373); // #06b6d4
const vec3 DEEP_INDIGO = vec3(0.545098, 0.360784, 0.964706);   // #8b5cf6

// 2D Hash function
vec2 hash2(vec2 p) {
    p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
}

// 2D Simplex/Value Noise
float noise2d(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(dot(hash2(i + vec2(0.0, 0.0)), f - vec2(0.0, 0.0)),
            dot(hash2(i + vec2(1.0, 0.0)), f - vec2(1.0, 0.0)), u.x),
        mix(dot(hash2(i + vec2(0.0, 1.0)), f - vec2(0.0, 1.0)),
            dot(hash2(i + vec2(1.0, 1.0)), f - vec2(1.0, 1.0)), u.x), u.y);
}

// Multi-octave Fractal Brownian Motion (FBM)
float fbm(vec2 p) {
    float val = 0.0;
    float amp = 0.5;
    mat2 rot = mat2(0.8, 0.6, -0.6, 0.8);
    for (int i = 0; i < 4; i++) {
        val += amp * noise2d(p);
        p = rot * p * 2.02;
        amp *= 0.5;
    }
    return val;
}

void main() {
    vec2 st = v_uv - vec2(0.5);
    float dist = length(st);
    float angle = atan(st.y, st.x);

    // Swirling rotational warp increasing near the center
    float swirl_speed = 3.0 + u_thought_trigger * 4.0;
    float swirl_angle = angle + (1.0 / (dist + 0.1)) * 0.4 + u_time * swirl_speed;
    vec2 v_uv_swirl = vec2(cos(swirl_angle), sin(swirl_angle)) * dist;

    // Multi-octave simplex noise vortex density
    float n = fbm(v_uv_swirl * 6.0 - vec2(u_time * 1.5));
    float n2 = fbm(v_uv_swirl * 12.0 + vec2(u_time * 2.0));

    float vortex_density = smoothstep(-0.2, 0.7, n + n2 * 0.5);
    float core_glow = smoothstep(0.48, 0.05, dist);

    // Color interpolation between Deep Indigo and Electric Cyan
    vec3 color = mix(DEEP_INDIGO, ELECTRIC_CYAN, vortex_density + u_thought_trigger * 0.3);
    color *= (1.0 + u_audio_amplitude * 1.2);

    float alpha = core_glow * (0.8 + 0.2 * vortex_density);
    fragColor = vec4(color, alpha);
}
"#;

/// Fragment shader source for Hermes A2UI Stream:
/// Fluid dynamic sine wave ripple reflecting dynamic UI generation.
pub const HERMES_A2UI_STREAM_SHADER: &str = r#"#version 450
precision highp float;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(set = 0, binding = 0) uniform OrbUniforms {
    float u_time;
    float u_audio_amplitude;
    float u_thought_trigger;
    float u_padding;
};

const vec3 ELECTRIC_CYAN = vec3(0.023529, 0.713725, 0.831373); // #06b6d4
const vec3 PURPLE_STREAM = vec3(0.658824, 0.356863, 0.952941); // #a85bf4

void main() {
    vec2 st = v_uv - vec2(0.5);
    float dist = length(st);

    // Fluid dynamic sine wave ripple layers
    float wave1 = sin(st.x * 20.0 + u_time * 6.0 + sin(st.y * 10.0 + u_time * 3.0));
    float wave2 = cos(st.y * 25.0 - u_time * 4.0 + cos(st.x * 12.0 + u_time * 5.0));
    float wave3 = sin((st.x + st.y) * 15.0 + u_time * 8.0 * (1.0 + u_audio_amplitude));

    float stream = (wave1 + wave2 + wave3) / 3.0;
    stream = smoothstep(-0.3, 0.7, stream);

    float boundary = smoothstep(0.49, 0.35, dist);
    vec3 color = mix(ELECTRIC_CYAN, PURPLE_STREAM, stream) * (1.1 + u_thought_trigger * 0.5);

    fragColor = vec4(color, boundary * (0.7 + stream * 0.3));
}
"#;

/// Fragment shader source for Hermes Idle Breathe:
/// Organic soft breathing gradient for ambient desktop state.
pub const HERMES_IDLE_BREATHE_SHADER: &str = r#"#version 450
precision highp float;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(set = 0, binding = 0) uniform OrbUniforms {
    float u_time;
    float u_audio_amplitude;
    float u_thought_trigger;
    float u_padding;
};

const vec3 INDIGO_SOFT = vec3(0.329412, 0.278431, 0.823529); // #5447d2
const vec3 CYAN_SOFT   = vec3(0.023529, 0.713725, 0.831373); // #06b6d4

void main() {
    vec2 st = v_uv - vec2(0.5);
    float dist = length(st);

    // Organic soft breathing cycle
    float breathe = sin(u_time * 1.8) * 0.5 + 0.5;
    float radius = 0.35 + breathe * 0.05 + u_audio_amplitude * 0.08;

    float gradient = smoothstep(radius + 0.1, radius - 0.1, dist);
    vec3 color = mix(INDIGO_SOFT, CYAN_SOFT, breathe * 0.6 + dist * 0.4);

    fragColor = vec4(color, gradient * 0.6);
}
"#;

/// Get the GLSL fragment shader source corresponding to Hermes state string.
pub fn get_hermes_shader(state: &str) -> &'static str {
    match state.to_lowercase().as_str() {
        "vortex" | "cognition_vortex" | "hermes_cognition_vortex" | "thinking" => {
            HERMES_COGNITION_VORTEX_SHADER
        }
        "stream" | "a2ui_stream" | "hermes_a2ui_stream" | "a2ui" => HERMES_A2UI_STREAM_SHADER,
        "breathe" | "idle_breathe" | "hermes_idle_breathe" | "idle" => HERMES_IDLE_BREATHE_SHADER,
        _ => HERMES_IDLE_BREATHE_SHADER,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermes_shader_sources_not_empty() {
        assert!(!HERMES_COGNITION_VORTEX_SHADER.is_empty());
        assert!(!HERMES_A2UI_STREAM_SHADER.is_empty());
        assert!(!HERMES_IDLE_BREATHE_SHADER.is_empty());
    }

    #[test]
    fn test_hermes_hex_color_comments() {
        assert!(HERMES_COGNITION_VORTEX_SHADER.contains("#06b6d4"));
        assert!(HERMES_COGNITION_VORTEX_SHADER.contains("#8b5cf6"));
    }

    #[test]
    fn test_get_hermes_shader() {
        assert_eq!(get_hermes_shader("vortex"), HERMES_COGNITION_VORTEX_SHADER);
        assert_eq!(
            get_hermes_shader("cognition_vortex"),
            HERMES_COGNITION_VORTEX_SHADER
        );
        assert_eq!(get_hermes_shader("stream"), HERMES_A2UI_STREAM_SHADER);
        assert_eq!(get_hermes_shader("a2ui_stream"), HERMES_A2UI_STREAM_SHADER);
        assert_eq!(get_hermes_shader("breathe"), HERMES_IDLE_BREATHE_SHADER);
        assert_eq!(get_hermes_shader("idle_breathe"), HERMES_IDLE_BREATHE_SHADER);
        assert_eq!(get_hermes_shader("unknown"), HERMES_IDLE_BREATHE_SHADER);
    }
}
