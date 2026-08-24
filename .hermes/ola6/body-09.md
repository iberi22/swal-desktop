# [Ola 6.09] feat-swal-59 — Agentic File Tool Plugin System in Pure Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- File actions in `swal-files` (compress, delete, rename, preview) are hardcoded in CLI dispatchers.
- Standalone and desktop users cannot dynamically load community or AI-generated action plugins (e.g., audio converter, image optimizer, code formatter, markdown summarizer).

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/plugin_system.rs`:
  - Structs & Enums:
    - `PluginTrigger`: `ContextMenu`, `OmnibarCommand(String)`, `FileOpenHook(String)`, `AutoTransform`.
    - `PluginTarget`: `SingleFile`, `MultiSelection`, `Directory`, `Any`.
    - `FilePluginManifest`: `id: String`, `name: String`, `version: String`, `author: String`, `description: String`, `triggers: Vec<PluginTrigger>`, `target_types: Vec<String>`, `executable_command: String`.
    - `PluginExecutionContext`: `selected_paths: Vec<String>`, `current_directory: String`, `environment_vars: std::collections::HashMap<String, String>`.
    - `PluginExecutionResult`: `success: bool`, `output_message: String`, `modified_files: Vec<String>`, `created_files: Vec<String>`.
    - `FilePluginRegistry`: Registry struct with methods:
      - `new() -> Self`
      - `load_from_directory(dir: &Path) -> Result<usize, String>`
      - `register_plugin(&mut self, manifest: FilePluginManifest)`
      - `find_matching_plugins(&self, file_path: &Path, trigger: &PluginTrigger) -> Vec<&FilePluginManifest>`
      - `execute_plugin(&self, plugin_id: &str, ctx: &PluginExecutionContext) -> Result<PluginExecutionResult, String>`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing manifest parsing, trigger matching by file extension, execution context assembly, and mock plugin execution with 100% test coverage.
- **File Target**: `crates/swal-files/src/plugin_system.rs`

## Web Research Required
1. search: "rust file manager plugin manifest json architecture"
2. search: "rust context menu action plugin registry"
3. search: "declarative file transformation plugin hook"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "FilePluginRegistry" crates/swal-files/src/plugin_system.rs` >= 1 match
- [ ] `grep -rn "FilePluginManifest" crates/swal-files/src/plugin_system.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/plugin_system.rs` | Non-existent | [NEW] Agentic file tool plugin system with 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/archive.rs` and `crates/swal-files/src/agent.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 9
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
