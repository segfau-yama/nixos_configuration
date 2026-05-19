{ ... }:
{
  services.mako = {
    enable = true;
    defaultTimeout = 5000;
  };

  services.swaybg = {
    enable = true;
    image = "/usr/share/backgrounds/cyber.png";
    mode = "fill";
  };
}
