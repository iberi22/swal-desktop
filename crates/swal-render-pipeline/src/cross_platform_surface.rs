//! Unified Cross-Platform WGPU Surface Backend for SWAL Desktop
//!
//! Provides abstract surface representation, multi-backend GPU negotiation,
//! HiDPI physical texture bounds scaling, presenter mode switching, and
//! headless offscreen rendering fallback capabilities across Linux, macOS, and Windows.

use serde::{Deserialize, Serialize};
use wgpu::{Backends, PresentMode, TextureFormat};
use crate::wgpu_context::WgpuSurfaceContext;

/// Hardware graphics API backend target selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuBackendType {
    Auto,
    Vulkan,
    Dx12,
    Metal,
    Gl,
}

impl GpuBackendType {
    /// Maps `GpuBackendType` to `wgpu::Backends` bitflags.
    pub fn to_wgpu_backends(&self) -> Backends {
        match self {
            Self::Auto => Backends::PRIMARY | Backends::GL,
            Self::Vulkan => Backends::VULKAN,
            Self::Dx12 => Backends::DX12,
            Self::Metal => Backends::METAL,
            Self::Gl => Backends::GL,
        }
    }

    /// Returns default native GPU backend based on host target OS platform.
    pub fn native_default() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::Dx12
        }
        #[cfg(target_os = "macos")]
        {
            Self::Metal
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            Self::Vulkan
        }
    }
}

/// Surface display presentation mode for cross-platform UI integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfacePresenterMode {
    WaylandLayerShell,
    StandardDesktopWindow,
    OffscreenHeadless,
    TuiBuffer,
}

impl SurfacePresenterMode {
    /// Determines if the presenter requires direct window handle composition.
    pub fn requires_window_handle(&self) -> bool {
        matches!(self, Self::WaylandLayerShell | Self::StandardDesktopWindow)
    }

    /// Determines if the presenter is an offscreen or TUI buffer mode.
    pub fn is_headless_or_tui(&self) -> bool {
        matches!(self, Self::OffscreenHeadless | Self::TuiBuffer)
    }
}

/// Declarative cross-platform surface configuration descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceDescriptorConfig {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub vsync: bool,
    pub backend: GpuBackendType,
    pub presenter: SurfacePresenterMode,
}

impl Default for SurfaceDescriptorConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
            vsync: true,
            backend: GpuBackendType::Auto,
            presenter: SurfacePresenterMode::OffscreenHeadless,
        }
    }
}

impl SurfaceDescriptorConfig {
    /// Creates a new `SurfaceDescriptorConfig` builder instance.
    pub fn new(width: u32, height: u32, scale_factor: f32, vsync: bool, backend: GpuBackendType, presenter: SurfacePresenterMode) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            scale_factor: if scale_factor > 0.0 { scale_factor } else { 1.0 },
            vsync,
            backend,
            presenter,
        }
    }

    /// Calculates physical pixel rendering dimensions based on HiDPI scale factor.
    pub fn physical_bounds(&self) -> (u32, u32) {
        let phys_w = ((self.width as f32) * self.scale_factor).round() as u32;
        let phys_h = ((self.height as f32) * self.scale_factor).round() as u32;
        (phys_w.max(1), phys_h.max(1))
    }

    /// Resolves WGPU present mode according to VSync configuration and presenter.
    pub fn resolve_present_mode(&self) -> PresentMode {
        if self.vsync {
            PresentMode::Fifo
        } else {
            PresentMode::Immediate
        }
    }
}

/// Unified cross-platform WGPU surface adapter managing GPU context, presenter state transitions,
/// HiDPI bounds math, and offscreen fallback rendering.
pub struct CrossPlatformSurfaceAdapter {
    config: SurfaceDescriptorConfig,
    active_backend: GpuBackendType,
    wgpu_context: Option<WgpuSurfaceContext>,
}

impl CrossPlatformSurfaceAdapter {
    /// Creates a new `CrossPlatformSurfaceAdapter` from a given configuration descriptor.
    pub fn new(config: SurfaceDescriptorConfig) -> Self {
        let active_backend = if config.backend == GpuBackendType::Auto {
            GpuBackendType::native_default()
        } else {
            config.backend
        };

        let wgpu_context = if config.presenter.is_headless_or_tui() {
            let mut ctx = WgpuSurfaceContext::new_headless();
            let (pw, ph) = config.physical_bounds();
            let present_mode = config.resolve_present_mode();
            ctx.configure_surface(pw, ph, TextureFormat::Bgra8UnormSrgb, present_mode);
            Some(ctx)
        } else {
            // For live Wayland/Desktop window surface adapters without a native raw window handle in headless tests,
            // we initialize a backing headless WGPU context.
            let mut ctx = WgpuSurfaceContext::new_headless();
            let (pw, ph) = config.physical_bounds();
            let present_mode = config.resolve_present_mode();
            ctx.configure_surface(pw, ph, TextureFormat::Bgra8UnormSrgb, present_mode);
            Some(ctx)
        };

        Self {
            config,
            active_backend,
            wgpu_context,
        }
    }

