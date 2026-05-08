# ⚡ SWAL Desktop — Guía de Setup

> Setup completo para NixOS + Hyprland en QEMU desde Windows

---

## 📋 Índice

1. [Prerrequisitos](#-prerrequisitos)
2. [Descarga e Instalación](#-descarga-e-instalación)
3. [Post-Instalación](#-post-instalación)
4. [Configuración Hyprland](#-configuración-hyprland)
5. [Solución de Problemas](#-solución-de-problemas)

---

## 🔧 Prerrequisitos

### Windows
```powershell
# Instalar QEMU via scoop
scoop install qemu

# Verificar instalación
qemu-system-x86_64 --version
```

### Recursos
- 40GB espacio disco
- 8GB RAM mínimo
- 4 CPUs disponibles

---

## 📥 Descarga e Instalación

### Paso 1: Descargar ISO
```powershell
cd ~/swal-desktop
.\swal-nixos.ps1 -DownloadISO
```

### Paso 2: Crear VM
```powershell
.\swal-nixos.ps1 -CreateVM
```

Esto:
- Crea imagen QCOW2 en `~/qemu/swal-nixos.qcow2`
- Inicia QEMU con NixOS
- SSH disponible en `localhost:2222`

### Paso 3: Instalar NixOS
1. Selecciona "NixOS install" en GRUB
2. Espera a que cargue la terminal
3. Conecta SSH:
```powershell
.\swal-nixos.ps1 -SSH
```
4. Ejecuta el script de instalación:
```bash
sudo bash /tmp/nixos-setup.sh
```

---

## 🔄 Post-Instalación

### Conectar por SSH
```powershell
.\swal-nixos.ps1 -SSH
```
**Usuario:** `bela`
**Password:** `swal123`

### Rebuild con Flake
```bash
# Ir al repo
cd ~/swal-desktop

# Rebuild
sudo nixos-rebuild switch --flake .#swal
```

### Verificar Hyprland
```bash
# Verificar que greeter está activo
systemctl status greetd

# Si hay errores
journalctl -xe | grep hyprland
```

---

## 🎨 Configuración Hyprland

### Copiar config local
```bash
mkdir -p ~/.config/hypr
cp ~/swal-desktop/hypr/hyprland.conf ~/.config/hypr/
```

### Reiniciar Hyprland
```bash
hyprctl reload
```

---

## 🐛 Solución de Problemas

### Hyprland no muestra UI
```bash
# Ver logs
journalctlxe | grep Hyprland

# Verificar GPU
lspci | grep -i vga

# Software rendering
export WLR_RENDERER_ALLOW_SOFTWARE=1
hyprctl reload
```

### SSH no conecta
```bash
# Verificar SSH
systemctl status sshd

# Ver puertos
ss -tlnp | grep 22
```

### Error de partición
```bash
# Ver discos
lsblk

# Re-particionar manualmente
sudo fdisk /dev/vda
```

---

## 📞 Comandos Útiles

| Comando | Descripción |
|---------|-------------|
| `systemctl status greetd` | Estado de greetd |
| `hyprctl reload` | Recargar Hyprland |
| `nixos-rebuild switch` | Aplicar config |
| `journalctl -xe` | Ver logs |
| `ip addr` | Ver IP |

---

## 🔗 Links

- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [Hyprland Wiki](https://wiki.hypr.land/)
- [Repo SWAL](https://github.com/iberi22/swal-desktop)

---

*SouthWest AI Labs ⚡*
