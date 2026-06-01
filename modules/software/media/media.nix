{ ... }:
{
  # media (Home Manager): capability に応じて GUI/TUI のメディアツールを切り替える。
  flake.modules.homeManager.media = { config, lib, pkgs, ... }: {
    config = lib.mkMerge [
      (lib.mkIf (config.my.capabilities.user_interface == "gui") {
        home.packages = with pkgs; [
          spotify    # 音楽ストリーミング
          mpv        # 汎用動画プレーヤー
          oculante   # 高速画像ビューアー
          playerctl  # MPRIS メディアコントロール
        ];
      })
      (lib.mkIf (config.my.capabilities.user_interface == "tui") {
        home.packages = with pkgs; [
          mpv
          yt-dlp
        ];
      })
    ];
  };
}
