# [Ola 1.09] SWAL-09 — feat(install-kit): Node installation bootstrap script for machine deployment

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Installer only cloned repository and symlinked /etc/nixos without configuring SWAL Node themes and schemas.

## Desired State (DELTA)
- Update `scripts/install.sh` to install theme assets, schemas, CLI tools, and agent skills on fresh machines.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "bash set -euo pipefail best practices"
2. search: "idempotent machine installer script"
3. search: "symlink management in nixos installations"
4. search: "curl bash installer security and safety"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- Idempotent bash scripting

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -x scripts/install.sh`
- [ ] `grep -q "swal-theme switch" scripts/install.sh`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `scripts/install.sh` | 70 lines | Update with Node Kit initialization | LOW |

## DO NOT touch (Anti-Regression)
- `eww/scripts/ram_panel.py`
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
test -x scripts/install.sh
grep -q "swal-theme switch" scripts/install.sh
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 3 (Release)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
