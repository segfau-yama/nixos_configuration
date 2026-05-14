{ ... }:
{
  imports = [
    ./packages.nix
    ./services.nix
    ./hyprland.nix
    ./tofi.nix
    ./ironbar.nix
  ];

  home.username = "yama";
  home.homeDirectory = "/home/yama";
  home.stateVersion = "25.05";

  programs.home-manager.enable = true;

  # Wayland session variables for Electron/Chromium apps.
  home.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    ELECTRON_OZONE_PLATFORM_HINT = "auto";
  };
}
