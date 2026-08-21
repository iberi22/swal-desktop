//! GLSL Fragment Shader Definitions for Ambient Voice & Thought Reactive Orb Surface

use crate::OrbState;

/// Fragment shader source for the Listening state:
/// Pulsing cyan energy ripple (#06b6d4).
pub const LISTENING_FRAGMENT_SHADER: &str = r#"#version 450
precision highp float;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(set = 0, binding = 0) uniform OrbUniforms {
    float u_time;
    float u_audio_amplitude;
    float u_thought_trigger;
    float u_padding;
};

const vec3 CYAN_CYBER = vec3(0.023529, 0.713725, 0.831373); // #06b6d4

void main() {
    vec2 center = v_uv - vec2(0.5);
    float dist = length(center);

    // Pulsing energy ripple radial waves
    float ripple = sin(u_time * 4.0 - dist * 25.0 + u_audio_amplitude * 8.0) * 0.5 + 0.5;
    float core = smoothstep(0.48, 0.10, dist);
    float rim = smoothstep(0.50, 0.45, dist) - smoothstep(0.45, 0.40, dist);

    float intensity = (ripple * 0.6 + 0.4) * (1.0 + u_audio_amplitude * 1.5);
    vec3 color = CYAN_CYBER * intensity + vec3(rim * 0.8);

    float alpha = smoothstep(0.5, 0.47, dist) * core;
    fragColor = vec4(color, alpha);
}
"#;

/// Fragment shader source for the Thinking state:
/// Orange multi-frequency interference (#f97316).
pub const THINKING_FRAGMENT_SHADER: &str = r#"#version 450
precision highp float;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(set = 0, binding = 0) uniform OrbUniforms {
    float u_time;
    float u_audio_amplitude;
    float u_thought_trigger;
    float u_padding;
};

const vec3 ORANGE_THOUGHT = vec3(0.976471, 0.450980, 0.086275); // #f97316
const vec3 GOLD_ACCENT = vec3(1.0, 0.733333, 0.2);

void main() {
    vec2 center = v_uv - vec2(0.5);
    float dist = length(center);

    // Multi-frequency interference patterns driven by Xavier thought triggers
    float freq1 = sin(center.x * 15.0 + u_time * 5.0 + u_thought_trigger * 10.0);
    float freq2 = cos(center.y * 22.0 - u_time * 3.5);
    float freq3 = sin((center.x + center.y) * 18.0 + u_time * 7.0);

    float interference = (freq1 + freq2 + freq3) / 3.0;
    interference = smoothstep(-0.2, 0.8, interference);

    float alpha = smoothstep(0.49, 0.42, dist);
    vec3 color = mix(ORANGE_THOUGHT, GOLD_ACCENT, interference * 0.7) * (1.0 + u_thought_trigger * 0.8);

    fragColor = vec4(color, alpha * (0.8 + interference * 0.2));
}
"#;

/// Fragment shader source for the Speaking state:
/// Morphed fluid particle boundary.
pub const SPEAKING_FRAGMENT_SHADER: &str = r#"#version 450
precision highp float;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(set = 0, binding = 0) uniform OrbUniforms {
    float u_time;
    float u_audio_amplitude;
    float u_thought_trigger;
    float u_padding;
};

const vec3 EMERALD_PARTICLE = vec3(0.0, 1.0, 0.5333); // #00ff88 SWAL Accent
const vec3 CYAN_GLOW = vec3(0.023529, 0.713725, 0.831373);

void main() {
    vec2 center = v_uv - vec2(0.5);
    float angle = atan(center.y, center.x);
    float dist = length(center);

    // Fluid boundary distortion based on audio amplitude and angular harmonics
    float morph = sin(angle * 6.0 + u_time * 6.0) * 0.04 * (1.0 + u_audio_amplitude * 2.0)
                + cos(angle * 11.0 - u_time * 4.0) * 0.02 * u_audio_amplitude;

    float radius = 0.38 + morph;
    float edge = smoothstep(radius + 0.08, radius - 0.02, dist);

    float particle_noise = sin(dist * 50.0 - u_time * 12.0) * cos(angle * 8.0);
    vec3 color = mix(EMERALD_PARTICLE, CYAN_GLOW, particle_noise * 0.5 + 0.5) * (1.2 + u_audio_amplitude);

    fragColor = vec4(color, edge);
}
"#;

/// Get the GLSL fragment shader source corresponding to an `OrbState`.
pub fn get_shader_for_state(state: &OrbState) -> &'static str {
    match state {
        OrbState::Listening => LISTENING_FRAGMENT_SHADER,
        OrbState::Thinking => THINKING_FRAGMENT_SHADER,
        OrbState::Speaking => SPEAKING_FRAGMENT_SHADER,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_sources_not_empty() {
        assert!(!LISTENING_FRAGMENT_SHADER.is_empty());
        assert!(!THINKING_FRAGMENT_SHADER.is_empty());
        assert!(!SPEAKING_FRAGMENT_SHADER.is_empty());
    }

    #[test]
    fn test_hex_color_comments() {
        assert!(LISTENING_FRAGMENT_SHADER.contains("#06b6d4"));
        assert!(THINKING_FRAGMENT_SHADER.contains("#f97316"));
    }
}
