# [Ola 1.03] SWAL-03 — feat(themes): Cyber Neon legacy theme and Nord Frost theme schemas

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Legacy theme was embedded directly into eww.scss without standalone definition file.

## Desired State (DELTA)
- Create `themes/cyber-neon.json` preserving matrix green (#00ff88) and `themes/nord-swal.json`.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "matrix cyber neon color scheme hex"
2. search: "nord palette hex colors official"
3. search: "hyprland dual gradient border syntax"
4. search: "dark mode UI surface layers"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- Validated theme.schema.json structure

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -f themes/cyber-neon.json`
- [ ] `test -f themes/nord-swal.json`
- [ ] `grep -q "#00ff88" themes/cyber-neon.json`
- [ ] `grep -q "#88c0d0" themes/nord-swal.json`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `themes/cyber-neon.json` | 0 lines | Matrix neon theme JSON | LOW |
| `themes/nord-swal.json` | 0 lines | Nord frost theme JSON | LOW |

## DO NOT touch (Anti-Regression)
- `scripts/swal-theme`
- `themes/hive-dark.json`
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
test -f themes/cyber-neon.json
test -f themes/nord-swal.json
grep -q "#00ff88" themes/cyber-neon.json
grep -q "#88c0d0" themes/nord-swal.json
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 1 (Foundation)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
