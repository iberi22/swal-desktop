{
  description = "⚡ SWAL Desktop — NixOS + Hyprland AI Workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # ── Optional AI Agent Flakes (uncomment after verifying on NixOS) ──
    # codex-cli = {
    #   url = "github:sadjow/codex-cli-nix";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
    # dank-material-shell = {
    #   url = "github:dank-space/dank-material-shell";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
    # hermes-agent = {
    #   url = "github:NousResearch/hermes-agent";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
  };

  outputs = { self, nixpkgs, home-manager, ... }@inputs: {
    nixosConfigurations = {
      swal = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = { inherit inputs; };
        modules = [
          ./nixos/configuration.nix
          home-manager.nixosModules.home-manager
          {
            home-manager.useGlobalPkgs = true;
            home-manager.extraSpecialArgs = { inherit inputs; };
          }
        ];
      };
    };
  };
}
