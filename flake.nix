{
  description = "Modular NixOS desktop: Wayland + Hyprland + NVIDIA + Gaming + Dev + CAD";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, nixpkgs-unstable, home-manager, ... }:
    let
      system = "x86_64-linux";
      unstablePkgs = import nixpkgs-unstable {
        inherit system;
        config.allowUnfree = true;
      };
    in {
      nixosConfigurations.default = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs unstablePkgs;
        };
        modules = [
          ./hosts/default.nix
          home-manager.nixosModules.home-manager
          {
            # Home Manager integrated into declarative NixOS rebuild flow.
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.extraSpecialArgs = {
              inherit unstablePkgs;
            };
            home-manager.users.yama = import ./home/home.nix;
          }
        ];
      };

      nixosConfigurations.nvidia = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs unstablePkgs;
        };
        modules = [
          ./hosts/nvidia.nix
          home-manager.nixosModules.home-manager
          {
            # Home Manager integrated into declarative NixOS rebuild flow.
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.extraSpecialArgs = {
              inherit unstablePkgs;
            };
            home-manager.users.yama = import ./home/home.nix;
          }
        ];
      };

      nixosConfigurations.amd = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs unstablePkgs;
        };
        modules = [
          ./hosts/amd.nix
          home-manager.nixosModules.home-manager
          {
            # Home Manager integrated into declarative NixOS rebuild flow.
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.extraSpecialArgs = {
              inherit unstablePkgs;
            };
            home-manager.users.yama = import ./home/home.nix;
          }
        ];
      };
    };
}