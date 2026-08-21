# [Ola 2.04] feat-swal-14 — Async Tokio Coordinator & Xavier Bridge

> Ola 2 — Rust Native Core & Generative A2UI.
> Labels: `ola2`, `wave-2` (sin `jules` todavía)

---

## Current State (MEDIBLE)
- Feature: `feat-swal-14` at 0% in `.gitcore/features.json`
- File: `crates/swal-node-daemon/` (NEW directory)
- Tests: 0 existing

## Desired State (DELTA)
- **Crate Scaffold**: Create `crates/swal-node-daemon/` with Tokio async daemon entrypoint.
- **Xavier Health Poller**: Monitor `http://127.0.0.1:8006/health` and MCP socket `:8100` asynchronously every 5s.
- **Edge-Mesh P2P Discovery**: Listen on local mesh control socket and broadcast node status.
- **File Island**: `crates/swal-node-daemon/`

## Web Research Required
1. search: "reqwest async health check client rust 2026"
2. search: "tokio background supervisor daemon pattern rust"
3. search: "xavier cognitive memory rest api"
4. search: "edge mesh p2p discovery protocol rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all tests pass
- [ ] `grep -rn "XavierClient" crates/swal-node-daemon/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/Cargo.toml` | NEW | Crate manifest with tokio, reqwest | LOW |
| `crates/swal-node-daemon/src/lib.rs` | NEW | Daemon supervisor & lifecycle coordinator | LOW |
| `crates/swal-node-daemon/src/xavier.rs` | NEW | HTTP client for Xavier Memory Core | LOW |

## DO NOT touch
- `crates/swal-telemetry-rs/` — assigned to Issue #11
- `crates/swal-a2ui-engine/` — assigned to Issue #12
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ before write: inspect `nixos/swal-node.nix` for exact ports (`:8006`, `:8100`)
2. Handle network errors gracefully with retry backoff

## Merge Order
- **Merge order within wave:** 4
- **Expected effort:** Medium (1-2h)
- **Parallel with:** #11, #12, #13, #15
