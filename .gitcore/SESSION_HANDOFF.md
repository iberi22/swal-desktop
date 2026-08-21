# SWAL Desktop — Canonical Session State & Agent Handoff

## 1. Project & Node Identification
- **Project**: `swal-desktop` (Periferia / SWAL Autonomous Node Shell)
- **Repo Remote**: `https://github.com/iberi22/swal-desktop.git` (branch `main`)
- **Protocol Version**: GitCore v3.8.0
- **Last Sync Timestamp**: 2026-08-20T23:15:00-05:00

---

## 2. Completed Milestones (Wave 1: 100% Ready)
- **Dynamic Theme Engine (`swal-theme`)**:
  - Full support for `@swal/ui` Hive Dark (`hive-dark.json`), Cyber Neon (`cyber-neon.json`), and Nord Frost (`nord-swal.json`).
  - Interactive Rofi Theme Picker (`swal-theme picker`).
  - Agent Skill created: `.agents/skills/swal-theme-creator/SKILL.md`.
- **System Doctor & Linter (`swal-doctor`)**:
  - 5-point verification with auto-fix (`swal-doctor --fix`).
  - 100% clean ASCII SCSS without `@charset` GTK3 bugs.
- **Interactive Manual Settings Panel (`swal_settings`)**:
  - Accessible via the orange Gear button (`⚙`) in the dashboard header.
  - Snug 320px layout with real-time design token swatch previews.
- **NixOS SWAL Node Module**:
  - `nixos/swal-node.nix` configuring `xavier-core` (`:8006` / `:8100`) and `edge-mesh` P2P services.

---

## 3. Active Wave (Wave 2: Rust Native Core & Generative A2UI)
- **Target Refresh Rate**: 200 Hz - 240 Hz (<5.0ms frame budget via `swal-render-pipeline`).
- **Cargo Workspace**: `Cargo.toml` at repository root.
- **Disjoint File Islands & Active Issues**:
  - `SWAL-11` (Issue #11): `crates/swal-telemetry-rs/` (Native /proc reader & Unix socket IPC).
  - `SWAL-12` (Issue #12): `crates/swal-a2ui-engine/` (Declarative JSON AST compiler).
  - `SWAL-13` (Issue #13): `crates/swal-ambient-orb/` (GLSL Shader Voice/Audio reactive surface).
  - `SWAL-14` (Issue #14): `crates/swal-node-daemon/` (Async Tokio coordinator & Xavier bridge).
  - `SWAL-15` (Issue #15): `crates/swal-widget-vault/` (Agent widget persistence & inventory manager).

---

## 4. Immediate Resumption Instructions for Any Incoming Agent
1. **Verification**:
   ```bash
   cd ~/proyectosSWAL/periferia/swal-desktop
   cargo test               # 3/3 crates tests passing in 0.00s
   swal-doctor              # 5/5 system checks passing with 0 errors
   ```
2. **Dispatch to Jules**:
   ```bash
   # Create issues on GitHub from .hermes/ola2/body-*.md
   gh issue create --title "[Ola 2.01] feat-swal-11..." --body-file .hermes/ola2/body-11.md --label "ola2,wave-2"
   # Once reviewed -> dispatch
   gh issue edit <ID> --add-label jules
   ```
3. **Monitoring & Merge**:
   - Monitor via `gh pr list --author iberi22 --state open`.
   - Merge PRs in wave order (1 -> 2 -> 3 -> 4 -> 5).
   - Reconcile `.gitcore/features.json` at the end of Wave 2.
