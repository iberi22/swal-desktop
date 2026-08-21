# [Ola 5.03] feat-swal-43 — Generative AUI Agent Action Card & Dynamic Response Streamer

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `crates/swal-a2ui-engine/` streams mock A2UI cards for the orb.
- No structured `AgentActionCard` model exists that pairs agent cognitive thoughts with dynamic actionable UI controls (before/after diffs, one-click execution buttons, rollback).

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-a2ui-engine/src/agent_action_card.rs`:
  - Struct `AgentActionCard`:
    - `agent_id: String`, `task_id: String`, `thought_summary: String`, `status: AgentActionStatus` (`Pending`, `Ready`, `Executed`, `RolledBack`, `Failed`).
    - `metric_impact: Option<MetricImpact>` (e.g. `label: "RAM Liberada"`, `before: "8.2 GB"`, `after: "4.1 GB"`).
    - `action_buttons: Vec<ActionButton>` (`id`, `label`, `action_payload`, `variant: Primary/Destructive/Subtle`).
    - `diff_preview: Option<String>`.
  - Methods: `new(agent_id, thought) -> Self`, `with_metric_impact(...)`, `add_action_button(...)`, `to_json(&self) -> String`, `from_json(json: &str) -> Result<Self>`.
  - **Embedded Unit Tests**: Include unit tests validating card builders, JSON roundtrips, and action button payload parsing.
- **File Target**: `crates/swal-a2ui-engine/src/agent_action_card.rs`

## Web Research Required
1. search: "agentic user interface actionable cards rust"
2. search: "generative ui action button callback payload rust"
3. search: "agent response card schema design"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `cargo test -p swal-a2ui-engine` — all unit tests pass
- [ ] `grep -rn "AgentActionCard" crates/swal-a2ui-engine/src/agent_action_card.rs` >= 1 match
- [ ] `grep -rn "MetricImpact" crates/swal-a2ui-engine/src/agent_action_card.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/agent_action_card.rs` | Non-existent | [NEW] Generative AUI Agent Action Card engine with full unit tests | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-a2ui-engine/src/hermes_streamer.rs` first.
2. Provide clean, robust JSON serializations.

## Merge Order
- **Merge order within wave:** 3
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
