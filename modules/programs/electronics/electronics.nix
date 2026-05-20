{ ... }:
{
  # electronics (Home Manager): 電気・電子設計ツール群。
  # KiCad 基板設計 + 回路シミュレーター + ガーバービューア。
  flake.modules.homeManager.electronics = { pkgs, ... }: {
    home.packages = with pkgs; [
      kicad    # オープンソース EDA / 基板設計ツール
    ];
  };
}
