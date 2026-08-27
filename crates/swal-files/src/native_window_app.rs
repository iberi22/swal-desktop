//! SWAL Files — Native Wayland Window (Zero-EWW / Zero-GTK / Zero-Terminal)
//!
//! A real xdg-toplevel window driven directly through the Wayland protocol
//! (smithay-client-toolkit 0.19), rendering with wl_shm + ab_glyph (CPU text).
//! Three-column layout mirroring the C# Files Explorer design:
//!   sidebar (pins + disk meters) | file list | preview panel
//!
//! Keyboard: j/k or arrows move, enter/l opens, h/backspace goes up,
//! r reloads, p cycles preview, q/esc quits.
//! Pointer: hover selects row, click opens.

use std::path::PathBuf;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface};
use wayland_client::{Connection, Dispatch, QueueHandle};
use smithay_client_toolkit::shell::xdg::XdgSurface;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use crate::gui::build_gui_payload;
use crate::scanner::{scan_directory, ScanOptions};
use crate::session::{load_session, save_session, TabState};

// ── SWAL cyber-neon palette (source of truth: @swal/ui tokens) ──────
const BG: u32       = 0xFF0A0F1D;  // rgba(10, 15, 29, 1) — main bg
const ELEVATED: u32 = 0xFF111827;  // sidebar/card surface
const ACCENT: u32   = 0xFF00FF88;  // cyber-neon green (primary)
const ACCENT2: u32  = 0xFF00CCFF;  // cyan (secondary)
const DANGER: u32   = 0xFFFF4444;
const WARNING: u32  = 0xFFFFBB00;
const SUCCESS: u32  = 0xFF00FF88;  // same as ACCENT
const TEXT: u32     = 0xFFE2E8F0;
const TEXT_DIM: u32 = 0xFF94A3B8;
const SELECTED: u32 = 0xFF1A3D2B;  // semi-dark green highlight (no real alpha in wl_shm)
const BORDER: u32   = 0xFF1E2A3A;  // subtle border
const DIR_COLOR: u32 = 0xFF60CDFF;

pub fn run_native_window() {
    let conn = Connection::connect_to_env().expect("WAYLAND_DISPLAY o XDG_RUNTIME_DIR requeridos");
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor no disponible");
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg-shell no disponible");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm no disponible");
    let font = load_font();

    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
    window.set_title("SWAL Files");
    window.set_app_id("io.github.southwest-ai-labs.swal-files");
    window.set_min_size(Some((640, 480)));
    window.commit();

    let session = load_session();
    let start_path = session
        .tabs
        .iter()
        .find(|t| t.id == session.active_tab_id)
        .map(|t| PathBuf::from(&t.path))
        .filter(|p| p.exists())
        .unwrap_or_else(home_path);

    let mut app = SwalFilesApp {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        window,
        pool: None,
        width: 1024,
        height: 720,
        configured: false,
        redraw: true,
        exit: false,
        keyboard: None,
        pointer: None,
        current_path: start_path,
        items: Vec::new(),
        selected_index: 0,
        scroll_offset: 0,
        font,
        tabs: session.tabs.clone(),
        active_tab_id: session.active_tab_id,
        _dummy: (),
    };
    app.reload_dir();

    let pid = std::process::id();
    let _ = std::fs::write("/tmp/swal-files.pid", pid.to_string());
    let _ = std::fs::write("/tmp/swal_files_visible.flag", "1");

    // Initial blank frame so the window maps even before configure arrives
    app.draw_frame();
    event_queue.blocking_dispatch(&mut app).unwrap();
    while !app.exit {
        if app.redraw {
            app.draw_frame();
            app.redraw = false;
        }
        event_queue.blocking_dispatch(&mut app).unwrap();
    }

    let _ = std::fs::remove_file("/tmp/swal-files.pid");
    let _ = std::fs::remove_file("/tmp/swal_files_visible.flag");
}

