#!/usr/bin/env bash
# Conmuta swal_files <-> swal_files_maximized.
#
# Decision: flag `is_maximized` de la sesion (lo mantiene este script con set-maximize,
# nunca toggle a ciegas). NO usar `eww active-windows`: tarda 1-2s en registrar ventanas
# recien abiertas y devolvia vacio (invertia la conmutacion).
#
# Sin reintentos ni esperas activas: con el daemon lento, reintentar abria ventanas
# DUPLICADAS. Un clic = un close + un open, todo con timeout.
#
# Auto-curacion: si EWW se cuelga, el cliente queda vivo y su superficie se queda encima
# del panel (parece que "no hizo nada"). Al final se reclaman los clientes colgados de
# ESTA ventana (>5s) y la superficie desaparece sola. El sentinel hace lo mismo a nivel
# global, pero aqui es inmediato.
FILES_BIN="/home/belal/proyectosSWAL/periferia/swal-desktop/target/debug/swal-files"
T=5

# Mata clientes eww colgados que operan sobre ventanas swal_files* (nunca otras).
reap_hung() {
  local p a age
  for p in $(pgrep -x ".eww-wrapped" 2>/dev/null) $(pgrep -x eww 2>/dev/null); do
    a="$(ps -o args= -p "$p" 2>/dev/null)" || continue
    case "$a" in *"swal_files"*) ;; *) continue ;; esac
    age="$(ps -o etimes= -p "$p" 2>/dev/null | tr -d ' ')"
    [ -n "$age" ] && [ "$age" -gt 5 ] && kill -9 "$p" 2>/dev/null
  done
  return 0
}

MAX=$("$FILES_BIN" view-json 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('is_maximized', False))" 2>/dev/null)

# Orden: cerrar la ventana actual y abrir la destino (una sola superficie cuando el cierre
# funciona). Si el cierre se cuelga, reap_hung mata el cliente y la superficie se va sola.
if [ "$MAX" = "True" ]; then
  timeout $T eww close swal_files_maximized 2>/dev/null
  timeout $T eww open swal_files 2>/dev/null
  "$FILES_BIN" set-maximize off 2>/dev/null
else
  timeout $T eww close swal_files 2>/dev/null
  timeout $T eww open swal_files_maximized 2>/dev/null
  "$FILES_BIN" set-maximize on 2>/dev/null
fi

reap_hung
