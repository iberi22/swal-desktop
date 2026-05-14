{ config, pkgs, inputs, ... }:

let
  system = pkgs.stdenv.hostPlatform.system;
in
{
  # ═══════════════════════════════════════════════════════════════════════════
  # ⚡ SWAL AI Agents Module
  # Configuración de herramientas CLI para agentes autónomos
  # ═══════════════════════════════════════════════════════════════════════════

  environment.systemPackages = [
    # ── CLI Agents ──────────────────────────────────────────────────────────
    pkgs.gemini-cli
    # inputs.codex-cli.packages.${system}.default     # Uncomment after verifying flake
    # inputs.hermes-agent.packages.${system}.default   # Uncomment after verifying flake
    
    # ── Agent Ecosystem (not already in configuration.nix) ──────────────────
    pkgs.direnv
    pkgs.yq-go
  ];

  # ─── Environment Variables for Agents ─────────────────────────────────────
  environment.variables = {
    GEMINI_API_KEY = "";       # Set via: export GEMINI_API_KEY=xxx in ~/.zshrc
    OPENAI_API_KEY = "";
    DEEPSEEK_API_KEY = "";
    OPENCODE_MODEL = "deepseek-v4-flash";
  };

  # ─── Direnv Integration ──────────────────────────────────────────────────
  programs.direnv = {
    enable = true;
    nix-direnv.enable = true;
  };
}
