{ config, pkgs, inputs, ... }:

let
  user = "beri";
  email = "you@example.com"; # override locally; do not commit a real address
in
{
  home.username = "bela";
  home.homeDirectory = "/home/bela";

  # ─── Packages ───────────────────────────────────────────────────────────
  home.packages = with pkgs; [
    zed-editor
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

  # ─── Niri Config Symlink ────────────────────────────────────────────────
  xdg.configFile."niri/config.kdl".source = ../niri/config.kdl;

  # ─── Git ───────────────────────────────────────────────────────────────
  programs.git = {
    enable = true;
    userName = user;
    userEmail = email;
  };

  # ─── Zed — default editor for text/code ───────────────────────────────
  xdg.mimeApps = {
    enable = true;
    defaultApplications = {
      "text/plain" = "dev.zed.Zed.desktop";
      "text/markdown" = "dev.zed.Zed.desktop";
      "text/x-markdown" = "dev.zed.Zed.desktop";
      "text/x-python" = "dev.zed.Zed.desktop";
      "text/x-shellscript" = "dev.zed.Zed.desktop";
      "text/x-c" = "dev.zed.Zed.desktop";
      "text/x-c++" = "dev.zed.Zed.desktop";
      "text/x-java" = "dev.zed.Zed.desktop";
      "text/javascript" = "dev.zed.Zed.desktop";
      "text/x-javascript" = "dev.zed.Zed.desktop";
      "application/json" = "dev.zed.Zed.desktop";
      "application/x-yaml" = "dev.zed.Zed.desktop";
      "application/yaml" = "dev.zed.Zed.desktop";
      "application/toml" = "dev.zed.Zed.desktop";
      "text/x-toml" = "dev.zed.Zed.desktop";
      "text/csv" = "dev.zed.Zed.desktop";
      "text/html" = "dev.zed.Zed.desktop";
      "text/css" = "dev.zed.Zed.desktop";
      "application/xml" = "dev.zed.Zed.desktop";
      "text/xml" = "dev.zed.Zed.desktop";
      "text/x-rust" = "dev.zed.Zed.desktop";
      "text/x-go" = "dev.zed.Zed.desktop";
      "text/x-nix" = "dev.zed.Zed.desktop";
      "text/x-lua" = "dev.zed.Zed.desktop";
      "application/x-shellscript" = "dev.zed.Zed.desktop";
      "x-scheme-handler/zed" = "dev.zed.Zed.desktop";
    };
  };
  home.sessionVariables = {
    EDITOR = "zeditor";
    VISUAL = "zeditor";
  };


  # ─── Zed config (declarativo) — barra lateral izquierda + tema Iberi22 ───
  xdg.configFile."zed/settings.json".source = ../../configs/zed/settings.json;
  xdg.configFile."zed/themes/iberi22-material-ocean.json".source = ../../configs/zed/themes/iberi22-material-ocean.json;
  xdg.configFile."zed/themes/iberi22-dark-eyecare.json".source = ../../configs/zed/themes/iberi22-dark-eyecare.json;

  home.stateVersion = "25.05";
}
