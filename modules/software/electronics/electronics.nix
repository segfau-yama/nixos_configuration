{ ... }:
{
  # electronics (Home Manager): KiCad 基板設計ツールを提供する。
  flake.modules.homeManager.electronics = { pkgs, ... }: {
    home.packages = with pkgs; [
      kicad  # オープンソース EDA / 基板設計ツール
    ];
  };
}
