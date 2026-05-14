#!/bin/bash
# ⚡ SWAL Desktop - VM Debugger
# Corre la configuración en un entorno QEMU seguro

echo "🛠️ Construyendo VM de prueba para SWAL Desktop..."
nixos-rebuild build-vm --flake .#swal

echo "🚀 Iniciando VM..."
./result/bin/run-swal-desktop-vm
