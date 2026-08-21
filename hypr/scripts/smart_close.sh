#!/usr/bin/env bash
# ⚡ SWAL Smart Window Close (SUPER + Q)
# Handles closing SWAL Files / Editor overlays or dispatched Hyprland active client

ACTIVE_EWW=$(eww active-windows 2>/dev/null)

if echo "$ACTIVE_EWW" | grep -q "swal_editor"; then
    eww close swal_editor
elif echo "$ACTIVE_EWW" | grep -q "swal_files_maximized"; then
    eww close swal_files_maximized
elif echo "$ACTIVE_EWW" | grep -q "swal_files"; then
    eww close swal_files
else
    hyprctl dispatch killactive
fi
