#!/usr/bin/env bash
# ⚡ Anti-regression gate: no hardcoded personal paths in Rust sources.
# Phase 2 portability — the repo must build on ANY machine, not just belal's.
#
# Fails if any `/home/belal` literal is found under crates/*/src/ (production
# code) or crates/*/tests/ (test code). Use dirs::home_dir(), env!("CARGO_MANIFEST_DIR"),
# std::env::temp_dir(), or the paths helpers instead.
set -euo pipefail

HITS=$(grep -rn "/home/belal" crates/*/src/ crates/*/tests/ || true)
if [ -n "$HITS" ]; then
  echo "❌ Paths hardcodeados detectados:"
  echo "$HITS"
  exit 1
fi
echo "✓ Sin paths hardcodeados"