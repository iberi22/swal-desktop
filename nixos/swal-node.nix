{ config, pkgs, lib, ... }:

{
  # ═══════════════════════════════════════════════════════════════════════════
  # ⚡ SWAL Node Core Services & Node Kit Module
  # SouthWest AI Labs — Xavier Cognitive Memory & Edge-Mesh Node Daemon
  # ═══════════════════════════════════════════════════════════════════════════

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
      ExecStart = "${pkgs.bash}/bin/bash -c 'if [ -f /home/belal/.local/bin/xavier-real ]; then exec /home/belal/.local/bin/xavier-real http 8006; elif [ -d /home/belal/proyectosSWAL/apps/xavier ]; then cd /home/belal/proyectosSWAL/apps/xavier && exec cargo run --release -- http 8006; fi'";
      Restart = "on-failure";
      RestartSec = "5s";
      Environment = [
        "XAVIER_WORKSPACE_DIR=/home/belal/proyectosSWAL"
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
      ExecStart = "${pkgs.bash}/bin/bash -c 'if [ -d /home/belal/proyectosSWAL/cores/edge-mesh ]; then cd /home/belal/proyectosSWAL/cores/edge-mesh && exec node dist/index.js --daemon; fi'";
      Restart = "on-failure";
      RestartSec = "10s";
    };
  };
}
