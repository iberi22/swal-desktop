//! swal-pointer — mini daemon de puntero para Hyprland (fase 1: movimiento).
//!
//! Sin dependencias: movimiento por el socket de Hyprland (`movecursor` ABSOLUTO,
//! verificado en vivo) y clicks/teclas por /dev/uinput con syscalls directas
//! (este Hyprland no expone virtual-pointer/virtual-keyboard).
//! Arquitectura ydotool: un daemon + clientes cortos por evento de tecla.
//!
//! Protocolo (una linea por conexion al socket unix):
//!   `down up|down|left|right`  press (repetidos por key-repeat se ignoran)
//!   `up <dir>`                 release: tap corto = nudge fino (fase 2: snap AT-SPI)
//!   `click left|right|middle`  click en la posicion actual
//!   `pos`                      responde `x,y` con la posicion actual
//!   `quit`                     apaga el daemon
//!
//! Binds directos (sin submapa: imposible quedar atrapado):
//!   bind = SUPER, up, exec, swal-pointer down up (+ bindr ... up up, etc.)
//!   bind = SUPER, Tab, exec, swal-pointer click left
//! movefocus vive en SUPER+ALT+flechas (SUPER+flechas = puntero).

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::{Duration, Instant};

// --- sintonia (fase 1) -------------------------------------------------------
const GRACE_MS: u64 = 180; // bajo esto es tap, sobre esto es hold con aceleracion
const TICK_MS: u64 = 16; // ~60 Hz, fluido
const BASE_PX_S: f64 = 220.0; // velocidad inicial (movimiento fino)
const MAX_PX_S: f64 = 2400.0; // tope (cruzar pantalla rapido)
const DOUBLING_MS: f64 = 550.0; // la velocidad se duplica cada X ms sostenido
const TAP_JUMP_PX: f64 = 48.0; // fase 1: nudge fino. Fase 2: snap AT-SPI.
const WATCHDOG_SECS: u64 = 5; // ningun hold sobrevive a esto (anti-atasco).

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn parse(s: &str) -> Option<Dir> {
        match s {
            "up" => Some(Dir::Up),
            "down" => Some(Dir::Down),
            "left" => Some(Dir::Left),
            "right" => Some(Dir::Right),
            _ => None,
        }
    }

    fn vec(self) -> (f64, f64) {
        match self {
            Dir::Up => (0.0, -1.0),
            Dir::Down => (0.0, 1.0),
            Dir::Left => (-1.0, 0.0),
            Dir::Right => (1.0, 0.0),
        }
    }
}

// --- socket Hyprland ----------------------------------------------------------
fn hypr_sock_path() -> String {
    let rt = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
    format!("{rt}/hypr/{his}/.socket.sock")
}

/// Envia un comando al socket1 y devuelve la respuesta (el servidor cierra).
fn hypr_cmd(cmd: &str) -> Option<String> {
    let mut s = UnixStream::connect(hypr_sock_path()).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    s.write_all(cmd.as_bytes()).ok()?;
    let mut buf = Vec::new();
    // read_to_end termina cuando Hyprland cierra; el timeout nos rescata si no.
    if s.read_to_end(&mut buf).is_err() && buf.is_empty() {
        return None;
    }
    String::from_utf8(buf).ok()
}

fn cursor_pos() -> Option<(f64, f64)> {
    let r = hypr_cmd("cursorpos")?;
    let mut it = r.trim().split(',');
    let x: f64 = it.next()?.trim().parse().ok()?;
    let y: f64 = it.next()?.trim().parse().ok()?;
    Some((x, y))
}

fn move_to(x: f64, y: f64) {
    let _ = hypr_cmd(&format!("dispatch movecursor {} {}", x.round(), y.round()));
}

// --- clicks y teclas via /dev/uinput (syscalls directas, sin libc) ------------------
// Este Hyprland no expone virtual-pointer/virtual-keyboard (cero strings en el
// binario), pero belal tiene ACL rw en /dev/uinput: emitimos ahi.
// Solo EMITIMOS input sintetico (como ydotool); nunca leemos el teclado.
use std::arch::asm;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

const SYS_IOCTL: i64 = 16;
const UI_SET_EVBIT: i64 = 0x4004_5564; // _IOW('U', 100, int)
const UI_SET_KEYBIT: i64 = 0x4004_5565; // _IOW('U', 101, int)
const UI_DEV_SETUP: i64 = 0x405c_5503; // _IOW('U', 3, uinput_setup de 92 bytes)
const UI_DEV_CREATE: i64 = 0x5501; // _IO('U', 1)
const UI_DEV_DESTROY: i64 = 0x5502; // _IO('U', 2)
const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const SYN_REPORT: u16 = 0;
const KEY_LEFTCTRL: u16 = 29;
const KEY_V: u16 = 47;

unsafe fn ioctl3(fd: i32, req: i64, arg: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        inlateout("rax") SYS_IOCTL => ret,
        in("rdi") i64::from(fd),
        in("rsi") req,
        in("rdx") arg as i64,
        out("rcx") _,
        out("r11") _,
    );
    ret
}

