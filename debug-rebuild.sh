#!/bin/bash
# Script to run on NixOS via SSH

# Save output to file
exec > /tmp/swal-debug.log 2>&1

echo "=== SWAL Debug Script ==="
echo "Date: $(date)"
echo ""

echo "1. Checking system..."
uname -a
echo ""

echo "2. Checking NixOS version..."
cat /etc/os-release
echo ""

echo "3. Checking /tmp/swal-desktop..."
ls -la /tmp/swal-desktop/
echo ""

echo "4. Checking nixos-rebuild..."
which nixos-rebuild
echo ""

echo "5. Running rebuild with trace..."
sudo nixos-rebuild switch --flake /tmp/swal-desktop#swal --show-trace 2>&1 | tee /tmp/rebuild-output.log

echo ""
echo "=== Done ==="
