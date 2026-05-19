{ ... }:
{
  # niri/greetd (NixOS): greetd + tuigreet で niri セッションを起動するログインマネージャー。
  # system.nix と同じ flake.modules.nixos.niri に追記する (Collector Aspect)。
  flake.modules.nixos.niri = { pkgs, ... }: {
    services.greetd = {
      enable = true;
      settings.default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd 'dbus-run-session niri --session'";
        user    = "greeter";
      };
    };
  };
}
