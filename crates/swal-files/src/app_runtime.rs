//! Cross-Platform Standalone Window & App Runtime Dispatcher in Rust
//!
//! Autonomous runtime mode selector and window lifecycle dispatcher for SWAL Files across
//! Linux Wayland Layer Shell, X11/Wayland Standalone Windows, macOS, Windows, TUI, and Headless environments.

use serde::{Deserialize, Serialize};
use std::env;

/// Application runtime mode variants for standalone & integrated operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppRuntimeMode {
    WaylandLayerShell,
    StandaloneWindow,
    TuiTerminal,
    HeadlessDaemon,
    WebCanvas,
}

impl Default for AppRuntimeMode {
    fn default() -> Self {
        Self::StandaloneWindow
    }
}

/// Window configuration settings for GUI window instances
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSettings {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub always_on_top: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            title: "SWAL Files".to_string(),
            width: 1080,
            height: 660,
            min_width: 800,
            min_height: 500,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
        }
    }
}

/// Context holding runtime mode, window settings, desktop presence, and active theme
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppRuntimeContext {
    pub mode: AppRuntimeMode,
    pub window_settings: WindowSettings,
    pub is_swal_desktop_present: bool,
    pub active_theme: String,
}

impl Default for AppRuntimeContext {
    fn default() -> Self {
        Self {
            mode: AppRuntimeMode::default(),
            window_settings: WindowSettings::default(),
            is_swal_desktop_present: false,
            active_theme: "fluent-dark".to_string(),
        }
    }
}

/// Dispatcher responsible for optimal mode auto-detection, context creation,
/// initial payload construction, and window lifecycle event management.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppRuntimeDispatcher {
    pub context: AppRuntimeContext,
    pub is_visible: bool,
    pub is_focused: bool,
    pub is_maximized: bool,
    pub is_minimized: bool,
}

impl Default for AppRuntimeDispatcher {
    fn default() -> Self {
        Self::new(AppRuntimeContext::default())
    }
}

impl AppRuntimeDispatcher {
    /// Creates a new `AppRuntimeDispatcher` wrapping the given runtime context.
    pub fn new(context: AppRuntimeContext) -> Self {
        Self {
            context,
            is_visible: true,
            is_focused: true,
            is_maximized: false,
            is_minimized: false,
        }
    }

