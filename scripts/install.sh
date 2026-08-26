#!/bin/bash
# ⚡ SWAL Desktop & Node Kit - Universal Installer
# SouthWest AI Labs ⚡
# Sets up a full autonomous SWAL Node with Xavier Core, Edge-Mesh, and Agentic UI
#
# Phase 2 portability:
#   - Never assumes a personal path: INSTALL_DIR is git-cloned if missing.
#   - Never touches /etc/nixos or runs `sudo nixos-rebuild` unless --nixos is passed
#     (opt-in, not opt-out). Without it, manual instructions are printed.
#   - --dry-run prints every action without executing anything.
#
# Usage:
#   ./scripts/install.sh [--dry-run] [--nixos]

set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/iberi22/swal-desktop}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/proyectosSWAL/periferia/swal-desktop}"

DRY_RUN=0
NIXOS_MODE=0

usage() {
  cat <<EOF
  ⚡ SWAL Desktop Installer

  Uso: $0 [opciones]

  Opciones:
    --dry-run   Imprime las acciones sin ejecutarlas.
    --nixos     OPT-IN: enlaza /etc/nixos y ejecuta sudo nixos-rebuild.
                (Sin este flag NO se toca nada del sistema NixOS.)
    -h, --help  Muestra esta ayuda.

  Variables de entorno:
    REPO_URL     URL del repositorio a clonar si INSTALL_DIR no existe.
    INSTALL_DIR  Directorio de instalación (por defecto: \$HOME/proyectosSWAL/periferia/swal-desktop).
EOF
}

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --nixos) NIXOS_MODE=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "❌ Opción desconocida: $arg"; usage; exit 1 ;;
  esac
done

# run <cmd...>: executes the command, or prints it under --dry-run.
run() {
  if [ "$DRY_RUN" = "1" ]; then
    echo "  ⏭ [dry-run] $*"
  else
    "$@"
  fi
}

# sudo_run <cmd...>: same as run(), but through sudo.
sudo_run() {
  if [ "$DRY_RUN" = "1" ]; then
    echo "  ⏭ [dry-run] sudo $*"
  else
    sudo "$@"
  fi
}

echo ""
echo "  ⚡ SWAL Desktop & Autonomous Node Kit Installer"
echo "  ────────────────────────────────────────────────"
echo ""

# ─── 0. Clone repository if missing (idempotent, portable) ───────────────
if [ ! -d "$INSTALL_DIR" ]; then
  echo "📦 Clonando swal-desktop desde $REPO_URL ..."
  run git clone "$REPO_URL" "$INSTALL_DIR"
else
  echo "✓ Repositorio ya presente en $INSTALL_DIR"
fi

# cd into the install dir only if it exists (under --dry-run with a missing
# dir there is nothing to enter yet — the remaining actions still get printed).
if [ -d "$INSTALL_DIR" ]; then
  cd "$INSTALL_DIR"
fi

# ─── 1. Ensure Local Config & Themes Directories ─────────────────────────
echo "📦 Configurando directorios de temas y esquemas SWAL..."
run mkdir -p "$HOME/.config/swal/themes" "$HOME/.config/swal/schemas" "$HOME/.config/swal/widgets" "$HOME/.local/bin"
run mkdir -p "$HOME/.agents/skills/swal-theme-creator" "$HOME/.hermes/skills/swal-theme-creator"

# ─── 2. Copy Theme Engine & Assets ───────────────────────────────────────
if [ -d "$INSTALL_DIR/themes" ]; then
  run cp -r "$INSTALL_DIR/themes/"*.json "$HOME/.config/swal/themes/" 2>/dev/null || true
fi
if [ -d "$INSTALL_DIR/schemas" ]; then
  run cp -r "$INSTALL_DIR/schemas/"*.json "$HOME/.config/swal/schemas/" 2>/dev/null || true
fi
if [ -f "$INSTALL_DIR/scripts/swal-theme" ]; then
  run cp "$INSTALL_DIR/scripts/swal-theme" "$HOME/.local/bin/swal-theme"
  run chmod +x "$HOME/.local/bin/swal-theme"
fi

# ─── 3. Activate Default Theme (Hive Dark) ───────────────────────────────
if command -v swal-theme &>/dev/null; then
  echo "🎨 Aplicando tema por defecto: SWAL Hive Dark (@swal/ui)..."
  run swal-theme switch hive-dark 2>/dev/null || true
fi

# ─── 4. NixOS integration (OPT-IN: solo con --nixos) ─────────────────────
if [ "$NIXOS_MODE" = "1" ]; then
  if command -v nixos-rebuild &>/dev/null; then
    echo "🔍 Detectando hardware real del sistema..."
    if [ "$DRY_RUN" = "1" ]; then
      echo "  ⏭ [dry-run] sudo nixos-generate-config --show-hardware-config > $INSTALL_DIR/nixos/hardware.nix"
    else
      sudo nixos-generate-config --show-hardware-config > "$INSTALL_DIR/nixos/hardware.nix" 2>/dev/null || true
    fi

    echo "🔗 Configurando enlaces simbólicos en /etc/nixos..."
    sudo_run mkdir -p /etc/nixos
    sudo_run ln -sf "$INSTALL_DIR/flake.nix" /etc/nixos/flake.nix
    sudo_run ln -sf "$INSTALL_DIR/nixos" /etc/nixos/nixos
    sudo_run ln -sf "$INSTALL_DIR/hypr" /etc/nixos/hypr
    sudo_run ln -sf "$INSTALL_DIR/themes" /etc/nixos/themes
    sudo_run ln -sf "$INSTALL_DIR/eww" /etc/nixos/eww

    echo "🚀 Reconstruyendo sistema NixOS con soporte de Nodo SWAL..."
    if [ "$DRY_RUN" = "1" ]; then
      echo "  ⏭ [dry-run] sudo nixos-rebuild switch --flake \"$INSTALL_DIR#swal\""
    else
      sudo nixos-rebuild switch --flake "$INSTALL_DIR#swal" 2>/dev/null || echo "⚠️ Rebuild opcional pospuesto."
    fi
  else
    echo "⚠️ nixos-rebuild no encontrado — omitiendo integración NixOS."
  fi
else
  echo ""
  echo "ℹ️ Integración NixOS omitida (usa --nixos para activarla). Nada de /etc/nixos fue tocado."
  echo "   Para integrar el nodo SWAL manualmente, ejecuta:"
  echo "     sudo mkdir -p /etc/nixos"
  echo "     sudo ln -sf \"$INSTALL_DIR/flake.nix\" /etc/nixos/flake.nix"
  echo "     sudo ln -sf \"$INSTALL_DIR/nixos\" /etc/nixos/nixos"
  echo "     sudo ln -sf \"$INSTALL_DIR/hypr\" /etc/nixos/hypr"
  echo "     sudo ln -sf \"$INSTALL_DIR/themes\" /etc/nixos/themes"
  echo "     sudo ln -sf \"$INSTALL_DIR/eww\" /etc/nixos/eww"
  echo "     sudo nixos-rebuild switch --flake \"$INSTALL_DIR#swal\""
  echo ""
fi

echo ""
echo "  ✅ ¡Nodo SWAL & Desktop configurado con éxito!"
echo "  • Tema activo: Hive Dark (@swal/ui) [Alterna con: swal-theme switch cyber-neon]"
echo "  • Memoria Cognitiva: Xavier Core (:8006 / :8100)"
echo "  • Red de Malla P2P: Edge-Mesh"
echo "  • Rieles Agénticos: ~/.config/swal/schemas & skills instaladas"
echo ""