/// Crea un dispositivo uinput efimero con los codigos dados. Vive una operacion.
fn uinput_open(codes: &[u16]) -> Option<std::fs::File> {
    let f = OpenOptions::new().write(true).open("/dev/uinput").ok()?;
    let fd = f.as_raw_fd();
    unsafe {
        if ioctl3(fd, UI_SET_EVBIT, u64::from(EV_SYN)) < 0 {
            return None;
        }
        if ioctl3(fd, UI_SET_EVBIT, u64::from(EV_KEY)) < 0 {
            return None;
        }
        for c in codes {
            if ioctl3(fd, UI_SET_KEYBIT, u64::from(*c)) < 0 {
                return None;
            }
        }
        // struct uinput_setup: id (8) + name[80] + ff_effects_max (4) = 92
        let mut setup = [0u8; 92];
        setup[0..2].copy_from_slice(&3u16.to_le_bytes()); // BUS_USB
        setup[2..4].copy_from_slice(&0x1209u16.to_le_bytes());
        setup[4..6].copy_from_slice(&1u16.to_le_bytes());
        setup[6..8].copy_from_slice(&1u16.to_le_bytes());
        let name = b"swal-pointer";
        setup[8..8 + name.len()].copy_from_slice(name);
        if ioctl3(fd, UI_DEV_SETUP, setup.as_ptr() as u64) < 0 {
            return None;
        }
        if ioctl3(fd, UI_DEV_CREATE, 0) < 0 {
            return None;
        }
    }
    std::thread::sleep(Duration::from_millis(200)); // el kernel da de alta el device
    Some(f)
}

/// struct input_event de 24 bytes en x86_64 (timeval 16 + type/code/value 8).
/// timeval en cero = el kernel estampa la hora actual.
fn emit(f: &mut std::fs::File, typ: u16, code: u16, val: i32) -> bool {
    let mut e = [0u8; 24];
    e[16..18].copy_from_slice(&typ.to_le_bytes());
    e[18..20].copy_from_slice(&code.to_le_bytes());
    e[20..24].copy_from_slice(&val.to_le_bytes());
    use std::io::Write as _;
    f.write_all(&e).is_ok()
}

fn key(f: &mut std::fs::File, code: u16, down: bool) -> bool {
    emit(f, EV_KEY, code, i32::from(down)) && emit(f, EV_SYN, SYN_REPORT, 0)
}

fn uinput_destroy(f: &std::fs::File) {
    unsafe {
        ioctl3(f.as_raw_fd(), UI_DEV_DESTROY, 0);
    }
}

/// Click completo (press + release). Botones: 0x110 izq, 0x111 der, 0x112 medio.
fn dev_click(btn: u16) -> bool {
    let mut f = match uinput_open(&[btn]) {
        Some(f) => f,
        None => return false,
    };
    let ok = key(&mut f, btn, true);
    std::thread::sleep(Duration::from_millis(60));
    let ok = ok && key(&mut f, btn, false);
    uinput_destroy(&f);
    ok
}

/// Ctrl+V (pega el portapapeles; lo usa el dictado PTT en vez de wtype).
fn dev_paste() -> bool {
    let mut f = match uinput_open(&[KEY_LEFTCTRL, KEY_V]) {
        Some(f) => f,
        None => return false,
    };
    let ok = key(&mut f, KEY_LEFTCTRL, true)
        && key(&mut f, KEY_V, true)
        && key(&mut f, KEY_V, false)
        && key(&mut f, KEY_LEFTCTRL, false);
    uinput_destroy(&f);
    ok
}

// --- daemon -------------------------------------------------------------------
fn sock_path() -> String {
    let rt = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    format!("{rt}/swal-pointer.sock")
}

struct Held {
    dir: Dir,
    since: Instant,
    pos: (f64, f64),
}

fn speed_px_s(elapsed_ms: u64) -> f64 {
    let v = BASE_PX_S * 2f64.powf(elapsed_ms as f64 / DOUBLING_MS);
    v.min(MAX_PX_S)
}

