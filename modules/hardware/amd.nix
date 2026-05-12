{ pkgs, ... }:
{
  services.xserver.videoDrivers = [ "amdgpu" ];

  # RADV is default on AMD and works well for gaming on Wayland.
  environment.systemPackages = with pkgs; [
    mesa
  ];
}