# [Ola 3.12] feat-swal-27 — Unit Test Suite for Files Fluent Theme Tokens & Schema Validator

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `tests/test_theme_engine.py` validates `hive-dark.json`, `cyber-neon.json`, `nord-swal.json`.
- No dedicated Python test suite validates the new `fluent-dark.json` and `fluent-mica.json` themes against `theme.schema.json`.

## Desired State (DELTA)
- **Specific Addition**: Create `tests/test_fluent_theme.py`:
  - Pytest / unittest suite:
    - Validates `fluent-dark.json` and `fluent-mica.json` against `schemas/theme.schema.json`.
    - Verifies color contrast ratios (text vs background >= 4.5:1).
    - Verifies presence of Hyprland border color definitions and Fluent Blue accents.
- **File Target**: `tests/test_fluent_theme.py`

## Web Research Required
1. search: "python jsonschema validate theme schema pytest"
2. search: "wcag color contrast calculation python rgb hex"
3. search: "files app fluent theme color validation"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `python3 -m unittest tests/test_fluent_theme.py` — all tests pass
- [ ] `grep -rn "test_fluent_dark_schema" tests/test_fluent_theme.py` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `tests/test_fluent_theme.py` | Non-existent | [NEW] Python test suite for Fluent theme tokens and schema validation | LOW |

## DO NOT touch
- `tests/test_theme_engine.py` — baseline theme test
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `schemas/theme.schema.json` to verify schema fields.
2. Use standard library `unittest` and `json` to ensure zero external pip dependencies.

## Merge Order
- **Merge order within wave:** 12
- **Expected effort:** Small (<20m)
- **Parallel with:** All other wave issues (disjoint file islands)
