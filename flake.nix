{
  description = "⚡ SWAL Desktop — NixOS + Hyprland AI Workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    hyprland = {
      url = "github:hyprwm/Hyprland";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # hyprland-plugins = {
    #   url = "github:hyprwm/hyprland-plugins";
    #   inputs.hyprland.follows = "hyprland";
    # };

    nixos-hardware.url = "github:NixOS/nixos-hardware";
    stylix.url = "github:danth/stylix";
    stylix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, home-manager, hyprland, stylix, ... }@inputs: {
    packages = forAllSystems (system:
      {
        swal-desktop = self.packages.${system}.default;
      }
    );

    nixosConfigurations = {
      swal = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit inputs; };
        modules = [
          ./nixos/configuration.nix
          stylix.nixosModules.stylix
          home-manager.nixosModules.home-manager
          {
            home-manager.useGlobalPkgs = true;
            home-manager.users.bela = import ./home.nix;
            programs.starship.enable = true;
          }
        ];
      };
    };

    defaultPackage = forAllSystems (system:
      self.nixosConfigurations.swal.config.package
    );
  };
}
