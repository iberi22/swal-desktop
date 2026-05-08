# ═══════════════════════════════════════════════════════════════════════════════
# ⚡ SWAL NixOS Hardware Configuration
# Plantilla para hardware real o QEMU/KVM
#
# Para QEMU/KVM: Usar valores por defecto (auto-detectados)
# Para hardware real: Generar con 'nixos-generate-config --root /mnt'
# ═══════════════════════════════════════════════════════════════════════════════

{ config, lib, pkgs, modulesPath, ... }:

{
  imports = [ ];

  # ─── QEMU/KVM Hardware ─────────────────────────────────────────────────
  boot.initrd.availableKernelModules = [
    "ahci"
    "xhci_pci"
    "virtio_pci"
    "virtio_scsi"
    "virtio_balloon"
    "virtio_blk"
    "sd_mod"
    "sr_mod"
  ];

  boot.initrd.kernelModules = [ ];
  boot.kernelModules = [ "kvm-amd" ];
  boot.extraModulePackages = [ ];

  # ─── Filesystems ──────────────────────────────────────────────────────
  # QEMU/KVM: these are placeholders for initial setup
  # After first boot, regenerate with: nixos-generate-config --root /mnt
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/-placeholder";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/placeholder";
    fsType = "vfat";
    options = [ "fmask=0022" "dmask=0022" ];
  };

  swapDevices = [ ];

  # ─── CPU ────────────────────────────────────────────────────────────────
  hardware.cpu.amd.updateMicrocode = true;

  # ─── Platform ──────────────────────────────────────────────────────────
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
}
