# [Ola 1.05] SWAL-05 — feat(agent-skills): swal-theme-creator agent skill for autonomous generation

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- AI agents lacked standard instructions on how to formulate and switch visual desktop themes.

## Desired State (DELTA)
- Implement `.agents/skills/swal-theme-creator/SKILL.md` documenting schema, CLI, and validation workflow.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "agent skill definition format yaml frontmatter"
2. search: "llm tool calling schema design"
3. search: "declarative agent UI customization"
4. search: "safe shell execution guidelines for agents"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- Standard Agent Skill YAML frontmatter + Markdown documentation

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -f .agents/skills/swal-theme-creator/SKILL.md`
- [ ] `grep -q "swal-theme" .agents/skills/swal-theme-creator/SKILL.md`
- [ ] `grep -q "swal-theme-creator" .agents/skills/swal-theme-creator/SKILL.md`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `.agents/skills/swal-theme-creator/SKILL.md` | 0 lines | Agent skill documentation | LOW |

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
test -f .agents/skills/swal-theme-creator/SKILL.md
grep -q "swal-theme" .agents/skills/swal-theme-creator/SKILL.md
grep -q "swal-theme-creator" .agents/skills/swal-theme-creator/SKILL.md
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 2 (Modules)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
