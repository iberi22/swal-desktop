#!/usr/bin/env bash
set -euo pipefail

titles=(
  "[Ola 4.01] feat-swal-31 — Wayland Layer Shell Protocol Surface Manager in Rust (Zero-Eww)"
  "[Ola 4.02] feat-swal-32 — WGPU Graphics Context & Hardware Surface Initializer (Zero-Eww)"
  "[Ola 4.03] feat-swal-33 — Mica Acrylic Blur & Rounded Geometry Quad Shader in WGSL (Zero-Eww)"
  "[Ola 4.04] feat-swal-34 — Hardware-Accelerated Hermes Ambient Orb Render Surface (Zero-Eww)"
  "[Ola 4.05] feat-swal-35 — GPU Typography & Glyph Rasterizer Engine (Zero-Eww)"
  "[Ola 4.06] feat-swal-36 — Wayland Pointer, Keyboard Focus & Drag Input Dispatcher (Zero-Eww)"
  "[Ola 4.07] feat-swal-37 — Direct A2UI AST GPU Node Rasterizer (Zero-Eww)"
  "[Ola 4.08] feat-swal-38 — Native SWAL Files GPU Window Layout Builder (Zero-Eww)"
  "[Ola 4.09] feat-swal-39 — Native Desktop Daemon Supervisor in swal-node-daemon (Zero-Eww Launcher)"
  "[Ola 4.10] feat-swal-40 — E2E Integration Test Suite for Pure Rust Native Desktop Pipeline"
)

created_issues=()

for i in $(seq 1 10); do
  idx=$(printf "%02d" "$i")
  body_file=".hermes/ola4/body-${idx}.md"
  title="${titles[$((i-1))]}"
  
  echo "Creating issue ${idx}: ${title}..."
  url=$(gh issue create --title "$title" --body-file "$body_file" --label "ola4,wave-4")
  num=$(echo "$url" | awk -F/ '{print $NF}')
  created_issues+=("$num")
  echo "  -> Issue #${num} created: $url"
done

echo ""
echo "Created Wave 4 Issue Numbers: ${created_issues[*]}"
echo "${created_issues[*]}" > .hermes/ola4/created_issue_ids.txt
