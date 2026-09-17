#!/usr/bin/env bash
# swal_ctx_open.sh — abre el MENU CONTEXTUAL del panel SWAL Files.
#
# Diseño (2026-09-11): el menú se dibuja DENTRO de la ventana del panel, activando
# la variable `swal_files_ctx_open` (el panel central pasa de lista a acciones).
# NO se usa una ventana layer-shell aparte: en EWW 0.6 + Hyprland esa segunda
# ventana no queda de forma fiable por encima del panel (con :stacking "overlay"
# y "foreground" el panel la tapaba; verificado muestreando píxeles con grim +
# ImageMagick). Dentro de la ventana el orden lo garantiza GTK.
#
# El payload de acciones se calcula CON la ruta explícita (`swal-files menu-json
# <ruta>`; sin argumento el backend devuelve {"actions":[],"target":""}) y se deja
# en ~/.cache/swal-files/ctx.json para que el defpoll lo relea.
#
# Uso: swal_ctx_open.sh <ruta>
set -u
FILES=${SWAL_ROOT:-$HOME/proyectosSWAL}/periferia/swal-desktop/target/debug/swal-files
CACHE="$HOME/.cache/swal-files/ctx.json"
TARGET="${1:-}"

# 1. Marcar la fila en el panel (feedback visual del clic derecho).
if [ -n "$TARGET" ]; then
  "$FILES" select-item "$TARGET" >/dev/null 2>&1
fi

# 2. Payload de acciones para ESE target.
if [ -n "$TARGET" ]; then
  JSON=$("$FILES" menu-json "$TARGET" 2>/dev/null)
else
  JSON=""
fi
case "${JSON:-}" in
  *'"actions"'*)
    mkdir -p "$(dirname "$CACHE")"
    printf '%s' "$JSON" > "$CACHE"
    timeout 6 eww update "swal_files_data_ctx=$JSON" >/dev/null 2>&1
    ;;
esac

# 3. Abrir el modo menú en el panel.
timeout 6 eww update "swal_files_ctx_open=true" >/dev/null 2>&1
exit 0
