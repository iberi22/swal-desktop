# [Ola 1.08] SWAL-08 — feat(process-monitor): Unified system & memory monitor with kill and mode selector

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Old RAM panel was a plain ASCII text dump without process kill buttons or profile selectors.

## Desired State (DELTA)
- Re-implement `eww/scripts/ram_panel.py` to output structured JSON with CPU/GPU/RAM meters and interactive kill commands.
- Add graceful escalation in `eww/scripts/ram_kill.sh`.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "proc statm python reading performance"
2. search: "process termination TERM to KILL escalation bash"
3. search: "json formatting for eww widgets"
4. search: "human readable memory formatting in python"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- Python /proc parser + clean JSON output

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `python3 eww/scripts/ram_panel.py | grep -q "procs"`
- [ ] `test -x eww/scripts/ram_kill.sh`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `eww/scripts/ram_panel.py` | 180 lines | Modern process & telemetry JSON producer | LOW |
| `eww/scripts/ram_kill.sh` | 15 lines | Graceful process kill script | LOW |

## DO NOT touch (Anti-Regression)
- `eww/eww.scss`
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
python3 eww/scripts/ram_panel.py | grep -q "procs"
test -x eww/scripts/ram_kill.sh
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 2 (Modules)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
