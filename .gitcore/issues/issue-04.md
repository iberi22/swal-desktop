# [Ola 1.04] SWAL-04 — feat(nixos-node): SWAL Node NixOS module with Xavier and Edge-Mesh services

> Ola 1 — SWAL Node Kit & Unified Theme Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- NixOS configuration did not include dedicated background service units for Xavier cognitive memory and Edge-Mesh P2P node.

## Desired State (DELTA)
- Create `nixos/swal-node.nix` configuring systemd user services for xavier-core (:8006) and edge-mesh.
- Import `swal-node.nix` into `nixos/configuration.nix`.

## 🌐 Web Research Required
**MANDATORY — 4-6 queries. The agent MUST research before implementing.**
1. search: "nixos systemd user services best practices"
2. search: "nixos module imports flake syntax"
3. search: "nixos environment.variables system-wide"
4. search: "systemd restart on-failure configuration"

## 🔬 Agent Session Prompt
"Before implementing, please:
1. Research the requested functionality — search the web for current best practices.
2. Read and understand existing codebase patterns.
3. Verify that changes remain strictly within the assigned File Island.
4. Document findings and verify all acceptance criteria."

## Existing Code Patterns
- NixOS module architecture with lib and pkgs

## Acceptance Criteria (VERIFIABLE BY COMMAND)
- [ ] `test -f nixos/swal-node.nix`
- [ ] `grep -q "xavier-core" nixos/swal-node.nix`
- [ ] `grep -q "edge-mesh" nixos/swal-node.nix`
- [ ] `grep -q "swal-node.nix" nixos/configuration.nix`
- [ ] `git status --porcelain` shows expected files in the assigned island

## Files to Modify
| File | Current State | Change | Risk |
|---|---|---|---|
| `nixos/swal-node.nix` | 0 lines | SWAL Node systemd module | MED |
| `nixos/configuration.nix` | 250 lines | Add ./swal-node.nix import | LOW |

## DO NOT touch (Anti-Regression)
- `nixos/hardware.nix`
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
test -f nixos/swal-node.nix
grep -q "xavier-core" nixos/swal-node.nix
grep -q "edge-mesh" nixos/swal-node.nix
grep -q "swal-node.nix" nixos/configuration.nix
```

## Dependencies & Merge Order
- **Merge order within wave:** Batch 2 (Modules)
- **Expected effort:** Small (<1h)

## Failure Recovery
| If this happens | Action |
|---|---|
| Command fails | Inspect stderr and fix syntax before committing |
| File does not exist | Create directory path with `mkdir -p` |
