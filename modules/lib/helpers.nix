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
    mkNixos = system: name: {
      ${name} = inputs.nixpkgs.lib.nixosSystem {
        modules = [
          inputs.self.modules.nixos.${name}
          { nixpkgs.hostPlatform = lib.mkDefault system; }
        ];
      };
    };
    
    # 新規: 環境変数から CPU/GPU を自動選択
    # 使い方:
    #   NIX_CPU=aarch64-linux NIX_GPU=nvidia nix flake show
    #   flake.nixosConfigurations = inputs.self.lib.mkNixosWithEnv;
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
          system = cpu;
          modules = [
            inputs.self.modules.nixos.nixos-base
            gpuModule
            { nixpkgs.hostPlatform = lib.mkDefault cpu; }
          ];
        };
      };
  };
}
