{ pkgs, ... }:
{
  home.packages = with pkgs; [
    chromium
    discord
    kicad
    freecad-wayland
    tofi
  ];
}
