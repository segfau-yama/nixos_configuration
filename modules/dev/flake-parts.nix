{ inputs, ... }:
{
  perSystem = { pkgs, ... }: {
    devShells.default = pkgs.mkShell {
      name = "nixos-config-dev";
      packages = [
        pkgs.nixd       # Nix 言語サーバー
        pkgs.alejandra  # Nix フォーマッタ
      ];
    };
  };
}
