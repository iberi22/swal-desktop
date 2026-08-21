# [Ola 3.14] feat-swal-29 — CLI Helper Script for Hermes Agent Voice Orb & Theme Switching

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `scripts/` contains `swal-theme` and `swal-doctor`.
- No single CLI command allows switching Hermes Ambient Orb states or applying the Files Fluent theme seamlessly from the terminal.

## Desired State (DELTA)
- **Specific Addition**: Create `scripts/swal-hermes-orb` (executable bash/python script):
  - Subcommands:
    - `swal-hermes-orb state <idle|listen|think|stream|error>`: Pushes state to `/tmp/swal_hermes_orb.sock`.
    - `swal-hermes-orb prompt "<prompt>"`: Triggers Hermes cognition flow.
    - `swal-hermes-orb theme fluent-dark`: Applies Fluent Dark theme tokens across Hyprland and Eww.
    - `swal-hermes-orb toggle`: Toggles orb overlay visibility.
- **File Target**: `scripts/swal-hermes-orb`

## Web Research Required
1. search: "cli helper script unix socket json client bash socat"
2. search: "hyprland dispatch eww open toggle cli helper"
3. search: "subcommand cli bash script argument parsing"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `test -x scripts/swal-hermes-orb` — file is executable
- [ ] `bash -n scripts/swal-hermes-orb` — 0 syntax errors
- [ ] `grep -rn "swal_hermes_orb.sock" scripts/swal-hermes-orb` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `scripts/swal-hermes-orb` | Non-existent | [NEW] CLI helper for Hermes Ambient Orb and Fluent theme control | LOW |

## DO NOT touch
- `scripts/swal-theme` — core theme switcher
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Add `#!/usr/bin/env bash` with `set -euo pipefail`.
2. Provide friendly `--help` documentation for all subcommands.

## Merge Order
- **Merge order within wave:** 14
- **Expected effort:** Small (<20m)
- **Parallel with:** All other wave issues (disjoint file islands)
