{ ... }:
{
  # programming/nix (Home Manager): Nix エコシステムのツール群。
  # 他の programming/* ファイルと同じ flake.modules.homeManager.programming に追記 (Collector Aspect)。
  flake.modules.homeManager.programming = { pkgs, ... }: {
    home.packages = with pkgs; [
      nix-index         # nix-locate で Nix パッケージを検索
      devenv            # 開発環境マネージャー
      nil               # Nix LSP サーバー
      nixfmt-rfc-style  # RFC 形式の Nix フォーマッター
    ];
  };
}
