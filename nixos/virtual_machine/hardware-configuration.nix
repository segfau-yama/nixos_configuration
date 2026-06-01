{ config, lib, ... }:
let
  installArgsPath = ./install-args.nix;
  generatedHardwarePath = ./generated-hardware-configuration.nix;
  installArgs =
    if builtins.pathExists installArgsPath
    then import installArgsPath
    else { };
  installDisk = installArgs.installDisk or config.my.installDisk;
in
{
  imports = lib.optionals (builtins.pathExists generatedHardwarePath) [
    generatedHardwarePath
  ];

  boot.initrd.availableKernelModules = [
    "ahci"
    "sd_mod"
    "sr_mod"
    "virtio_blk"
    "virtio_net"
    "virtio_pci"
    "virtio_scsi"
    "virtio_gpu"
    "xhci_pci"
    "usb_storage"
  ];

  fileSystems."/" = lib.mkForce {
    device = installDisk.root;
    fsType = "ext4";
  };

  fileSystems."/boot" = lib.mkForce {
    device = installDisk.boot;
    fsType = "vfat";
  };

  swapDevices = lib.mkForce (
    lib.optional (installDisk.swap != null && installDisk.swap != "") {
      device = installDisk.swap;
    }
  );

  networking.useDHCP = lib.mkDefault true;

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;
}
