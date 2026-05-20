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
      # builtins.getEnv は pure evaluation（デフォルト）では常に空文字列を返す。
      # NIX_CPU / NIX_GPU を有効にするには nixos-rebuild / nixos-install に
      # --impure フラグが必要。指定しない場合は以下のデフォルト値が使われる。
      #
      #   NIX_CPU=x86_64-linux NIX_GPU=default
      #
      # 使用例:
      #   NIX_GPU=nvidia sudo nixos-rebuild switch --impure --flake .#nixos
      nixCpu = builtins.getEnv "NIX_CPU";
      nixGpu = builtins.getEnv "NIX_GPU";

      # デフォルト値: 環境変数が空（pure eval 含む）の場合に使用される。
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
