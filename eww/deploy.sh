#!/usr/bin/env bash
set -euo pipefail

EWW_CONF_DIR="${HOME}/.config/eww"

if [ "${1:-}" = "--dry-run" ]; then
  echo "Deploying EWW UI configuration..."
  echo "Target directory: ${EWW_CONF_DIR}"
  echo "Files to deploy: eww.yuck, eww.scss, files-fluent.scss, hermes_orb.yuck, scripts/"
  exit 0
fi

mkdir -p "${EWW_CONF_DIR}"
cp -r eww/* "${EWW_CONF_DIR}/"
if command -v eww >/dev/null 2>&1; then
  eww reload
fi
