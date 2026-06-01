{ inputs, ... }:
{
  flake.modules.nixos.laptop = { lib, ... }: {
    imports = with inputs.self.modules.nixos; [
      base
      home-manager
      jade-core
      jade-full
    ] ++ [ "${inputs.self}/nixos/laptop/hardware-configuration.nix" ];

    networking.hostName = "laptop";

    my.hardware.gpu = lib.mkDefault "amd";
    my.hardware.cpu = lib.mkDefault "amd";
    my.hardware.storage.enable = false;
    my.installDisk = {
      boot = "/dev/disk/by-label/boot";
      root = "/dev/disk/by-label/nixos";
      swap = "/dev/disk/by-label/swap";
    };

    my.capabilities.window_manager = "hyprland";

    console.keyMap = "us";
  };
}
