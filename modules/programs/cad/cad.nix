{ ... }:
{
  # cad (Home Manager): KiCad 基板設計 + FreeCAD 3D モデリングを提供する。
  flake.modules.homeManager.cad = { pkgs, ... }: {
    home.packages = with pkgs; [
      kicad          # オープンソース EDA / 基板設計ツール
      freecad-wayland  # Wayland 対応 FreeCAD
    ];
  };
}