fn home_path() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn load_font() -> FontArc {
    // Path exacto verificado (probe: gids H=43 e=72 o=82 correctos, upem 2048).
    // NO escanear /nix/store completo: hay variantes corruptas/parciales.
    let candidates = [
        "/nix/store/ang6yzsv32vnkdq7bqr41dgna2knkz8w-dejavu-fonts-minimal-2.37/share/fonts/truetype/DejaVuSans.ttf",
        "/nix/store/xvy8dq43r9hi9qrnwgg7kjjny8y0lr0g-dejavu-fonts-minimal-2.37/share/fonts/truetype/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/DejaVuSans.ttf",
    ];
    for c in candidates {
        if let Ok(data) = std::fs::read(c) {
            if let Ok(font) = FontArc::try_from_vec(data) {
                // Sanity check: 'H' debe existir (gid != 0)
                if font.glyph_id('H').0 != 0 {
                    return font;
                }
            }
        }
    }
    panic!("No se encontró un DejaVuSans.ttf válido — SWAL Files necesita una fuente TTF");
}

struct SwalFilesApp {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    window: Window,
    pool: Option<smithay_client_toolkit::shm::slot::SlotPool>,
    width: u32,
    height: u32,
    configured: bool,
    redraw: bool,
    exit: bool,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,
    current_path: PathBuf,
    items: Vec<String>,
    selected_index: usize,
    scroll_offset: usize,
    font: FontArc,
    tabs: Vec<TabState>,
    active_tab_id: usize,
    _dummy: (),
}

impl SwalFilesApp {
    fn reload_dir(&mut self) {
        self.items = scan_directory(&self.current_path, &ScanOptions::default())
            .unwrap_or_default()
            .into_iter()
            .map(|e| {
                let icon = if e.is_dir { "[D]" } else { "[F]" };
                format!("{} {}  {}", icon, e.name, e.formatted_size)
            })
            .collect();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.redraw = true;
    }

    fn raw_name(&self, idx: usize) -> Option<String> {
        let line = self.items.get(idx)?;
        let bare = line
            .strip_prefix("[D] ")
            .or_else(|| line.strip_prefix("[F] "))
            .unwrap_or(line);
        // bytes 4+2: "name  size" → split at last double-space
        let name = bare.rsplit_once("  ").map(|(n, _)| n).unwrap_or(bare);
        Some(name.trim().to_string())
    }

    fn is_dir_entry(&self, idx: usize) -> bool {
        self.items.get(idx).map(|l| l.starts_with("[D]")).unwrap_or(false)
    }

    fn open_selected(&mut self) {
        if let Some(name) = self.raw_name(self.selected_index) {
            let target = self.current_path.join(&name);
            if target.is_dir() {
                self.current_path = target;
                self.reload_dir();
            } else {
                let _ = std::process::Command::new("xdg-open").arg(&target).spawn();
            }
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            if parent.exists() {
                self.current_path = parent.to_path_buf();
                self.reload_dir();
            }
        }
    }

