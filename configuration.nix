{ ... }:
{
  imports = [
    # Core system modules
    ./modules/core/boot.nix
    ./modules/core/nix.nix
    ./modules/core/gc.nix
    ./modules/core/networking.nix
    ./modules/core/timezone.nix

    # Locale and input method
    ./modules/locale/locale.nix
    ./modules/locale/fonts.nix
    ./modules/locale/fcitx5-mozc.nix

    # Desktop stack
    ./modules/desktop/display-manager.nix
    ./modules/desktop/hyprland.nix
    ./modules/desktop/ironbar.nix
    ./modules/desktop/terminal.nix
    ./modules/desktop/xdg-portal.nix

    # Hardware acceleration
    ./modules/hardware/opengl.nix
    ./modules/hardware/vulkan.nix

    # Audio stack
    ./modules/audio/pipewire.nix
    ./modules/audio/audio-utils.nix

    # Gaming stack
    ./modules/gaming/steam.nix
    ./modules/gaming/lutris.nix
    ./modules/gaming/gamemode.nix
    ./modules/gaming/wine.nix

    # Development/CAD
    ./modules/development/zed.nix
    ./modules/development/tools.nix

    # Media apps
    ./modules/media/music.nix
    ./modules/media/video.nix

    # Users
    ./modules/users/default.nix
  ];

  system.stateVersion = "25.05";
}