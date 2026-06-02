{ ... }:
{
  # plasma (NixOS): KDE Plasma Wayland セッションを提供する。
  flake.modules.nixos.plasma = { ... }: {
    services.desktopManager.plasma6.enable = true;
  };

  # plasma (Home Manager): Plasma ユーザー用の明示的な DE フック。
  flake.modules.homeManager.plasma = { config, lib, ... }: {
    config = lib.mkIf (config.my.capabilities.window_manager == "plasma") { };
  };
}