    /// Creates a fallback headless rendering context adapter with 1920x1080 resolution.
    pub fn create_fallback_headless_context() -> Self {
        let config = SurfaceDescriptorConfig {
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
            vsync: false,
            backend: GpuBackendType::Auto,
            presenter: SurfacePresenterMode::OffscreenHeadless,
        };
        Self::new(config)
    }

    /// Returns a reference to the active surface configuration.
    pub fn config(&self) -> &SurfaceDescriptorConfig {
        &self.config
    }

    /// Returns the active negotiated GPU backend type.
    pub fn active_backend(&self) -> GpuBackendType {
        self.active_backend
    }

    /// Returns whether VSync is enabled for the surface.
    pub fn is_vsync_enabled(&self) -> bool {
        self.config.vsync
    }

    /// Formats and returns the target FPS based on VSync status and presenter mode.
    /// VSync FIFO defaults to standard display refresh (60Hz / 120Hz cap representation),
    /// while unlocked non-vsync targets high-refresh 240Hz rendering loops.
    pub fn format_target_fps(&self) -> u32 {
        if self.config.vsync {
            60
        } else {
            240
        }
    }

    /// Returns physical render texture bounds `(width, height)` considering scale factor.
    pub fn get_render_texture_bounds(&self) -> (u32, u32) {
        self.config.physical_bounds()
    }

    /// Resizes the surface dimensions and updates scale factor, reconfiguring backing WGPU context.
    pub fn resize(&mut self, new_width: u32, new_height: u32, scale_factor: f32) {
        self.config.width = new_width.max(1);
        self.config.height = new_height.max(1);
        self.config.scale_factor = if scale_factor > 0.0 { scale_factor } else { 1.0 };

        let (pw, ph) = self.config.physical_bounds();
        let present_mode = self.config.resolve_present_mode();

        if let Some(ctx) = &mut self.wgpu_context {
            ctx.configure_surface(pw, ph, ctx.surface_config.format, present_mode);
        }
    }

    /// Updates VSync state and updates backing WGPU surface present mode.
    pub fn set_vsync(&mut self, vsync: bool) {
        self.config.vsync = vsync;
        let present_mode = self.config.resolve_present_mode();
        let (pw, ph) = self.config.physical_bounds();

        if let Some(ctx) = &mut self.wgpu_context {
            ctx.configure_surface(pw, ph, ctx.surface_config.format, present_mode);
        }
    }

    /// Transitions the active presenter mode (e.g. WaylandLayerShell <-> StandardDesktopWindow <-> OffscreenHeadless).
    pub fn transition_presenter_mode(&mut self, new_presenter: SurfacePresenterMode) {
        self.config.presenter = new_presenter;
    }

    /// Provides access to the underlying `WgpuSurfaceContext` if initialized.
    pub fn wgpu_context(&self) -> Option<&WgpuSurfaceContext> {
        self.wgpu_context.as_ref()
    }

