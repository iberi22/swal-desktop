# [Ola 1.01] feat-a2ui: CalendarGrid and AgendaList AST Component Nodes

> Ola 1 — UI & Declarative Engine.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-a2ui` in `crates/swal-a2ui-engine`
- File: `crates/swal-a2ui-engine/src/lib.rs` (350 lines, supports Card, Grid, Slider, ProcessTable)
- Tests: 14 existing, 14 passing

## Desired State (DELTA)
- **Specific Addition**: Implement `CalendarGrid { year: u32, month: u32, highlighted_days: Vec<u32> }` and `AgendaList { events: Vec<AgendaEvent> }` in `crates/swal-a2ui-engine/src/calendar.rs`.
- Wire `ComponentNode::CalendarGrid` and `ComponentNode::AgendaList` into AST parser and token resolver.
- Add `DrawCalendar` and `DrawAgenda` in `crates/swal-a2ui-engine/src/native_render.rs`.

## Web Research Required
1. search: "rust calendar date calculation zero allocation"
2. search: "declarative agenda ui AST schema json"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-a2ui-engine` — 0 errors
- [ ] `cargo test -p swal-a2ui-engine` — all tests pass
- [ ] `grep -rn "CalendarGrid" crates/swal-a2ui-engine/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-a2ui-engine/src/calendar.rs` | None (New) | Implement calendar AST types and date math | LOW |
| `crates/swal-a2ui-engine/src/lib.rs` | 350 lines | Export module and add enum variants | LOW |

## DO NOT touch
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-telemetry-rs/` — assigned to Issue #3
- `crates/swal-render-pipeline/` — assigned to Issue #4

## Anti-Hallucination Guard
1. READ before write: inspect all existing files in `crates/swal-a2ui-engine/`
2. Follow Rust 2021 idiomatic patterns with zero unsafe code

## Merge Order
- **Merge order within wave:** 1
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
