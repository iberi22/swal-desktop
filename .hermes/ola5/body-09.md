# [Ola 5.09] feat-swal-49 — SWAL Doctor Embedded Self-Healing & Diagnostic Engine in Rust

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- `scripts/swal-doctor` is a Python script checking Eww and SCSS files.
- No native pure Rust self-healing and system diagnostic engine exists in `swal-node-daemon`.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/doctor_engine.rs`:
  - Struct `SwalDoctorReport`:
    - `checks: Vec<DiagnosticCheck>`, `all_passed: bool`, `error_count: usize`, `warning_count: usize`.
    - `DiagnosticCheck { name: String, category: String, status: CheckStatus (Pass, Warning, Fail), message: String, auto_fixable: bool }`.
  - Struct `SwalDoctorEngine`:
    - Methods:
      - `run_full_diagnostics() -> SwalDoctorReport`: probes Wayland socket, GPU graphics acceleration, Xavier GraphRAG connectivity (`:8006`), settings store validity, and storage disk space.
      - `attempt_auto_fix(check_name: &str) -> bool`: repairs broken configs and creates missing directory structures.
  - **Embedded Unit Tests**: Include complete unit tests mocking diagnostic probes, report formatting, and auto-fix execution.
- **File Target**: `crates/swal-node-daemon/src/doctor_engine.rs`

## Web Research Required
1. search: "system diagnostics self healing health check rust"
2. search: "health check probe reporting framework rust"
3. search: "socket ping url health check pure rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "SwalDoctorEngine" crates/swal-node-daemon/src/doctor_engine.rs` >= 1 match
- [ ] `grep -rn "run_full_diagnostics" crates/swal-node-daemon/src/doctor_engine.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/doctor_engine.rs` | Non-existent | [NEW] Native SWAL Doctor diagnostic and self-healing engine in Rust | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Design diagnostic checks with safe fallbacks in sandbox environments.
2. Include comprehensive unit test assertions.

## Merge Order
- **Merge order within wave:** 9
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
