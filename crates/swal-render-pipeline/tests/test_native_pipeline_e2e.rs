//! E2E Integration Test Suite for Pure Rust Native Desktop Pipeline
//! SWAL Ola 4.10 - Native Wayland Layer Shell & GPU Rendering Engine

use std::time::Duration;
use swal_ambient_orb::{LockFreeAudioConsumer, OrbController, OrbInputSignal, OrbState};
use swal_render_pipeline::FrameScheduler;

/// Layer Shell Anchor flags representing Wayland zwlr_layer_surface_v1 anchors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerAnchors {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayerShellLayer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone)]
pub struct LayerSurfaceConfig {
    pub namespace: String,
    pub layer: LayerShellLayer,
    pub width: u32,
    pub height: u32,
    pub anchors: LayerAnchors,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub exclusive_zone: i32,
    pub keyboard_interactivity: bool,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            namespace: "swal-desktop-overlay".to_string(),
            layer: LayerShellLayer::Overlay,
            width: 1920,
            height: 1080,
            anchors: LayerAnchors {
                top: true,
                bottom: true,
                left: true,
                right: true,
            },
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
            exclusive_zone: -1,
            keyboard_interactivity: false,
        }
    }
}

/// Simulated WGSL Mica Shader Uniform buffer packed according to std140 layout
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicaUniformBuffer {
    pub tint_color: [f32; 4],
    pub opacity: f32,
    pub noise_scale: f32,
    pub blur_radius: f32,
    pub _padding0: f32,
    pub resolution: [f32; 2],
    pub time: f32,
    pub _padding1: f32,
}

impl MicaUniformBuffer {
    pub fn new(tint_color: [f32; 4], opacity: f32, noise_scale: f32, blur_radius: f32, resolution: [f32; 2], time: f32) -> Self {
        Self {
            tint_color,
            opacity,
            noise_scale,
            blur_radius,
            _padding0: 0.0,
            resolution,
            time,
            _padding1: 0.0,
        }
    }

    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

/// Spatial Bounding Box for Hit Testing
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= (self.x + self.width) && py >= self.y && py <= (self.y + self.height)
    }
}

/// Simulated Headless Glyph Renderer for Text Layout Measurements
pub struct GlyphRenderer {
    pub font_size: f32,
    pub line_height: f32,
    pub char_width: f32,
}

impl GlyphRenderer {
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            line_height: font_size * 1.25,
            char_width: font_size * 0.60,
        }
    }

    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let lines: Vec<&str> = text.lines().collect();
        let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let width = max_len as f32 * self.char_width;
        let height = lines.len().max(1) as f32 * self.line_height;
        (width, height)
    }
}

#[test]
fn test_layer_surface_config_and_anchors() {
    let mut config = LayerSurfaceConfig::default();
    assert_eq!(config.namespace, "swal-desktop-overlay");
    assert_eq!(config.layer, LayerShellLayer::Overlay);
    assert!(config.anchors.top && config.anchors.bottom && config.anchors.left && config.anchors.right);

    // Reconfigure for top status bar layer shell
    config.namespace = "swal-status-bar".to_string();
    config.layer = LayerShellLayer::Top;
    config.height = 36;
    config.anchors = LayerAnchors {
        top: true,
        bottom: false,
        left: true,
        right: true,
    };
    config.exclusive_zone = 36;

    assert_eq!(config.namespace, "swal-status-bar");
    assert_eq!(config.layer, LayerShellLayer::Top);
    assert_eq!(config.height, 36);
    assert!(!config.anchors.bottom);
    assert_eq!(config.exclusive_zone, 36);
}

#[test]
fn test_wgpu_context_headless_render_tick_200hz() {
    let scheduler = FrameScheduler::new(200); // 200Hz tick rate = 5ms frame budget
    assert_eq!(scheduler.frame_budget, Duration::from_millis(5));

    let mut mock_gpu_pass_count = 0;

    for _ in 0..10 {
        let (elapsed, within_budget) = scheduler.benchmark_render_tick(|| {
            // Simulated Headless WGPU Command Encoder & Render Pass execution
            mock_gpu_pass_count += 1;
            std::thread::sleep(Duration::from_micros(100)); // 0.1ms simulated GPU load
        });

        assert!(within_budget, "Headless render tick must complete within 5.0ms budget");
        assert!(elapsed < Duration::from_millis(4));
    }

    assert_eq!(mock_gpu_pass_count, 10);
    assert_eq!(scheduler.total_frames(), 10);
}

#[test]
fn test_mica_shader_uniform_packing() {
    let uniform = MicaUniformBuffer::new(
        [0.1, 0.2, 0.3, 0.85], // tint_color (RGBA)
        0.92,                  // opacity
        1.5,                   // noise_scale
        24.0,                  // blur_radius
        [1920.0, 1080.0],      // resolution
        16.4,                  // time
    );

    assert_eq!(std::mem::size_of::<MicaUniformBuffer>(), 48); // 12 x 4 bytes = 48 bytes (16-byte aligned)
    let bytes = uniform.to_bytes();
    assert_eq!(bytes.len(), 48);

    // Verify correct byte layout for tint color float components
    let tint_r = f32::from_ne_bytes(bytes[0..4].try_into().unwrap());
    assert!((tint_r - 0.1).abs() < f32::EPSILON);
}

#[test]
fn test_orb_surface_hermes_state_rendering() {
    let audio_consumer = LockFreeAudioConsumer::new();
    let mut orb_controller = OrbController::new(audio_consumer.clone());
    let scheduler = FrameScheduler::new(200);

    // Default state: Listening
    assert_eq!(orb_controller.state(), OrbState::Listening);

    // Simulate incoming Hermes state packet via audio consumer
    audio_consumer.process_signal(OrbInputSignal::SetState(OrbState::Thinking));
    assert_eq!(audio_consumer.get_state(), OrbState::Thinking);

    // Tick orb controller and check uniforms inside scheduler render benchmark
    let (elapsed, within_budget) = scheduler.benchmark_render_tick(|| {
        orb_controller.transition_to(audio_consumer.get_state());
        let _uniforms = orb_controller.tick(0.016);
        assert_eq!(orb_controller.state(), OrbState::Thinking);
    });

    assert!(within_budget, "Orb render tick must complete under 5ms budget");
    assert!(elapsed < Duration::from_millis(2));
    assert_eq!(orb_controller.state(), OrbState::Thinking);
}

#[test]
fn test_spatial_hit_testing_pointer_events() {
    let panel_rect = Rect {
        x: 100.0,
        y: 50.0,
        width: 400.0,
        height: 300.0,
    };

    // Test inside bounds
    assert!(panel_rect.contains(150.0, 100.0));
    assert!(panel_rect.contains(100.0, 50.0)); // Top-left edge
    assert!(panel_rect.contains(500.0, 350.0)); // Bottom-right edge

    // Test outside bounds
    assert!(!panel_rect.contains(99.0, 50.0));
    assert!(!panel_rect.contains(150.0, 350.1));
    assert!(!panel_rect.contains(500.1, 100.0));
}

#[test]
fn test_glyph_renderer_text_layout_measurements() {
    let glyph_renderer = GlyphRenderer::new(16.0); // 16px font
    let sample_text = "SWAL Desktop\nNative Pipeline";

    let (width, height) = glyph_renderer.measure_text(sample_text);

    // 15 chars max line ("Native Pipeline") * (16 * 0.6 = 9.6) = 144.0px
    assert!((width - 144.0).abs() < f32::EPSILON);
    // 2 lines * (16 * 1.25 = 20.0) = 40.0px
    assert!((height - 40.0).abs() < f32::EPSILON);
}
