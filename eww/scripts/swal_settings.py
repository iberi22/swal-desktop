#!/usr/bin/env python3
"""swal_settings.py — Backend state and control provider for SWAL Settings Panel."""
import json
import os
import subprocess
import sys
import urllib.request


def get_status():
    # Check Xavier Core
    xavier_online = False
    try:
        req = urllib.request.Request("http://127.0.0.1:8006/health", headers={"User-Agent": "swal-settings"})
        with urllib.request.urlopen(req, timeout=0.8) as res:
            if res.status == 200:
                xavier_online = True
    except Exception:
        xavier_online = False

    # Check active theme
    cur_theme = "hive-dark"
    theme_file = os.path.expanduser("~/.config/swal/current_theme.json")
    if os.path.exists(theme_file):
        try:
            with open(theme_file) as f:
                cur_theme = json.load(f).get("id", "hive-dark")
        except Exception:
            pass

    return {
        "xavier_online": xavier_online,
        "active_theme": cur_theme,
    }


def main():
    if len(sys.argv) > 1:
        cmd = sys.argv[1]
        if cmd == "status":
            print(json.dumps(get_status()))
        elif cmd == "switch_theme":
            t = sys.argv[2]
            subprocess.run(["swal-theme", "switch", t])
        elif cmd == "restart_xavier":
            subprocess.run(["systemctl", "--user", "restart", "xavier-core"])
        elif cmd == "doctor":
            subprocess.run(["ghostty", "-e", "bash -c 'swal-doctor; read -p \"Presiona Enter para cerrar...\"'"])
        elif cmd == "doctor_fix":
            subprocess.run(["ghostty", "-e", "bash -c 'swal-doctor --fix; read -p \"Presiona Enter para cerrar...\"'"])
        elif cmd == "rebuild_nix":
            subprocess.run(["ghostty", "-e", "bash -c 'sudo nixos-rebuild switch --flake /etc/nixos#swal; read -p \"Presiona Enter para cerrar...\"'"])
        elif cmd == "set_profile":
            prof = sys.argv[2]
            subprocess.run(["python3", os.path.expanduser("~/.config/eww/scripts/ram_panel.py"), "profile", prof])
    else:
        print(json.dumps(get_status()))


if __name__ == "__main__":
    main()
