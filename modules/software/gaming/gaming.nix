{ ... }:
{
  # gaming (Home Manager): Lutris + Wine + Winetricks を提供する。
  # NixOS 側の Steam / Gamemode は my.hardware.gpu の設定で自動有効化される。
  flake.modules.homeManager.gaming = { pkgs, ... }: {
    home.packages = with pkgs; [
      lutris
      wineWowPackages.stable  # 32/64bit Wine を同梱
      winetricks
    ];
  };
}
