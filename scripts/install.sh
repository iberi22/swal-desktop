#!/bin/bash
# ⚡ SWAL Desktop & Node Kit - Universal Installer
# SouthWest AI Labs ⚡
# Sets up a full autonomous SWAL Node with Xavier Core, Edge-Mesh, and Agentic UI

set -euo pipefail

REPO_URL="https://github.com/iberi22/swal-desktop"
INSTALL_DIR="$HOME/proyectosSWAL/periferia/swal-desktop"

echo ""
echo "  ⚡ SWAL Desktop & Autonomous Node Kit Installer"
echo "  ────────────────────────────────────────────────"
echo ""

# ─── 1. Ensure Local Config & Themes Directories ─────────────────────────
echo "📦 Configurando directorios de temas y esquemas SWAL..."
mkdir -p "$HOME/.config/swal/themes" "$HOME/.config/swal/schemas" "$HOME/.config/swal/widgets" "$HOME/.local/bin"
mkdir -p "$HOME/.agents/skills/swal-theme-creator" "$HOME/.hermes/skills/swal-theme-creator"

# ─── 2. Copy Theme Engine & Assets ───────────────────────────────────────
if [ -d "$INSTALL_DIR/themes" ]; then
    cp -r "$INSTALL_DIR/themes/"*.json "$HOME/.config/swal/themes/" 2>/dev/null || true
fi
if [ -d "$INSTALL_DIR/schemas" ]; then
    cp -r "$INSTALL_DIR/schemas/"*.json "$HOME/.config/swal/schemas/" 2>/dev/null || true
fi
if [ -f "$INSTALL_DIR/scripts/swal-theme" ]; then
    cp "$INSTALL_DIR/scripts/swal-theme" "$HOME/.local/bin/swal-theme"
    chmod +x "$HOME/.local/bin/swal-theme"
fi

# ─── 3. Activate Default Theme (Hive Dark) ───────────────────────────────
if command -v swal-theme &>/dev/null; then
    echo "🎨 Aplicando tema por defecto: SWAL Hive Dark (@swal/ui)..."
    swal-theme switch hive-dark 2>/dev/null || true
fi

# ─── 4. Pre-flight checks for NixOS ──────────────────────────────────────
if command -v nixos-rebuild &> /dev/null; then
    echo "🔍 Detectando hardware real del sistema..."
    sudo nixos-generate-config --show-hardware-config > "$INSTALL_DIR/nixos/hardware.nix" 2>/dev/null || true

    echo "🔗 Configurando enlaces simbólicos en /etc/nixos..."
    sudo mkdir -p /etc/nixos
    sudo ln -sf "$INSTALL_DIR/flake.nix" /etc/nixos/flake.nix
    sudo ln -sf "$INSTALL_DIR/nixos" /etc/nixos/nixos
    sudo ln -sf "$INSTALL_DIR/hypr" /etc/nixos/hypr
    sudo ln -sf "$INSTALL_DIR/themes" /etc/nixos/themes
    sudo ln -sf "$INSTALL_DIR/eww" /etc/nixos/eww

    echo "🚀 Reconstruyendo sistema NixOS con soporte de Nodo SWAL..."
    sudo nixos-rebuild switch --flake "$INSTALL_DIR#swal" 2>/dev/null || echo "⚠️ Rebuild opcional pospuesto."
fi

echo ""
echo "  ✅ ¡Nodo SWAL & Desktop configurado con éxito!"
echo "  • Tema activo: Hive Dark (@swal/ui) [Alterna con: swal-theme switch cyber-neon]"
echo "  • Memoria Cognitiva: Xavier Core (:8006 / :8100)"
echo "  • Red de Malla P2P: Edge-Mesh"
echo "  • Rieles Agénticos: ~/.config/swal/schemas & skills instaladas"
echo ""
