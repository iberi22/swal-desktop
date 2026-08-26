{ config, pkgs, lib, inputs, ... }:

let
  swal-config-menu = pkgs.writeShellScriptBin "swal-config-menu" ''
    export PATH=${lib.makeBinPath (with pkgs; [ rofi libnotify hyprland niri eww findutils swww hyprpaper ghostty kitty curl jq procps systemd utillinux coreutils bash ])}:$PATH
    exec ${pkgs.bash}/bin/bash ${../scripts/swal-config-menu.sh} "$@"
  '';
  swal-session = pkgs.writeShellScriptBin "swal-session" ''
    export PATH=${lib.makeBinPath (with pkgs; [ hyprland niri libnotify bash coreutils ])}:$PATH
    exec ${pkgs.bash}/bin/bash ${../scripts/swal-session.sh} "$@"
  '';
in
{
  # ═══════════════════════════════════════════════════════════════════════════
  # ⚡ SWAL NixOS Configuration — SouthWest AI Labs
  # NixOS 25.05 + Hyprland + Dank Material Shell
  # 
  # Para rebuild: sudo nixos-rebuild switch --flake .#swal
  # ═══════════════════════════════════════════════════════════════════════════

  imports = [
    ./hardware.nix # Generado por el instalador
    ./ai-agents.nix
    ./swal-node.nix
  ];
  # Nota: hardware-configuration.nix ya existe en /etc/nixos/

  # ─── Boot ────────────────────────────────────────────────────────────────
  boot.loader.systemd-boot.enable = true;
  boot.loader.systemd-boot.editor = true;
  boot.loader.efi.canTouchEfiVariables = true;
  boot.loader.timeout = 5;
  boot.loader.grub.enable = false;
  boot.supportedFilesystems = [ "ntfs" "ext4" "vfat" ];
  boot.kernelModules = [ "kvm-amd" ];

  # ─── Locales & Time ─────────────────────────────────────────────────────
  time.timeZone = "America/Bogota";
  i18n.defaultLocale = "es_CO.UTF-8";
  i18n.extraLocaleSettings = {
    LC_TIME = "es_CO.UTF-8";
    LC_MONETARY = "es_CO.UTF-8";
    LC_ADDRESS = "es_CO.UTF-8";
    LC_TELEPHONE = "es_CO.UTF-8";
    LC_MEASUREMENT = "es_CO.UTF-8";
  };
  console.keyMap = "la-latin1";
  console.font = "Lat2-Terminus16";

  # ─── Networking ──────────────────────────────────────────────────────────
  networking.hostName = "swal-desktop";
  networking.networkmanager.enable = true;
  networking.firewall.enable = true;
  networking.firewall.allowedTCPPorts = [ 22 80 443 2222 ];
  networking.firewall.allowedUDPPorts = [ ];

  # ─── Users ───────────────────────────────────────────────────────────────
  users.users.root.hashedPassword = "!"; # Disabled
  users.mutableUsers = true;
  users.users.bela = {
    isNormalUser = true;
    description = "Bela — SWAL";
    extraGroups = [
      "networkmanager"
      "wheel"
      "docker"
      "video"
      "audio"
      "input"
    ];
    shell = pkgs.zsh;
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKH6FMEJmkBG8sLCEBLHvoSjMnRxhtO4hI3CQzT1k5Ny belal@DESKTOP-EDITOR1"
    ];
    initialPassword = "swal123";
  };

  # ─── Security ────────────────────────────────────────────────────────────
  security.sudo.wheelNeedsPassword = false;
  security.polkit.enable = true;
  security.rtkit.enable = true;

  # ─── Programs ────────────────────────────────────────────────────────────
  programs.zsh.enable = true;
  programs.starship.enable = true;
  programs.git.enable = true;
  programs.dconf.enable = true;
  programs.ssh.startAgent = true;

  # ─── Services ────────────────────────────────────────────────────────────
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "prohibit-password";
      PasswordAuthentication = false;
    };
  };
  services.dbus.enable = true;
  services.udisks2.enable = true;

  # ─── SWAL Node (Xavier + Edge-Mesh) ──────────────────────────────────────
  # Parametrized module (nixos/swal-node.nix): user = primary OS user (bela),
  # workspaceDir defaults to /home/<user>/proyectosSWAL.
  services.swal-node = {
    enable = true;
    user = "bela";
  };

  # ─── Virtualization ────────────────────────────────────────────────────────
  # virtualisation.docker.enable = true;
  # Deshabilitado en VM para evitar problemas

  # ─── Display: greetd → Hyprland ──────────────────────────────────────────
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${swal-session}/bin/swal-session";
        user = "bela";
      };
    };
  };

  programs.hyprland = {
    enable = true;
    withUWSM = false;
    xwayland.enable = true;
    # plugins = [
    #   inputs.hyprland-plugins.packages.${pkgs.system}.hyprtrails
    # ];
  };

  # ─── Niri Compositor (Archcraft Style & Fallback) ──────────────────────
  programs.niri.enable = true;

  # ─── Fonts ────────────────────────────────────────────────────────────────
  fonts.packages = with pkgs; [
    noto-fonts
    font-awesome
    fira-code
    fira-code-symbols
  ];

  # ─── XDG Portals (screen sharing!) ───────────────────────────────────────
  xdg.portal = {
    enable = true;
    extraPortals = with pkgs; [
      xdg-desktop-portal-hyprland
      xdg-desktop-portal-gtk
    ];
    configPackages = [ pkgs.hyprland ];
    xdgOpenUsePortal = true;
  };

  # ─── Audio: PipeWire ───────────────────────────────────────────────────────
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    wireplumber.enable = true;
  };

  # ─── Environment Packages ─────────────────────────────────────────────────
  environment.systemPackages = with pkgs; [
    # ── Shell & Terminal ──────────────────────────────────────────────────
    kitty
    ghostty
    zsh
    starship
    neovim
    vim

    # ── Dev Tools ────────────────────────────────────────────────────────
    git
    gh
    lazygit
    gitui
    delta
    ripgrep
    fd
    fzf
    bat
    eza
    zoxide
    jq
    tmux
    btop

    # ── Languages ────────────────────────────────────────────────────────
    python3
    python3Packages.pip
    nodejs_22
    nodePackages_latest.pnpm
    go
    rustup
    gcc
    gnumake
    cmake

    # ── Network ─────────────────────────────────────────────────────────
    curl
    wget
    openssh
    openssl
    nmap

    # ── Containers ───────────────────────────────────────────────────────
    docker
    docker-compose

    # ── Hyprland & WM ────────────────────────────────────────────────────
    hyprland
    niri
    fuzzel
    hyprpaper
    hyprlock
    hypridle
    hyprpicker
    waybar
    rofi
    dunst
    libnotify
    wl-clipboard
    wlogout
    swww
    eww
    swal-config-menu
    swal-session

    # ── Utilities ────────────────────────────────────────────────────────
    polkit_gnome
    networkmanagerapplet
    pavucontrol
    blueman
    brightnessctl
    playerctl
    mako
    slurp
    grim
    swappy

    # ── Browser & Media ─────────────────────────────────────────────────
    firefox
    chromium
    vlc
    mpv

    # ── Monitoring ─────────────────────────────────────────────────────
    fastfetch
    nix-output-monitor

    # ── NixOS ───────────────────────────────────────────────────────────
    home-manager
    devenv
    
    # ── Appearance ─────────────────────────────────────────────────────
    phinger-cursors
    catppuccin-gtk
    catppuccin-kvantum
    # inputs.dank-material-shell.packages.${pkgs.system}.default  # Uncomment when flake input is enabled
  ];

  # ─── Nix Settings ─────────────────────────────────────────────────────────
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
  nix.settings.auto-optimise-store = true;
  nix.settings.trusted-users = [ "root" "bela" ];
  nix.gc = {
    automatic = true;
    dates = "weekly";
    options = "--delete-older-than 7d";
  };

  home-manager.users.bela = import ./home.nix;

  system.stateVersion = "25.05";
}
