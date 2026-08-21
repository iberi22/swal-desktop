#!/usr/bin/env bash
ACTION="${1:-toggle}"

close_all() {
    for win in dashboard ram_panel agent_admin keybinds_panel agent_chat swal_settings; do
        if eww active-windows | grep -q "$win"; then
            eww close "$win" 2>/dev/null || true
        fi
    done
}

if [ "$ACTION" = "close" ]; then
    close_all
elif [ "$ACTION" = "open" ]; then
    eww open dashboard 2>/dev/null || true
else
    ACTIVE=$(eww active-windows)
    if [ -n "$ACTIVE" ]; then
        close_all
    else
        eww open dashboard 2>/dev/null || true
    fi
fi
