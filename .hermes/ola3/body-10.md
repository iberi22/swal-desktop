# [Ola 3.10] feat-swal-25 — Radial Agent Quick Action Menu for Hermes Orb in Eww

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `eww/` contains system panels for RAM, Network, and SWAL Files.
- No interactive radial / popup widget exists for the Hermes Ambient Orb to trigger quick actions (`@summarize`, `@refactor`, `@execute`, `@chat`).

## Desired State (DELTA)
- **Specific Addition**: Create `eww/hermes_orb.yuck` and `eww/scripts/hermes_orb_menu.py`:
  - `eww/hermes_orb.yuck`: Defwidget `hermes_orb_overlay` displaying reactive animated orb container and radial action buttons on click.
  - `eww/scripts/hermes_orb_menu.py`: Python dispatcher script communicating with Hermes CLI and Orb IPC socket.
- **File Target**: `eww/hermes_orb.yuck`, `eww/scripts/hermes_orb_menu.py`

## Web Research Required
1. search: "eww yuck radial menu widget circular buttons"
2. search: "voice assistant floating orb click popup eww"
3. search: "python unix domain socket send json payload"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `test -f eww/hermes_orb.yuck` — file exists
- [ ] `test -f eww/scripts/hermes_orb_menu.py` — file exists
- [ ] `python3 -m py_compile eww/scripts/hermes_orb_menu.py` — 0 syntax errors

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `eww/hermes_orb.yuck` | Non-existent | [NEW] Eww widget for Hermes Ambient Orb overlay | LOW |
| `eww/scripts/hermes_orb_menu.py` | Non-existent | [NEW] Python action dispatcher for Hermes Orb actions | LOW |

## DO NOT touch
- `eww/eww.scss` — global stylesheet
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Follow standard Eww Yuck defwidget syntax.
2. Ensure Python script has `#!/usr/bin/env python3` and executable permissions.

## Merge Order
- **Merge order within wave:** 10
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
