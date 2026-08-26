#!/usr/bin/env bash
# ⚡ SWAL Smart Fullscreen / Maximize Toggle
# Zero-Eww: native swal-files handles its own maximize; everything else is Hyprland fullscreen.

if pgrep -x swal-files >/dev/null 2>&1; then
    swal-files toggle-maximize 2>/dev/null || hyprctl dispatch fullscreen 0
else
    hyprctl dispatch fullscreen 0
fi