    fn draw_frame(&mut self) {
        if !self.configured {
            return;
        }
        let (w, h) = (self.width as usize, self.height as usize);
        if w == 0 || h == 0 {
            return;
        }
        let mut buf: Vec<u32> = vec![BG; w * h];

        let session_state = load_session();
        let session = build_gui_payload(&session_state);

        // Layout dimensions
        let sidebar_w = ((w as f32) * 0.18).max(140.0) as usize;
        let preview_w = ((w as f32) * 0.28) as usize;
        let content_x = sidebar_w;
        let content_w = w.saturating_sub(sidebar_w + preview_w);

        // ── 1. Tab strip (height=32px, above toolbar) ──────────────────
        fill_rect(&mut buf, w, h, 0, 0, w, 32, ELEVATED);
        let tab_w = 160usize;
        for (i, tab) in self.tabs.iter().enumerate() {
            let tab_x = i * tab_w;
            if tab_x + tab_w > w {
                break;
            }
            let is_active = tab.id == self.active_tab_id || tab.active;
            let bg_col = if is_active { ELEVATED } else { BG };
            fill_rect(&mut buf, w, h, tab_x, 0, tab_w, 32, bg_col);
            if is_active {
                fill_rect(&mut buf, w, h, tab_x, 30, tab_w, 2, ACCENT);
            } else {
                draw_line_h(&mut buf, w, h, tab_x, 31, tab_w, BORDER);
            }
            let title_trunc: String = tab.title.chars().take(16).collect();
            let text_col = if is_active { TEXT } else { TEXT_DIM };
            draw_text_trunc(&mut buf, w, h, &self.font, 13.0, tab_x + 12, 8, &title_trunc, text_col, is_active, tab_w - 20);
            draw_separator_v(&mut buf, w, h, tab_x + tab_w - 1, 0, 32, BORDER);
        }
        let plus_x = self.tabs.len() * tab_w + 12;
        if plus_x < w {
            draw_text(&mut buf, w, h, &self.font, 15.0, plus_x, 7, "+", TEXT_DIM, false);
            draw_line_h(&mut buf, w, h, self.tabs.len() * tab_w, 31, w.saturating_sub(self.tabs.len() * tab_w), BORDER);
        }

        // ── 2. Toolbar / breadcrumb strip (height=28px) ───────────────
        fill_rect(&mut buf, w, h, 0, 32, w, 28, BG);
        // Action buttons
        draw_text(&mut buf, w, h, &self.font, 13.0, content_x + 10, 38, "⮜  ⮝  ⟳", TEXT_DIM, false);
        draw_separator_v(&mut buf, w, h, content_x + 75, 34, 24, BORDER);

        // Breadcrumb path display
        let breadcrumbs = session.breadcrumbs.clone();
        let mut bx = content_x + 85;
        for (i, item) in breadcrumbs.iter().enumerate() {
            if bx >= content_x + content_w - 80 {
                break;
            }
            let is_last = i + 1 == breadcrumbs.len();
            let col = if is_last { ACCENT } else { TEXT_DIM };
            draw_text_trunc(&mut buf, w, h, &self.font, 13.0, bx, 38, &item.name, col, is_last, 120);
            bx += item.name.len() * 7 + 10;
            if !is_last {
                draw_text(&mut buf, w, h, &self.font, 13.0, bx, 38, "›", TEXT_DIM, false);
                bx += 14;
            }
        }
        let filter_label = format!("[{}]", session.filter_type);
        draw_text(&mut buf, w, h, &self.font, 12.0, content_x + content_w - 75, 38, &filter_label, ACCENT2, false);
        draw_line_h(&mut buf, w, h, 0, 59, w, BORDER);

        // ── 3. Sidebar (pins + disk meters from session) ───────────────
        fill_rect(&mut buf, w, h, 0, 60, sidebar_w, h.saturating_sub(80), ELEVATED);
        draw_separator_v(&mut buf, w, h, sidebar_w - 1, 60, h.saturating_sub(80), BORDER);

        let mut sy = 68usize;
        // Section: FAVORITOS
        draw_text(&mut buf, w, h, &self.font, 10.0, 12, sy, "FAVORITOS", TEXT_DIM, true);
        sy += 18;
        for fav in &session.favorites {
            if sy + 22 > h - 140 {
                break;
            }
            let is_act = fav.is_active || fav.path == self.current_path.to_string_lossy();
            if is_act {
                fill_rect(&mut buf, w, h, 0, sy - 2, 3, 18, ACCENT);
            }
            let col = if is_act { TEXT } else { TEXT_DIM };
            let line = format!("{} {}", fav.icon, fav.name);
            draw_text_trunc(&mut buf, w, h, &self.font, 13.0, 12, sy, &line, col, is_act, sidebar_w - 20);
            sy += 22;
        }

        // Section: ESPACIOS
        sy += 6;
        draw_text(&mut buf, w, h, &self.font, 10.0, 12, sy, "ESPACIOS", TEXT_DIM, true);
        sy += 18;
        for ws_item in &session.workspaces {
            if sy + 22 > h - 100 {
                break;
            }
            let is_act = ws_item.is_active || ws_item.path == self.current_path.to_string_lossy();
            if is_act {
                fill_rect(&mut buf, w, h, 0, sy - 2, 3, 18, ACCENT);
            }
            let col = if is_act { TEXT } else { TEXT_DIM };
            let line = format!("{} {}", ws_item.icon, ws_item.name);
            draw_text_trunc(&mut buf, w, h, &self.font, 13.0, 12, sy, &line, col, is_act, sidebar_w - 20);
            sy += 22;
        }

        // Section: UNIDADES
        sy += 6;
        draw_text(&mut buf, w, h, &self.font, 10.0, 12, sy, "UNIDADES", TEXT_DIM, true);
        sy += 18;
        for disk in &session.disks {
            if sy + 32 > h - 30 {
                break;
            }
            let label = format!("{} {} ({}%)", disk.icon, disk.label, disk.used_pct as u8);
            draw_text_trunc(&mut buf, w, h, &self.font, 11.0, 12, sy, &label, TEXT_DIM, false, sidebar_w - 20);
            sy += 16;
            fill_rect(&mut buf, w, h, 12, sy, sidebar_w - 24, 5, SELECTED);
            let bar_w = ((sidebar_w - 24) as f32 * (disk.used_pct / 100.0).clamp(0.0, 1.0)) as usize;
            let bar_col = if disk.used_pct > 90.0 { DANGER } else { SUCCESS };
            fill_rect(&mut buf, w, h, 12, sy, bar_w, 5, bar_col);
            sy += 14;
        }

        // ── 4. File list with scroll ────────────────────────────────────
        let row_h = 24usize;
        let list_top = 62usize;
        let visible_rows = (h.saturating_sub(list_top + 24)) / row_h;
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
        if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index.saturating_sub(visible_rows - 1);
        }
        let mut y = list_top;
        for i in self.scroll_offset..self.items.len().min(self.scroll_offset + visible_rows) {
            let is_sel = i == self.selected_index;
            if is_sel {
                fill_rect(&mut buf, w, h, content_x, y - 2, content_w, row_h, SELECTED);
            }
            if let Some(line) = self.items.get(i) {
                let color = if is_sel {
                    ACCENT
                } else if self.is_dir_entry(i) {
                    DIR_COLOR
                } else {
                    TEXT
                };
                draw_text_trunc(&mut buf, w, h, &self.font, 14.0, content_x + 16, y + 4, line, color, is_sel, content_w - 32);
            }
            y += row_h;
        }

