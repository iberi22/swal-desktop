# Checklist de Paridad Eww → Shell Nativa Rust

> **Objetivo:** Verificar que cada superficie que Eww renderiza hoy tiene equivalente
> nativo (swal-node-daemon + render-pipeline + A2UI) ANTES de ejecutar el Task 3.5
> (`git rm -r eww/`). Este documento es la precondición del hito Zero-Eww.
>
> **Regla:** una fila solo puede marcarse ✅ cuando el binario nativo levanta la
> superficie y responde a su keybind sin proceso Eww corriendo.

---

## 1. Inventario Eww (11 ventanas activas)

Fuente: `eww/eww.yuck` + `eww/hermes_orb.yuck` (grep `defwindow`, verificado 2026-08-25).

| # | Ventana Eww | Función | Superficie nativa | Estado |
|---|---|---|---|---|
| 1 | `dashboard` | Dashboard principal con telemetría | `TelemetryBar` / render-pipeline orb_surface | 🟡 PARCIAL — existe superficie, falta paridad visual completa |
| 2 | `ram_panel` | Monitor de procesos + kill | A2UI ProcessTable (`native_render.rs` ya rasteriza) | 🟡 PARCIAL |
| 3 | `osd` | On-screen display de volumen/brillo | Notification surface (Wave 7) | 🔴 PENDIENTE |
| 4 | `keybinds_panel` | Ayuda de atajos | A2UI modal | 🔴 PENDIENTE |
| 5 | `agent_chat` | Chat overlay del agente | Hermes streamer (a2ui) | 🔴 PENDIENTE |
| 6 | `agent_admin` | Panel admin de agentes | A2UI settings components | 🔴 PENDIENTE |
| 7 | `swal_editor` | Editor rápido de texto | swal-files viewer/editor | 🟡 PARCIAL |
| 8 | `swal_files` | File manager dual-pane | **Nativo completo** (`swal-files` bin, supervisor SwalFiles) | ✅ HECHO |
| 9 | `swal_files_maximized` | Variante maximizada | Nativo (is_maximized en SessionState) | ✅ HECHO |
| 10 | `swal_settings` | Settings GUI (backend `swal_settings.py`) | settings_window.rs + settings_cli.rs (Task 3.2) | 🟡 PARCIAL — backend listo, CLI bin pendiente |
| 11 | `hermes_orb` | Orbe ambiente GLSL | **Nativo completo** (`swal-orb`, supervisor HermesOrb) | ✅ HECHO |

## 2. Scripts Eww y su reemplazo

| Script | Llamado por | Reemplazo nativo | Estado |
|---|---|---|---|
| `hermes_orb_menu.py` | keybinds del orbe | Cliente IPC Rust (Task 3.3) | 🟡 código escrito, bin CLI pendiente |
| `swal_settings.py` | ventana settings | `settings_cli.rs` → bin `swal-settings` (Task 3.2) | 🟡 módulo existe como lib, falta exponer bin |
| `toggle_dashboard.sh` | SUPER+Escape | Acción nativa toggle (Task 3.1) | 🟡 evento existe, fallback .sh aún activo |
| `toggle_orb_hud.sh` | keybind orbe | Ídem | 🟡 ídem |
| `ram_kill.sh` | botón kill en ram_panel | IPC action card | 🔴 PENDIENTE |
| `ai_status.sh` / `sys_info.sh` | polling dashboard | Telemetry socket (<0.2ms) | 🟡 datos fluyen, formato JSON pendiente |

## 3. Keybinds a re-cablear (Task 3.5)

Verificar en `hypr/` antes de apagar:

- [ ] SUPER+Escape → `toggle-dashboard`: hoy cae a `/home/belal/.config/eww/scripts/toggle_dashboard.sh`
- [ ] SUPER+E → `swal-files` nativo (ya no `eww open swal_files`)
- [ ] SUPER+Q → `close-files`: hoy usa `eww close swal_files*`
- [ ] SUPER+O → orbe: hoy `toggle_orb_hud.sh`

**Nota:** tras Task 2.x (portabilidad) estos paths ya no deben ser absolutos.

## 4. Criterio de apagado (gate para Task 3.5)

NO ejecutar `git rm -r eww/` hasta cumplir TODO:

1. Las 11 filas de la tabla 1 en ✅ o con decisión documentada de descarte.
2. Los 6 scripts de la tabla 2 eliminados o reemplazados.
3. Los 4 keybinds de la sección 3 operativos sin eww en el comando.
4. `grep -rn "eww" crates/ hypr/ nixos/ flake.nix scripts/swal-session.sh` devuelve solo comentarios históricos.
5. `cargo test --workspace` verde + smoke test visual de 24h con Eww desinstalado del perfil.
6. Backup histórico previo en `~/proyectosSWAL/periferia/archivo/swal-desktop-eww-legacy/` (FUERA del repo).

## 5. Registro

| Fecha | Avance |
|---|---|
| 2026-08-25 | Inventario inicial creado; clasificación por ventana (3 ✅, 4 🟡, 4 🔴). Tasks hijas derivadas: OSD nativo, keybinds panel, agent chat overlay, agent admin panel, ram_kill IPC, bins `swal-settings` y `swal-orb-action`. |
