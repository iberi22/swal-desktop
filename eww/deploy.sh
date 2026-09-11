#!/usr/bin/env bash
# Copia la UI de este repo al config vivo de EWW y recarga.
# Uso: bash eww/deploy.sh [--dry-run]
set -euo pipefail
SRC="$(cd "$(dirname "$0")" && pwd)"
DST="${EWW_CONFIG_DIR:-$HOME/.config/eww}"
DRY=0; [ "${1:-}" = "--dry-run" ] && DRY=1
for f in eww.yuck eww.scss; do
  if [ "$DRY" = 1 ]; then echo "[dry-run] cp $SRC/$f -> $DST/$f"; else cp "$SRC/$f" "$DST/$f"; fi
done
if [ -d "$SRC/scripts" ]; then
  for f in "$SRC"/scripts/swal_*.sh; do
    [ -e "$f" ] || continue
    if [ "$DRY" = 1 ]; then echo "[dry-run] cp $f -> $DST/scripts/"; else cp "$f" "$DST/scripts/"; fi
  done
fi
if [ "$DRY" = 1 ]; then echo "[dry-run] eww reload"; else eww reload && echo "deployed + reloaded"; fi
