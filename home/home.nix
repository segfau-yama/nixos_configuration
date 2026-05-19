{ ... }:
{
  imports = [
    ./packages.nix
    ./browser.nix
    ./sns.nix
    ./services.nix
    ./niri.nix
    ./tofi.nix
    ./ironbar.nix
  ];

  home.username = "yama";
  home.homeDirectory = "/home/yama";
  home.stateVersion = "25.05";

  programs.home-manager.enable = true;

  # Electron/Chromium 系アプリ向けの Wayland セッション変数。
  home.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    ELECTRON_OZONE_PLATFORM_HINT = "auto";
  };
}
