{ config, pkgs, ... }:
{
  boot.kernelParams = [
    "nvidia-drm.modeset=1"
  ];

  services.xserver.videoDrivers = [ "nvidia" ];

  hardware.nvidia = {
    modesetting.enable = true;
    powerManagement.enable = false;
    powerManagement.finegrained = false;
    open = false;
    nvidiaSettings = true;
    # Prefer long-lived branch for older GPUs like GTX 1080.
    package = config.boot.kernelPackages.nvidiaPackages.production
      or config.boot.kernelPackages.nvidiaPackages.stable;
  };

  environment.sessionVariables = {
    NIXOS_OZONE_WL = "1";
  };

  environment.systemPackages = with pkgs; [
    nvtopPackages.nvidia
  ];
}