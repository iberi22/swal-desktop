# [Ola 1.07] SWAL-07 — feat(dashboard-ui): Dashboard dynamic palette reactivity and theme indicator

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Dashboard used hardcoded colors instead of theme-driven SCSS variables.

## Desired State (DELTA)
- Refactor `eww/eww.scss` and `eww/eww.yuck` to consume dynamic theme variables ($theme-bg, $theme-accent, etc.).

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "eww scss variable compilation"
2. search: "gtk 3 css custom properties"
3. search: "dynamic stylesheet reloads in layer shell"
4. search: "smooth glassmorphism container styling"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- SCSS variables + Eww GTK3 CSS standards

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `grep -q "$theme-accent" eww/eww.scss`
- [ ] `grep -q "$theme-bg" eww/eww.scss`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `eww/eww.scss` | 380 lines | Refactor to dynamic SCSS variables | MED |
| `eww/eww.yuck` | 420 lines | Add theme switch bindings | LOW |

## DO NOT touch (Anti-Regression)
- `scripts/swal-theme`
- `nixos/swal-node.nix`
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
grep -q "$theme-accent" eww/eww.scss
grep -q "$theme-bg" eww/eww.scss
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 2 (Modules)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
