{ ... }:
{
  # desktopPlasma (NixOS): KDE Plasma Wayland セッションを提供する。
  flake.modules.nixos.desktopPlasma = { config, lib, pkgs, ... }: {
    config = lib.mkIf (config.my.capabilities.window_manager == "plasma") {
      services.desktopManager.plasma6.enable = true;

      services.greetd = {
        enable = true;
        settings.default_session = {
          command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd ${pkgs.kdePackages.plasma-workspace}/bin/startplasma-wayland";
          user = "greeter";
        };
      };
    };
  };

  # desktopPlasma (Home Manager): Plasma ユーザー用の明示的な DE フック。
  flake.modules.homeManager.desktopPlasma = { config, lib, ... }: {
    config = lib.mkIf (config.my.capabilities.window_manager == "plasma") { };
  };
}
