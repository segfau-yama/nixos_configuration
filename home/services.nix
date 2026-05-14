{ ... }:
{
  services.mako = {
    enable = true;
    defaultTimeout = 5000;
  };

  services.hyprpaper = {
    enable = true;
    settings = {
      splash = false;
      ipc = true;
      preload = [ "/usr/share/backgrounds/cyber.png" ];
      wallpaper = [ "eDP-1,/usr/share/backgrounds/cyber.png" ];
    };
  };
}
