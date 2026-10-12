#!/usr/bin/env bash
# ⚡ Anti-regression gate: no hardcoded personal paths in Rust sources.
# Phase 2 portability — the repo must build on ANY machine, not just belal's.
#
# Fails if any absolute personal home path is found under crates/*/src/ (production
# code) or crates/*/tests/ (test code). Use dirs::home_dir(), env!("CARGO_MANIFEST_DIR"),
# std::env::temp_dir(), or the paths helpers instead.
set -euo pipefail

# Allowlist (narrow): /home/user/... is the conventional SYNTHETIC fixture
# placeholder (like example.com) used in #[cfg(test)] sample data — never a
# real login. Any other /home/<name>/... is a personal path and must fail.
HITS=$(grep -rn "/home/" crates/*/src/ crates/*/tests/ | grep -v "/home/user/" || true)
if [ -n "$HITS" ]; then
  echo "❌ Paths hardcodeados detectados:"
  echo "$HITS"
  exit 1
fi
echo "✓ Sin paths hardcodeados"