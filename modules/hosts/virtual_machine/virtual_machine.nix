{ inputs, ... }:
{
  flake.modules.nixos.virtual_machine = { lib, ... }: {
    imports = with inputs.self.modules.nixos; [
      base
      home-manager
      desktop
      jade
      admin
    ] ++ [ "${inputs.self}/nixos/virtual_machine/hardware-configuration.nix" ];

    networking.hostName = "virtual_machine";

    my.hardware.gpu = lib.mkDefault "virtio";
    my.hardware.cpu = lib.mkDefault "amd";
    my.hardware.storage.enable = false;
    my.installDisk = {
      boot = "/dev/vda1";
      root = "/dev/vda2";
      swap = "/dev/vda3";
    };

    console.keyMap = "us";

    services.qemuGuest.enable = true;
    systemd.services.NetworkManager-wait-online.enable = false;
  };
}
