//! POC: real Wayland (xdg-toplevel) + wgpu surface — window with a clear color.
//! Validates the native window path before building the full A2UI renderer.
use std::rc::Rc;

use smithay_client_toolkit::app::{App, AppData, AppExt};
use smithay_client_toolkit::compositor::CompositorHandler;
use smithay_client_toolkit::output::OutputHandler;
use smithay_client_toolkit::seat::keyboard::{KeyboardEvent, KeyboardHandler};
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use smithay_client_toolkit::seat::SeatHandler;
use smithay_client_toolkit::shell::xdg::window::{Window, WindowConfigure, WindowHandler};
use smithay_client_toolkit::shell::xdg::XdgShellHandler;
use smithay_client_toolkit::shm::ShmHandler;
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::{delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry, delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window};

use wayland_client::globals::registry_queue_init;
use wayland_client::{Connection, QueueHandle};

pub struct PocApp {
    registry_state: RegistryState,
    compositor_state: smithay_client_toolkit::compositor::CompositorState,
    xdg_shell_state: smithay_client_toolkit::shell::xdg::XdgShellState,
    window: Option<Rc<Window>>,
    running: bool,
}

impl PocApp {
    fn new(connection: &Connection, qh: &QueueHandle<Self>) -> Self {
        Self {
            registry_state: RegistryState::new(&connection, &qh),
            compositor_state: smithay_client_toolkit::compositor::CompositorState::new(&connection, &qh),
            xdg_shell_state: smithay_client_toolkit::shell::xdg::XdgShellState::new(&connection, &qh),
            window: None,
            running: true,
        }
    }
}

impl AppData for PocApp {}

impl CompositorHandler for PocApp {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _factor: i32) {}
    fn frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _time: u32) {}
}

impl OutputHandler for PocApp {
    fn output_state(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: &wayland_client::protocol::wl_output::WlOutput) {}
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
}

impl SeatHandler for PocApp {
    fn seat_state(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: &smithay_client_toolkit::seat::Seat, _state: smithay_client_toolkit::seat::seat::SeatState) {}
    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: smithay_client_toolkit::seat::Seat) {}
    fn new_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: smithay_client_toolkit::seat::Seat, capability: smithay_client_toolkit::seat::Capability) {}
    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: smithay_client_toolkit::seat::Seat, capability: smithay_client_toolkit::seat::Capability) {}
    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: smithay_client_toolkit::seat::Seat) {}
}

impl PointerHandler for PocApp {
    fn pointer_frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &smithay_client_toolkit::seat::pointer::Pointer, _events: &[PointerEvent]) {}
    fn pointer_enter(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &smithay_client_toolkit::seat::pointer::Pointer, _evt: &PointerEvent) {}
    fn pointer_leave(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &smithay_client_toolkit::seat::pointer::Pointer, _evt: &PointerEvent) {}
    fn pointer_move(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &smithay_client_toolkit::seat::pointer::Pointer, _evt: &PointerEvent) {}
    fn pointer_button(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &smithay_client_toolkit::seat::pointer::Pointer, _evt: &PointerEvent) {
        if let PointerEventKind::Press = _evt.kind {
            if let Some(w) = &self.window {
                w.close();
            }
            self.running = false;
        }
    }
    fn pointer_axis(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &smithay_client_toolkit::seat::pointer::Pointer, _evt: &PointerEvent) {}
}

impl KeyboardHandler for PocApp {
    fn enter(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _keyboard: &smithay_client_toolkit::seat::keyboard::Keyboard, _surface: &wayland_client::protocol::wl_surface::WlSurface) {}
    fn leave(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _keyboard: &smithay_client_toolkit::seat::keyboard::Keyboard, _surface: &wayland_client::protocol::wl_surface::WlSurface) {}
    fn key_state(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _keyboard: &smithay_client_toolkit::seat::keyboard::Keyboard, _key: KeyboardEvent) {}
    fn modifiers(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _keyboard: &smithay_client_toolkit::seat::keyboard::Keyboard, _mods: wayland_client::protocol::wl_keyboard::WlKeyboard) {}
}

impl ShmHandler for PocApp {
    fn shm_state(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _shm: &wayland_client::protocol::wl_shm::WlShm) {}
}

impl XdgShellHandler for PocApp {
    fn xdg_shell_state(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _state: smithay_client_toolkit::shell::xdg::XdgShellState) {}
}

impl WindowHandler for PocApp {
    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &Window) {
        self.running = false;
    }
    fn configure(
        &mut self,
        conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
    ) {
        // POC: just ack immediately; real renderer will set up wgpu surface here
        let (tw, th) = (configure.new_size.width as i32, configure.new_size.height as i32);
        println!("configure: {}x{}", tw, th);
        _window.ack_configure(configure.serial);
    }
}

impl ProvidesRegistryState for PocApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    fn registry_handles(&mut self) -> Vec<(u32, wayland_client::protocol::wl_registry::WlRegistry)> {
        self.registry_state.registry_handles()
    }
}

delegate_compositor!(PocApp);
delegate_output!(PocApp);
delegate_registry!(PocApp);
delegate_seat!(PocApp);
delegate_pointer!(PocApp);
delegate_keyboard!(PocApp);
delegate_shm!(PocApp);
delegate_xdg_shell!(PocApp);
delegate_xdg_window!(PocApp);

fn main() {
    let conn = Connection::connect_to_env().expect("Conectar a WAYLAND_DISPLAY");
    let globals = registry_queue_init(&conn).unwrap();
    let qh = globals.queue_handle();
    let mut app = PocApp::new(&conn, &qh);

    let (window, surface) = smithay_client_toolkit::shell::xdg::window::Window::create(
        &conn,
        &qh,
        globals.registry().get_global::<wayland_client::protocol::wl_compositor::WlCompositor>()?,
        &app.xdg_shell_state,
        "SWAL Files (POC)",
        1024,
        768,
    ).expect("crear ventana");

    app.window = Some(window);
    app.xdg_shell_state.ping_handle();

    // Minimal render loop: no wgpu surface yet, just keep the event loop alive
    while app.running {
        let mut event_queue = globals.event_queue();
        event_queue.blocking_dispatch(&mut app).unwrap();
    }
    println!("POC window done");
    std::process::exit(0);
}