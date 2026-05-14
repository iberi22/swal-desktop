{ config, pkgs, inputs, ... }:

let
  user = "beri";
  email = "beri22@gmail.com";
in
{
  home.username = "bela";
  home.homeDirectory = "/home/bela";

  # ─── Packages ───────────────────────────────────────────────────────────
  home.packages = with pkgs; [
    # Packages specific to the user
  ];

  # ─── Hyprland Configuration ─────────────────────────────────────────────
  wayland.windowManager.hyprland = {
    enable = true;
    xwayland.enable = true;
    
    # Plugins from nixpkgs (version-matched with nixpkgs Hyprland)
    plugins = with pkgs.hyprlandPlugins; [
      # hyprtrails  # Uncomment after verifying: nix search nixpkgs#hyprlandPlugins
    ];

    extraConfig = builtins.readFile ../hypr/hyprland.conf;
  };

  # ─── Waybar ─────────────────────────────────────────────────────────────
  programs.waybar = {
    enable = true;
  };

  # ─── Git ───────────────────────────────────────────────────────────────
  programs.git = {
    enable = true;
    userName = user;
    userEmail = email;
  };

  home.stateVersion = "25.05";
}
