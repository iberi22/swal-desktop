#!/usr/bin/env bash
# ⚡ SWAL Smart Fullscreen / Maximize Toggle
# Supports native Hyprland windows, SWAL Files and SWAL QuickLook Editor

ACTIVE_EWW=$(eww active-windows 2>/dev/null)

if echo "$ACTIVE_EWW" | grep -qE "swal_files|swal_files_maximized"; then
    swal-files toggle-maximize
elif echo "$ACTIVE_EWW" | grep -q "swal_editor"; then
    eww close swal_editor
else
    hyprctl dispatch fullscreen 0
fi
