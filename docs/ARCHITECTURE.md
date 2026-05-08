# ⚡ SWAL Desktop — Arquitectura

> Arquitectura del sistema NixOS + Hyprland para SWAL

---

## 🏗️ Stack Tecnológico

```
┌─────────────────────────────────────────────────────────────┐
│                     NIXOS 25.05                            │
│              (Declarative Configuration)                   │
├─────────────────────────────────────────────────────────────┤
│                    HYPRLAND                                │
│              (Wayland Compositor - Rust)                   │
├─────────────┬─────────────┬─────────────┬──────────────────┤
│ Workspace 1  │ Workspace 2  │ Workspace 3  │ Workspace 4     │
│   (Dev)      │   (Web)     │  (AI Canvas) │  (Dock)        │
├─────────────┴─────────────┴─────────────┴──────────────────┤
│                     EWW Widgets                             │
│                   (CSS/JS Custom)                          │
├─────────────────────────────────────────────────────────────┤
│              DANK MATERIAL SHELL                           │
│           (Quickshell Desktop Shell)                       │
├─────────────────────────────────────────────────────────────┤
│              TAURI AI CANVAS APP                           │
│             (Rust + Svelte)                                │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Componentes

### NixOS (Base)
- **Gestión de paquetes:** Nix flakes
- **Configuración:** `configuration.nix` (declarativo)
- **Boot:** systemd-boot (EFI)

### Hyprland (Window Manager)
- **Tipo:** Wayland compositor
- **Lenguaje:** Rust
- **Plugins:** hy3, hyprexpo, hyprbars
- **Config:** `hyprland.conf`

### Dank Material Shell (Desktop Shell)
- **Base:** Quickshell
- **Themes:** JSON + matugen
- **Reemplaza:** Waybar, rofi, dunst

### EWW (Widgets)
- **Lenguaje:** Yuck (config) + CSS (styling)
- **Performance:** Rust (compilado)
- **Uso:** Clock, system stats, workspace indicators

### Tauri AI Canvas
- **Backend:** Rust
- **Frontend:** Svelte
- **Audio:** Whisper (Groq API)
- **Agent:** Xavier2 (localhost:8006)

---

## 🔧 Estructura de Archivos

```
swal-desktop/
├── README.md              # Este archivo
├── swal-nixos.ps1         # Launcher PowerShell
├── flake.nix              # Nix flake entry
│
├── nixos/
│   ├── configuration.nix  # Config principal
│   └── hardware.nix       # Hardware config
│
├── hypr/
│   └── hyprland.conf      # Hyprland config
│
├── themes/
│   └── swal-cyber.json    # Theme Dank Material Shell
│
├── eww/
│   └── def.nix            # EWW widgets config
│
└── docs/
    ├── SETUP.md           # Guía de setup
    ├── ARCHITECTURE.md    # Este archivo
    └── TROUBLESHOOTING.md # Solución de problemas
```

---

## 🌐 Red y Conectividad

### Puertos
| Puerto | Servicio | Uso |
|--------|----------|-----|
| 22 | SSH | Acceso remoto |
| 2222 | SSH (host) | Forward desde Windows |
| 5900 | VNC | Acceso gráfico (opcional) |
| 8006 | Xavier2 | Sistema de memoria |
| 3000 | AI Canvas | Web interface |

### Configuración
- NetworkManager para WiFi/Ethernet
- SSH con key pública (no password)
- Firewall permite 22, 80, 443, 2222

---

## 🎨 Branding SWAL

### Colores
```css
:root {
  --swal-green: #00FF88;
  --swal-blue: #7DCFFF;
  --swal-purple: #BB9AF7;
  --swal-caoba: #9D4A14;
  --swal-bg: #0D1117;
  --swal-surface: #161B22;
}
```

### Tipografía
- **Code:** FiraCode (ligatures)
- **UI:** Noto Sans

---

## 🔄 Pipeline de Desarrollo

```
1. Editar archivos en ~/swal-desktop
   ↓
2. Push a GitHub (git push)
   ↓
3. En NixOS: git pull
   ↓
4. Rebuild: sudo nixos-rebuild switch --flake .
   ↓
5. Verificar cambios en Hyprland
```

---

## 📊 Estados del Sistema

| Componente | Estado | Notas |
|------------|--------|-------|
| NixOS base | ✅ Listo | 25.05 |
| Hyprland | ✅ Listo | Config completo |
| Hy3 (tiling) | 🔄 Por probar | Requiere hyprpm |
| Hyprexpo | 🔄 Por probar | Requiere hyprpm |
| Dank Material Shell | 🔄 Por instalar | Requiere Quickshell |
| EWW Widgets | 🔄 Por crear | Diseños listos |
| AI Canvas | 🔄 Por crear | Tauri + Svelte |

---

## 🔗 Referencias

- [Hyprland](https://hyprland.org/)
- [EWW](https://elkowar.github.io/eww/)
- [Dank Material Shell](https://danklinux.com/docs/dankmaterialshell/)
- [Tauri](https://tauri.app/)
- [NixOS](https://nixos.org/)

---

*SouthWest AI Labs ⚡*
