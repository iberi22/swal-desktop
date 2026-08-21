# [Ola 3.09] feat-swal-24 — Hermes Direct A2UI Component Streamer in swal-a2ui-engine

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-a2ui-engine/` provides AST definition and declarative schema parser.
- Hermes agent currently lacks a high-level component streamer helper to emit incremental A2UI fragments (Cards, ProgressBars, StatBadges, ActionButtons).

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-a2ui-engine/src/hermes_streamer.rs`:
  - Struct `HermesA2uiStreamer` with builder methods:
    - `.create_agent_card(title, summary)`
    - `.add_step(label, status)`
    - `.add_action_button(label, callback_cmd)`
    - `.add_metric(label, value, unit)`
    - `.to_json_fragment() -> String`
    - `.to_eww_yuck_snippet() -> String`
- **File Target**: `crates/swal-a2ui-engine/src/hermes_streamer.rs`

## Web Research Required
1. search: "declarative a2ui component builder rust"
2. search: "streaming ui json ast compiler rust"
3. search: "eww yuck dynamic widget snippet generator"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `grep -rn "HermesA2uiStreamer" crates/swal-a2ui-engine/src/hermes_streamer.rs` >= 1 match
- [ ] `grep -rn "to_eww_yuck_snippet" crates/swal-a2ui-engine/src/hermes_streamer.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/hermes_streamer.rs` | Non-existent | [NEW] Hermes A2UI component streamer and Yuck snippet compiler | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/src/schema.rs` — base schema definition
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-a2ui-engine/src/lib.rs` and `schema.rs` before writing.
2. Include comprehensive unit tests inside `hermes_streamer.rs`.

## Merge Order
- **Merge order within wave:** 9
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
