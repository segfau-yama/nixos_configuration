{ pkgs, ... }:
{
  programs.hyprland = {
    enable = true;
    xwayland.enable = true;
  };

  programs.dconf.enable = true;

  services.dbus.enable = true;

  environment.systemPackages = with pkgs; [
    hyprpaper
    mako
    wl-clipboard
    wayshot
  ];
}