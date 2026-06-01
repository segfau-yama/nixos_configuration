{ ... }:
{
  # programming (Home Manager): 開発用 CLI・言語ツール・シェル設定。GUI では Zed も追加する。
  flake.modules.homeManager.programming = { config, lib, pkgs, ... }: {
    config = lib.mkMerge [
      {
        programs.nushell.enable = true;
        programs.direnv = {
          enable = true;
          nix-direnv.enable = true;
        };
        home.packages = with pkgs; [
          xh          # 使いやすい HTTP クライアント
          jaq         # jq 互換の高速 JSON プロセッサー
          just        # コマンドランナー (Makefile 代替)
          pkg-config  # ライブラリパス解決ヘルパー
          rustc       # Rust コンパイラ
          clang       # C/C++ コンパイラ
          mold        # 高速リンカー
          python3     # Python 3 インタープリター
        ];
      }
      (lib.mkIf (config.my.capabilities.user_interface == "gui") {
        home.packages = with pkgs; [
          unstable.zed-editor # Zed エディター
        ];
      })
      (lib.mkIf (config.my.capabilities.user_interface == "tui") {
        home.packages = with pkgs; [
          gitui  # TUI Git クライアント
        ];
      })
    ];
  };
}
