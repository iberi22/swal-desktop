# [Ola 1.05] feat-daemon: Hermes Live Reasoning SSE Stream Listener and State Broadcaster

> Ola 1 — Node Daemon & Hermes Coordination.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-daemon` in `crates/swal-node-daemon`
- File: `crates/swal-node-daemon/src/lib.rs` (12 tests passing)
- Problem: The daemon supervises local layers and polls Xavier health, but does not subscribe to real-time agent reasoning tokens from Hermes Gateway (`:8000/v1/events` or `:8006/events`).

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-node-daemon/src/hermes_stream.rs` with `HermesEventListener` using `reqwest-eventsource` or native Tokio SSE line parsing.
- Parse events: `AgentThinking(token_chunk)`, `AgentSpeaking(audio_url)`, `AgentIdle`.
- Broadcast `ShellEvent::OrbStateChanged` to `NativeShellSupervisor` in real-time.

## Web Research Required
1. search: "tokio async SSE stream parser line by line rust"
2. search: "server sent events client rust without heavy dependencies"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all tests pass
- [ ] `grep -rn "HermesEventListener" crates/swal-node-daemon/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/hermes_stream.rs` | None (New) | Implement asynchronous SSE event parser | LOW |
| `crates/swal-node-daemon/src/lib.rs` | 260 lines | Export `pub mod hermes_stream;` | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-render-pipeline/` — assigned to Issue #4

## Anti-Hallucination Guard
1. Use auto-reconnect backoff (1s, 2s, 5s) if Hermes Gateway is restarting
2. Ensure stream reading is non-blocking and spawned in its own Tokio task

## Merge Order
- **Merge order within wave:** 5
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