    /// Auto-detects the optimal runtime mode based on environment variables
    /// (`SWAL_HEADLESS`, `SWAL_WEB`, `WAYLAND_DISPLAY`, `DISPLAY`, `OS`, `SWAL_DESKTOP_ACTIVE`, TTY status).
    pub fn detect_optimal_mode() -> AppRuntimeMode {
        if let Ok(val) = env::var("SWAL_HEADLESS") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                return AppRuntimeMode::HeadlessDaemon;
            }
        }

        if let Ok(val) = env::var("SWAL_WEB") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                return AppRuntimeMode::WebCanvas;
            }
        }

        let is_tui_env = env::var("SWAL_TUI")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if is_tui_env {
            return AppRuntimeMode::TuiTerminal;
        }

        let is_swal_active = env::var("SWAL_DESKTOP_ACTIVE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let has_wayland = env::var("WAYLAND_DISPLAY").map(|v| !v.trim().is_empty()).unwrap_or(false);
        let has_x11 = env::var("DISPLAY").map(|v| !v.trim().is_empty()).unwrap_or(false);

        let os_env = env::var("OS").unwrap_or_default().to_lowercase();
        let target_os = env::consts::OS;

        let is_windows_or_mac = target_os == "windows"
            || target_os == "macos"
            || os_env.contains("windows")
            || os_env.contains("darwin");

        if is_swal_active && has_wayland {
            return AppRuntimeMode::WaylandLayerShell;
        }

        if has_wayland || has_x11 || is_windows_or_mac {
            return AppRuntimeMode::StandaloneWindow;
        }

        let is_tty = env::var("SSH_TTY").is_ok()
            || env::var("TERM").map(|v| !v.trim().is_empty() && v != "dumb").unwrap_or(false);

        if is_tty {
            return AppRuntimeMode::TuiTerminal;
        }

        AppRuntimeMode::HeadlessDaemon
    }

    /// Constructs a new `AppRuntimeContext`, applying an optional mode override or auto-detecting the optimal mode.
    pub fn new_context(mode_override: Option<AppRuntimeMode>) -> AppRuntimeContext {
        let mode = mode_override.unwrap_or_else(Self::detect_optimal_mode);
        let is_swal_desktop_present = env::var("SWAL_DESKTOP_ACTIVE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let active_theme = env::var("SWAL_THEME").unwrap_or_else(|_| "fluent-dark".to_string());

        let mut window_settings = WindowSettings::default();

        // Adjust window defaults based on runtime mode
        match mode {
            AppRuntimeMode::WaylandLayerShell => {
                window_settings.decorations = false;
                window_settings.transparent = true;
                window_settings.always_on_top = true;
            }
            AppRuntimeMode::StandaloneWindow => {
                window_settings.decorations = true;
                window_settings.transparent = false;
            }
            AppRuntimeMode::TuiTerminal => {
                window_settings.decorations = false;
                window_settings.width = 120;
                window_settings.height = 40;
            }
            AppRuntimeMode::HeadlessDaemon => {
                window_settings.decorations = false;
                window_settings.width = 0;
                window_settings.height = 0;
            }
            AppRuntimeMode::WebCanvas => {
                window_settings.decorations = true;
                window_settings.transparent = true;
            }
        }

        AppRuntimeContext {
            mode,
            window_settings,
            is_swal_desktop_present,
            active_theme,
        }
    }

    /// Builds a JSON-encoded initial payload string describing the runtime state and window configuration.
    pub fn build_initial_payload(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Handles window lifecycle events ("open", "show", "hide", "focus", "blur", "resize", "minimize", "maximize", "restore", "close", "quit").
    /// Returns `true` if the event was recognized and processed successfully, `false` otherwise.
    pub fn handle_window_lifecycle_event(&mut self, event: &str) -> bool {
        match event.to_lowercase().trim() {
            "open" | "show" => {
                self.is_visible = true;
                self.is_minimized = false;
                true
            }
            "hide" => {
                self.is_visible = false;
                self.is_focused = false;
                true
            }
            "focus" => {
                self.is_focused = true;
                true
            }
            "blur" => {
                self.is_focused = false;
                true
            }
            "resize" => true,
            "minimize" => {
                self.is_minimized = true;
                self.is_focused = false;
                true
            }
            "maximize" => {
                self.is_maximized = true;
                self.is_minimized = false;
                true
            }
            "restore" => {
                self.is_maximized = false;
                self.is_minimized = false;
                self.is_visible = true;
                true
            }
            "close" | "quit" => {
                self.is_visible = false;
                self.is_focused = false;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    /// Serializes tests that mutate process-global environment variables.
    /// Without this, `cargo test` runs them in parallel and one test's
    /// `SWAL_HEADLESS=1` leaks into another's mode detection (flaky).
    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let mutex = LOCK.get_or_init(|| Mutex::new(()));
        mutex.lock().unwrap_or_else(|p| p.into_inner())
    }

    struct EnvGuard {
        key: &'static str,
        old_val: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, val: &str) -> Self {
            let old_val = env::var(key).ok();
            env::set_var(key, val);
            Self { key, old_val }
        }

        fn remove(key: &'static str) -> Self {
            let old_val = env::var(key).ok();
            env::remove_var(key);
            Self { key, old_val }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(ref val) = self.old_val {
                env::set_var(self.key, val);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn test_app_runtime_mode_variants_and_default() {
        assert_eq!(AppRuntimeMode::default(), AppRuntimeMode::StandaloneWindow);
        let mode = AppRuntimeMode::WaylandLayerShell;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: AppRuntimeMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mode, deserialized);

        let modes = [
            AppRuntimeMode::WaylandLayerShell,
            AppRuntimeMode::StandaloneWindow,
            AppRuntimeMode::TuiTerminal,
            AppRuntimeMode::HeadlessDaemon,
            AppRuntimeMode::WebCanvas,
        ];
        assert_eq!(modes.len(), 5);
    }

    #[test]
    fn test_window_settings_default_and_serde() {
        let settings = WindowSettings::default();
        assert_eq!(settings.title, "SWAL Files");
        assert_eq!(settings.width, 1080);
        assert_eq!(settings.height, 660);
        assert_eq!(settings.min_width, 800);
        assert_eq!(settings.min_height, 500);
        assert!(settings.resizable);
        assert!(settings.decorations);
        assert!(!settings.transparent);
        assert!(!settings.always_on_top);

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: WindowSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }

    #[test]
    fn test_app_runtime_context_default_and_serde() {
        let ctx = AppRuntimeContext::default();
        assert_eq!(ctx.mode, AppRuntimeMode::StandaloneWindow);
        assert_eq!(ctx.active_theme, "fluent-dark");
        assert!(!ctx.is_swal_desktop_present);

        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: AppRuntimeContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, parsed);
    }

    #[test]
    fn test_mode_auto_detection_headless() {
        let _lock = env_lock();
        let _g = EnvGuard::set("SWAL_HEADLESS", "1");
        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::HeadlessDaemon);

        let _g2 = EnvGuard::set("SWAL_HEADLESS", "true");
        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::HeadlessDaemon);
    }

    #[test]
    fn test_mode_auto_detection_web() {
        let _lock = env_lock();
        let _g_h = EnvGuard::remove("SWAL_HEADLESS");
        let _g_w = EnvGuard::set("SWAL_WEB", "true");
        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::WebCanvas);
    }

    #[test]
    fn test_mode_auto_detection_wayland_layer_shell() {
        let _lock = env_lock();
        let _g_h = EnvGuard::remove("SWAL_HEADLESS");
        let _g_w = EnvGuard::remove("SWAL_WEB");
        let _g_desktop = EnvGuard::set("SWAL_DESKTOP_ACTIVE", "1");
        let _g_wayland = EnvGuard::set("WAYLAND_DISPLAY", "wayland-0");

        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::WaylandLayerShell);
    }

    #[test]
    fn test_mode_auto_detection_standalone_window() {
        let _lock = env_lock();
        let _g_h = EnvGuard::remove("SWAL_HEADLESS");
        let _g_w = EnvGuard::remove("SWAL_WEB");
        let _g_desktop = EnvGuard::remove("SWAL_DESKTOP_ACTIVE");
        let _g_wayland = EnvGuard::set("WAYLAND_DISPLAY", "wayland-0");

        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::StandaloneWindow);

        let _g_wayland_off = EnvGuard::remove("WAYLAND_DISPLAY");
        let _g_x11 = EnvGuard::set("DISPLAY", ":0");
        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::StandaloneWindow);
    }

    #[test]
    fn test_mode_auto_detection_tui() {
        let _lock = env_lock();
        let _g_h = EnvGuard::remove("SWAL_HEADLESS");
        let _g_w = EnvGuard::remove("SWAL_WEB");
        let _g_desktop = EnvGuard::remove("SWAL_DESKTOP_ACTIVE");
        let _g_wayland = EnvGuard::remove("WAYLAND_DISPLAY");
        let _g_x11 = EnvGuard::remove("DISPLAY");
        let _g_os = EnvGuard::remove("OS");
        let _g_term = EnvGuard::set("SWAL_TUI", "1");

        assert_eq!(AppRuntimeDispatcher::detect_optimal_mode(), AppRuntimeMode::TuiTerminal);
    }

    #[test]
    fn test_mode_auto_detection_fallback() {
        let _lock = env_lock();
        let _g_h = EnvGuard::remove("SWAL_HEADLESS");
        let _g_w = EnvGuard::remove("SWAL_WEB");
        let _g_desktop = EnvGuard::remove("SWAL_DESKTOP_ACTIVE");
        let _g_wayland = EnvGuard::remove("WAYLAND_DISPLAY");
        let _g_x11 = EnvGuard::remove("DISPLAY");
        let _g_os = EnvGuard::remove("OS");
        let _g_term = EnvGuard::remove("TERM");
        let _g_ssh = EnvGuard::remove("SSH_TTY");
        let _g_tui = EnvGuard::remove("SWAL_TUI");

        // On Linux without display/term, falls back to HeadlessDaemon; on macOS/Windows, falls back to StandaloneWindow
        let mode = AppRuntimeDispatcher::detect_optimal_mode();
        assert!(mode == AppRuntimeMode::HeadlessDaemon || mode == AppRuntimeMode::StandaloneWindow);
    }

    #[test]
    fn test_new_context_with_override_and_defaults() {
        let ctx = AppRuntimeDispatcher::new_context(Some(AppRuntimeMode::WaylandLayerShell));
        assert_eq!(ctx.mode, AppRuntimeMode::WaylandLayerShell);
        assert!(!ctx.window_settings.decorations);
        assert!(ctx.window_settings.transparent);
        assert!(ctx.window_settings.always_on_top);

        let ctx_tui = AppRuntimeDispatcher::new_context(Some(AppRuntimeMode::TuiTerminal));
        assert_eq!(ctx_tui.mode, AppRuntimeMode::TuiTerminal);
        assert_eq!(ctx_tui.window_settings.width, 120);
        assert_eq!(ctx_tui.window_settings.height, 40);

        let ctx_headless = AppRuntimeDispatcher::new_context(Some(AppRuntimeMode::HeadlessDaemon));
        assert_eq!(ctx_headless.mode, AppRuntimeMode::HeadlessDaemon);
        assert_eq!(ctx_headless.window_settings.width, 0, "Width should be 0 for headless");

        let ctx_web = AppRuntimeDispatcher::new_context(Some(AppRuntimeMode::WebCanvas));
        assert_eq!(ctx_web.mode, AppRuntimeMode::WebCanvas);
        assert!(ctx_web.window_settings.transparent);
    }

    #[test]
    fn test_build_initial_payload_json() {
        let ctx = AppRuntimeContext::default();
        let dispatcher = AppRuntimeDispatcher::new(ctx);
        let payload = dispatcher.build_initial_payload();

        assert!(payload.contains("StandaloneWindow"));
        assert!(payload.contains("SWAL Files"));
        assert!(payload.contains("is_visible"));

        let parsed: AppRuntimeDispatcher = serde_json::from_str(&payload).unwrap();
        assert_eq!(dispatcher, parsed);
    }

    #[test]
    fn test_handle_window_lifecycle_events() {
        let mut dispatcher = AppRuntimeDispatcher::default();

        // 1. hide
        assert!(dispatcher.handle_window_lifecycle_event("hide"));
        assert!(!dispatcher.is_visible);
        assert!(!dispatcher.is_focused);

        // 2. show / open
        assert!(dispatcher.handle_window_lifecycle_event("SHOW"));
        assert!(dispatcher.is_visible);
        assert!(!dispatcher.is_minimized);

        // 3. focus / blur
        assert!(dispatcher.handle_window_lifecycle_event("focus"));
        assert!(dispatcher.is_focused);

        assert!(dispatcher.handle_window_lifecycle_event("blur"));
        assert!(!dispatcher.is_focused);

        // 4. minimize / maximize / restore
        assert!(dispatcher.handle_window_lifecycle_event("minimize"));
        assert!(dispatcher.is_minimized);
        assert!(!dispatcher.is_focused);

        assert!(dispatcher.handle_window_lifecycle_event("maximize"));
        assert!(dispatcher.is_maximized);
        assert!(!dispatcher.is_minimized);

        assert!(dispatcher.handle_window_lifecycle_event("restore"));
        assert!(!dispatcher.is_maximized);
        assert!(!dispatcher.is_minimized);
        assert!(dispatcher.is_visible);

        // 5. resize
        assert!(dispatcher.handle_window_lifecycle_event("resize"));

        // 6. close / quit
        assert!(dispatcher.handle_window_lifecycle_event("close"));
        assert!(!dispatcher.is_visible);

        // 7. unknown event
        assert!(!dispatcher.handle_window_lifecycle_event("unknown_event_xyz"));
    }
}
