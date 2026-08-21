# SWAL Desktop — Long-Term Development Planning

## Scope & Architectural Vision
Build a blazing fast, zero-overhead desktop environment for SouthWest AI Labs that acts as an autonomous node connected to Xavier Memory Core and Edge-Mesh P2P network with declarative agentic generative UI.

## Wave Breakdown
- **Wave 1 (Completed)**: Theme Engine (`swal-theme`), `@swal/ui` Hive Dark, Cyber Neon, SWAL Doctor, Settings Modal, NixOS Node services.
- **Wave 2 (Active)**: Native Rust Core (`crates/`), `swal-telemetry-rs` /proc reader, `swal-a2ui-engine` JSON UI compiler, Widget Vault.
- **Wave 3 (Upcoming)**: GLSL Ambient Orb Voice/Thought Visualizer, full Wayland Layer Shell composition.

## Guidelines
- Follow GitCore v3.8.0 rules.
- Strictly disjoint file islands across all concurrent Jules agent branches.
