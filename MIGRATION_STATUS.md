# SWAL Files — Estado de Migración EWW → Rust Pure

> **Para el agente trabajando en la migración**: Lee esto antes de empezar.
> El backend lógico YA ESTÁ COMPLETO. Solo falta la capa de renderizado.

## Backend Lógico: ✅ 100% COMPLETO

| Módulo | Archivo | Estado |
|--------|---------|--------|
| Scanner directorios | `scanner.rs` | ✅ sort/filter/group |
| Git integration | `git.rs` | ✅ status por dir |
| Preview Yazi sidebar | `preview.rs` | ✅ syntax highlighting |
| Session management | `session.rs` | ✅ persistence JSON |
| Tabs extendidos | `tabs_extended.rs` | ✅ multi-tab |
| Omnibar búsqueda | `omnibar.rs` | ✅ command palette |
| Dual pane | `dual_pane.rs` | ✅ navegación |
| Disk usage | `storage.rs` | ✅ scan drives |
| Archive inspector | `archive.rs` | ✅ tar/zip/etc |
| Directory watcher | `watcher.rs` | ✅ inotify |
| Agent protocol | `agent.rs` | ✅ AI integration |
| CLI dispatcher | `cli.rs` | ✅ view-json/nav/tab-close |
| GUI payload | `gui.rs` | ✅ (temporary para EWW) |
| Config persistence | `config.rs` | ✅ |
| Native window tree | `native_window.rs` | 🟡 75% — build A2UI tree |

## Lo que FALTA (capa de renderizado) — ENFOCAR AQUÍ

### 1. `swal-render-pipeline` (crates/swal-render-pipeline/)
- Conectar el `ComponentNode` tree al GPU renderer
- Implementar Wayland surface creation directamente (zwlr_layer_shell_v1)
- Event loop para input handling (clicks, teclado, resize)

### 2. `swal-a2ui-engine` (crates/swal-a2ui-engine/)
- Layout engine: posicionar componentes Card/Grid/Tabs en pantalla
- Hit testing: detectar qué componente fue clickeado
- Text rendering: dibujar labels con FreeType/skia

### 3. Wayland Integration
- `wl_surface` + `zwlr_layer_surface_v1` para crear ventanas
- Keyboard interactivity: OnDemand para overlays, Exclusive para apps
- No más dependencia de EWW/GTK

## UI Design (inspirada en C# File Explorer)
```
┌─────────────────────────────────────────────────────┐
│ [📂 Home] [📁 Proyectos] [+]              ─  □  ✕  │  ← Tab strip
├─────────────────────────────────────────────────────┤
│ ⮜ ⮝ │ Home › Proyectos › swal-desktop │ 📌 Pine │  │  ← Toolbar
├────────┬──────────────────────┬────────────────────┤
│ 📌 Home│ 📂 cores             │ Vista Previa:      │
│ 📁 Down│ 📂 apps              │ cli.rs             │
│ 📁 Docs│ 📂 periferia         │                    │
│        │ 📄 README.md         │ 1: //! CLI...      │
│ 🖴 /   │ 📄 Cargo.toml        │ 2: use std::...    │
│ 65%    │ 📄 AGENTS.md         │ 3: ...             │
│ 🖴 SSD │                      │                    │
│ 80%    │                      │ [Terminal] [Copiar]│
└────────┴──────────────────────┴────────────────────┘
  Sidebar      Content (file list)     Preview panel
```

## Bugs de EWW que motivan la migración
- EWW daemon pierde track de ventanas (zombie windows)
- `:ondblclick` no soportado → 43K warnings
- defpoll empty string corrompe daemon
- Focus events no reportados en Wayland

## Commands CLI útiles
```bash
# Ver JSON del estado actual
swal-files view-json

# Navegar a directorio
swal-files nav /path/to/dir

# Cerrar tab
swal-files tab-close <id>

# Abrir item
swal-files open-item /path/to/file
```

## Archivos clave
- **Repo**: `~/proyectosSWAL/periferia/swal-desktop/`
- **Binary**: `target/debug/swal-files`
- **EWW config (legacy)**: `~/.config/eww/eww.yuck`
- **Cargo.toml**: workspace con 8 crates
