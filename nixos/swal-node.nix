# ⚡ SWAL Node Core Services & Node Kit Module
# SouthWest AI Labs — Xavier Cognitive Memory & Edge-Mesh Node Daemon
#
# Phase 2 portability: fully parameterized NixOS module. Works for ANY user and
# workspace via `services.swal-node.{enable,user,workspaceDir}` — zero hardcoded
# personal paths. Default `user = "belal"` preserves backwards compatibility for
# flakes that relied on the legacy behavior, but hosts may override it freely
# (e.g. `user = "bela"` for the primary NixOS user).
{ config, pkgs, lib, ... }:
with lib;
let
  cfg = config.services.swal-node;
  # Real homeDirectory of the target user when defined, else /home/<user>.
  userHome = config.users.users.${cfg.user}.home or "/home/${cfg.user}";
in
{
  options.services.swal-node = {
    enable = mkEnableOption "SWAL Autonomous Node (Xavier + Edge-Mesh services)";
    user = mkOption {
      type = types.str;
      default = "belal";
      description = "OS user that owns the SWAL node services and workspace.";
    };
    workspaceDir = mkOption {
      type = types.str;
      default = "/home/${cfg.user}/proyectosSWAL";
      example = "/home/user/proyectosSWAL";
      description = "Root directory of the SWAL ecosystem workspace.";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      # ── Node Networking & RAG Dependencies ──────────────────────────────────
      nodejs_22
      pnpm
      sqlite
      curl
      jq
      ghostty
    ];

    # ─── Environment Variables for SWAL Node ──────────────────────────────────
    environment.variables = {
      XAVIER_API_URL = "http://127.0.0.1:8006";
      XAVIER_MCP_PORT = "8100";
      SWAL_NODE_ENV = "production";
      SWAL_THEME_DEFAULT = "hive-dark";
    };

    # ─── Systemd User Services: Xavier Memory Core ────────────────────────────
    systemd.user.services.xavier-core = {
      description = "SWAL Xavier Cognitive Memory & GraphRAG Server";
      after = [ "network.target" ];
      wantedBy = [ "default.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.bash}/bin/bash -c 'if [ -f ${userHome}/.local/bin/xavier-real ]; then exec ${userHome}/.local/bin/xavier-real http 8006; elif [ -d ${cfg.workspaceDir}/apps/xavier ]; then cd ${cfg.workspaceDir}/apps/xavier && exec cargo run --release -- http 8006; fi'";
        Restart = "on-failure";
        RestartSec = "5s";
        Environment = [
          "XAVIER_WORKSPACE_DIR=${cfg.workspaceDir}"
          "XAVIER_EMBEDDING_CACHE_ENABLED=true"
        ];
      };
    };

    # ─── Systemd User Services: Edge-Mesh P2P Node ────────────────────────────
    systemd.user.services.edge-mesh = {
      description = "SWAL Edge-Mesh P2P Discovery and CRDT Sync Daemon";
      after = [ "network.target" ];
      wantedBy = [ "default.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.bash}/bin/bash -c 'if [ -d ${cfg.workspaceDir}/cores/edge-mesh ]; then cd ${cfg.workspaceDir}/cores/edge-mesh && exec node dist/index.js --daemon; fi'";
        Restart = "on-failure";
        RestartSec = "10s";
      };
    };
  };
}