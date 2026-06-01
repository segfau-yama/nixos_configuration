{ ... }:
{
  # desktopPlasma (NixOS): KDE Plasma Wayland セッションを提供する。
  flake.modules.nixos.desktopPlasma = { ... }: {
    services.desktopManager.plasma6.enable = true;
  };

  # desktopPlasma (Home Manager): Plasma ユーザー用の明示的な DE フック。
  flake.modules.homeManager.desktopPlasma = { config, lib, ... }: {
    config = lib.mkIf (config.my.capabilities.window_manager == "plasma") { };
  };
}
