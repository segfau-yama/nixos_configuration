{ ... }:
{
  # development/nix (Home Manager): Nix エコシステムのツール群。
  # 他の development/* ファイルと同じ flake.modules.homeManager.development に追記 (Collector Aspect)。
  flake.modules.homeManager.development = { pkgs, ... }: {
    home.packages = with pkgs; [
      nix-index         # nix-locate で Nix パッケージを検索
      devenv            # 開発環境マネージャー
      nil               # Nix LSP サーバー
      nixfmt-rfc-style  # RFC 形式の Nix フォーマッター
    ];
  };
}
