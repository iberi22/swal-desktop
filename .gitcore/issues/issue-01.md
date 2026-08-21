# SWAL-01: Dynamic JSON Theme Engine & swal-theme CLI Dispatcher

## Current State
Previously, theme values were hardcoded in SCSS and Lisp DSL files without programmatic validation.

## Scope
Implement a unified Python/CLI theme engine (`swal-theme`) that validates theme JSON files against `theme.schema.json`, regenerates runtime SCSS, updates Hyprland window border colors, and triggers hot-reloads with zero downtime.

## Acceptance Criteria
- [ ] `swal-theme list` prints all installed themes with active indicator.
- [ ] `swal-theme switch <name>` applies color tokens across Eww and Hyprland.
- [ ] Validated with `theme.schema.json`.

## Files to Modify
- `scripts/swal-theme` (NEW)
- `schemas/theme.schema.json` (NEW)
