{ ... }:
{
  # nix-tools (Home Manager): Nix エコシステム開発補助ツール群。
  flake.modules.homeManager.nix-tools = { pkgs, ... }: {
    home.packages = with pkgs; [
      nix-index         # nix-locate で Nix パッケージを検索
      devenv            # 開発環境マネージャー
      nil               # Nix LSP サーバー
      nixfmt-rfc-style  # RFC 形式の Nix フォーマッター
    ];
  };
}
