# ⚡ SWAL Desktop — Setup Guide

> Comprehensive guide for deploying the SWAL Desktop environment (NixOS + Hyprland).

---

## 📋 Table of Contents

1. [Prerequisites](#-prerequisites)
2. [Installation](#-installation)
3. [Post-Installation](#-post-installation)
4. [Agent Configuration](#-agent-configuration)
5. [Troubleshooting](#-troubleshooting)

---

## 🔧 Prerequisites

### Hardware Requirements
- **Disk Space:** 40GB minimum (SSD recommended).
- **RAM:** 8GB minimum (16GB+ recommended for AI agents).
- **GPU:** Integrated or dedicated (Nvidia/AMD/Intel). Wayland support is required.

### Network
- A stable internet connection is required to fetch Nix flakes and packages.

---

## 📥 Installation

### 1. Bare Metal Installation (Recommended)
If you already have a basic NixOS installation or are running from the installer environment:

```bash
curl -sSL https://raw.githubusercontent.com/iberi22/swal-desktop/main/scripts/install.sh | bash
```

**What this script does:**
1. Detects your hardware and generates `hardware.nix`.
2. Clones the `swal-desktop` repository.
3. Configures Nix flakes and Home Manager.
4. Performs a `nixos-rebuild switch`.

### 2. VM Installation (Testing)
If you are on Windows and want to test the environment using QEMU:

```powershell
# In PowerShell (with QEMU installed)
cd ~/swal-desktop
.\swal-nixos.ps1 -DownloadISO
.\swal-nixos.ps1 -CreateVM
```

---

## 🔄 Post-Installation

### Default Credentials
- **User:** `bela`
- **Password:** `swal123` (Change this immediately using `passwd`)
- **Keybinds:**
    - `SUPER + D`: Toggle EWW Dashboard
    - `SUPER + Enter`: Open Terminal (Kitty)
    - `SUPER + Q`: Close Window
    - `SUPER + M`: Exit Hyprland

### Applying Changes
Whenever you modify the configuration in `~/swal-desktop`:
```bash
cd ~/swal-desktop
sudo nixos-rebuild switch --flake .#swal
```

---

## 🤖 Agent Configuration

To initialize the AI ecosystem (Hermes, Gemini, Codex):

1. **Onboarding Script:**
   ```bash
   bash scripts/hermes-onboarding.sh
   ```
2. **API Keys:**
   Define your API keys in `nixos/ai-agents.nix` or export them in your shell:
   - `OPENCODE_API_KEY`
   - `GEMINI_API_KEY`
   - `DEEPSEEK_API_KEY`

---

## 🐛 Troubleshooting

### Hyprland Fails to Start
- **Check logs:** `journalctl -xe | grep Hyprland`
- **GPU Drivers:** Ensure `hardware.opengl.enable = true` is set in `configuration.nix`.
- **Software Rendering:** If on a VM without 3D acceleration:
  `export WLR_RENDERER_ALLOW_SOFTWARE=1`

### Package Collisions
If you see errors regarding `gh` or `rustup`, ensure you haven't manually added them to `environment.systemPackages` as they are handled by the `ai-agents.nix` module.

### EWW Dashboard Errors
Run `eww logs` to see real-time error reporting for widgets.

---

## 📞 Useful Commands

| Command | Description |
|---------|-------------|
| `systemctl status greetd` | Check Display Manager status |
| `hyprctl reload` | Reload Hyprland configuration |
| `nix flake check` | Verify flake syntax |
| `nix-collect-garbage -d` | Clean old Nix generations |

---

## 🔗 Links

- [NixOS Official Manual](https://nixos.org/manual/nixos/stable/)
- [Hyprland Wiki](https://wiki.hypr.land/)
- [SWAL GitHub Repository](https://github.com/iberi22/swal-desktop)

---

*Created by SouthWest AI Labs ⚡*
