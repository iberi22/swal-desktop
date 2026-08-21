# [Ola 3.01] feat-swal-16 — Windows / Files-Community Fluent Dark & Fluent Mica Theme Tokens

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- Themes directory `themes/` contains `hive-dark.json`, `cyber-neon.json`, `nord-swal.json`.
- No Windows 11 / Fluent 2 / Files App (`files-community/Files`) inspired theme tokens currently exist.
- Schema in `schemas/theme.schema.json` defines required tokens (`bg`, `elevated`, `accent_primary`, `border_active`, etc.).

## Desired State (DELTA)
- **Specific Addition**: Create `themes/fluent-dark.json` and `themes/fluent-mica.json` matching the Windows 11 / Files App design system:
  - Deep Mica acrylic background: `rgba(32, 32, 32, 0.94)` / `rgba(44, 44, 44, 0.88)`
  - Fluent Blue primary accent: `#60cdff` / `#0078d4`
  - Subtle highlight borders: `rgba(255, 255, 255, 0.08)`
  - Elevated surfaces: `rgba(44, 44, 44, 0.70)`
  - Hyprland border gradients with Fluent blue & cyan glow.
- **File Target**: `themes/fluent-dark.json`, `themes/fluent-mica.json`

## Web Research Required
1. search: "files-community Files app theme colors windows fluent"
2. search: "windows 11 fluent design system color palette dark mode"
3. search: "fluent 2 mica acrylic background color rgba"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `jq . themes/fluent-dark.json` — 0 syntax errors
- [ ] `jq . themes/fluent-mica.json` — 0 syntax errors
- [ ] `grep -rn "accent_primary" themes/fluent-dark.json` >= 1 match
- [ ] `grep -rn "accent_primary" themes/fluent-mica.json` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `themes/fluent-dark.json` | Non-existent | [NEW] Fluent Dark theme tokens matching Files App | LOW |
| `themes/fluent-mica.json` | Non-existent | [NEW] Fluent Mica theme tokens with acrylic blur | LOW |

## DO NOT touch
- `themes/hive-dark.json` — assigned to baseline
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `schemas/theme.schema.json` and `themes/hive-dark.json` before writing.
2. Ensure all required JSON fields (`id`, `name`, `author`, `version`, `colors`, `hyprland`) are fully defined.

## Merge Order
- **Merge order within wave:** 1
- **Expected effort:** Small (<20m)
- **Parallel with:** All other wave issues (disjoint file islands)
