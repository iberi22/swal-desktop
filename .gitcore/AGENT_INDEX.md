# SWAL Desktop — Agent Role Routing & Context Index

## Role Routing

| Agent Role | Primary Tooling | Responsibilities | Context Files |
|---|---|---|---|
| **Hermes Orchestrator** | `agent`, `gitcore-swal-workflow` | High-level synthesis, wave design, cross-node delegation | `.gitcore/planning/PLANNING.md`, `.gitcore/features.json` |
| **Jules Worker** | `gh issue`, isolated worktrees | Autonomous issue execution within strict file islands | `.gitcore/issues/*.md`, `.gitcore/AGENT_MAP.md` |
| **Antigravity Lead** | Pair-programming, MCP tools | Architecture review, E2E test validation, self-healing | `.gitcore/ARCHITECTURE.md`, `tests/` |
| **Xavier Core** | HTTP `:8006`, MCP `:8100` | Global memory persistence, GraphRAG retrieval | `apps/xavier/README.md` |

## Canonical Workflows
- **Wave Execution Loop**: `Plan -> Generate Canonical Issues -> Verify Pre-Dispatch -> Dispatch to Jules -> Reconcile features.json`.
- **System Diagnosis**: `swal-doctor` / `swal-doctor --fix`.
- **Theme Deployment**: `swal-theme switch <name>`.
