#!/usr/bin/env bash
set -euo pipefail

titles=(
  "[Ola 3.01] feat-swal-16 — Windows / Files-Community Fluent Dark & Fluent Mica Theme Tokens"
  "[Ola 3.02] feat-swal-17 — Eww SCSS Fluent 2 Styling & Acrylic Mica Effects for SWAL Files"
  "[Ola 3.03] feat-swal-18 — Windows Files Dual-Pane Layout Controller in Rust (swal-files)"
  "[Ola 3.04] feat-swal-19 — Storage Drive & Disk Space Usage Visualizer Engine in Rust"
  "[Ola 3.05] feat-swal-20 — Windows Files Tab Tooltips & Pin Drag/Reorder Metadata"
  "[Ola 3.06] feat-swal-21 — Hermes Agent Protocol & Cognition State Machine in swal-ambient-orb"
  "[Ola 3.07] feat-swal-22 — GLSL Shaders for Hermes Thinking Particle Vortex & A2UI Waves"
  "[Ola 3.08] feat-swal-23 — Async Unix Domain Socket IPC Daemon for Hermes Ambient Orb"
  "[Ola 3.09] feat-swal-24 — Hermes Direct A2UI Component Streamer in swal-a2ui-engine"
  "[Ola 3.10] feat-swal-25 — Radial Agent Quick Action Menu for Hermes Orb in Eww"
  "[Ola 3.11] feat-swal-26 — E2E Integration Test Suite for Hermes Orb & A2UI Streamer"
  "[Ola 3.12] feat-swal-27 — Unit Test Suite for Files Fluent Theme Tokens & Schema Validator"
  "[Ola 3.13] feat-swal-28 — Integration Test Suite for Dual-Pane and Storage Engine"
  "[Ola 3.14] feat-swal-29 — CLI Helper Script for Hermes Agent Voice Orb & Theme Switching"
  "[Ola 3.15] feat-swal-30 — Architecture Documentation for Fluent Theme & Hermes Ambient Orb"
)

created_issues=()

for i in $(seq 1 15); do
  idx=$(printf "%02d" "$i")
  body_file=".hermes/ola3/body-${idx}.md"
  title="${titles[$((i-1))]}"
  
  echo "Creating issue ${idx}: ${title}..."
  url=$(gh issue create --title "$title" --body-file "$body_file" --label "ola3,wave-3")
  num=$(echo "$url" | awk -F/ '{print $NF}')
  created_issues+=("$num")
  echo "  -> Issue #${num} created: $url"
done

echo ""
echo "Created Issue Numbers: ${created_issues[*]}"
echo "${created_issues[*]}" > .hermes/ola3/created_issue_ids.txt
