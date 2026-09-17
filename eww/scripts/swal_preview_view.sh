#!/usr/bin/env bash
# Abre el archivo del preview en un PAGER DE SOLO LECTURA (less) para poder
# seleccionar y copiar texto. NO abre editores: no hay forma de modificar el archivo.
# Uso desde el panel EWW: bash swal_preview_view.sh
FILES_BIN="${SWAL_ROOT:-$HOME/proyectosSWAL}/periferia/swal-desktop/target/debug/swal-files"

P_PATH=$("$FILES_BIN" view-json 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('preview',{}).get('path',''))" 2>/dev/null)

if [ -z "$P_PATH" ] || [ ! -e "$P_PATH" ]; then
  notify-send "SWAL Files" "No hay archivo en el preview" 2>/dev/null
  exit 0
fi

if [ -d "$P_PATH" ]; then
  exec ls -la "$P_PATH" | less -R
fi

# less -R: pager read-only con soporte de color; q para salir. Seleccion con el mouse + Ctrl+Shift+C.
if command -v ghostty >/dev/null 2>&1; then
  ghostty -e less -R "$P_PATH" &
else
  less -R "$P_PATH"
fi
