# [Ola 3.15] feat-swal-30 — Architecture Documentation for Fluent Theme & Hermes Ambient Orb

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `docs/` contains baseline architecture docs.
- No unified specification and user manual exists describing the Windows Files inspired theme tokens and the Hermes Agent A2UI Ambient Orb pipeline.

## Desired State (DELTA)
- **Specific Addition**: Create `docs/HERMES_ORB_FLUENT_THEME.md`:
  - Complete architectural breakdown:
    1. Files App Inspired Fluent 2 Design System (Mica acrylic blur, token hierarchy, breadcrumbs).
    2. Hermes Ambient Orb State Machine (`Idle`, `ListeningVoice`, `DecomposingPlan`, `StreamingA2Ui`).
    3. Unix Domain Socket IPC Protocol specification (`/tmp/swal_hermes_orb.sock`).
    4. Keybindings, CLI commands (`swal-hermes-orb`), and Eww widget integration.
- **File Target**: `docs/HERMES_ORB_FLUENT_THEME.md`

## Web Research Required
1. search: "markdown technical architecture document template"
2. search: "ambient voice agent system architecture diagram"
3. search: "fluent design system token specification documentation"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `test -f docs/HERMES_ORB_FLUENT_THEME.md` — file exists
- [ ] `grep -rn "Hermes Ambient Orb" docs/HERMES_ORB_FLUENT_THEME.md` >= 1 match
- [ ] `grep -rn "Fluent 2 Design System" docs/HERMES_ORB_FLUENT_THEME.md` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `docs/HERMES_ORB_FLUENT_THEME.md` | Non-existent | [NEW] Complete technical architecture and user guide | LOW |

## DO NOT touch
- `docs/ARCHITECTURE.md` — system base architecture
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Follow standard GitHub Flavored Markdown with clean headings and code snippets.
2. Accurately document the components implemented across Wave 3.

## Merge Order
- **Merge order within wave:** 15
- **Expected effort:** Small (<20m)
- **Parallel with:** All other wave issues (disjoint file islands)
