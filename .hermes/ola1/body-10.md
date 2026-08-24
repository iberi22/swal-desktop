# [Ola 1.10] feat-telemetry: RAPL Energy and Battery Discharge Rate Profiler

> Ola 1 — Telemetry & Power Profiling.
> Labels: `ola1`, `wave-1`

---

## Current State (MEDIBLE)
- Feature: `feat-telemetry` in `crates/swal-telemetry-rs`
- File: `crates/swal-telemetry-rs/src/lib.rs` (7 tests passing, CPU/RAM/GPU/Temps/Net covered)
- Problem: CPU package energy is read indirectly through GPU power, but missing direct Intel/AMD RAPL energy counter delta calculation (`/sys/class/powercap/intel-rapl:0/energy_uj`).

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-telemetry-rs/src/rapl.rs` containing `RaplPowerMeter`.
- Calculate CPU package power consumption (Watts) from microjoule counters between two sample ticks: `watts = (delta_uj / 1_000_000.0) / elapsed_secs`.
- Add `pub cpu_power_watts: f32` to `SystemMetrics`.

## Web Research Required
1. search: "linux rapl energy_uj powercap sysfs reading rust zero allocation"
2. search: "calculating instantaneous power from microjoule counters rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-telemetry-rs` — 0 errors
- [ ] `cargo test -p swal-telemetry-rs` — all tests pass
- [ ] `grep -rn "RaplPowerMeter" crates/swal-telemetry-rs/` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-telemetry-rs/src/rapl.rs` | None (New) | Implement RAPL power meter and delta math | LOW |
| `crates/swal-telemetry-rs/src/lib.rs` | 320 lines | Export `pub mod rapl;` and embed power | LOW |

## DO NOT touch
- `crates/swal-a2ui-engine/` — assigned to Issue #1
- `crates/swal-files/` — assigned to Issue #2
- `crates/swal-render-pipeline/` — assigned to Issue #4

## Anti-Hallucination Guard
1. Handle RAPL counter overflow (32-bit/64-bit wraparound) using `.wrapping_sub()` or `saturating_sub()`
2. If powercap is not present (e.g. virtualized environment), return 0.0 without failing

## Merge Order
- **Merge order within wave:** 10
- **Expected effort:** Small (<30m)
- **Parallel with:** All other wave issues (disjoint file islands)
