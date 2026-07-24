#!/usr/bin/env bash
# ⚡ SWAL Desktop Session Starter with Niri Fallback
# SouthWest AI Labs

SELECTED_WM="${1:-hyprland}"

export XDG_CURRENT_DESKTOP="SWAL-Wayland"
export XDG_SESSION_TYPE="wayland"

case "$SELECTED_WM" in
    niri)
        echo "⚡ Starting Niri (Archcraft Style)..."
        export XDG_CURRENT_DESKTOP="niri"
        exec niri
        ;;
    hyprland|*)
        echo "⚡ Starting Hyprland..."
        export XDG_CURRENT_DESKTOP="Hyprland"
        Hyprland
        EXIT_CODE=$?

        if [ $EXIT_CODE -ne 0 ]; then
            echo "⚠️ [SWAL Session] Hyprland exited with code $EXIT_CODE."
            echo "🔄 Launching Niri as Emergency Recovery Environment..."
            if command -v notify-send >/dev/null 2>&1; then
                notify-send -u critical "SWAL Recovery" "Hyprland falló. Cargando entorno Niri..."
            fi
            export XDG_CURRENT_DESKTOP="niri"
            exec niri
        fi
        ;;
esac
