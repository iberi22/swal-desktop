#!/usr/bin/env bash
# Conmuta swal_files <-> swal_files_maximized.
#
# Decision: flag `is_maximized` de la sesion (lo mantiene este script con set-maximize,
# nunca toggle a ciegas). NO usar `eww active-windows`: en esta version devuelve vacio
# aunque la ventana exista (verificado 2026-09-11), por eso se verifica por geometria
# en `hyprctl layers`.
#
# Sin reintentos ni esperas activas: con el daemon lento, reintentar abria ventanas
# DUPLICADAS. Un clic = un close + un open, todo con timeout.
#
# Auto-curacion (2026-09-11): si al abrir el cliente no puede hablar con el daemon
# ("Failed to connect to daemon" / "Initializing eww daemon" en la salida) o la
# superficie no aparece, el daemon esta trabado: se pide al sentinel `recover-eww`,
# se espera y se reintenta UNA vez. Si aun asi no aparece, se avisa por notificacion
# en vez de dejar una ventana a medias (era el bug reportado: ventana bugueada que
# habia que matar a mano).
FILES_BIN="/home/belal/proyectosSWAL/periferia/swal-desktop/target/debug/swal-files"
SENTINEL="/home/belal/.local/bin/swal-sentinel"
T=5

# Mata clientes eww colgados que operan sobre ventanas swal_files* (nunca otras).
reap_hung() {
  local p a age
  for p in $(pgrep -x ".eww-wrapped" 2>/dev/null) $(pgrep -x eww 2>/dev/null); do
    a="$(ps -o args= -p "$p" 2>/dev/null)" || continue
    case "$a" in *"swal_files"*) ;; *) continue ;; esac
    case "$a" in *daemon*) continue ;; esac
    age="$(ps -o etimes= -p "$p" 2>/dev/null | tr -d ' ')"
    [ -n "$age" ] && [ "$age" -gt 5 ] && kill -9 "$p" 2>/dev/null
  done
  return 0
}

window_present() { # $1 = "ancho alto" exactos de la ventana esperada
  hyprctl layers 2>/dev/null | command grep -q " $1"
}

open_win() { # $1 = ventana, $2 = "ancho alto"
  local win="$1" geo="$2" out tmpf
  # OJO: nada de `out="$(eww open ...)"`: si el cliente arranca su propio daemon,
  # ese hijo hereda el pipe de la sustitucion y la deja bloqueada PARA SIEMPRE
  # (bug observado 2026-09-11: el script colgaba 420s). A archivo y listo.
  tmpf="$(mktemp -t swal_open_XXXXXX)"
  timeout $T eww open "$win" >"$tmpf" 2>&1 </dev/null
  out="$(cat "$tmpf" 2>/dev/null)"
  rm -f "$tmpf"
  if printf '%s' "$out" | command grep -qiE "initializing eww daemon|failed to connect"; then
    # El cliente no alcanzo el daemon: esta trabado. Recuperar y reintentar una vez.
    timeout 40 "$SENTINEL" recover-eww >/dev/null 2>&1 </dev/null
    sleep 4
    timeout $T eww open "$win" >/dev/null 2>&1 </dev/null
  fi
  local i=0
  while [ "$i" -lt 8 ]; do
    window_present "$geo" && return 0
    sleep 0.4
    i=$((i + 1))
  done
  notify-send "SWAL Files" "No pude abrir el panel (daemon EWW trabado). Mira: ~/.cache/swal-sentinel/sentinel.log" -i dialog-warning 2>/dev/null || true
  return 1
}

MAX=$("$FILES_BIN" view-json 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('is_maximized', False))" 2>/dev/null)

OK=1
if [ "$MAX" = "True" ]; then
  timeout $T eww close swal_files_maximized 2>/dev/null
  open_win swal_files "1080 660" || OK=0
  "$FILES_BIN" set-maximize off 2>/dev/null
else
  timeout $T eww close swal_files 2>/dev/null
  open_win swal_files_maximized "1881 1004" || OK=0
  "$FILES_BIN" set-maximize on 2>/dev/null
fi

reap_hung
[ "$OK" = "1" ] && exit 0
exit 1
