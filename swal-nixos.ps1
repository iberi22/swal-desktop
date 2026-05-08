# ═══════════════════════════════════════════════════════════════════════════════
# ⚡ SWAL NixOS Launcher — SouthWest AI Labs
# NixOS + Hyprland AI Workspace Environment
# 
# Uso:
#   .\swal-nixos.ps1 -DownloadISO      Descargar ISO
#   .\swal-nixos.ps1 -CreateVM         Crear imagen e iniciar VM
#   .\swal-nixos.ps1 -SSH              Conectar SSH
#   .\swal-nixos.ps1 -FullInstall      Instalación automática completa
# ═══════════════════════════════════════════════════════════════════════════════

param(
    [switch]$DownloadISO,
    [switch]$CreateVM,
    [switch]$SSH,
    [switch]$FullInstall,
    [switch]$Status,
    [string]$DiskPath = "$HOME\qemu\swal-nixos.qcow2",
    [string]$ISOPath = "$HOME\qemu\nixos-minimal-25.05.iso",
    [int]$DiskSizeGB = 40,
    [int]$CPUs = 4,
    [int]$MemoryMB = 8192,
    [int]$SSHPort = 2222
)

$ErrorActionPreference = "Stop"
$QEMU_DIR = "$HOME\qemu"
$QEMU_BIN = "C:\Users\belal\scoop\apps\qemu\current"
$REPO_DIR = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$VBoxManage = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"

# ─── Helpers ──────────────────────────────────────────────────────────────────

function Write-Banner {
    param([string]$Title)
    Write-Host ""
    Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  ⚡ $Title" -ForegroundColor White
    Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
}

