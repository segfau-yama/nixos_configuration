{ ... }:
{
  # programming/tools (Home Manager): 汎用 CLI ツール群。
  # 他の programming/* ファイルと同じ flake.modules.homeManager.programming に追記 (Collector Aspect)。
  flake.modules.homeManager.programming = { pkgs, ... }: {
    home.packages = with pkgs; [
      git         # バージョン管理
      xh          # 使いやすい HTTP クライアント
      jaq         # jq 互換の高速 JSON プロセッサー
      just        # コマンドランナー (Makefile 代替)
      pkg-config  # ライブラリパス解決ヘルパー
    ];
  };
}
