{ ... }:
{
  # mechanical (Home Manager): 機械設計・3D プリント向けツール群。
  # FreeCAD 3D モデリング + スライサー + メッシュ操作 + コードベースモデリング。
  flake.modules.homeManager.mechanical = { pkgs, ... }: {
    home.packages = with pkgs; [
      freecad-wayland  # Wayland 対応 FreeCAD 3D モデラー
      meshlab          # メッシュ修復・解析・変換
    ];
  };
}
