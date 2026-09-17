#!/usr/bin/env bash
# Buscador del panel SWAL Files: filtra por nombre en el directorio actual.
# EWW llama este script con el texto del input como $1.
FILES_BIN="${SWAL_ROOT:-$HOME/proyectosSWAL}/periferia/swal-desktop/target/debug/swal-files"
QUERY="$1"

if [ -z "$QUERY" ]; then
  # Vacío -> limpiar búsqueda
  "$FILES_BIN" omnibar "? " 2>/dev/null
else
  "$FILES_BIN" omnibar "?$QUERY" 2>/dev/null
fi
