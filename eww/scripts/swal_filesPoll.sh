#!/bin/bash
# EWW defpoll wrapper for swal-files view-json
# v3 (2026-09-11):
#   - timeout 6s (dirs gigantes como /tmp con 43k items tardan ~1.2s + serializacion)
#   - poda de groups > 300 entries (evita 12MB por IPC cada 2s)
#   - cache del ultimo resultado bueno: un timeout NUNCA mas muestra un "Home" falso
set -u

TIMEOUT=6
CMD="${SWAL_ROOT:-$HOME/proyectosSWAL}/periferia/swal-desktop/target/debug/swal-files"
LOG="/tmp/swal-files-poll.log"
CACHE="/tmp/swal-files-last-good.json"
FALLBACK='{"current_path":"HOME_DIR_PLACEHOLDER","parent_path":"/","total_items":0,"active_tab_id":1,"tabs":[{"id":1,"title":"Home","path":"HOME_DIR_PLACEHOLDER","active":true}],"breadcrumbs":[{"name":"Home","path":"HOME_DIR_PLACEHOLDER"}],"view_mode":"details","show_hidden":false,"dual_pane":false,"sort_by":"name","sort_order":"asc","group_by":"none","filter_type":"all","preview_mode":"sidebar","is_maximized":false,"is_current_pinned":false,"theme_id":"hive-dark","favorites":[],"workspaces":[],"entries":[],"groups":[],"disks":[],"git_status":{"is_git_repo":false,"branch":"","ahead":0,"behind":0,"staged_count":0,"modified_count":0,"untracked_count":0,"conflicted_count":0,"is_clean":true,"summary":"","badge":""},"col_chars":{"name":24,"date":16,"type":12,"size":8},"preview_wrap":true,"row_density":"comfortable","preview":{"path":"HOME_DIR_PLACEHOLDER","file_name":"Home","file_type":"Carpeta","size_formatted":"0 items","date_modified":"","is_image":false,"is_text":false,"is_dir":true,"is_git_repo":false,"image_path":"","line_count":0,"content":"","gutter_lines":"","git_status_summary":""}}'
FALLBACK="${FALLBACK//HOME_DIR_PLACEHOLDER/${HOME:-/root}}"

TMP=$(mktemp) || exit 0
trap 'rm -f "$TMP"' EXIT

if timeout "$TIMEOUT" "$CMD" view-json > "$TMP" 2>/dev/null && [ -s "$TMP" ]; then
    if OUT=$(python3 - "$TMP" <<'PY' 2>/dev/null
import json, sys
# Poda SOLO para payloads gigantes (dirs de decenas de miles de archivos).
# Carpetas normales (p.ej. 563 capturas) se sirven completas: el panel debe poder
# listar todo lo que el usuario ve en su carpeta.
MAX_BYTES = 1_500_000
MAX_ENTRIES = 1200
with open(sys.argv[1]) as fh:
    raw = fh.read()
d = json.loads(raw)
if len(raw) > MAX_BYTES:
    for g in d.get('groups', []):
        e = g.get('entries', [])
        if len(e) > MAX_ENTRIES:
            g['entries'] = e[:MAX_ENTRIES]
            g['truncated'] = True
            g['count'] = g.get('count', len(e))
print(json.dumps(d, separators=(',', ':')))
PY
    ) && [ -n "$OUT" ]; then
        printf '%s' "$OUT" > "$CACHE"
        printf '%s' "$OUT"
        exit 0
    fi
    echo "[$(date -Is)] swal-files-poll: scan ok pero json invalido -> cache" >> "$LOG"
else
    echo "[$(date -Is)] swal-files-poll: timeout/scan fail -> cache" >> "$LOG"
fi

# Fallback: ultimo resultado bueno (nunca un Home falso); si no hay cache, placeholder
if [ -s "$CACHE" ]; then
    cat "$CACHE"
else
    printf '%s' "$FALLBACK"
fi
