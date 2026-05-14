# ⚡ SWAL Desktop: Cyber Edition

### NixOS + Hyprland — The Ultimate AI Workspace Environment

<p align="center">
  <a href="https://nixos.org">
    <img src="https://img.shields.io/badge/NixOS-25.05-5277C3?style=for-the-badge&logo=nixos&logoColor=white" alt="NixOS"/>
  </a>
  <a href="https://hyprland.org">
    <img src="https://img.shields.io/badge/Hyprland-v0.50-00ff88?style=for-the-badge&logo=hyprland&logoColor=white" alt="Hyprland"/>
  </a>
  <a href="https://github.com/iberi22/swal-desktop">
    <img src="https://img.shields.io/github/stars/iberi22/swal-desktop?style=for-the-badge&color=ffd700" alt="Stars"/>
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/iberi22/swal-desktop?style=for-the-badge&color=bb9af7" alt="License"/>
  </a>
</p>

---

> **"⚡ Execute, Automate, Resolve"** — SouthWest AI Labs

**SWAL Desktop** is a highly optimized, modular NixOS configuration designed for AI researchers, developers, and autonomous agent ecosystems. It provides a premium, "Cyberpunk" aesthetic paired with a robust declarative foundation.

## 🚀 Key Features

- **4 Preconfigured Workspaces**: Optimized workflow for Dev, Web, AI Canvas, and Docking.
- **Hyprland WM**: Ultra-fast, tiled window management with custom animations.
- **EWW Dashboard**: A premium, CSS/JS-powered system dashboard (SUPER + D).
- **AI Agent Ecosystem**: Native support for **Hermes**, **Gemini**, **Codex**, and **OpenCode**.
- **AI Canvas**: An interactive Tauri-based canvas for real-time agent orchestration.
- **Dank Material Shell**: Branded UI shell with custom colors and layout.
- **Verified Stability**: Audited and tested via Docker/Nix-VM (12+ critical bugs resolved).

---

## 🛠️ One-Line Installation

Deploy the full environment on any clean NixOS system with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/iberi22/swal-desktop/main/scripts/install.sh | bash
```

*For more detailed instructions, see [SETUP.md](docs/SETUP.md).*

---

## 🤖 AI Agent Integration

This desktop is built from the ground up to support the **SouthWest AI Labs** ecosystem:
- **Hermes Agent**: Autonomous research and execution agent.
- **Gemini & Codex CLI**: Built-in CLI agents for code generation and task management.
- **Node.js 22 + pnpm**: Modern stack for running agentic toolchains.
- **DeepSeek V4 Support**: Configured for high-performance inference via OpenCode.

To onboard your agents after installation:
```bash
bash scripts/hermes-onboarding.sh
```

---

## 🎨 Branding & Aesthetics

| Element | Hex Code | Description |
|----------|----------|-------------|
| **Neon Green** | `#00FF88` | SWAL Primary Accent |
| **Deep Blue** | `#7DCFFF` | System & Information |
| **Purple** | `#BB9AF7` | Secondary Accent |
| **Mahogany** | `#9D4A14` | Shell Details |
| **Background** | `#0D1117` | Deep Space Matte |

---

## 📦 Project Structure

```text
swal-desktop/
├── flake.nix              # Main Nix Flake (Stable 25.05)
├── nixos/
│   ├── configuration.nix  # Core system configuration
│   ├── ai-agents.nix      # Agent environment module
│   └── home.nix           # Home Manager (User space)
├── hypr/
│   └── hyprland.conf      # Window manager rules & binds
├── eww/
│   ├── eww.yuck           # Widget definitions
│   └── eww.scss           # Custom Cyber styling
└── scripts/               # Automation & Onboarding
```

---

## 🔧 Verified Components

| Component | Status | Details |
|------------|------------|---------|
| **Window Manager** | ✅ Stable | Hyprland + Plugins |
| **Dashboard** | ✅ Verified | EWW Modular Dashboard |
| **Package Set** | ✅ Validated | NixOS 25.05 (Unstable for AI) |
| **AI Canvas** | 🚧 Beta | Tauri + Svelte Foundation |
| **Docker Test** | ✅ Passed | Full system evaluation successful |

---

## 🤝 Contributing

We welcome contributions to the SWAL ecosystem!
1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/cool-new-thing`).
3. Commit your changes.
4. Push to the branch and open a Pull Request.

---

## 📄 License

Distributed under the **MIT License**. Created with ⚡ by **SouthWest AI Labs**.

*Repository: [github.com/iberi22/swal-desktop](https://github.com/iberi22/swal-desktop)*
