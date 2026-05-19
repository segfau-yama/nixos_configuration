{ pkgs, ... }:
{
  programs.niri.enable = true;

  programs.dconf.enable = true;

  services.dbus.enable = true;

  environment.systemPackages = with pkgs; [
    niri
    swaybg
    mako
    wl-clipboard
    wayshot
    wlsunset
  ];
}
