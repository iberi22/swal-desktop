# [Ola 3.02] feat-swal-17 — Eww SCSS Fluent 2 Styling & Acrylic Mica Effects for SWAL Files

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `eww/eww.scss` contains base stylesheet for SWAL Desktop and file manager.
- No dedicated modular stylesheet for Windows Files style segmented breadcrumbs, acrylic card elevations, and rounded pills exists.

## Desired State (DELTA)
- **Specific Addition**: Create `eww/files-fluent.scss` containing modular SCSS mixins and classes:
  - `.fluent-mica-window`: 12px border radius, backdrop blur, 1px subtle top border highlight.
  - `.fluent-segmented-breadcrumb`: Segmented chevron path pills inspired by Windows Files Explorer.
  - `.fluent-tab-strip`: Windows 11 style tab headers with smooth active indicator line.
  - `.fluent-toolbar-action`: Micro-interaction hover states with fluid transitions.
- **File Target**: `eww/files-fluent.scss`

## Web Research Required
1. search: "files-community/Files breadcrumb bar style scss"
2. search: "fluent design 2 rounded corners tabs scss css"
3. search: "gtk3 eww scss backdrop-filter mica window styling"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `test -f eww/files-fluent.scss` — file exists
- [ ] `grep -rn "fluent-mica-window" eww/files-fluent.scss` >= 1 match
- [ ] `grep -rn "fluent-segmented-breadcrumb" eww/files-fluent.scss` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `eww/files-fluent.scss` | Non-existent | [NEW] Modular Fluent 2 styling and SCSS mixins for Files App look | LOW |

## DO NOT touch
- `eww/eww.yuck` — baseline UI file
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `eww/eww.scss` before writing to match GTK3/Eww SCSS syntax constraints.
2. Use valid SCSS color variables and pseudo-classes (`:hover`, `:active`).

## Merge Order
- **Merge order within wave:** 2
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