        // ── 5. Preview panel ───────────────────────────────────────────
        fill_rect(&mut buf, w, h, content_x + content_w, 60, preview_w, h.saturating_sub(80), ELEVATED);
        draw_separator_v(&mut buf, w, h, content_x + content_w, 60, h.saturating_sub(80), BORDER);
        draw_text(&mut buf, w, h, &self.font, 14.0, content_x + content_w + 12, 68, "Vista Previa", TEXT_DIM, false);
        if let Some(name) = self.raw_name(self.selected_index) {
            let p = self.current_path.join(&name);
            let mut lines: Vec<String> = Vec::new();
            if p.is_dir() {
                lines.push(format!("📁 {}", name));
            } else if let Ok(content) = std::fs::read_to_string(&p) {
                lines = content.lines().take(40).map(|l| l.to_string()).collect();
            } else {
                lines.push("(binario — sin preview de texto)".to_string());
            }
            let mut py = 92;
            for (i, line) in lines.iter().enumerate() {
                if py + 16 > h - 24 || i > 60 {
                    break;
                }
                let lineno = format!("{:>3} │ {}", i + 1, line);
                draw_text_trunc(&mut buf, w, h, &self.font, 12.0, content_x + content_w + 12, py, &lineno, TEXT_DIM, false, preview_w - 24);
                py += 16;
            }
        }

        // ── 6. Status bar (bottom 20px) ─────────────────────────────────
        fill_rect(&mut buf, w, h, 0, h - 20, w, 20, ELEVATED);
        draw_line_h(&mut buf, w, h, 0, h - 20, w, BORDER);
        draw_text(
            &mut buf, w, h, &self.font, 12.0, 12, h - 15,
            &format!("{} elementos", self.items.len()),
            TEXT_DIM, false,
        );
        if session.git_status.is_git_repo {
            let git_info = format!(" {}", session.git_status.branch);
            draw_text(&mut buf, w, h, &self.font, 12.0, w / 2 - 40, h - 15, &git_info, ACCENT2, false);
        }
        draw_text(&mut buf, w, h, &self.font, 12.0, w - 110, h - 15, "SWAL Files ⚡", ACCENT, true);