/// Si otro daemon responde en el socket, no duplicar (evita holds huerfanos
/// peleando por el puntero).
fn another_lives(path: &str) -> bool {
    let mut s = match UnixStream::connect(path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    s.set_read_timeout(Some(Duration::from_secs(1))).ok();
    if s.write_all(b"pos\n").is_err() {
        return false;
    }
    let mut b = [0u8; 1];
    loop {
        match s.read(&mut b) {
            Ok(0) => return true, // respondio y cerro = vivo
            Ok(_) => {
                if b[0] == b'\n' {
                    return true;
                }
            }
            Err(e) if e.kind() == ErrorKind::TimedOut => return false,
            Err(_) => return false,
        }
    }
}

fn run_daemon() {
    let path = sock_path();
    if another_lives(&path) {
        eprintln!("swal-pointer: otro daemon ya corre, saliendo");
        std::process::exit(1);
    }
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind swal-pointer.sock");
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");
    eprintln!("swal-pointer daemon en {path}");

    let mut held: Option<Held> = None;

    loop {
        // Atiende clientes pendientes sin bloquear.
        loop {
            match listener.accept() {
                Ok((mut c, _)) => {
                    let mut line = String::new();
                    let mut b = [0u8; 1];
                    loop {
                        match c.read(&mut b) {
                            Ok(0) => break,
                            Ok(_) => {
                                if b[0] == b'\n' {
                                    break;
                                }
                                line.push(b[0] as char);
                            }
                            Err(_) => break,
                        }
                    }
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    let reply = match parts.as_slice() {
                        ["down", d] => {
                            if let Some(dir) = Dir::parse(d) {
                                match &held {
                                    // repeat del compositor: mantener aceleracion, no reiniciar
                                    Some(h) if h.dir == dir => "ok repeat",
                                    _ => {
                                        let pos = cursor_pos().unwrap_or((0.0, 0.0));
                                        held = Some(Held {
                                            dir,
                                            since: Instant::now(),
                                            pos,
                                        });
                                        "ok down"
                                    }
                                }
                            } else {
                                "err dir"
                            }
                        }
                        ["up", d] => {
                            if let (Some(h), Some(dir)) = (&held, Dir::parse(d)) {
                                if h.dir == dir {
                                    let ms = h.since.elapsed().as_millis() as u64;
                                    let (dx, dy) = dir.vec();
                                    if ms < GRACE_MS {
                                        // TAP: fase 1 = nudge fino; fase 2 = snap AT-SPI.
                                        let (px, py) = h.pos;
                                        move_to(px + dx * TAP_JUMP_PX, py + dy * TAP_JUMP_PX);
                                        held = None;
                                        "ok tap"
                                    } else {
                                        held = None;
                                        "ok up"
                                    }
                                } else {
                                    "ok other"
                                }
                            } else {
                                "ok none"
                            }
                        }
                        ["click", which] => {
                            let btn: u16 = match *which {
                                "left" => 0x110,
                                "right" => 0x111,
                                "middle" => 0x112,
                                _ => 0,
                            };
                            if btn != 0 && dev_click(btn) {
                                "ok click"
                            } else {
                                "err click"
                            }
                        }
                        ["paste"] => {
                            if dev_paste() {
                                "ok paste"
                            } else {
                                "err paste"
                            }
                        }
                        ["pos"] => {
                            let (x, y) = cursor_pos().unwrap_or((-1.0, -1.0));
                            let r = format!("{x},{y}");
                            let _ = c.write_all(r.as_bytes());
                            let _ = c.write_all(b"\n");
                            continue;
                        }
                        ["quit"] => {
                            let _ = c.write_all(b"ok bye\n");
                            return;
                        }
                        _ => "err uso: down|up <dir> | click <btn> | pos | quit",
                    };
                    let _ = c.write_all(reply.as_bytes());
                    let _ = c.write_all(b"\n");
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        // Watchdog: ningun hold vive mas de WATCHDOG_SECS (anti-atasco).
        if held
            .as_ref()
            .is_some_and(|h| h.since.elapsed() > Duration::from_secs(WATCHDOG_SECS))
        {
            eprintln!("swal-pointer: watchdog libero un hold atascado");
            held = None;
        }
        // Paso de movimiento si hay tecla sostenida mas alla de la gracia.
        if let Some(h) = &mut held {
            let ms = h.since.elapsed().as_millis() as u64;
            if ms >= GRACE_MS {
                let dt = TICK_MS as f64 / 1000.0;
                let step = speed_px_s(ms) * dt;
                let (dx, dy) = h.dir.vec();
                h.pos.0 += dx * step;
                h.pos.1 += dy * step;
                move_to(h.pos.0, h.pos.1); // Hyprland clampea solo a los bordes
            }
        }

        std::thread::sleep(Duration::from_millis(TICK_MS));
    }
}

fn client(args: &[String]) -> i32 {
    let mut s = match UnixStream::connect(sock_path()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("daemon no corre: lanza `swal-pointer daemon` primero");
            return 1;
        }
    };
    s.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let line = args.join(" ");
    if s.write_all(line.as_bytes()).is_err() || s.write_all(b"\n").is_err() {
        eprintln!("no se pudo enviar al daemon");
        return 1;
    }
    let mut resp = String::new();
    let mut b = [0u8; 1];
    loop {
        match s.read(&mut b) {
            Ok(0) => break,
            Ok(_) => {
                if b[0] == b'\n' {
                    break;
                }
                resp.push(b[0] as char);
            }
            Err(_) => break,
        }
    }
    let resp = resp.trim().to_string();
    println!("{resp}");
    if resp.starts_with("ok") { 0 } else { 2 }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("daemon") => run_daemon(),
        Some(_) => std::process::exit(client(&args)),
        None => {
            eprintln!("uso: swal-pointer daemon | down|up <dir> | click <btn> | pos | quit");
            std::process::exit(1);
        }
    }
}
