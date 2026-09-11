#!/usr/bin/env bash
# Abre el archivo del preview con la app adecuada:
#   texto/código -> Sublime Text (edición real)
#   imagen       -> visor por defecto (xdg-open)
#   carpeta      -> terminal ghostty en esa carpeta
FILES_BIN="/home/belal/proyectosSWAL/periferia/swal-desktop/target/debug/swal-files"

read -r P_PATH P_TEXT P_IMG <<< "$("$FILES_BIN" view-json 2>/dev/null | python3 -c "
import sys, json
d = json.load(sys.stdin).get('preview', {})
p = d.get('path','')
print(p.replace(' ', '\\ '), '1' if d.get('is_text') else '0', '1' if d.get('is_image') else '0')
" 2>/dev/null)"

P_PATH="${P_PATH//\\ / }"

if [ -z "$P_PATH" ]; then
  exit 0
fi

if [ -d "$P_PATH" ]; then
  ghostty --working-directory="$P_PATH" &
elif [ "$P_TEXT" = "1" ]; then
  subl "$P_PATH" &
elif [ "$P_IMG" = "1" ]; then
  xdg-open "$P_PATH" &
else
  # Binario u otro: abrir con la app por defecto
  xdg-open "$P_PATH" &
fi
