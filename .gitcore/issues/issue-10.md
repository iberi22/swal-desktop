# [Ola 1.10] SWAL-10 — test(e2e-node): End-to-end test suite for theme switching and daemon health

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- No automated E2E test suite verifying theme switching, SCSS generation, and process monitoring.

## Desired State (DELTA)
- Implement `tests/test_theme_engine.py` validating theme loading, switching, and schema conformance.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "pytest python test suite for cli tools"
2. search: "subprocess test automation in python"
3. search: "json schema validation tests"
4. search: "mocking eww and hyprctl in tests"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- Python unittest / pytest pattern

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -f tests/test_theme_engine.py`
- [ ] `python3 tests/test_theme_engine.py | grep -q "OK"`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `tests/test_theme_engine.py` | 0 lines | E2E test suite for SWAL Desktop | LOW |

## DO NOT touch (Anti-Regression)
- `scripts/install.sh`
- `eww/eww.scss`
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard ⚠️
1. **READ before write**: Read all existing files in the island before modifying.
2. **Match existing patterns**: Follow SWAL coding conventions and design tokens.
3. **No inventar imports**: Use only standard libraries or declared packages.
4. **Command verification**: Every single Acceptance Criteria must pass with exit code 0.

## PR Delivery Requirements (ANTI-EMPTY-PR)
- [ ] `git status --porcelain` lists new/modified files BEFORE opening PR
- [ ] `git diff --stat HEAD` is non-empty
- [ ] The PR contains >= 1 modified file
- [ ] IF the task cannot be completed: DO NOT open PR — report the blocker

## Verification
```bash
# Verify acceptance criteria
test -f tests/test_theme_engine.py
python3 tests/test_theme_engine.py | grep -q "OK"
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 3 (Release)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
