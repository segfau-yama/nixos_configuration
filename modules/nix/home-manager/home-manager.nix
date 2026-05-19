{ inputs, ... }:
{
  # home-manager: Home Manager を NixOS モジュールとして統合する。
  # useGlobalPkgs = true により pkgs (unstable overlay 含む) を共有する。
  flake.modules.nixos.home-manager = { ... }: {
    imports = [ inputs.home-manager.nixosModules.home-manager ];
    home-manager = {
      useGlobalPkgs = true;
      useUserPackages = true;
      backupFileExtension = "bak";
    };
  };
}
