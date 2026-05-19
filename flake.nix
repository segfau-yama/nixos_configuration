{
  description = "Modular NixOS desktop: Wayland + Niri + NVIDIA + Gaming + Dev + CAD";

  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    import-tree.url = "github:vic/import-tree";

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ nixpkgs, nixpkgs-unstable, home-manager, ... }:
    let
      system = "x86_64-linux";
      unstablePkgs = import nixpkgs-unstable {
        inherit system;
        config.allowUnfree = true;
      };

      legacyConfigurations = {
        default = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs unstablePkgs;
          };
          modules = [
            ./legacy/hosts/default.nix
            home-manager.nixosModules.home-manager
            {
              # Home Manager を宣言的な NixOS 再構築フローに統合。
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.extraSpecialArgs = {
                inherit unstablePkgs;
              };
              home-manager.users.yama = import ./legacy/home/home.nix;
            }
          ];
        };

        nvidia = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs unstablePkgs;
          };
          modules = [
            ./legacy/hosts/nvidia.nix
            home-manager.nixosModules.home-manager
            {
              # Home Manager を宣言的な NixOS 再構築フローに統合。
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.extraSpecialArgs = {
                inherit unstablePkgs;
              };
              home-manager.users.yama = import ./legacy/home/home.nix;
            }
          ];
        };

        amd = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs unstablePkgs;
          };
          modules = [
            ./legacy/hosts/amd.nix
            home-manager.nixosModules.home-manager
            {
              # Home Manager を宣言的な NixOS 再構築フローに統合。
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.extraSpecialArgs = {
                inherit unstablePkgs;
              };
              home-manager.users.yama = import ./legacy/home/home.nix;
            }
          ];
        };

        # /nix ストアを HDD に配置するプロファイル。
        # 使用方法: sudo nixos-rebuild switch --flake .#with-hdd
        # 事前に hdd.nix の UUID を実際の HDD UUID に書き換えること。
        with-hdd = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs unstablePkgs;
          };
          modules = [
            ./legacy/hosts/default.nix
            ./legacy/modules/hardware/hdd.nix
            home-manager.nixosModules.home-manager
            {
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.extraSpecialArgs = {
                inherit unstablePkgs;
              };
              home-manager.users.yama = import ./legacy/home/home.nix;
            }
          ];
        };

        # HDD 未接続時のフォールバックプロファイル（デフォルトと同等）。
        # 使用方法: sudo nixos-rebuild switch --flake .#without-hdd
        without-hdd = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs unstablePkgs;
          };
          modules = [
            ./legacy/hosts/default.nix
            home-manager.nixosModules.home-manager
            {
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.extraSpecialArgs = {
                inherit unstablePkgs;
              };
              home-manager.users.yama = import ./legacy/home/home.nix;
            }
          ];
        };
      };

      dendriticOutputs =
        inputs.flake-parts.lib.mkFlake { inherit inputs; } {
          systems = [ "x86_64-linux" ];
          imports = [ (inputs.import-tree ./modules) ];
        };
    in
    dendriticOutputs
    // {
      nixosConfigurations =
        legacyConfigurations
        // (dendriticOutputs.nixosConfigurations or { });
    };
}
