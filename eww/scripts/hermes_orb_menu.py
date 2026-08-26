#!/usr/bin/env python3
"""hermes_orb_menu.py — Action dispatcher and IPC socket bridge for Hermes Ambient Orb in Eww."""

import json
import os
import shlex
import socket
import subprocess
import sys
import time

SOCKET_PATHS = [
    os.path.join(os.environ.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}"), "swal", "hermes_orb.sock"),
    os.path.join(os.environ.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}"), "hermes_orb.sock"),
    "/tmp/hermes_orb.sock",
]

DEFAULT_ACTIONS = {
    "@summarize": "hermes --prompt 'Resumir selección o contexto actual'",
    "@refactor": "hermes --prompt 'Refactorizar código seleccionado'",
    "@execute": "hermes --prompt 'Ejecutar tarea agéntica en espacio SWAL'",
    "@chat": "eww open --toggle agent_chat",
}


def send_unix_socket_payload(payload: dict) -> bool:
    """Sends a JSON payload over the Hermes Orb Unix domain socket if available."""
    data = json.dumps(payload).encode("utf-8")
    for path in SOCKET_PATHS:
        if os.path.exists(path):
            try:
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as sock:
                    sock.settimeout(1.5)
                    sock.connect(path)
                    sock.sendall(data)
                    return True
            except Exception as e:
                sys.stderr.write(f"IPC socket connection warning ({path}): {e}\n")
    return False


def dispatch_action(action: str, extra_args: str = ""):
    """Dispatches quick action to Hermes CLI / Eww overlays and notifies IPC socket.

    Security note: never build shell strings from user input. Arguments are
    passed as an argv list so no command injection is possible.
    """
    payload = {
        "event": "action_triggered",
        "action": action,
        "extra_args": extra_args,
        "timestamp": time.time(),
    }

    socket_sent = send_unix_socket_payload(payload)

    prompts = {
        "@summarize": "Resumir el contexto activo o selección de texto",
        "@refactor": "Refactorizar y optimizar el código actual",
        "@execute": "Ejecutar acción y herramientas agénticas",
    }

    if action == "@chat":
        subprocess.run(["eww", "open", "--toggle", "agent_chat"])
    else:
        prompt = prompts.get(action, action.lstrip("@"))
        argv = ["ghostty", "-e", "hermes", "--prompt", prompt]
        if extra_args:
            argv.extend(shlex.split(extra_args))
        subprocess.Popen(argv)

    status_str = f"Action {action} dispatched (IPC: {'Connected' if socket_sent else 'Fallback CLI'})"
    print(json.dumps({"ok": True, "action": action, "ipc_sent": socket_sent, "message": status_str}))


def main():
    if len(sys.argv) < 2:
        print(json.dumps({
            "ok": True,
            "actions": list(DEFAULT_ACTIONS.keys()),
            "usage": "hermes_orb_menu.py [dispatch|send-ipc|status] <action> [args]"
        }))
        return

    subcmd = sys.argv[1]

    if subcmd == "dispatch":
        action = sys.argv[2] if len(sys.argv) > 2 else "@chat"
        extra = " ".join(sys.argv[3:]) if len(sys.argv) > 3 else ""
        dispatch_action(action, extra)

    elif subcmd == "send-ipc":
        payload_raw = sys.argv[2] if len(sys.argv) > 2 else "{}"
        try:
            payload = json.loads(payload_raw)
        except Exception:
            payload = {"raw": payload_raw, "timestamp": time.time()}
        success = send_unix_socket_payload(payload)
        print(json.dumps({"ok": success, "payload": payload}))

    elif subcmd == "status":
        socket_active = any(os.path.exists(p) for p in SOCKET_PATHS)
        print(json.dumps({
            "ok": True,
            "socket_active": socket_active,
            "socket_paths": SOCKET_PATHS,
            "available_actions": list(DEFAULT_ACTIONS.keys())
        }))
    else:
        dispatch_action(subcmd)


if __name__ == "__main__":
    main()
