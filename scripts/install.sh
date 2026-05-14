#!/bin/bash
# ⚡ SWAL Desktop - One-Line Installer
# SouthWest AI Labs ⚡
# Usage: curl -sSL https://raw.githubusercontent.com/iberi22/swal-desktop/main/scripts/install.sh | bash

set -euo pipefail

REPO_URL="https://github.com/iberi22/swal-desktop"
INSTALL_DIR="$HOME/swal-desktop"

echo ""
echo "  ⚡ SWAL Desktop Installer"
echo "  ─────────────────────────"
echo ""

# ─── Pre-flight checks ───────────────────────────────────────────────────
if ! command -v nixos-rebuild &> /dev/null; then
    echo "❌ Error: 'nixos-rebuild' not found. Are you running this on NixOS?"
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo "❌ Error: 'git' not found. Install it first: nix-env -iA nixos.git"
    exit 1
fi

# ─── 1. Clone or update repo ─────────────────────────────────────────────
if [ -d "$INSTALL_DIR/.git" ]; then
    echo "📦 Repositorio ya existe. Actualizando..."
    cd "$INSTALL_DIR"
    git pull --ff-only || echo "⚠️  No se pudo actualizar, continuando con versión local."
else
    echo "📥 Clonando repositorio..."
    git clone "$REPO_URL" "$INSTALL_DIR"
fi

cd "$INSTALL_DIR"

# ─── 2. Backup de /etc/nixos ─────────────────────────────────────────────
if [ -d "/etc/nixos" ] && [ ! -L "/etc/nixos/flake.nix" ]; then
    echo "📦 Respaldando /etc/nixos actual..."
    sudo cp -r /etc/nixos "/etc/nixos.bak.$(date +%Y%m%d%H%M%S)"
fi

# ─── 3. Generate real hardware config (CRITICAL) ─────────────────────────
# The repo ships with a placeholder hardware.nix.
# We MUST generate the real one for this specific machine.
echo "🔍 Detectando hardware real del sistema..."
sudo nixos-generate-config --show-hardware-config > "$INSTALL_DIR/nixos/hardware.nix"
echo "✅ Hardware detectado y guardado en nixos/hardware.nix"

# ─── 4. Symlinks ─────────────────────────────────────────────────────────
echo "🔗 Configurando enlaces simbólicos..."
sudo mkdir -p /etc/nixos
sudo ln -sf "$INSTALL_DIR/flake.nix" /etc/nixos/flake.nix
sudo ln -sf "$INSTALL_DIR/nixos" /etc/nixos/nixos
sudo ln -sf "$INSTALL_DIR/hypr" /etc/nixos/hypr
sudo ln -sf "$INSTALL_DIR/themes" /etc/nixos/themes
sudo ln -sf "$INSTALL_DIR/eww" /etc/nixos/eww

# ─── 5. Dry-run first ────────────────────────────────────────────────────
echo "🧪 Ejecutando dry-run para verificar la configuración..."
if sudo nixos-rebuild dry-activate --flake "$INSTALL_DIR#swal" 2>&1; then
    echo "✅ Dry-run exitoso. Aplicando configuración..."
else
    echo "❌ Error en dry-run. Revisa los errores arriba."
    echo "💡 Tip: Puedes editar $INSTALL_DIR/nixos/configuration.nix y volver a intentar."
    exit 1
fi

# ─── 6. Rebuild ──────────────────────────────────────────────────────────
echo "🚀 Aplicando configuración (esto puede tardar ~10 min la primera vez)..."
sudo nixos-rebuild switch --flake "$INSTALL_DIR#swal"

echo ""
echo "  ✅ ¡SWAL Desktop instalado con éxito!"
echo ""
echo "  Usuario: bela"
echo "  Password: swal123"
echo "  Escritorio: Hyprland (auto-start via greetd)"
echo ""
echo "  📌 Post-instalación:"
echo "    • Reinicia el sistema: sudo reboot"
echo "    • Configura agentes AI: bash ~/swal-desktop/scripts/hermes-onboarding.sh"
echo "    • Configura API keys en: /etc/nixos/nixos/ai-agents.nix"
echo ""
