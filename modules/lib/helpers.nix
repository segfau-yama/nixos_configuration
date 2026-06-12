{ inputs, lib, ... }:
{
  # flake.lib: 全ホストから使うヘルパー関数を公開する。
  options.flake.lib = lib.mkOption {
    type = lib.types.attrsOf lib.types.unspecified;
    default = { };
    description = "Dendritic パターン用のヘルパー関数群。";
  };

  config.flake.lib = {
    # 明示的なホスト名で設定を生成する。
    # 使い方:
    #   flake.nixosConfigurations = inputs.self.lib.mkNixos "x86_64-linux" "laptop";
    # hostPlatform は通常優先度 (100) で設定することで、hardware-configuration.nix の
    # lib.mkDefault (1000) を上書きできる。
    mkNixos = system: name: {
      ${name} = inputs.nixpkgs.lib.nixosSystem {
        specialArgs = { inherit inputs; };
        modules = [
          (inputs.self + "/nixos/${name}/configuration.nix")
          { nixpkgs.hostPlatform = system; }
        ];
      };
    };
  };
}
