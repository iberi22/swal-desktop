#!/usr/bin/env bash
# ⚡ SWAL Smart Window Close (SUPER + Q)
# Zero-Eww: closes the native swal-files window if present, else kills active client.

# Native swal-files first (check via its PID file, then hyprland clients)
if [ -f "${XDG_RUNTIME_DIR:-/tmp}/swal-files.pid" ] || pgrep -x swal-files >/dev/null 2>&1; then
    swal-desktop-ctl close-files 2>/dev/null && exit 0
    pkill -x swal-files 2>/dev/null && exit 0
fi

hyprctl dispatch killactive
