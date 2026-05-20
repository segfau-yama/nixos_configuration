{ inputs, lib, ... }:
{
  # flake.lib: 全ホストから使うヘルパー関数を公開する。
  options.flake.lib = lib.mkOption {
    type = lib.types.attrsOf lib.types.unspecified;
    default = { };
    description = "Dendritic パターン用のヘルパー関数群。";
  };

  config.flake.lib = {
    # 既存: 明示的なホスト名で設定を生成
    # 使い方:
    #   flake.nixosConfigurations = inputs.self.lib.mkNixos "x86_64-linux" "nixos-default";
    # hostPlatform は通常優先度 (100) で設定することで、hardware-configuration.nix の
    # lib.mkDefault (1000) を上書きできる。
    mkNixos = system: name: {
      ${name} = inputs.nixpkgs.lib.nixosSystem {
        modules = [
          inputs.self.modules.nixos.${name}
          { nixpkgs.hostPlatform = system; }
        ];
      };
    };

    # 新規: 環境変数から CPU/GPU を自動選択
    # 使い方 (--impure 必須):
    #   NIX_CPU=aarch64-linux NIX_GPU=nvidia sudo nixos-rebuild switch --impure --flake .#nixos
    #
    # 優先度の流れ:
    #   hardware-configuration.nix : lib.mkDefault "x86_64-linux"  → 優先度 1000 (低)
    #   このモジュール  : nixpkgs.hostPlatform = cpu        → 優先度 100  (通常)
    #   → NIX_CPU で指定した値が常に勝つ。
    mkNixosWithEnv =
      let
        cpu = inputs.self.lib.nixCpu or "x86_64-linux";
        gpu = inputs.self.lib.nixGpu or "default";

        gpuModule =
          if gpu == "nvidia" then inputs.self.modules.nixos.nvidia-gpu
          else if gpu == "amd" then inputs.self.modules.nixos.amd-gpu
          else inputs.self.modules.nixos.default-gpu;
      in
      {
        nixos = inputs.nixpkgs.lib.nixosSystem {
          # system = cpu は NixOS 22.05 以降 deprecated。
          # nixpkgs.hostPlatform を modules 内で指定することで代替する。
          modules = [
            inputs.self.modules.nixos.nixos-base
            gpuModule
            # 通常優先度 (100) で設定し hardware-configuration.nix の mkDefault を上書きする。
            { nixpkgs.hostPlatform = cpu; }
          ];
        };
      };
  };
}
