{ ... }:
{
  # media (Home Manager): 音楽・動画再生ソフトとメディアコントロールツールを提供する。
  flake.modules.homeManager.media = { pkgs, ... }: {
    home.packages = with pkgs; [
      spotify    # 音楽ストリーミング
      mpv        # 汎用動画プレイヤー
      oculante   # 高速画像ビューアー
      playerctl  # MPRIS メディアコントロール (再生・停止・トラック切替)
    ];
  };
}