        // Present via wl_shm
        let (w32, h32) = (w as i32, h as i32);
        let stride = w32 * 4;
        let mut pool = self.pool.take().unwrap_or_else(|| {
            smithay_client_toolkit::shm::slot::SlotPool::new((w * h * 4) as usize, &self.shm).expect("pool")
        });
        let buffer = pool
            .create_buffer(w32, h32, stride, wl_shm::Format::Argb8888)
            .expect("create buffer")
            .0;
        if let Some(canvas) = pool.canvas(&buffer) {
            for (dst, src) in canvas.chunks_exact_mut(4).zip(buf.iter()) {
                let px = *src;
                dst[0] = (px >> 16) as u8; // 0RGB → B,G,R,A little-endian
                dst[1] = (px >> 8) as u8;
                dst[2] = px as u8;
                dst[3] = (px >> 24) as u8;
            }
        }
        buffer.attach_to(self.window.wl_surface()).expect("attach");
        self.window.wl_surface().damage_buffer(0, 0, w32, h32);
        self.window.commit();
        self.pool = Some(pool);
    }

    fn save_current_path(&self) {
        let mut session = load_session();
        session.active_tab_id = self.active_tab_id;
        if let Some(tab) = session.tabs.iter_mut().find(|t| t.id == self.active_tab_id) {
            tab.path = self.current_path.to_string_lossy().to_string();
            tab.title = self.current_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "/".to_string());
        } else if let Some(first) = session.tabs.first_mut() {
            first.path = self.current_path.to_string_lossy().to_string();
            first.title = self.current_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "/".to_string());
        } else {
            session.tabs.push(TabState {
                id: 1,
                title: self.current_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "/".to_string()),
                path: self.current_path.to_string_lossy().to_string(),
                active: true,
            });
        }
        let _ = save_session(&session);
    }
}

// ── drawing helpers ──────────────────────────────────────────────────────

fn draw_line_h(buf: &mut [u32], w: usize, _h: usize, x: usize, y: usize, len: usize, color: u32) {
    if y >= _h { return; }
    let end_x = (x + len).min(w);
    let row_start = y * w;
    for px in x.min(w)..end_x {
        buf[row_start + px] = color;
    }
}

fn draw_separator_v(buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, len: usize, color: u32) {
    if x >= w { return; }
    let end_y = (y + len).min(h);
    for py in y.min(h)..end_y {
        buf[py * w + x] = color;
    }
}

fn draw_badge_dot(buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, color: u32) {
    fill_rect(buf, w, h, cx.saturating_sub(2), cy.saturating_sub(2), 4, 4, color);
}

fn fill_rect(buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, rw: usize, rh: usize, color: u32) {
    let x1 = (x + rw).min(w);
    let y1 = (y + rh).min(h);
    for row in y.min(h)..y1 {
        let start = row * w + x.min(w);
        let end = row * w + x1;
        buf[start..end].fill(color);
    }
}

fn draw_text(buf: &mut [u32], w: usize, h: usize, font: &FontArc, px: f32, x: usize, y: usize, text: &str, color: u32, bold: bool) {
    draw_text_trunc(buf, w, h, font, px, x, y, text, color, bold, usize::MAX);
}

fn draw_text_trunc(
    buf: &mut [u32],
    w: usize,
    h: usize,
    font: &FontArc,
    px: f32,
    x: usize,
    y: usize,
    text: &str,
    color: u32,
    bold: bool,
    max_w: usize,
) {
    let scale = PxScale::from(px * if bold { 1.05 } else { 1.0 });
    let scaled = font.as_scaled(scale);
    let mut cx = x;
    let limit = x.saturating_add(max_w);
    for ch in text.chars() {
        if cx >= limit {
            break;
        }
        let gid = font.glyph_id(ch);
        // Notdef (emoji etc. ausentes en DejaVu): avanzar sin dibujar para no
        // empujar el resto de la línea fuera del ancho visible.
        if gid.0 == 0 {
            cx += (px * 0.55) as usize;
            continue;
        }
        let glyph = scaled.scaled_glyph(ch);
        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            let ox = bounds.min.x as i32;
            let oy = bounds.min.y as i32;
            outline.draw(|gx, gy, cov| {
                if cov <= 0.0 {
                    return;
                }
                let xx = x as i32 + ox + gx as i32;
                let yy = y as i32 + oy + gy as i32;
                if xx >= 0 && yy >= 0 && xx < w as i32 && yy < h as i32 {
                    let idx = (yy as usize) * w + (xx as usize);
                    if idx < buf.len() {
                        buf[idx] = blend(buf[idx], color, cov);
                    }
                }
            });
        }
        cx += scaled.h_advance(gid) as usize;
    }
}

