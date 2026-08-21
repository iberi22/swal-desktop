# [Ola 3.06] feat-swal-21 — Hermes Agent Protocol & Cognition State Machine in swal-ambient-orb

> Ola 3 — Files App Inspired Theme & Hermes Orb Pipeline.
> Labels: `ola3`, `wave-3`

---

## Current State (MEDIBLE)
- `crates/swal-ambient-orb/src/lib.rs` defines basic `OrbState` (`Listening`, `Thinking`, `Speaking`).
- Default agent in SWAL ecosystem is **Hermes**. No specialized Hermes cognition states (`HermesState::Decomposing`, `HermesState::StreamingA2Ui`, `HermesState::ExecutingTask`, `HermesState::IdleAwaitingUser`) currently exist.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-ambient-orb/src/hermes.rs`:
  - Enum `HermesAgentState` with: `Idle`, `ListeningVoice`, `DecomposingPlan`, `StreamingA2Ui`, `ExecutingToolAction`, `AwaitingUserFeedback`, `ErrorAlert`.
  - Struct `HermesOrbPacket` with `agent_id: String` (defaults to `"hermes"`), `state: HermesAgentState`, `prompt_summary: String`, `audio_level: f32`, `progress_pct: f32`.
  - State transition validator and event dispatching callback channel.
- **File Target**: `crates/swal-ambient-orb/src/hermes.rs`

## Web Research Required
1. search: "hermes agent orchestrator state machine rust"
2. search: "ambient voice orb agent state machine states"
3. search: "async channel agent event dispatching tokio rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-ambient-orb` — 0 errors
- [ ] `grep -rn "HermesAgentState" crates/swal-ambient-orb/src/hermes.rs` >= 1 match
- [ ] `grep -rn "HermesOrbPacket" crates/swal-ambient-orb/src/hermes.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-ambient-orb/src/hermes.rs` | Non-existent | [NEW] Hermes agent protocol and ambient orb cognition state machine | LOW |

## DO NOT touch
- `crates/swal-ambient-orb/src/lib.rs` — core orb pipeline
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-ambient-orb/src/lib.rs` before writing.
2. Include unit tests covering all `HermesAgentState` transitions.

## Merge Order
- **Merge order within wave:** 6
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
