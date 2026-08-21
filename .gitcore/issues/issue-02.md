# [Ola 1.02] SWAL-02 — feat(tokens): Canonical Hive Dark design tokens port from @swal/ui

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- No unified theme matching @swal/ui Hive Dark palette in desktop configuration.

## Desired State (DELTA)
- Port Hive Dark tokens (#020617, #0f172a, #06b6d4, #f97316, #10b981) to `themes/hive-dark.json`.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "swal design system tokens hive dark"
2. search: "accessible slate color palette contrast"
3. search: "wcag contrast ratios for dark theme cyan"
4. search: "hex to rgba CSS conversions"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- @swal/ui CSS variable mappings

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -f themes/hive-dark.json`
- [ ] `grep -q "#06b6d4" themes/hive-dark.json`
- [ ] `grep -q "hive-dark" themes/hive-dark.json`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `themes/hive-dark.json` | 0 lines | Canonical Hive Dark theme JSON definition | LOW |

## DO NOT touch (Anti-Regression)
- `scripts/swal-theme`
- `eww/eww.yuck`
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
test -f themes/hive-dark.json
grep -q "#06b6d4" themes/hive-dark.json
grep -q "hive-dark" themes/hive-dark.json
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 1 (Foundation)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
