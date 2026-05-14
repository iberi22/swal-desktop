# ⚡ SWAL Desktop — Architecture

> System Architecture for the NixOS + Hyprland AI Workspace (SWAL).

---

## 🏗️ Technology Stack

```text
┌─────────────────────────────────────────────────────────────┐
│                     NIXOS 25.05                            │
│              (Declarative Configuration)                   │
├─────────────────────────────────────────────────────────────┤
│                    HYPRLAND                                │
│              (Wayland Compositor - C++/Rust)               │
├─────────────┬─────────────┬─────────────┬──────────────────┤
│ Workspace 1  │ Workspace 2  │ Workspace 3  │ Workspace 4     │
│   (Dev)      │   (Web)     │  (AI Canvas) │  (Dock)        │
├─────────────┴─────────────┴─────────────┴──────────────────┤
│                     EWW Widgets                             │
│                   (Yuck + SCSS Custom)                     │
├─────────────────────────────────────────────────────────────┤
│              AI AGENT ECOSYSTEM                            │
│           (Hermes, Gemini, Codex, Node.js)                 │
├─────────────────────────────────────────────────────────────┤
│              TAURI AI CANVAS APP                           │
│             (Rust Backend + Svelte UI)                     │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Core Components

### NixOS (Foundation)
- **Package Management:** Nix Flakes for reproducible builds.
- **System Logic:** `configuration.nix` (Declarative OS definition).
- **Home Manager:** `home.nix` for user-specific configs (Dotfiles).

### Hyprland (Window Manager)
- **Compositor:** High-performance Wayland compositor.
- **Workflow:** Dynamic tiling with 4 persistent, labeled workspaces.
- **Integration:** Custom window rules to pin AI tools to specific workspaces.

### EWW (System Widgets)
- **UI:** Custom Dashboard written in Yuck/SCSS.
- **Monitoring:** Real-time CPU, RAM, and Disk metrics.
- **AI Integration:** Visual status indicators for active AI agents.

### AI Agent Ecosystem
- **Local Agents:** Integrated CLI tools for Gemini and Codex.
- **Remote Agents:** Hermes Agent integration with DeepSeek V4 API support.
- **Development:** Built-in Node.js 22 and pnpm for fast agentic prototyping.

---

## 🔧 File Structure

```text
swal-desktop/
├── flake.nix              # Flake entry point & inputs
├── nixos/
│   ├── configuration.nix  # System-wide configuration
│   ├── ai-agents.nix      # Agent-specific module
│   ├── home.nix           # User-space / Dotfiles
│   └── hardware.nix       # Auto-generated hardware config
├── hypr/
│   ├── hyprland.conf      # WM layout & binds
│   └── waybar/            # Status bar configuration
├── eww/
│   ├── eww.yuck           # Widget definitions
│   ├── eww.scss           # Widget styling
│   └── scripts/           # Logic scripts (Vol, Brightness, AI)
└── canvas/                # AI Canvas source code (Tauri)
```

---

## 🌐 Networking & Ports

| Port | Service | Usage |
|------|---------|-------|
| 22 | SSH | Remote management |
| 18789 | OpenClaw | Gateway for AI agents |
| 8006 | Xavier | Memory system API |
| 3000 | AI Canvas | Frontend development port |

---

## 🎨 Branding Guidelines

### Color Palette
- **Neon Green (`#00FF88`)**: Primary accent, active states.
- **Deep Blue (`#7DCFFF`)**: Secondary accent, labels.
- **Purple (`#BB9AF7`)**: Decoration, tertiary actions.
- **Background (`#0D1117`)**: Deep space matte for focus.

### Typography
- **Monospace:** `FiraCode Nerd Font` (with ligatures).
- **UI Text:** `Inter` or `Noto Sans`.

---

## 📊 Component Status

| Component | Status | Notes |
|------------|--------|-------|
| NixOS Base | ✅ Stable | Using 25.05 stable branch |
| Hyprland | ✅ Ready | Custom rules & workspaces enabled |
| EWW Widgets | ✅ Ready | Fully functional Dashboard |
| AI Agents | ✅ Ready | Integrated Hermes/Gemini/Codex |
| AI Canvas | 🔄 Beta | Foundation created (Tauri+Svelte) |
| Install Script| ✅ Verified | Dynamic hardware generation ready |

---

## 🔗 References

- [NixOS Documentation](https://nixos.org/learn)
- [Hyprland Wiki](https://wiki.hypr.land/)
- [EWW Documentation](https://elkowar.github.io/eww/)
- [SouthWest AI Labs](https://github.com/iberi22)

---

*SouthWest AI Labs ⚡*
