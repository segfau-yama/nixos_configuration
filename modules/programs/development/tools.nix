{ ... }:
{
  # development/tools (Home Manager): 汎用 CLI ツール群。
  # 他の development/* ファイルと同じ flake.modules.homeManager.development に追記 (Collector Aspect)。
  flake.modules.homeManager.development = { pkgs, ... }: {
    home.packages = with pkgs; [
      git         # バージョン管理
      xh          # 使いやすい HTTP クライアント
      jaq         # jq 互換の高速 JSON プロセッサー
      just        # コマンドランナー (Makefile 代替)
      pkg-config  # ライブラリパス解決ヘルパー
    ];
  };
}