function New-Directory {
    param([string]$Path)
    if (!(Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Get-NixOSISO {
    # NixOS 25.05 minimal - versión estable
    $version = "25.05.813558.d2b1213bf5ec"
    $url = "https://releases.nixos.org/nixos/25.05/nixos-$version/nixos-minimal-$version-x86_64-linux.iso"
    
    New-Directory $QEMU_DIR
    
    if (Test-Path $ISOPath) {
        $size = (Get-Item $ISOPath).Length / 1MB
        Write-Host "  [OK] NixOS ISO existe: $("{0:N1}MB" -f $size)" -ForegroundColor Green
        return $ISOPath
    }
    
    Write-Host "  [DESCARGANDO] NixOS 25.05 minimal ISO..." -ForegroundColor Cyan
    Write-Host "  URL: $url" -ForegroundColor Gray
    Write-Host "  Guardando en: $ISOPath" -ForegroundColor Gray
    Write-Host ""
    
    try {
        (New-Object System.Net.WebClient).DownloadFile($url, $ISOPath)
        $size = (Get-Item $ISOPath).Length / 1MB
        Write-Host "  [OK] ISO descargado: $("{0:N1}MB" -f $size)" -ForegroundColor Green
        return $ISOPath
    }
    catch {
        Write-Host "  [ERROR] Fallo descarga: $_" -ForegroundColor Red
        Write-Host "  [INFO] Descarga manual: $url" -ForegroundColor Yellow
        return $null
    }
}

function Initialize-DiskImage {
    param([int]$SizeGB)
    
    New-Directory (Split-Path $DiskPath -Parent)
    
    if (Test-Path $DiskPath) {
        $size = (Get-Item $DiskPath).Length / 1GB
        Write-Host "  [OK] Disco existe: $DiskPath ($("{0:N1}GB" -f $size))" -ForegroundColor Green
        return $DiskPath
    }
    
    Write-Host "  [CREANDO] Imagen QCOW2 de ${SizeGB}GB..." -ForegroundColor Yellow
    $qemuImg = Join-Path $QEMU_BIN "qemu-img.exe"
    
    & $qemuImg create -f qcow2 $DiskPath "${SizeGB}G"
    if ($LASTEXITCODE -ne 0) { throw "Error creando disco" }
    
    Write-Host "  [OK] Disco creado: $DiskPath" -ForegroundColor Green
    return $DiskPath
}

function Find-QEMU {
    $qemuSystem = Join-Path $QEMU_BIN "qemu-system-x86_64.exe"
    if (!(Test-Path $qemuSystem)) {
        throw "QEMU no encontrado en $QEMU_BIN. Instala con: scoop install qemu"
    }
    return $qemuSystem
}

function Start-NixOSVM {
    param([switch]$InstallerMode)
    
    $iso = Get-NixOSISO
    if (!$iso) { throw "ISO requerido. Ejecuta con -DownloadISO primero." }
    
    $disk = Initialize-DiskImage -SizeGB $DiskSizeGB
    $qemuSystem = Find-QEMU
    
    Write-Banner "SWAL NixOS VM — Iniciando..."
    Write-Host "  CPU: $CPUs cores | RAM: ${MemoryMB}MB" -ForegroundColor Gray
    Write-Host "  Disco: $disk" -ForegroundColor Gray
    Write-Host "  ISO: $iso" -ForegroundColor Gray
    Write-Host "  SSH: localhost:$SSHPort" -ForegroundColor Cyan
    Write-Host ""
    
    # Parámetros QEMU optimizados para NixOS + Hyprland
    $qemuArgs = @(
        "-name SWAL-NixOS",
        "-m ${MemoryMB}",
        "-smp $CPUs",
        "-enable-kvm",
        "-cpu host",
        "-machine q35",
        
        # GPU virtual con aceleración 3D
        "-device virtio-vga-gl",
        "-render-gl es",
        "-display gtk,gl=on",
        
        # Audio
        "-device intel-hda",
        "-device hda-output",
        
        # Red: SSH forwarding
        "-netdev user,id=net0,hostfwd=tcp::${SSHPort}-:22",
        "-device virtio-net-pci,netdev=net0",
        
        # Discos
        "-drive file=`"$iso`",media=cdrom,index=0",
        "-drive file=`"$DiskPath`",media=disk,index=1,if=virtio",
        
        # Input
        "-usb",
        "-device usb-tablet",
        
        # Boot
        "-boot order=dc"
    )
    
    Write-Host "  [INFO] Ejecutando QEMU..." -ForegroundColor Cyan
    Write-Host ""
    
    $proc = Start-Process -FilePath $qemuSystem -ArgumentList $qemuArgs -PassThru -NoNewWindow
    
    Write-Host "  [OK] VM iniciada (PID: $($proc.Id))" -ForegroundColor Green
    Write-Host ""
    
    if ($InstallerMode) {
        Write-Host "  SIGUE ESTOS PASOS:" -ForegroundColor Yellow
        Write-Host "  1. Selecciona 'NixOS install' en GRUB" -ForegroundColor White
        Write-Host "  2. Espera a que termine de cargar" -ForegroundColor White
        Write-Host "  3. Conecta: .\swal-nixos.ps1 -SSH" -ForegroundColor White
        Write-Host ""
    }
    
    return $proc
}

function Connect-SSH {
    Write-Host "  [SSH] Conectando a NixOS..." -ForegroundColor Cyan
    Write-Host "  [CMD] ssh -p $SSHPort bela@localhost" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  Credenciales:" -ForegroundColor Yellow
    Write-Host "    Usuario: bela" -ForegroundColor Gray
    Write-Host "    Password: swal123" -ForegroundColor Gray
    Write-Host ""
    
    $sshArgs = @(
        "-o StrictHostKeyChecking=no",
        "-o UserKnownHostsFile=$env:USERPROFILE\.ssh\known_hosts",
        "-p $SSHPort",
        "bela@localhost"
    )
    
    & ssh $sshArgs
}

function Get-VMStatus {
    Write-Banner "VM Status"
    
    $diskExists = Test-Path $DiskPath
    $isoExists = Test-Path $ISOPath
    
    Write-Host "  [ISO] NixOS 25.05     $($isoExists ? '✅' : '❌')" -ForegroundColor ($isoExists ? 'Green' : 'Red')
    Write-Host "  [DISK] Imagen QCOW2   $($diskExists ? '✅' : '❌')" -ForegroundColor ($diskExists ? 'Green' : 'Red')
    
    if ($diskExists) {
        $size = (Get-Item $DiskPath).Length / 1GB
        Write-Host "  [SIZE] Tamaño disco    $("{0:N1}GB" -f $size)" -ForegroundColor Gray
    }
    
    Write-Host "  [SSH] Puerto          localhost:$SSHPort" -ForegroundColor Gray
    Write-Host ""
}

function Install-NixOS {
    # Script de auto-instalación para ejecutar dentro de NixOS
    Write-Banner "Generando Script de Instalación"
    
    $setupScript = @"
#!/run/current-system/sw/bin/bash
set -e

echo "══════════════════════════════════════════════════════════"
echo "  ⚡ SWAL NixOS Installer"
echo "══════════════════════════════════════════════════════════"

DEVICE="/dev/vda"

echo "[1/7] Partitioning $DEVICE..."
parted $DEVICE --script mklabel gpt
parted $DEVICE --script mkpart boot fat32 1MiB 512MiB
parted $DEVICE --script mkpart root ext4 512MiB 100%
parted $DEVICE --script set 1 boot on

echo "[2/7] Formatting..."
mkfs.fat -F 32 /dev/vda1
mkfs.ext4 -F /dev/vda2

echo "[3/7] Mounting..."
mount /dev/vda2 /mnt
mkdir -p /mnt/boot
mount /dev/vda1 /mnt/boot

echo "[4/7] Generating NixOS config..."
nixos-generate-config --root /mnt

echo "[5/7] Applying SWAL configuration..."
# Copiar configuration.nix
cp /tmp/configuration.nix /mnt/etc/nixos/configuration.nix

echo "[6/7] Installing NixOS (puede tardar 10-20 minutos)..."
nixos-install --no-root-password --flake /mnt#swal

echo "[7/7] ¡Listo!"
echo ""
echo "Reinicia con: reboot"
echo "Luego conecta SSH con:"
echo "  ssh -p 2222 bela@localhost"
echo "Password: swal123"
"@

    Write-Host "  [OK] Script generado" -ForegroundColor Green
    return $setupScript
}

# ─── Main ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  ⚡ SWAL NixOS Launcher — SouthWest AI Labs" -ForegroundColor White
Write-Host "  NixOS 25.05 + Hyprland AI Workspace" -ForegroundColor Gray
Write-Host "══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

switch ($true) {
    $Status { Get-VMStatus }
    $DownloadISO { Get-NixOSISO }
    $CreateVM { Start-NixOSVM -InstallerMode }
    $SSH { Connect-SSH }
    $FullInstall {
        Write-Host "[AUTO] Iniciando instalación completa..." -ForegroundColor Cyan
        Get-NixOSISO
        Start-NixOSVM -InstallerMode
    }
    default {
        Write-Host "Uso:" -ForegroundColor White
        Write-Host "  -DownloadISO   Descargar NixOS 25.05 minimal ISO" -ForegroundColor Gray
        Write-Host "  -CreateVM      Crear imagen e iniciar VM" -ForegroundColor Gray
        Write-Host "  -SSH           Conectar via SSH" -ForegroundColor Gray
        Write-Host "  -FullInstall   Instalación automática" -ForegroundColor Gray
        Write-Host "  -Status        Ver estado de VMs" -ForegroundColor Gray
        Write-Host ""
        Write-Host "Ejemplo:" -ForegroundColor Yellow
        Write-Host "  .\swal-nixos.ps1 -DownloadISO" -ForegroundColor Gray
        Write-Host "  .\swal-nixos.ps1 -CreateVM" -ForegroundColor Gray
    }
}
