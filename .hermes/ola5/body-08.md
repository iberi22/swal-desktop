# [Ola 5.08] feat-swal-48 — Settings CLI Companion Tool in Rust (swal-node-daemon)

> Ola 5 — macOS-Inspired Centralized System Settings & Generative AUI Component System.
> Labels: `ola5`, `wave-5`

---

## Current State (MEDIBLE)
- Modifying configurations from the terminal currently relies on ad-hoc python scripts or editing files manually.
- No compiled Rust CLI command runner exists for querying, setting, and exporting system settings.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-node-daemon/src/settings_cli.rs`:
  - Struct `SettingsCliRunner`:
    - Commands:
      - `get(key: &str) -> String`: returns the value for dot-notation key.
      - `set(key: &str, value: &str) -> Result<String>`: updates value and triggers IPC broadcast.
      - `list() -> String`: formats all settings as pretty-printed JSON or key-value table.
      - `reset() -> Result<String>`: resets settings to factory defaults.
      - `export_schema() -> String`: outputs the canonical JSON schema.
  - **Embedded Unit Tests**: Include complete unit tests executing CLI command branches, validating get/set mutations, and testing error handling on unknown keys.
- **File Target**: `crates/swal-node-daemon/src/settings_cli.rs`

## Web Research Required
1. search: "cli subcommand runner rust pure string parsing"
2. search: "pretty print json table terminal rust"
3. search: "settings cli tool getter setter rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-node-daemon` — 0 errors
- [ ] `cargo test -p swal-node-daemon` — all unit tests pass
- [ ] `grep -rn "SettingsCliRunner" crates/swal-node-daemon/src/settings_cli.rs` >= 1 match
- [ ] `grep -rn "export_schema" crates/swal-node-daemon/src/settings_cli.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-node-daemon/src/settings_cli.rs` | Non-existent | [NEW] Settings CLI companion tool with comprehensive unit tests | LOW |

## DO NOT touch
- `crates/swal-node-daemon/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. Support friendly error messages when keys don't exist.
2. Include complete unit test coverage.

## Merge Order
- **Merge order within wave:** 8
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
