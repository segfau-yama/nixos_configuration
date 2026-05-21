{ ... }:
{
  # freecad (Home Manager): FreeCAD 3D モデラー・メッシュ処理ツールを提供する。
  flake.modules.homeManager.freecad = { pkgs, ... }: {
    home.packages = with pkgs; [
      freecad-wayland  # Wayland 対応 FreeCAD 3D モデラー
      meshlab          # メッシュ修復・解析・変換
    ];
  };
}
