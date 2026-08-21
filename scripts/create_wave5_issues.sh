#!/usr/bin/env bash
set -euo pipefail

titles=(
  "[Ola 5.01] feat-swal-41 — Canonical System Settings JSON Schema & Store Engine in Rust"
  "[Ola 5.02] feat-swal-42 — A2UI Rich Settings Component Nodes (Toggle, Slider, Select, ColorSwatch)"
  "[Ola 5.03] feat-swal-43 — Generative AUI Agent Action Card & Dynamic Response Streamer"
  "[Ola 5.04] feat-swal-44 — macOS-Inspired Centralized Settings Window Layout Builder"
  "[Ola 5.05] feat-swal-45 — GPU Rasterizer Extension for Interactive Settings Controls"
  "[Ola 5.06] feat-swal-46 — Interactive Settings Hit-Testing & Value Mutation Controller"
  "[Ola 5.07] feat-swal-47 — Agent Real-Time Configuration Mutation IPC Protocol in Rust"
  "[Ola 5.08] feat-swal-48 — Settings CLI Companion Tool in Rust (swal-node-daemon)"
  "[Ola 5.09] feat-swal-49 — SWAL Doctor Embedded Self-Healing & Diagnostic Engine in Rust"
  "[Ola 5.10] feat-swal-50 — E2E Integration Test Suite for Centralized Settings & Generative AUI"
)

created_issues=()

for i in $(seq 1 10); do
  idx=$(printf "%02d" "$i")
  body_file=".hermes/ola5/body-${idx}.md"
  title="${titles[$((i-1))]}"
  
  echo "Creating issue ${idx}: ${title}..."
  url=$(gh issue create --title "$title" --body-file "$body_file" --label "ola5,wave-5")
  num=$(echo "$url" | awk -F/ '{print $NF}')
  created_issues+=("$num")
  echo "  -> Issue #${num} created: $url"
done

echo ""
echo "Created Wave 5 Issue Numbers: ${created_issues[*]}"
echo "${created_issues[*]}" > .hermes/ola5/created_issue_ids.txt
