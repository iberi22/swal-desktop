//! WGPU graphics context & hardware surface initializer for SWAL Desktop
//!
//! Provides `WgpuSurfaceContext` for GPU hardware acceleration, supporting
//! both headless testing environments and high-refresh Wayland layer shell surfaces.

use wgpu::{
    Adapter, CommandEncoder, Device, Instance, PresentMode, Queue, SurfaceConfiguration,
    Texture, TextureFormat, TextureUsages,
};

/// Hardware WGPU Graphics & Surface Context for high-performance rendering.
pub struct WgpuSurfaceContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
}

impl WgpuSurfaceContext {
    /// Creates a headless WGPU rendering context suitable for testing and offscreen rendering.
    pub fn new_headless() -> Self {
        let instance = Instance::default();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .or_else(|| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: true,
            }))
        })
        .expect("Failed to find a suitable WGPU graphics adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("SWAL Headless Graphics Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to create WGPU logical device and queue");

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: TextureFormat::Bgra8UnormSrgb,
            width: 1920,
            height: 1080,
            present_mode: PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Self {
            instance,
            adapter,
            device,
            queue,
            surface_config,
        }
    }

    /// Reconfigures the surface parameters (width, height, pixel format, and presentation mode).
    pub fn configure_surface(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        present_mode: PresentMode,
    ) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface_config.format = format;
        self.surface_config.present_mode = present_mode;
    }

    /// Creates a new WGPU command encoder for submitting draw passes.
    pub fn create_command_encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("SWAL Frame Command Encoder"),
            })
    }

    /// Creates an offscreen render target texture matching current surface dimensions and format.
    pub fn render_frame_target(&self) -> Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SWAL Offscreen Render Target"),
            size: wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_config.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_adapter_creation() {
        let ctx = WgpuSurfaceContext::new_headless();
        let info = ctx.adapter.get_info();
        assert!(!info.name.is_empty(), "Adapter should have a valid device name");
        assert_eq!(ctx.surface_config.width, 1920);
        assert_eq!(ctx.surface_config.height, 1080);
    }

    #[test]
    fn test_surface_configuration_math() {
        let mut ctx = WgpuSurfaceContext::new_headless();

        // Configure surface for 200Hz+ unlocked high refresh rate
        ctx.configure_surface(2560, 1440, TextureFormat::Rgba8UnormSrgb, PresentMode::Immediate);
        assert_eq!(ctx.surface_config.width, 2560);
        assert_eq!(ctx.surface_config.height, 1440);
        assert_eq!(ctx.surface_config.format, TextureFormat::Rgba8UnormSrgb);
        assert_eq!(ctx.surface_config.present_mode, PresentMode::Immediate);

        // Reconfigure surface for VSync mode
        ctx.configure_surface(1920, 1080, TextureFormat::Bgra8UnormSrgb, PresentMode::Fifo);
        assert_eq!(ctx.surface_config.width, 1920);
        assert_eq!(ctx.surface_config.height, 1080);
        assert_eq!(ctx.surface_config.format, TextureFormat::Bgra8UnormSrgb);
        assert_eq!(ctx.surface_config.present_mode, PresentMode::Fifo);

        // Verify minimum dimension clamp math (width/height = 0 should clamp to 1)
        ctx.configure_surface(0, 0, TextureFormat::Bgra8UnormSrgb, PresentMode::Fifo);
        assert_eq!(ctx.surface_config.width, 1);
        assert_eq!(ctx.surface_config.height, 1);
    }

    #[test]
    fn test_queue_command_submission() {
        let ctx = WgpuSurfaceContext::new_headless();
        let mut encoder = ctx.create_command_encoder();
        let frame_target = ctx.render_frame_target();
        let view = frame_target.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Test Clear Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        let cmd_buffer = encoder.finish();
        ctx.queue.submit(std::iter::once(cmd_buffer));
        ctx.device.poll(wgpu::Maintain::Wait);
    }
}