fn blend(bg: u32, fg: u32, alpha: f32) -> u32 {
    let a = alpha.clamp(0.0, 1.0);
    let (bgr, bgg, bgb) = ((bg >> 16) & 0xFF, (bg >> 8) & 0xFF, bg & 0xFF);
    let (fgr, fgg, fgb) = ((fg >> 16) & 0xFF, (fg >> 8) & 0xFF, fg & 0xFF);
    let r = (bgr as f32 * (1.0 - a) + fgr as f32 * a) as u32;
    let g = (bgg as f32 * (1.0 - a) + fgg as f32 * a) as u32;
    let b = (bgb as f32 * (1.0 - a) + fgb as f32 * a) as u32;
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ── sctk 0.19 handlers ───────────────────────────────────────────────────

impl CompositorHandler for SwalFilesApp {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for SwalFilesApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl SeatHandler for SwalFilesApp {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            // Manual bind: sctk keyboard module needs xkbcommon (not in offline
            // cache), so we drive wl_keyboard events ourselves (keycodes X11).
            let kb: wl_keyboard::WlKeyboard = seat.get_keyboard(qh, ());
            self.keyboard = Some(kb);
        }
        if capability == Capability::Pointer && self.pointer.is_none() {
            if let Ok(pt) = self.seat_state.get_pointer(qh, &seat) {
                self.pointer = Some(pt);
            }
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }
        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
        }
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

// ── Manual wl_keyboard dispatch (xkbcommon no está en el cache offline) ──
// Mapeo de keycodes X11 hacia las teclas que usa la UI (layout US/ES basta).
fn handle_x11_keycode(app: &mut SwalFilesApp, keycode: u32) {
    match keycode {
        // q=24, esc=9
        24 | 9 => {
            app.save_current_path();
            app.exit = true;
        }
        // j=44, down=116
        44 | 116 => {
            if app.selected_index + 1 < app.items.len() {
                app.selected_index += 1;
                app.redraw = true;
            }
        }
        // k=45, up=111
        45 | 111 => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
                app.redraw = true;
            }
        }
        // l=46, right=114, enter=36
        46 | 114 | 36 => app.open_selected(),
        // h=43, left=113, backspace=22
        43 | 113 | 22 => app.go_up(),
        // r=27
        27 => app.reload_dir(),
        // p=33
        33 => app.redraw = true,
        _ => {}
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for SwalFilesApp {
    fn event(
        state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                key, state: kstate, ..
            } => {
                if matches!(kstate, wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed)) {
                    handle_x11_keycode(state, key);
                }
            }
            _ => {}
        }
    }
}

impl PointerHandler for SwalFilesApp {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            if &event.surface != self.window.wl_surface() {
                continue;
            }
            match event.kind {
                PointerEventKind::Motion { .. } => {
                    let (sx, sy) = event.position;
                    self.hit_test(sx as usize, sy as usize);
                }
                PointerEventKind::Press { button: 0x110, .. } => {
                    self.open_selected();
                }
                _ => {}
            }
        }
    }
}

impl ShmHandler for SwalFilesApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl WindowHandler for SwalFilesApp {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.save_current_path();
        self.exit = true;
    }
    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window, configure: WindowConfigure, serial: u32) {
        if let (Some(w), Some(h)) = (configure.new_size.0, configure.new_size.1) {
            self.width = w.get().max(200);
            self.height = h.get().max(200);
        }
        self.configured = true;
        self.redraw = true;
        self.window.xdg_surface().ack_configure(serial);
    }
}

impl ProvidesRegistryState for SwalFilesApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

impl SwalFilesApp {
    fn hit_test(&mut self, sx: usize, sy: usize) {
        let (w, h) = (self.width as usize, self.height as usize);
        let sidebar_w = ((w as f32) * 0.18).max(140.0) as usize;
        let preview_w = ((w as f32) * 0.28) as usize;
        let content_w = w.saturating_sub(sidebar_w + preview_w);
        if sx < sidebar_w || sx >= sidebar_w + content_w || sy < 60 || sy >= h.saturating_sub(20) {
            return;
        }
        let row_h = 24usize;
        let row = (sy - 60) / row_h;
        let idx = self.scroll_offset + row;
        if idx < self.items.len() {
            self.selected_index = idx;
            self.redraw = true;
        }
    }
}

delegate_compositor!(SwalFilesApp);
delegate_output!(SwalFilesApp);
delegate_registry!(SwalFilesApp);
delegate_seat!(SwalFilesApp);
delegate_pointer!(SwalFilesApp);
delegate_shm!(SwalFilesApp);
delegate_xdg_shell!(SwalFilesApp);
delegate_xdg_window!(SwalFilesApp);