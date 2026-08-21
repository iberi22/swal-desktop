# [Ola 2.02] feat-swal-12 — Declarative A2UI JSON AST Compiler

> Ola 2 — Rust Native Core & Generative A2UI.
> Labels: `ola2`, `wave-2` (sin `jules` todavía)

---

## Current State (MEDIBLE)
- Feature: `feat-swal-12` at 10% in `.gitcore/features.json`
- File: `crates/swal-a2ui-engine/src/lib.rs` (42 lines, basic AST enum)
- Tests: 1 existing, 1 passing (`test_parse_sample_widget`)

## Desired State (DELTA)
- **Component AST Expansion**: Support the complete `@swal/ui` design component catalog: `Card`, `Grid`, `StatusBadge`, `MetricPill`, `Button`, `LogViewer`, `Terminal`, `Tabs`.
- **JSON Schema Validator**: Compile dynamic widget JSON strings into strongly typed component trees with color token resolving (`hive-dark` / `cyber-neon`).
- **File Island**: `crates/swal-a2ui-engine/`

## Web Research Required
1. search: "serde enum untagged vs tagged json ast rust"
2. search: "declarative ui component tree compiler rust"
3. search: "swal ui design tokens schema"
4. search: "hot reload json widgets wayland rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `cargo test -p swal-a2ui-engine` — all tests pass
- [ ] `grep -rn "MetricPill" crates/swal-a2ui-engine/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/lib.rs` | 42 lines | Expand component tree & token resolvers | LOW |
| `crates/swal-a2ui-engine/src/schema.rs` | NEW | JSON schema validation routines | LOW |

## DO NOT touch
- `crates/swal-telemetry-rs/` — assigned to Issue #11
- `crates/swal-ambient-orb/` — assigned to Issue #13
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ before write: inspect `schemas/widget.schema.json` and `crates/swal-a2ui-engine/src/lib.rs`
2. Preserve existing `parse_widget_json` function signature

## Merge Order
- **Merge order within wave:** 2
- **Expected effort:** Medium (1-2h)
- **Parallel with:** #11, #13, #14, #15
