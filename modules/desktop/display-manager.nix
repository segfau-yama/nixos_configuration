{ pkgs, ... }:
{
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${pkgs.greetd.tuigreet}/bin/tuigreet --time --cmd 'dbus-run-session niri --session'";
        user = "greeter";
      };
    };
  };

  security.polkit.enable = true;
  services.seatd.enable = true;
}