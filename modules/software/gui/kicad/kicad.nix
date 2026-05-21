{ ... }:
{
  # kicad (Home Manager): KiCad 基板設計ツールを提供する。
  flake.modules.homeManager.kicad = { pkgs, ... }: {
    home.packages = with pkgs; [
      kicad  # オープンソース EDA / 基板設計ツール
    ];
  };
}