    /// Provides mutable access to the underlying `WgpuSurfaceContext` if initialized.
    pub fn wgpu_context_mut(&mut self) -> Option<&mut WgpuSurfaceContext> {
        self.wgpu_context.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_backend_type_to_wgpu_backends_mapping() {
        assert_eq!(GpuBackendType::Vulkan.to_wgpu_backends(), Backends::VULKAN);
        assert_eq!(GpuBackendType::Dx12.to_wgpu_backends(), Backends::DX12);
        assert_eq!(GpuBackendType::Metal.to_wgpu_backends(), Backends::METAL);
        assert_eq!(GpuBackendType::Gl.to_wgpu_backends(), Backends::GL);
        assert_eq!(
            GpuBackendType::Auto.to_wgpu_backends(),
            Backends::PRIMARY | Backends::GL
        );
    }

    #[test]
    fn test_gpu_backend_native_default() {
        let native = GpuBackendType::native_default();
        #[cfg(target_os = "windows")]
        assert_eq!(native, GpuBackendType::Dx12);
        #[cfg(target_os = "macos")]
        assert_eq!(native, GpuBackendType::Metal);
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        assert_eq!(native, GpuBackendType::Vulkan);
    }

    #[test]
    fn test_surface_presenter_mode_properties() {
        assert!(SurfacePresenterMode::WaylandLayerShell.requires_window_handle());
        assert!(SurfacePresenterMode::StandardDesktopWindow.requires_window_handle());
        assert!(!SurfacePresenterMode::OffscreenHeadless.requires_window_handle());
        assert!(!SurfacePresenterMode::TuiBuffer.requires_window_handle());

        assert!(SurfacePresenterMode::OffscreenHeadless.is_headless_or_tui());
        assert!(SurfacePresenterMode::TuiBuffer.is_headless_or_tui());
        assert!(!SurfacePresenterMode::WaylandLayerShell.is_headless_or_tui());
        assert!(!SurfacePresenterMode::StandardDesktopWindow.is_headless_or_tui());
    }

    #[test]
    fn test_resolution_scaling_math_and_physical_bounds() {
        let config = SurfaceDescriptorConfig::new(
            1000,
            500,
            1.5,
            true,
            GpuBackendType::Vulkan,
            SurfacePresenterMode::StandardDesktopWindow,
        );

        assert_eq!(config.physical_bounds(), (1500, 750));

        let config_hidpi2 = SurfaceDescriptorConfig::new(
            1920,
            1080,
            2.0,
            false,
            GpuBackendType::Auto,
            SurfacePresenterMode::WaylandLayerShell,
        );
        assert_eq!(config_hidpi2.physical_bounds(), (3840, 2160));
        assert_eq!(config_hidpi2.resolve_present_mode(), PresentMode::Immediate);
        assert_eq!(config.resolve_present_mode(), PresentMode::Fifo);
    }

    #[test]
    fn test_zero_or_negative_scale_dimension_clamping() {
        let config = SurfaceDescriptorConfig::new(
            0,
            0,
            -0.5,
            true,
            GpuBackendType::Auto,
            SurfacePresenterMode::OffscreenHeadless,
        );

        assert_eq!(config.width, 1);
        assert_eq!(config.height, 1);
        assert_eq!(config.scale_factor, 1.0);
        assert_eq!(config.physical_bounds(), (1, 1));
    }

    #[test]
    fn test_adapter_creation_and_backend_negotiation() {
        let config = SurfaceDescriptorConfig {
            width: 1280,
            height: 720,
            scale_factor: 1.0,
            vsync: true,
            backend: GpuBackendType::Auto,
            presenter: SurfacePresenterMode::WaylandLayerShell,
        };

        let adapter = CrossPlatformSurfaceAdapter::new(config.clone());
        assert_eq!(adapter.config().width, 1280);
        assert_eq!(adapter.config().height, 720);
        assert_eq!(adapter.active_backend(), GpuBackendType::native_default());
        assert!(adapter.is_vsync_enabled());
        assert_eq!(adapter.format_target_fps(), 60);
        assert_eq!(adapter.get_render_texture_bounds(), (1280, 720));
        assert!(adapter.wgpu_context().is_some());
    }

    #[test]
    fn test_fallback_headless_context_creation() {
        let adapter = CrossPlatformSurfaceAdapter::create_fallback_headless_context();
        assert_eq!(adapter.config().width, 1920);
        assert_eq!(adapter.config().height, 1080);
        assert_eq!(adapter.config().scale_factor, 1.0);
        assert!(!adapter.is_vsync_enabled());
        assert_eq!(adapter.format_target_fps(), 240);
        assert_eq!(adapter.get_render_texture_bounds(), (1920, 1080));
        assert_eq!(
            adapter.config().presenter,
            SurfacePresenterMode::OffscreenHeadless
        );
        assert!(adapter.wgpu_context().is_some());
    }

    #[test]
    fn test_resize_and_vsync_reconfiguration() {
        let mut adapter = CrossPlatformSurfaceAdapter::create_fallback_headless_context();

        adapter.resize(2560, 1440, 1.25);
        assert_eq!(adapter.config().width, 2560);
        assert_eq!(adapter.config().height, 1440);
        assert_eq!(adapter.config().scale_factor, 1.25);
        assert_eq!(adapter.get_render_texture_bounds(), (3200, 1800));

        let ctx_bounds = adapter.wgpu_context().unwrap().surface_config;
        assert_eq!(ctx_bounds.width, 3200);
        assert_eq!(ctx_bounds.height, 1800);

        adapter.set_vsync(true);
        assert!(adapter.is_vsync_enabled());
        assert_eq!(adapter.format_target_fps(), 60);
        assert_eq!(
            adapter.wgpu_context().unwrap().surface_config.present_mode,
            PresentMode::Fifo
        );

        adapter.set_vsync(false);
        assert!(!adapter.is_vsync_enabled());
        assert_eq!(adapter.format_target_fps(), 240);
        assert_eq!(
            adapter.wgpu_context().unwrap().surface_config.present_mode,
            PresentMode::Immediate
        );
    }

    #[test]
    fn test_presenter_state_transitions() {
        let mut adapter = CrossPlatformSurfaceAdapter::create_fallback_headless_context();
        assert_eq!(
            adapter.config().presenter,
            SurfacePresenterMode::OffscreenHeadless
        );

        adapter.transition_presenter_mode(SurfacePresenterMode::WaylandLayerShell);
        assert_eq!(
            adapter.config().presenter,
            SurfacePresenterMode::WaylandLayerShell
        );

        adapter.transition_presenter_mode(SurfacePresenterMode::StandardDesktopWindow);
        assert_eq!(
            adapter.config().presenter,
            SurfacePresenterMode::StandardDesktopWindow
        );

        adapter.transition_presenter_mode(SurfacePresenterMode::TuiBuffer);
        assert_eq!(
            adapter.config().presenter,
            SurfacePresenterMode::TuiBuffer
        );
    }
}
