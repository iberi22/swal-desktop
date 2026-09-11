#!/usr/bin/env bash
# Copia al portapapeles el CONTENIDO del archivo en preview (texto/código).
# Para imágenes/carpetas copia la ruta. Lee el estado real del backend.
FILES_BIN="/home/belal/proyectosSWAL/periferia/swal-desktop/target/debug/swal-files"

JSON=$("$FILES_BIN" view-json 2>/dev/null)
eval "$(echo "$JSON" | python3 -c "
import sys, json
d = json.load(sys.stdin).get('preview', {})
def q(s): return \"'\" + s.replace(\"'\", \"'\\\\''\") + \"'\"
print('P_PATH=' + q(d.get('path','')))
print('P_TEXT=' + ('1' if d.get('is_text') else '0'))
print('P_IMG=' + ('1' if d.get('is_image') else '0'))
print('P_NAME=' + q(d.get('file_name','')))
" 2>/dev/null)"

if [ "$P_TEXT" = "1" ] && [ -f "$P_PATH" ]; then
  wl-copy < "$P_PATH"
  notify-send "SWAL Files" "Contenido de '$P_NAME' copiado al portapapeles" 2>/dev/null
elif [ -n "$P_PATH" ]; then
  wl-copy "$P_PATH"
  notify-send "SWAL Files" "Ruta copiada (no es archivo de texto): $P_NAME" 2>/dev/null
fi
