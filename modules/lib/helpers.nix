{ inputs, lib, ... }:
{
  # flake.lib: 全ホストから使うヘルパー関数を公開する。
  options.flake.lib = lib.mkOption {
    type = lib.types.attrsOf lib.types.unspecified;
    default = { };
    description = "Dendritic パターン用のヘルパー関数群。";
  };

  config.flake.lib = {
    # nixosConfigurations エントリを生成するヘルパー。
    # 使い方:
    #   flake.nixosConfigurations = inputs.self.lib.mkNixos "x86_64-linux" "nixos-default";
    mkNixos = system: name: {
      ${name} = inputs.nixpkgs.lib.nixosSystem {
        modules = [
          inputs.self.modules.nixos.${name}
          { nixpkgs.hostPlatform = lib.mkDefault system; }
        ];
      };
    };
  };
}
