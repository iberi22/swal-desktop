#!/usr/bin/env bash
# ⚡ SWAL Desktop Configuration & Environment Menu
# SouthWest AI Labs

ROFI_CMD="rofi -dmenu -i -p ⚡ SWAL Desktop -theme-str window{width:520px;} listview{lines:9;}"

OPTIONS="🎨  1. Cambiar Tema Visual (Hive Dark, Cyber Neon, Nord)
󰕮  2. Iniciar / Cambiar a Hyprland
󰍹  3. Iniciar / Cambiar a Niri (Archcraft Style)
📊  4. Alternar SWAL Dashboard Lateral
🖼️  5. Cambiar Fondo de Pantalla (SWAL Wallpapers)
🩺  6. Ejecutar SWAL Doctor (Linter & Diagnóstico)
⚙️  7. Reconstruir NixOS (sudo nixos-rebuild switch)
🤖  8. Verificar Estado de Xavier Core & Hermes
🚪  9. Cerrar Sesión / Salir"

CHOSEN=$(echo -e "$OPTIONS" | $ROFI_CMD)

case "$CHOSEN" in
    *Tema*)
        swal-theme picker
        ;;
    *Hyprland*)
        notify-send "SWAL Desktop" "Cambiando a Hyprland..."
        hyprctl dispatch exit || pkill -9 hyprland
        ;;
    *Niri*)
        notify-send "SWAL Desktop" "Cambiando a Niri..."
        pkill -9 Hyprland || true
        niri &
        ;;
    *Dashboard*)
        ~/.config/eww/scripts/toggle_dashboard.sh toggle
        ;;
    *Doctor*)
        ghostty -e "swal-doctor; read -p 'Presiona Enter para cerrar...'" &
        ;;
    *Fondo*)
        WALLPAPER=$(find ~/Wallpapers -type f \( -name "*.png" -o -name "*.jpg" \) 2>/dev/null | rofi -dmenu -i -p "Seleccionar Wallpaper:")
        if [ -n "$WALLPAPER" ]; then
            swww img "$WALLPAPER" --transition-type wipe || hyprpaper
            notify-send "SWAL Desktop" "Wallpaper actualizado: $(basename $WALLPAPER)"
        fi
        ;;
    *Reconstruir*)
        ghostty -e "sudo nixos-rebuild switch --flake /etc/nixos#swal; read -p 'Presiona Enter para cerrar...'" &
        ;;
    *Xavier*)
        STATUS=$(curl -s http://localhost:8006/health | jq -r '.status' 2>/dev/null || echo "Offline")
        notify-send "Estado de Xavier Core" "Estado: $STATUS\nEndpoint: http://localhost:8006"
        ;;
    *Cerrar*)
        hyprctl dispatch exit || pkill -9 niri || loginctl terminate-user $USER
        ;;
esac
