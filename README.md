# ⚡ SWAL Desktop

### NixOS + Hyprland — AI Workspace Environment

<p align="center">
  <a href="https://nixos.org">
    <img src="https://img.shields.io/badge/NixOS-25.05-5277C3?style=for-the-badge&logo=nixos&logoColor=white" alt="NixOS"/>
  </a>
  <a href="https://hyprland.org">
    <img src="https://img.shields.io/badge/Hyprland-v0.49-00ff88?style=for-the-badge&logo=hyprland&logoColor=white" alt="Hyprland"/>
  </a>
  <a href="https://github.com/iberi22/swal-desktop">
    <img src="https://img.shields.io/github/stars/iberi22/swal-desktop?style=for-the-badge&color=ffd700" alt="Stars"/>
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/iberi22/swal-desktop?style=for-the-badge&color=bb9af7" alt="License"/>
  </a>
</p>

---

> **"⚡ Ejecuto, automatizo, resuelvo"** — SouthWest AI Labs

Desktop environment personalizado para el laboratorio de IA. Incluye:
- **4 Workspaces preconfigurados** (Dev, Web, AI Canvas, Dock)
- **Hyprland** como window manager (Wayland)
- **Dank Material Shell** con branding SWAL (#00ff88 verde + caoba)
- **EWW Widgets** custom en CSS/JS
- **AI Canvas** — Lienzo interactivo para agentes IA
- **Tauri + Svelte** app para rendering de componentes

---

## 🚀 Quick Start

### Prerrequisitos (Windows)
- [QEMU](https://scoop.sh/) → `scoop install qemu`
- [Git](https://git-scm.com/)
- 30GB espacio libre

### Instalación en 3 Pasos

```powershell
# 1. Clonar el repo
git clone https://github.com/iberi22/swal-desktop ~/swal-desktop
cd ~/swal-desktop

# 2. Ejecutar el launcher
.\swal-nixos.ps1 -DownloadISO

# 3. Iniciar y configurar
.\swal-nixos.ps1 -CreateVM
```

### Dentro de NixOS (después de instalar)
```bash
# Conectar SSH
ssh -p 2222 bela@localhost

# Aplicar configuración SWAL
sudo nixos-rebuild switch --flake .#swal
```

---

## 📸 Gallery

| Workspace | Descripción |
|-----------|-------------|
| **WS 1: Dev** | Terminal + Neovim + Firefox |
| **WS 2: Web** | Browser fullscreen |
| **WS 3: AI Canvas** | Tauri app interactiva |
| **WS 4: Dock** | Widgets + Dank Material Shell |

---

## 🎨 Branding SWAL

| Elemento | Valor |
|----------|-------|
| Neon Green | `#00FF88` |
| Deep Blue | `#7DCFFF` |
| Purple | `#BB9AF7` |
| Caoba | `#9D4A14` |
| Background | `#0D1117` |

---

## 📦 Estructura

```
swal-desktop/
├── swal-nixos.ps1        # Launcher principal
├── README.md
├── flake.nix              # NixOS flake
├── nixos/
│   ├── configuration.nix  # Config principal
│   └── hardware.nix       # Hardware auto-generado
├── hypr/
│   └── hyprland.conf      # Hyprland config
├── themes/
│   └── swal-cyber.json    # Theme Dank Material Shell
├── eww/
│   └── def.nix            # EWW widgets
└── docs/
    ├── SETUP.md
    ├── ARCHITECTURE.md
    └── TROUBLESHOOTING.md
```

---

## 🔧 Componentes

| Componente | Tecnología | Estado |
|------------|------------|--------|
| Window Manager | Hyprland | ✅ Listo |
| Desktop Shell | Dank Material Shell | 🔄 En desarrollo |
| Widgets | EWW (CSS/JS) | 🔄 En desarrollo |
| AI Canvas | Tauri + Svelte | 🔄 En desarrollo |
| Theme | Custom SWAL | ✅ Listo |

---

## 🤝 Contribuir

1. Fork → `iberi22/swal-desktop`
2. Crear branch: `git checkout -b feature/mi-feature`
3. Commit → `git push`
4. Open PR

---

## 📄 Licencia

MIT — SouthWest AI Labs

---

*Repo: github.com/iberi22/swal-desktop*
