{ pkgs, ... }:
{
  services.xserver.videoDrivers = [ "amdgpu" ];

  # RADV は AMD で標準となり、Wayland のゲーム用途でも安定しやすい。
  environment.systemPackages = with pkgs; [
    mesa
  ];
}