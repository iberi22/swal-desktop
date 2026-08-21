# [Ola 2.05] feat-swal-15 — Agent Widget Vault & Addon Inventory Manager

> Ola 2 — Rust Native Core & Generative A2UI.
> Labels: `ola2`, `wave-2` (sin `jules` todavía)

---

## Current State (MEDIBLE)
- Feature: `feat-swal-15` at 0% in `.gitcore/features.json`
- File: `crates/swal-widget-vault/` (NEW directory)
- Tests: 0 existing

## Desired State (DELTA)
- **Crate Scaffold**: Create `crates/swal-widget-vault/` with Cargo manifest.
- **Directory Watcher & Indexer**: Monitor `~/.config/swal/widgets/*.json` using `notify` crate.
- **Vault Operations**: Provide API to list installed widgets, pin/unpin widgets on desktop, and export/import widget bundles.
- **File Island**: `crates/swal-widget-vault/`

## Web Research Required
1. search: "notify crate recursive file watcher async rust"
2. search: "desktop widget catalog inventory json rust"
3. search: "swal widgets schema specification"
4. search: "zero copy json indexing rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-widget-vault` — 0 errors
- [ ] `cargo test -p swal-widget-vault` — all tests pass
- [ ] `grep -rn "WidgetVault" crates/swal-widget-vault/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-widget-vault/Cargo.toml` | NEW | Crate manifest & dependencies | LOW |
| `crates/swal-widget-vault/src/lib.rs` | NEW | Widget storage & indexer implementation | LOW |

## DO NOT touch
- `crates/swal-telemetry-rs/` — assigned to Issue #11
- `crates/swal-a2ui-engine/` — assigned to Issue #12
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `schemas/widget.schema.json` before writing
2. Use path resolution based on standard XDG directories (`dirs::config_dir()`)

## Merge Order
- **Merge order within wave:** 5
- **Expected effort:** Medium (1-2h)
- **Parallel with:** #11, #12, #13, #14
