# [Ola 1.06] SWAL-06 — feat(widget-vault): Declarative JSON A2UI widget schema and storage vault

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- No persistent schema or storage directory for storing agent-generated generative UI widgets.

## Desired State (DELTA)
- Create `schemas/widget.schema.json` and initialize `widgets/` directory for desktop widget persistence.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "declarative UI schema json draft 07"
2. search: "agent generative UI json contract"
3. search: "generic component registry architecture"
4. search: "hot reloadable desktop widgets"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- JSON Schema Draft-07 specification

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -f schemas/widget.schema.json`
- [ ] `test -d widgets`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `schemas/widget.schema.json` | 0 lines | A2UI Widget JSON Schema | LOW |

## DO NOT touch (Anti-Regression)
- `eww/eww.yuck`
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
test -f schemas/widget.schema.json
test -d widgets
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 2 (Modules)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
