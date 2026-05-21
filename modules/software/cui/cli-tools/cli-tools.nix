{ ... }:
{
  # cli-tools (Home Manager): 汎用 CLI ユーティリティ群。
  flake.modules.homeManager.cli-tools = { pkgs, ... }: {
    home.packages = with pkgs; [
      git         # バージョン管理
      xh          # 使いやすい HTTP クライアント
      jaq         # jq 互換の高速 JSON プロセッサー
      just        # コマンドランナー (Makefile 代替)
      pkg-config  # ライブラリパス解決ヘルパー
    ];
  };
}
