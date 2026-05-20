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

  outputs = inputs@{ ... }:
    let
      # 環境変数から CPU と GPU を読み込み（デフォルト値付き）
      nixCpu = builtins.getEnv "NIX_CPU";
      nixGpu = builtins.getEnv "NIX_GPU";
      
      # デフォルト値の設定
      cpu = if nixCpu != "" then nixCpu else "x86_64-linux";
      gpu = if nixGpu != "" then nixGpu else "default";
      
      dendriticOutputs =
        inputs.flake-parts.lib.mkFlake { inherit inputs; } {
          systems = [ "x86_64-linux" "aarch64-linux" ];
          imports = [ (inputs.import-tree ./modules) ];
        };
    in
    dendriticOutputs // {
      lib = dendriticOutputs.lib or { } // {
        nixCpu = cpu;
        nixGpu = gpu;
      };
    };
}
