#!/bin/bash

# Get volume using pactl
get_volume() {
    pactl get-sink-volume @DEFAULT_SINK@ | grep -Po '[0-9]+(?=%)' | head -n 1
}

# Get icon based on volume level
get_vol_icon() {
    vol=$(get_volume)
    if [ "$vol" -eq 0 ]; then
        echo "󰝟"
    elif [ "$vol" -lt 30 ]; then
        echo ""
    elif [ "$vol" -lt 70 ]; then
        echo ""
    else
        echo ""
    fi
}

# Get brightness using brightnessctl
get_brightness() {
    brightnessctl i | grep -Po '[0-9]+(?=%)'
}

case $1 in
    vol) get_volume ;;
    vol_icon) get_vol_icon ;;
    bright) get_brightness ;;
esac
